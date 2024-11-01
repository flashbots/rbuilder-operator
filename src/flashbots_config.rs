//! Config should always be deserializable, default values should be used
//! This code has lots of copy/paste from the example config but it's not really copy/paste since we use our own private types.
//! @Pending make this copy/paste generic code on the library
use alloy_signer_local::PrivateKeySigner;
use eyre::Context;
use http::StatusCode;
use jsonrpsee::RpcModule;
use rbuilder::building::builders::merging_builder::merging_build_backtest;
use rbuilder::building::builders::UnfinishedBlockBuildingSinkFactory;
use rbuilder::live_builder::base_config::EnvOrValue;
use rbuilder::live_builder::block_output::bid_observer::{BidObserver, NullBidObserver};
use rbuilder::live_builder::block_output::bid_value_source::interfaces::BidValueSource;
use rbuilder::live_builder::block_output::bid_value_source::null_bid_value_source::NullBidValueSource;
use rbuilder::live_builder::block_output::bidding::interfaces::{
    BiddingService, BiddingServiceWinControl, LandedBlockInfo,
};
use rbuilder::live_builder::block_output::bidding::wallet_balance_watcher::WalletBalanceWatcher;
use rbuilder::live_builder::block_output::block_sealing_bidder_factory::BlockSealingBidderFactory;
use rbuilder::live_builder::config::{
    create_builders, BuilderConfig, SpecificBuilderConfig, WALLET_INIT_HISTORY_SIZE,
};
use rbuilder::live_builder::watchdog::spawn_watchdog_thread;
use rbuilder::primitives::mev_boost::MevBoostRelay;
use rbuilder::{
    building::builders::{BacktestSimulateBlockInput, Block},
    live_builder::{
        base_config::BaseConfig, cli::LiveBuilderConfig, config::L1Config,
        payload_events::MevBoostSlotDataGenerator, LiveBuilder,
    },
    utils::build_info::Version,
};
use reth::payload::database::CachedReads;
use reth::providers::ProviderFactory;
use reth_db::DatabaseEnv;
use serde::Deserialize;
use serde_with::serde_as;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};
use url::Url;

use crate::best_bid_ws::BestBidWSConnector;
use crate::bidding_service_wrapper::client::bidding_service_client_adapter::BiddingServiceClientAdapter;
use crate::block_descriptor_bidding::bidding_service_adapter::BiddingServiceAdapter;
use crate::blocks_processor::{
    BlocksProcessorClient, BlocksProcessorClientBidObserver,
    SIGNED_BLOCK_CONSUME_BUILT_BLOCK_METHOD,
};
use crate::build_info::rbuilder_version;
use crate::true_block_value_push::unfinished_block_building_sink_factory_wrapper::UnfinishedBlockBuildingSinkFactoryWrapper;

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
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct FlashbotsConfig {
    #[serde(flatten)]
    pub base_config: BaseConfig,

    #[serde(flatten)]
    pub l1_config: L1Config,

    #[serde(flatten)]
    clickhouse: ClickhouseConfig,

    #[serde(default)]
    pub flashbots_builder_pubkeys: Vec<String>,

    /// ws stream url for getting best relay bid info
    pub top_bid_ws_url: Option<EnvOrValue<String>>,
    /// ws stream url authentication
    pub top_bid_ws_basic_auth: Option<EnvOrValue<String>>,

    // bidding server ipc path config.
    bidding_service_ipc_path: String,

    /// selected builder configurations
    pub builders: Vec<BuilderConfig>,

    /// If this is Some then blocks_processor_url MUST be some and:
    /// - signed mode is used for blocks_processor.
    /// - tbv_push is done via blocks_processor_url (signed block-processor also handles flashbots_reportBestTrueValue).
    pub key_registration_url: Option<String>,

    pub blocks_processor_url: Option<String>,

    /// Cfg to push tbv to redis.
    /// For production we always need some tbv push (since it's used by smart-multiplexing.) so:
    /// !Some(key_registration_url) => Some(tbv_push_redis)
    tbv_push_redis: Option<TBVPushRedisConfig>,
}

impl LiveBuilderConfig for FlashbotsConfig {
    fn base_config(&self) -> &BaseConfig {
        &self.base_config
    }

    async fn create_builder(
        &self,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<LiveBuilder<Arc<DatabaseEnv>, MevBoostSlotDataGenerator>> {
        let provider_factory = self.base_config.provider_factory()?;

        let (sink_factory, relays, bidding_service_win_control) = self
            .create_sink_factory_and_relays(
                provider_factory.provider_factory_unchecked(),
                cancellation_token.clone(),
            )
            .await?;

        let payload_event = MevBoostSlotDataGenerator::new(
            self.l1_config.beacon_clients()?,
            relays,
            self.base_config.blocklist()?,
            cancellation_token.clone(),
        );

        let mut res = self
            .base_config
            .create_builder_with_provider_factory(
                cancellation_token.clone(),
                sink_factory,
                payload_event,
                provider_factory,
            )
            .await?;

        let mut module = RpcModule::new(());
        module.register_async_method("bid_subsidiseBlock", move |params, _| {
            handle_subsidise_block(bidding_service_win_control.clone(), params)
        })?;
        res = res.with_extra_rpc(module);
        let root_hash_config = self.base_config.live_root_hash_config()?;
        let root_hash_task_pool = self.base_config.root_hash_task_pool()?;
        let builders = create_builders(
            self.live_builders()?,
            root_hash_config,
            root_hash_task_pool,
            self.base_config.sbundle_mergeabe_signers(),
        );
        res = res.with_builders(builders);
        Ok(res)
    }

    fn version_for_telemetry(&self) -> Version {
        rbuilder_version()
    }

    /// @Pending fix this ugly copy/paste
    fn build_backtest_block(
        &self,
        building_algorithm_name: &str,
        input: BacktestSimulateBlockInput<'_, Arc<DatabaseEnv>>,
    ) -> eyre::Result<(Block, CachedReads)> {
        let builder_cfg = self.builder(building_algorithm_name)?;
        match builder_cfg.builder {
            SpecificBuilderConfig::OrderingBuilder(config) => {
                rbuilder::building::builders::ordering_builder::backtest_simulate_block(
                    config, input,
                )
            }
            SpecificBuilderConfig::MergingBuilder(config) => merging_build_backtest(input, config),
        }
    }
}

async fn handle_subsidise_block(
    win_control: Arc<dyn BiddingServiceWinControl>,
    params: jsonrpsee::types::Params<'static>,
) {
    match params.one() {
        Ok(block_number) => win_control.must_win_block(block_number),
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
        _cancellation_token: CancellationToken,
    ) -> eyre::Result<Box<dyn BiddingService>> {
        let client = Box::new(
            BiddingServiceClientAdapter::new(&self.bidding_service_ipc_path, landed_blocks_history)
                .await
                .map_err(|e| eyre::Report::new(e).wrap_err("Unable to connect to remote bidder"))?,
        );
        Ok(Box::new(BiddingServiceAdapter::new(client)))
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
    ) -> eyre::Result<Box<dyn BidObserver + Send + Sync>> {
        let bid_observer: Box<dyn BidObserver + Send + Sync> = if let Some(url) =
            &self.blocks_processor_url
        {
            if let Some(block_processor_key) = block_processor_key {
                let client = crate::signed_http_client::create_client(url, block_processor_key)?;
                let block_processor =
                    BlocksProcessorClient::new(client, SIGNED_BLOCK_CONSUME_BUILT_BLOCK_METHOD);
                Box::new(BlocksProcessorClientBidObserver::new(block_processor))
            } else {
                let client = BlocksProcessorClient::try_from(url)?;
                Box::new(BlocksProcessorClientBidObserver::new(client))
            }
        } else {
            if block_processor_key.is_some() {
                return Self::bail_blocks_processor_url_not_set();
            }
            Box::new(NullBidObserver {})
        };
        Ok(bid_observer)
    }

    fn bail_blocks_processor_url_not_set<T>() -> Result<T, eyre::Report> {
        eyre::bail!("blocks_processor_url should always be set if key_registration_url is set");
    }

    /// Connects (UnfinishedBlockBuildingSinkFactoryWrapper->BlockSealingBidderFactory)->RelaySubmitSinkFactory
    /// RelaySubmitSinkFactory: submits final blocks to relays
    /// BlockSealingBidderFactory: performs sealing/bidding. Sends bids to the RelaySubmitSinkFactory
    /// UnfinishedBlockBuildingSinkFactoryWrapper: sends all the tbv info via redis and forwards to BlockSealingBidderFactory
    #[allow(clippy::type_complexity)]
    async fn create_sink_factory_and_relays(
        &self,
        provider_factory: ProviderFactory<Arc<DatabaseEnv>>,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<(
        Box<dyn UnfinishedBlockBuildingSinkFactory>,
        Vec<MevBoostRelay>,
        Arc<dyn BiddingServiceWinControl>,
    )> {
        let block_processor_key = if let Some(key_registration_url) = &self.key_registration_url {
            if self.blocks_processor_url.is_none() {
                return Self::bail_blocks_processor_url_not_set();
            }
            Some(self.register_key(key_registration_url).await?)
        } else {
            None
        };
        // RelaySubmitSinkFactory
        let bid_observer = self.create_block_processor_client(block_processor_key.clone())?;
        let (sink_sealed_factory, relays) = self
            .l1_config
            .create_relays_sealed_sink_factory(self.base_config.chain_spec()?, bid_observer)?;

        // BlockSealingBidderFactory
        let (wallet_balance_watcher, wallet_history) = WalletBalanceWatcher::new(
            provider_factory,
            self.base_config.coinbase_signer()?.address,
            WALLET_INIT_HISTORY_SIZE,
        )?;
        let bid_value_source = self.create_bid_value_source(cancellation_token.clone())?;
        let bidding_service: Box<dyn BiddingService> = self
            .create_bidding_service(&wallet_history, cancellation_token.clone())
            .await?;
        let bidding_service_win_control = bidding_service.win_control();

        let sink_factory = Box::new(BlockSealingBidderFactory::new(
            bidding_service,
            sink_sealed_factory,
            bid_value_source.clone(),
            wallet_balance_watcher,
            self.l1_config.max_concurrent_seals as usize,
        ));

        // UnfinishedBlockBuildingSinkFactoryWrapper
        let wrapped_sink_factory = self.wrap_with_tbv_pusher(
            &cancellation_token,
            &bid_value_source,
            block_processor_key,
            sink_factory,
        )?;

        Ok((wrapped_sink_factory, relays, bidding_service_win_control))
    }

    /// Wraps the factory with one that sends a to TBV stream to our infra.
    /// block_processor_key == Some -> We use signed block processor API.
    /// block_processor_key == None -> If Some(tbv_push_redis) -> We send directly to a redis channel.
    fn wrap_with_tbv_pusher(
        &self,
        cancellation_token: &CancellationToken,
        bid_value_source: &Arc<dyn BidValueSource + Send + Sync>,
        block_processor_key: Option<PrivateKeySigner>,
        factory: Box<dyn UnfinishedBlockBuildingSinkFactory>,
    ) -> eyre::Result<Box<dyn UnfinishedBlockBuildingSinkFactory>> {
        if let Some(block_processor_key) = block_processor_key {
            if let Some(blocks_processor_url) = &self.blocks_processor_url {
                Ok(Box::new(
                    UnfinishedBlockBuildingSinkFactoryWrapper::new_block_processor(
                        factory,
                        bid_value_source.clone(),
                        blocks_processor_url.clone(),
                        block_processor_key,
                        cancellation_token.clone(),
                    )?,
                ))
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
            Ok(Box::new(
                UnfinishedBlockBuildingSinkFactoryWrapper::new_redis(
                    factory,
                    bid_value_source.clone(),
                    tbv_push_redis_url_value,
                    cfg.channel.clone(),
                    cancellation_token.clone(),
                )?,
            ))
        } else {
            Ok(factory)
        }
    }

    fn create_bid_value_source(
        &self,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<Arc<dyn BidValueSource + Send + Sync>> {
        if let (Some(top_bid_ws_url), Some(top_bid_ws_basic_auth)) =
            (&self.top_bid_ws_url, &self.top_bid_ws_basic_auth)
        {
            let ws_url = top_bid_ws_url.value().context("top_bid_ws_url")?;
            let ws_auth = top_bid_ws_basic_auth
                .value()
                .context("top_bid_ws_basic_auth")?;
            let connector = Arc::new(BestBidWSConnector::new(&ws_url, &ws_auth)?);
            let connector_clone = connector.clone();

            let watch_dog_sender = spawn_watchdog_thread(self.base_config().watchdog_timeout())?;
            std::thread::spawn(move || {
                connector_clone.run_ws_stream(watch_dog_sender, cancellation_token);
            });
            Ok(connector)
        } else {
            error!("No BidValueSource configured, using NullBidValueSource");
            Ok(Arc::new(NullBidValueSource {}))
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
