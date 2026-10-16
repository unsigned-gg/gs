# Makefile for gs (Godseat — Estate Control Surface)
# Host Throttling: Physical host ceres runs multiple active agent sessions.
# Shared workstations keep JOBS=2 to prevent CPU starvation and OOM.

SHELL := bash
.SHELLFLAGS := -euo pipefail -c
.DEFAULT_GOAL := help

JOBS ?= 2
export JOBS

.PHONY: help default xwork check hygiene graph-audit build fmt fmt-check clippy test doc-test deny smoke ci

##@ General

help: ## Show this help message
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z0-9_-]+:.*?##/ { printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

default: help

##@ Verification & CI Gates (cargo xtask)

xwork: ## Run full verification DAG via cargo xwork
	cargo xwork

check: ## Run verification pipeline with break-check
	cargo xtask check --break-check

ci: ## Run full CI verification suite
	cargo xwork

hygiene: ## Verify estate configuration files and absence of Justfile
	cargo xtask hygiene

graph-audit: ## Verify workspace DAG and Wave-0 gs-domain purity
	cargo xtask graph-audit --break-check

##@ Build & Smoke

build: ## Compile all workspace crates (-j 2)
	cargo build --workspace --locked -j $(JOBS)

smoke: ## Build and smoke-test the gs CLI (-j 2)
	cargo xtask smoke

##@ Code Quality & Supply Chain

fmt: ## Format all workspace Rust code
	cargo fmt --all

fmt-check: ## Check formatting without modifying files
	cargo fmt --all -- --check

clippy: ## Lint workspace with clippy (-D warnings, -j 2)
	cargo clippy --workspace --all-targets --locked -j $(JOBS) -- -D warnings

test: ## Run unit and integration test suite (-j 2)
	cargo test --workspace --locked -j $(JOBS)

doc-test: ## Run documentation tests (-j 2)
	cargo test --doc --workspace --locked -j $(JOBS)

deny: ## Audit licenses, bans, and advisories via cargo-deny
	cargo deny check
