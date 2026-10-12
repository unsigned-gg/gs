// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Policy gate tier classification engine (operating contract §3).
//!
//! Pure-Rust, zero-I/O leaf module holding gate tier classifications,
//! action contexts, canonical action spelling, and decision predicates.

use serde::{Deserialize, Serialize};

/// Gate classification result — the mask's expression.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum GateTier {
    /// 和面 wamen — calm face: autonomous, reversible, observable, internal, certain.
    Green,
    /// 怒眼 igan — angry eyes: uncertainty or multi-target, downgrades to Red.
    Yellow,
    /// 赤鬼 akaoni — red demon: operator sign-off required (irreversible, outward, live peer).
    Red,
    /// 封印 fūin — sealed mark: absolute block, no action ever.
    Never,
}

impl GateTier {
    /// The bare tier word — for audit fields and metric labels.
    #[must_use]
    pub const fn short(self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Red => "red",
            Self::Never => "never",
        }
    }

    /// Every tier in canonical order.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [Self::Green, Self::Yellow, Self::Red, Self::Never]
    }

    /// Returns true if autonomous execution is permitted.
    #[must_use]
    pub const fn is_autonomous(self) -> bool {
        matches!(self, Self::Green)
    }

    /// Returns true if the action is permanently prohibited.
    #[must_use]
    pub const fn is_sealed(self) -> bool {
        matches!(self, Self::Never)
    }
}

impl std::fmt::Display for GateTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Green => write!(f, "和面 wamen — GREEN"),
            Self::Yellow => write!(f, "怒眼 igan — YELLOW->RED"),
            Self::Red => write!(f, "赤鬼 akaoni — RED"),
            Self::Never => write!(f, "封印 fūin — NEVER"),
        }
    }
}

/// Classification criteria for an estate action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionContext {
    /// What the action does (e.g. "pause", "reap", "merge-to-main", "rotate-secret").
    pub action: String,
    /// Target of the action (session id, repo, key, pod, etc.).
    pub target: String,
    /// Is the action reversible?
    pub reversible: bool,
    /// Is it observable after execution?
    pub observable: bool,
    /// Is it internal-only (no outward-facing effect)?
    pub internal: bool,
    /// Is it single-target (not multi-target/mass-action)?
    pub single_target: bool,
    /// Is the ground truth certain (all sensors agree)?
    pub ground_truth_certain: bool,
    /// Does it touch a live peer's work?
    pub touches_live_peer: bool,
    /// Does it affect something outward-facing (external send, deploy)?
    pub outward_facing: bool,
}

impl ActionContext {
    /// Constructs a safe default internal action context.
    #[must_use]
    pub fn safe_internal(action: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            target: target.into(),
            reversible: true,
            observable: true,
            internal: true,
            single_target: true,
            ground_truth_certain: true,
            touches_live_peer: false,
            outward_facing: false,
        }
    }
}

/// Normalizes an action name into a canonical hyphen-separated lowercase identifier.
///
/// Folds ASCII case and collapses runs of non-alphanumeric characters into a single `-`.
#[must_use]
pub fn canonical_action(action: &str) -> String {
    let mut out = String::with_capacity(action.len());
    let mut pending_sep = false;
    for c in action.trim().chars() {
        if c.is_ascii_alphanumeric() {
            if pending_sep && !out.is_empty() {
                out.push('-');
            }
            pending_sep = false;
            out.push(c.to_ascii_lowercase());
        } else {
            pending_sep = true;
        }
    }
    out
}

/// Classifies an action per the 4-tier policy gate lattice.
/// Returns the tier and reasoning.
#[must_use]
pub fn classify(ctx: &ActionContext) -> (GateTier, String) {
    let action = canonical_action(&ctx.action);

    // 1. 封印 fūin (sealed mark — absolute refusal)
    let never_actions = [
        "force-push-main",
        "commit-secrets",
        "print-secrets",
        "no-gpg-sign",
        "operator-direct-override",
        "silent-mass-action",
        "silent-truncation",
        "drop-database",
        "disable-audit",
    ];
    if never_actions.contains(&action.as_str()) {
        return (
            GateTier::Never,
            format!(
                "'{}' is on the sealed list — the mask refuses, ever",
                ctx.action
            ),
        );
    }

    // 2. Accelerated estate operations (merge, deploy, sync)
    let accelerated = ["merge-to-main", "deploy", "argocd-sync"];
    if accelerated.contains(&action.as_str()) {
        return if ctx.single_target && ctx.ground_truth_certain {
            (
                GateTier::Green,
                format!(
                    "'{}' accelerated: single-target + ground truth certain — calm face",
                    ctx.action
                ),
            )
        } else {
            (
                GateTier::Red,
                format!(
                    "'{}' accelerates only when single-target and certain — fangs bared",
                    ctx.action
                ),
            )
        };
    }

    // 3. Explicit 赤鬼 akaoni actions (operator sign-off mandatory)
    let red_actions = [
        "terraform-apply",
        "secret-rotate",
        "secret-delete",
        "external-send",
        "kill-session",
        "pause-session",
        "redirect-session",
        "control-action-other-session",
        "openbao-write",
        "kyverno-edit",
        "pause-agent",
        "resume-agent",
        "scale-runnerset",
        "kill-pod",
        "drain-node",
    ];
    if red_actions.contains(&action.as_str()) {
        return (
            GateTier::Red,
            format!("'{}' bares fangs — operator sign-off required", ctx.action),
        );
    }

    // 4. Structural bared fangs (irreversible, live peer, outward-facing)
    if ctx.outward_facing || ctx.touches_live_peer || !ctx.reversible {
        return (
            GateTier::Red,
            "outward-facing, touches a live peer, or irreversible — fangs bared".to_owned(),
        );
    }

    // 5. 怒眼 igan (uncertainty or multi-target — downgrade to red)
    if !ctx.single_target || !ctx.ground_truth_certain {
        return (
            GateTier::Yellow,
            "multi-target or ground truth uncertain — eyes narrow, downgrade to fangs".to_owned(),
        );
    }

    // 6. 和面 wamen (calm face — autonomous execution permitted)
    if ctx.reversible && ctx.observable && ctx.internal {
        return (
            GateTier::Green,
            format!(
                "'{}' calm face — autonomous execution permitted",
                ctx.action
            ),
        );
    }

    (
        GateTier::Red,
        "fails autonomous criteria — default to operator sign-off".to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_action_formatting() {
        assert_eq!(canonical_action("Print-Secrets"), "print-secrets");
        assert_eq!(canonical_action("print_secrets"), "print-secrets");
        assert_eq!(canonical_action("  no:::gpg...sign  "), "no-gpg-sign");
        assert_eq!(canonical_action("merge_to_main"), "merge-to-main");
    }

    #[test]
    fn test_sealed_actions_always_never() {
        let mut ctx = ActionContext::safe_internal("print_secrets", "session-1");
        let (tier, _) = classify(&ctx);
        assert_eq!(tier, GateTier::Never);

        ctx.action = "force-push-main".to_owned();
        let (tier, _) = classify(&ctx);
        assert_eq!(tier, GateTier::Never);
    }

    #[test]
    fn test_accelerated_actions_autonomous_when_certain() {
        let ctx = ActionContext::safe_internal("merge-to-main", "repo-kd");
        let (tier, _) = classify(&ctx);
        assert_eq!(tier, GateTier::Green);

        let mut uncertain = ctx;
        uncertain.ground_truth_certain = false;
        let (tier, _) = classify(&uncertain);
        assert_eq!(tier, GateTier::Red);
    }

    #[test]
    fn test_structural_red_actions() {
        let mut ctx = ActionContext::safe_internal("update-cache", "slot-1");
        ctx.touches_live_peer = true;
        let (tier, _) = classify(&ctx);
        assert_eq!(tier, GateTier::Red);

        ctx.touches_live_peer = false;
        ctx.reversible = false;
        let (tier, _) = classify(&ctx);
        assert_eq!(tier, GateTier::Red);
    }
}
