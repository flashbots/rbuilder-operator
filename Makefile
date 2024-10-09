# Heavily inspired by Lighthouse: https://github.com/sigp/lighthouse/blob/stable/Makefile
# and Reth: https://github.com/paradigmxyz/reth/blob/main/Makefile
.DEFAULT_GOAL := help

GIT_VER ?= $(shell git describe --tags --always --dirty="-dev")
GIT_TAG ?= $(shell git describe --tags --abbrev=0)

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
build: ## Build (debug version)
	cargo build

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
