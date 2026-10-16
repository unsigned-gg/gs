// SPDX-License-Identifier: Apache-2.0 OR MIT
//! # gs-domain — Godseat Zero-I/O Domain Primitives
//!
//! Perception before action, cryptographic gate tiers, and Article 73 duty clocks.
//!
//! This crate contains pure data models and algebraic state transitions.
//! It depends on NO filesystem, networking, process execution, or async runtimes.

#![forbid(unsafe_code)]

pub mod gate;
pub mod incident;

pub use gate::{
    canonical_action, classify, classify_with_principal, ActionContext, GatePrincipalClaims,
    GateTier, WimsePrincipalKind, MAX_DELEGATION_DEPTH,
};
pub use incident::{DutyWindow, IncidentRecord, IncidentState};
