.PHONY: help build run test test-unit test-integration crd-install crd-uninstall

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build the controller binary
	cargo build

release: ## Build optimized release binary
	cargo build --release

run: ## Run the controller locally
	RUST_LOG=info cargo run

test-unit: ## Run unit tests
	cargo test --lib

test-integration: crd-install ## Run integration tests
	@echo "Make sure controller is running in another terminal!"
	@echo "Run: make run"
	@echo ""
	cargo test --test '*' -- --ignored --test-threads=1 --nocapture

test-namespace: crd-install ## Run only namespace tests
	cargo test --test namespace_tests -- --ignored --test-threads=1 --nocapture

test-deployment: crd-install ## Run only deployment tests
	cargo test --test deployment_tests -- --ignored --test-threads=1 --nocapture

test: test-unit ## Run all tests (use test-integration for integration tests)

crd-install: ## Install the CRD
	@echo "Generating and installing CRD..."
	@cargo run --bin crdgen > manifests/crd.yaml
	@kubectl apply -f manifests/crd.yaml
	@echo "CRD installed"

crd-uninstall: ## Uninstall the CRD
	@kubectl delete -f manifests/crd.yaml --ignore-not-found=true

clean: ## Clean build artifacts
	cargo clean

check: ## Run cargo check
	cargo check

fmt: ## Format code
	cargo fmt

lint: ## Run clippy
	cargo clippy -- -D warnings