// SPDX-License-Identifier: Apache-2.0 OR MIT
//! # gs — Godseat Control Surface
//!
//! Perception before action, cryptographic gate tiers, and operator cluster perception.
//!
//! Re-exports all zero-I/O domain types from [`gs_domain`] alongside sensing and CLI tools.

#![forbid(unsafe_code)]

pub mod sensing;

pub use gs_domain::*;
pub use sensing::{is_paused, pause_flag_paths, probe_tcp_port};

/// Evaluates a gate tier for an action against a target using safe internal defaults.
#[must_use]
pub fn evaluate_gate(action: &str, target: &str) -> (GateTier, String) {
    let ctx = ActionContext::safe_internal(action, target);
    classify(&ctx)
}

/// Evaluates a gate tier for an action with WIMSE principal and RFC 8693 `act` delegation claims.
#[must_use]
pub fn evaluate_gate_with_principal(
    action: &str,
    target: &str,
    claims: &GatePrincipalClaims,
    expected_trust_domain: &str,
) -> (GateTier, String) {
    let ctx = ActionContext::safe_internal(action, target);
    classify_with_principal(&ctx, claims, expected_trust_domain)
}
