//! Config should always be deserializable, default values should be used
//! This code has lots of copy/paste from the example config but it's not really copy/paste since we use our own private types.
//! @Pending make this copy/paste generic code on the library
use alloy_primitives::U256;
use alloy_signer_local::PrivateKeySigner;
use derivative::Derivative;
use eyre::Context;
use http::StatusCode;
use jsonrpsee::RpcModule;
use rbuilder::building::builders::parallel_builder::parallel_build_backtest;
use rbuilder::building::order_priority::{FullProfitInfoGetter, NonMempoolProfitInfoGetter};
use rbuilder::building::{BuiltBlockTrace, PartialBlockExecutionTracer};
use rbuilder::live_builder::base_config::EnvOrValue;
use rbuilder::live_builder::block_output::bidding_service_interface::{
    BidObserver, LandedBlockInfo,
};
use rbuilder::live_builder::config::{
    build_backtest_block_ordering_builder, create_builder_from_sink, create_builders,
    create_sink_factory_and_relays, create_wallet_balance_watcher, BuilderConfig,
    SpecificBuilderConfig,
};
use rbuilder::live_builder::payload_events::MevBoostSlotData;
use rbuilder::provider::StateProviderFactory;
use rbuilder::{
    building::builders::{BacktestSimulateBlockInput, Block},
    live_builder::{
        base_config::BaseConfig, cli::LiveBuilderConfig, config::L1Config, LiveBuilder,
    },
    utils::build_info::Version,
};
use rbuilder_primitives::mev_boost::SubmitBlockRequest;
use reth::primitives::SealedBlock;
use serde::Deserialize;
use serde_with::serde_as;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};
use url::Url;

use crate::bidding_service_wrapper::client::bidding_service_client_adapter::BiddingServiceClientAdapter;
use crate::blocks_processor::{
    BlocksProcessorClient, BlocksProcessorClientBidObserver,
    SIGNED_BLOCK_CONSUME_BUILT_BLOCK_METHOD,
};
use crate::build_info::rbuilder_version;
use crate::true_block_value_push::best_true_value_observer::BestTrueValueObserver;

use clickhouse::Client;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct ClickhouseConfig {
    /// clickhouse host url (starts with http/https)
    pub clickhouse_host_url: Option<EnvOrValue<String>>,
    pub clickhouse_user: Option<EnvOrValue<String>>,
    pub clickhouse_password: Option<EnvOrValue<String>>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
/// Config to push TBV to a redis channel.
struct TBVPushRedisConfig {
    /// redis connection string for pushing best bid value
    /// Option so we can have Default for Deserialize but always required.
    pub url: Option<EnvOrValue<String>>,

    /// redis channel name for syncing best bid value
    pub channel: String,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Derivative)]
#[serde(default, deny_unknown_fields)]
#[derivative(Default)]
pub struct FlashbotsConfig {
    #[serde(flatten)]
    pub base_config: BaseConfig,

    #[serde(flatten)]
    pub l1_config: L1Config,

    #[serde(flatten)]
    clickhouse: ClickhouseConfig,

    #[serde(default)]
    pub flashbots_builder_pubkeys: Vec<String>,

    // bidding server ipc path config.
    bidding_service_ipc_path: String,

    /// selected builder configurations
    pub builders: Vec<BuilderConfig>,

    /// If this is Some then blocks_processor_url MUST be some and:
    /// - signed mode is used for blocks_processor.
    /// - tbv_push is done via blocks_processor_url (signed block-processor also handles flashbots_reportBestTrueValue).
    pub key_registration_url: Option<String>,

    pub blocks_processor_url: Option<String>,

    #[serde(default = "default_blocks_processor_max_concurrent_requests")]
    #[derivative(Default(value = "default_blocks_processor_max_concurrent_requests()"))]
    pub blocks_processor_max_concurrent_requests: usize,
    #[serde(default = "default_blocks_processor_max_request_size_bytes")]
    #[derivative(Default(value = "default_blocks_processor_max_request_size_bytes()"))]
    pub blocks_processor_max_request_size_bytes: u32,

    /// Cfg to push tbv to redis.
    /// For production we always need some tbv push (since it's used by smart-multiplexing.) so:
    /// !Some(key_registration_url) => Some(tbv_push_redis)
    tbv_push_redis: Option<TBVPushRedisConfig>,
}

impl LiveBuilderConfig for FlashbotsConfig {
    fn base_config(&self) -> &BaseConfig {
        &self.base_config
    }

    async fn new_builder<P>(
        &self,
        provider: P,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<LiveBuilder<P>>
    where
        P: StateProviderFactory + Clone + 'static,
    {
        if self.l1_config.scraped_bids_publisher_url.is_none() {
            eyre::bail!("scraped_bids_publisher_url is not set");
        }

        let (wallet_balance_watcher, landed_blocks) =
            create_wallet_balance_watcher(provider.clone(), &self.base_config).await?;

        let bidding_service = self.create_bidding_service(&landed_blocks).await?;

        let bid_observer = self.create_bid_observer(&cancellation_token).await?;

        let (sink_factory, slot_info_provider, adjustment_fee_payers) =
            create_sink_factory_and_relays(
                &self.base_config,
                &self.l1_config,
                wallet_balance_watcher,
                bid_observer,
                bidding_service.clone(),
                cancellation_token.clone(),
            )
            .await?;

        let live_builder = create_builder_from_sink(
            &self.base_config,
            &self.l1_config,
            provider,
            sink_factory,
            slot_info_provider,
            adjustment_fee_payers,
            cancellation_token,
        )
        .await?;

        let mut module = RpcModule::new(());
        module.register_async_method("bid_subsidiseBlock", move |params, _| {
            handle_subsidise_block(bidding_service.clone(), params)
        })?;
        let live_builder = live_builder.with_extra_rpc(module);
        let builders = create_builders(self.live_builders()?);
        Ok(live_builder.with_builders(builders))
    }

    fn version_for_telemetry(&self) -> Version {
        rbuilder_version()
    }

    /// @Pending fix this ugly copy/paste
    fn build_backtest_block<
        P,
        PartialBlockExecutionTracerType: PartialBlockExecutionTracer + Clone + Send + Sync + 'static,
    >(
        &self,
        building_algorithm_name: &str,
        input: BacktestSimulateBlockInput<'_, P>,
        partial_block_execution_tracer: PartialBlockExecutionTracerType,
    ) -> eyre::Result<Block>
    where
        P: StateProviderFactory + Clone + 'static,
    {
        let builder_cfg = self.builder(building_algorithm_name)?;
        match builder_cfg.builder {
            SpecificBuilderConfig::OrderingBuilder(config) => {
                if config.ignore_mempool_profit_on_bundles {
                    build_backtest_block_ordering_builder::<
                        P,
                        NonMempoolProfitInfoGetter,
                        PartialBlockExecutionTracerType,
                    >(config, input, partial_block_execution_tracer)
                } else {
                    build_backtest_block_ordering_builder::<
                        P,
                        FullProfitInfoGetter,
                        PartialBlockExecutionTracerType,
                    >(config, input, partial_block_execution_tracer)
                }
            }
            SpecificBuilderConfig::ParallelBuilder(config) => {
                parallel_build_backtest::<P>(input, config)
            }
        }
    }
}

async fn handle_subsidise_block(
    bidding_service: Arc<BiddingServiceClientAdapter>,
    params: jsonrpsee::types::Params<'static>,
) {
    match params.one() {
        Ok(block_number) => bidding_service.must_win_block(block_number).await,
        Err(err) => warn!(?err, "Failed to parse block_number"),
    };
}

#[derive(thiserror::Error, Debug)]
enum RegisterKeyError {
    #[error("Register key error parsing url: {0:?}")]
    UrlParse(#[from] url::ParseError),
    #[error("Register key network error: {0:?}")]
    Network(#[from] reqwest::Error),
    #[error("Register key service error: {0:?}")]
    Service(StatusCode),
}

impl FlashbotsConfig {
    /// Returns the BiddingService + an optional FlashbotsBlockSubsidySelector so smart multiplexing can force blocks.
    /// FlashbotsBlockSubsidySelector can be None if subcidy is disabled.
    pub async fn create_bidding_service(
        &self,
        landed_blocks_history: &[LandedBlockInfo],
    ) -> eyre::Result<Arc<BiddingServiceClientAdapter>> {
        let bidding_service_client =
            BiddingServiceClientAdapter::new(&self.bidding_service_ipc_path, landed_blocks_history)
                .await
                .map_err(|e| eyre::Report::new(e).wrap_err("Unable to connect to remote bidder"))?;
        Ok(Arc::new(bidding_service_client))
    }

    /// Creates a new PrivateKeySigner and registers the associated address on key_registration_url
    async fn register_key(
        &self,
        key_registration_url: &str,
    ) -> Result<PrivateKeySigner, RegisterKeyError> {
        let signer = PrivateKeySigner::random();
        let client = reqwest::Client::new();
        let url = {
            let mut url = Url::parse(key_registration_url)?;
            url.set_path("/api/l1-builder/v1/register_credentials/rbuilder");
            url
        };
        let body = format!("{{ \"ecdsa_pubkey_address\": \"{}\" }}", signer.address());
        let res = client.post(url).body(body).send().await?;
        if res.status().is_success() {
            Ok(signer)
        } else {
            Err(RegisterKeyError::Service(res.status()))
        }
    }

    /// Depending on the cfg may create:
    /// - Dummy sink (no blocks_processor_url)
    /// - Standard block processor client
    /// - Secure block processor client (using block_processor_key to sign)
    fn create_block_processor_client(
        &self,
        block_processor_key: Option<PrivateKeySigner>,
    ) -> eyre::Result<Option<Box<dyn BidObserver + Send + Sync>>> {
        if let Some(url) = &self.blocks_processor_url {
            let bid_observer: Box<dyn BidObserver + Send + Sync> = if let Some(
                block_processor_key,
            ) = block_processor_key
            {
                let client = crate::signed_http_client::create_client(
                    url,
                    block_processor_key,
                    self.blocks_processor_max_request_size_bytes,
                    self.blocks_processor_max_concurrent_requests,
                )?;
                let block_processor =
                    BlocksProcessorClient::new(client, SIGNED_BLOCK_CONSUME_BUILT_BLOCK_METHOD);
                Box::new(BlocksProcessorClientBidObserver::new(block_processor))
            } else {
                eyre::bail!("Unsigned block processing is not supported: if blocks_processor_url then key_registration_url must be set");
            };
            Ok(Some(bid_observer))
        } else {
            if block_processor_key.is_some() {
                return Self::bail_blocks_processor_url_not_set();
            }
            Ok(None)
        }
    }

    fn bail_blocks_processor_url_not_set<T>() -> Result<T, eyre::Report> {
        eyre::bail!("blocks_processor_url should always be set if key_registration_url is set");
    }

    /// Depending on the cfg add a BlocksProcessorClientBidObserver and/or a true value pusher.
    async fn create_bid_observer(
        &self,
        cancellation_token: &CancellationToken,
    ) -> eyre::Result<Box<dyn BidObserver + Send + Sync>> {
        let block_processor_key = if let Some(key_registration_url) = &self.key_registration_url {
            if self.blocks_processor_url.is_none() {
                return Self::bail_blocks_processor_url_not_set();
            }
            Some(self.register_key(key_registration_url).await?)
        } else {
            None
        };

        let bid_observer = RbuilderOperatorBidObserver {
            block_processor: self.create_block_processor_client(block_processor_key.clone())?,
            tbv_pusher: self.create_tbv_pusher(block_processor_key, cancellation_token)?,
        };
        Ok(Box::new(bid_observer))
    }

    fn create_tbv_pusher(
        &self,
        block_processor_key: Option<PrivateKeySigner>,
        cancellation_token: &CancellationToken,
    ) -> eyre::Result<Option<Box<dyn BidObserver + Send + Sync>>> {
        // Avoid sending TBV is we are not on buildernet
        if self.key_registration_url.is_none() {
            return Ok(None);
        }

        if let Some(block_processor_key) = block_processor_key {
            if let Some(blocks_processor_url) = &self.blocks_processor_url {
                Ok(Some(Box::new(BestTrueValueObserver::new_block_processor(
                    blocks_processor_url.clone(),
                    block_processor_key,
                    self.blocks_processor_max_concurrent_requests,
                    cancellation_token.clone(),
                )?)))
            } else {
                Self::bail_blocks_processor_url_not_set()
            }
        } else if let Some(cfg) = &self.tbv_push_redis {
            let tbv_push_redis_url_value = cfg
                .url
                .as_ref()
                .ok_or(eyre::Report::msg("Missing tbv_push_redis_url"))?
                .value()
                .context("tbv_push_redis_url")?;
            Ok(Some(Box::new(BestTrueValueObserver::new_redis(
                tbv_push_redis_url_value,
                cfg.channel.clone(),
                cancellation_token.clone(),
            )?)))
        } else {
            Ok(None)
        }
    }

    fn live_builders(&self) -> eyre::Result<Vec<BuilderConfig>> {
        self.base_config
            .live_builders
            .iter()
            .map(|cfg_name| self.builder(cfg_name))
            .collect()
    }

    fn builder(&self, name: &str) -> eyre::Result<BuilderConfig> {
        self.builders
            .iter()
            .find(|b| b.name == name)
            .cloned()
            .ok_or_else(|| eyre::eyre!("Builder {} not found in builders list", name))
    }

    pub fn clickhouse_client(&self) -> eyre::Result<Option<Client>> {
        let host_url = if let Some(host) = &self.clickhouse.clickhouse_host_url {
            host.value()?
        } else {
            return Ok(None);
        };
        let user = self
            .clickhouse
            .clickhouse_user
            .as_ref()
            .ok_or(eyre::eyre!("clickhouse_user not found"))?
            .value()?;
        let password = self
            .clickhouse
            .clickhouse_password
            .as_ref()
            .ok_or(eyre::eyre!("clickhouse_password not found"))?
            .value()?;

        let client = Client::default()
            .with_url(host_url)
            .with_user(user)
            .with_password(password);
        Ok(Some(client))
    }
}

pub fn default_blocks_processor_max_concurrent_requests() -> usize {
    1024
}

pub fn default_blocks_processor_max_request_size_bytes() -> u32 {
    31457280 // 30MB
}

#[derive(Debug)]
struct RbuilderOperatorBidObserver {
    block_processor: Option<Box<dyn BidObserver + Send + Sync>>,
    tbv_pusher: Option<Box<dyn BidObserver + Send + Sync>>,
}

impl BidObserver for RbuilderOperatorBidObserver {
    fn block_submitted(
        &self,
        slot_data: &MevBoostSlotData,
        sealed_block: &SealedBlock,
        submit_block_request: &SubmitBlockRequest,
        built_block_trace: &BuiltBlockTrace,
        builder_name: String,
        best_bid_value: U256,
    ) {
        if let Some(p) = self.block_processor.as_ref() {
            p.block_submitted(
                slot_data,
                sealed_block,
                submit_block_request,
                built_block_trace,
                builder_name.clone(),
                best_bid_value,
            )
        }
        if let Some(p) = self.tbv_pusher.as_ref() {
            p.block_submitted(
                slot_data,
                sealed_block,
                submit_block_request,
                built_block_trace,
                builder_name,
                best_bid_value,
            )
        }
    }
}

#[cfg(test)]
mod test {
    use rbuilder::live_builder::base_config::load_config_toml_and_env;

    use super::*;
    use std::{env, path::PathBuf};

    #[test]
    fn test_default_config() {
        let config: FlashbotsConfig = serde_json::from_str("{}").unwrap();
        let config_default = FlashbotsConfig::default();

        assert_eq!(config, config_default);
    }

    #[test]
    fn test_parse_example_config() {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("config-live-example.toml");

        load_config_toml_and_env::<FlashbotsConfig>(p.clone()).expect("Config load");
    }
}
