# Heavily inspired by Lighthouse: https://github.com/sigp/lighthouse/blob/stable/Makefile
# and Reth: https://github.com/paradigmxyz/reth/blob/main/Makefile
.DEFAULT_GOAL := help

GIT_VER ?= $(shell git describe --tags --always --dirty="-dev")
GIT_TAG ?= $(shell git describe --tags --abbrev=0)

# Deb package supported targets
DEB_SUPPORTED_TARGETS = x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

# Set the target for the build, default to x86_64
TARGET ?= x86_64-unknown-linux-gnu

# Profile for builds, default to release
PROFILE ?= release

##@ Help

.PHONY: help
help: ## Display this help.
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.PHONY: v
v: ## Show the current version
	@echo "Version: ${GIT_VER}"

##@ Build

.PHONY: clean
clean: ## Clean up
	cargo clean

.PHONY: build
build: ## Build static binary for x86_64
	cargo build --profile $(PROFILE) --target $(TARGET)

build-x86_64-unknown-linux-gnu: ## Build for x86_64 Linux
	$(MAKE) build TARGET=x86_64-unknown-linux-gnu

build-aarch64-unknown-linux-gnu: ## Build for aarch64 Linux
	cargo install cross --git https://github.com/cross-rs/cross
	cross build --profile $(PROFILE) --target aarch64-unknown-linux-gnu

.PHONY: build-reproducible
build-reproducible: ## Build reproducible static binary for x86_64
	@if [ "$(TARGET)" != "x86_64-unknown-linux-gnu" ]; then \
		echo "Error: Reproducible builds are only supported for x86_64-unknown-linux-gnu, not $(TARGET)"; \
		exit 1; \
	fi
	SOURCE_DATE_EPOCH=$(shell git log -1 --pretty=%ct) \
	RUSTFLAGS="-C link-arg=-Wl,--build-id=none -C metadata='' --remap-path-prefix $$(pwd)=." \
	CARGO_INCREMENTAL=0 \
	LC_ALL=C \
	TZ=UTC \
	cargo build --bin rbuilder --profile "reproducible" --locked --target x86_64-unknown-linux-gnu

build-deb-%: ## Build debian package for supported targets
	@case "$*" in \
		x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu) \
			echo "Building debian package for $*"; \
			;; \
		*) \
			echo "Error: Debian packages are only supported for $(DEB_SUPPORTED_TARGETS), not $*"; \
			exit 1; \
			;; \
	esac
	cargo install cargo-deb@3.6.0 --locked
	cargo deb --profile $(PROFILE) --no-build --no-dbgsym --no-strip \
		--target $* \
		$(if $(VERSION),--deb-version "1~$(VERSION)") \
		$(if $(VERSION),--output "target/$*/$(PROFILE)/rbuilder-operator-$(VERSION)-$*-$(PROFILE).deb")

.PHONY: build-deb-x86_64
build-deb-x86_64: ## Build debian package for x86_64
	$(MAKE) build-deb-x86_64-unknown-linux-gnu

.PHONY: build-deb-aarch64
build-deb-aarch64: ## Build debian package for aarch64
	$(MAKE) build-deb-aarch64-unknown-linux-gnu

.PHONY: docker-image
docker-image: ## Build a rbuilder Docker image
	docker build --platform linux/amd64 . -t rbuilder

##@ Dev

.PHONY: lint
lint: ## Run the linters
	cargo fmt -- --check
	cargo clippy -- -D warnings

.PHONY: test
test: ## Run the tests
	cargo test --verbose

.PHONY: lt
lt: lint test ## Run "lint" and "test"

fmt: ## Format the code
	cargo fmt
	cargo fix --allow-staged
	cargo clippy --fix --allow-staged
