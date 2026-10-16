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

/// Maximum allowed RFC 8693 `act` delegation chain depth at the Godseat policy gate.
pub const MAX_DELEGATION_DEPTH: usize = 8;

/// Parsed WIMSE (`draft-klrc-aiagent-auth-03`) or operator principal kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WimsePrincipalKind {
    /// Autonomous agent (`spiffe://<trust_domain>/agent/<team>/<agent_id>`).
    Agent {
        /// SPIFFE trust domain.
        trust_domain: String,
        /// Owning team or lane.
        team: String,
        /// Agent instance identifier.
        agent_id: String,
    },
    /// Conclave Elf (`spiffe://<trust_domain>/conclave/<mesh>/elf/<elf_id>`).
    ConclaveElf {
        /// SPIFFE trust domain.
        trust_domain: String,
        /// Mesh identifier.
        mesh: String,
        /// Elf identifier.
        elf_id: String,
    },
    /// Human operator principal (`user:<id>` or `operator:<id>`).
    Operator {
        /// Operator handle or identifier.
        id: String,
    },
}

impl WimsePrincipalKind {
    /// Parse a principal subject URI into a [`WimsePrincipalKind`].
    pub fn parse(subject: &str) -> Result<Self, String> {
        let subject = subject.trim();
        if subject.is_empty() {
            return Err("principal subject must not be empty".to_owned());
        }
        if let Some(id) = subject
            .strip_prefix("user:")
            .or_else(|| subject.strip_prefix("operator:"))
        {
            if id.is_empty() || id.contains('/') || id.chars().any(char::is_whitespace) {
                return Err(format!("invalid operator principal '{subject}'"));
            }
            return Ok(Self::Operator { id: id.to_owned() });
        }
        let rest = subject
            .strip_prefix("spiffe://")
            .ok_or_else(|| format!("unsupported principal scheme in '{subject}'"))?;
        let parts: Vec<&str> = rest.split('/').collect();
        match parts.as_slice() {
            [td, "agent", team, agent_id]
                if valid_segment(td) && valid_segment(team) && valid_segment(agent_id) =>
            {
                Ok(Self::Agent {
                    trust_domain: (*td).to_owned(),
                    team: (*team).to_owned(),
                    agent_id: (*agent_id).to_owned(),
                })
            }
            [td, "conclave", mesh, "elf", elf_id]
                if valid_segment(td) && valid_segment(mesh) && valid_segment(elf_id) =>
            {
                Ok(Self::ConclaveElf {
                    trust_domain: (*td).to_owned(),
                    mesh: (*mesh).to_owned(),
                    elf_id: (*elf_id).to_owned(),
                })
            }
            _ => Err(format!(
                "invalid WIMSE SPIFFE path in '{subject}' (expected /agent/<team>/<id> or /conclave/<mesh>/elf/<id>)"
            )),
        }
    }

    /// Returns the SPIFFE trust domain for workload principals, or `None` for operators.
    #[must_use]
    pub fn trust_domain(&self) -> Option<&str> {
        match self {
            Self::Agent { trust_domain, .. } | Self::ConclaveElf { trust_domain, .. } => {
                Some(trust_domain.as_str())
            }
            Self::Operator { .. } => None,
        }
    }

    /// Returns `true` if this principal represents a human operator.
    #[must_use]
    pub const fn is_operator(&self) -> bool {
        matches!(self, Self::Operator { .. })
    }
}

fn valid_segment(seg: &str) -> bool {
    !seg.is_empty()
        && !seg
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == ':')
}

/// Introspected OAuth2 / Ory Hydra and WIMSE (`draft-klrc-aiagent-auth-03`) principal claims
/// presented at the Godseat policy gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatePrincipalClaims {
    /// Primary subject (`sub`) of the WIMSE JWT-SVID or OAuth2 access token.
    pub subject: String,
    /// Ordered RFC 8693 `act` delegation chain (outermost actor first, root delegator last).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegation_chain: Vec<String>,
    /// Granted OAuth2 / Hydra scopes (`scope` claim split on whitespace).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

impl GatePrincipalClaims {
    /// Construct claims for a subject with no delegation chain or scopes.
    #[must_use]
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            delegation_chain: Vec::new(),
            scopes: Vec::new(),
        }
    }

    /// Validate the primary subject and every actor in the RFC 8693 `act` chain
    /// against `expected_trust_domain`.
    pub fn validate(&self, expected_trust_domain: &str) -> Result<WimsePrincipalKind, String> {
        if self.delegation_chain.len() > MAX_DELEGATION_DEPTH {
            return Err(format!(
                "RFC 8693 act delegation chain depth {} exceeds maximum {}",
                self.delegation_chain.len(),
                MAX_DELEGATION_DEPTH
            ));
        }
        let subject_kind = WimsePrincipalKind::parse(&self.subject)?;
        if let Some(td) = subject_kind.trust_domain() {
            if td != expected_trust_domain {
                return Err(format!(
                    "subject trust domain '{td}' does not match '{expected_trust_domain}'"
                ));
            }
        }
        for (idx, actor) in self.delegation_chain.iter().enumerate() {
            if actor == &self.subject {
                return Err(format!(
                    "subject '{}' cannot appear in its own RFC 8693 act delegation chain",
                    self.subject
                ));
            }
            if self.delegation_chain[..idx].contains(actor) {
                return Err(format!(
                    "duplicate actor '{actor}' in RFC 8693 act delegation chain"
                ));
            }
            let actor_kind = WimsePrincipalKind::parse(actor)?;
            if let Some(td) = actor_kind.trust_domain() {
                if td != expected_trust_domain {
                    return Err(format!(
                        "delegation actor trust domain '{td}' does not match '{expected_trust_domain}'"
                    ));
                }
            }
        }
        Ok(subject_kind)
    }

    /// Returns `true` if any actor in the RFC 8693 `act` chain is a human operator.
    #[must_use]
    pub fn has_operator_delegator(&self) -> bool {
        self.delegation_chain.iter().any(|actor| {
            WimsePrincipalKind::parse(actor)
                .map(|k| k.is_operator())
                .unwrap_or(false)
        })
    }

    /// Returns `true` if the token carries `scope`.
    #[must_use]
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

/// Classifies an action with WIMSE (`draft-klrc-aiagent-auth-03`) principal verification
/// and RFC 8693 `act` delegation chain enforcement.
///
/// - Invalid principals, foreign SPIFFE trust domains, or cyclic/over-depth `act` chains
///   are immediately sealed (`GateTier::Never`).
/// - Actions classified as `GateTier::Red` may be satisfied by an agent only when its
///   RFC 8693 `act` chain includes a verified human operator delegator (`user:<id>` or
///   `operator:<id>`) and the explicit approval scope `gate:approve:<canonical-action>`
///   (or `estate:admin`).
#[must_use]
pub fn classify_with_principal(
    ctx: &ActionContext,
    claims: &GatePrincipalClaims,
    expected_trust_domain: &str,
) -> (GateTier, String) {
    let subject_kind = match claims.validate(expected_trust_domain) {
        Ok(kind) => kind,
        Err(err) => {
            return (
                GateTier::Never,
                format!("WIMSE / RFC 8693 principal validation failed ({err}) — the mask refuses"),
            );
        }
    };

    let (base_tier, base_reason) = classify(ctx);
    if base_tier == GateTier::Never {
        return (GateTier::Never, base_reason);
    }

    let action = canonical_action(&ctx.action);
    let approval_scope = format!("gate:approve:{action}");

    match base_tier {
        GateTier::Red | GateTier::Yellow => {
            if subject_kind.is_operator()
                || (claims.has_operator_delegator()
                    && (claims.has_scope(&approval_scope) || claims.has_scope("estate:admin")))
            {
                (
                    GateTier::Green,
                    format!(
                        "'{}' authorized via verified RFC 8693 operator delegation for '{}' — calm face",
                        ctx.action, claims.subject
                    ),
                )
            } else {
                (
                    GateTier::Red,
                    format!(
                        "{base_reason} (principal '{}' lacks operator act delegation and '{approval_scope}' scope)",
                        claims.subject
                    ),
                )
            }
        }
        GateTier::Green => (
            GateTier::Green,
            format!(
                "{base_reason} (verified WIMSE principal '{}')",
                claims.subject
            ),
        ),
        GateTier::Never => (GateTier::Never, base_reason),
    }
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

    #[test]
    fn test_wimse_principal_and_rfc8693_delegation_at_gate() {
        let red_ctx = ActionContext::safe_internal("secret-rotate", "openbao/prod");

        // Agent without operator delegation remains Red.
        let unapproved_agent = GatePrincipalClaims {
            subject: "spiffe://unsigned.gg/agent/estate/builder-01".to_owned(),
            delegation_chain: vec!["spiffe://unsigned.gg/agent/estate/coordinator-01".to_owned()],
            scopes: vec!["estate:admin".to_owned()],
        };
        let (tier, _) = classify_with_principal(&red_ctx, &unapproved_agent, "unsigned.gg");
        assert_eq!(tier, GateTier::Red);

        // Agent with operator in RFC 8693 act chain + explicit gate scope upgrades Red -> Green.
        let delegated_agent = GatePrincipalClaims {
            subject: "spiffe://unsigned.gg/agent/estate/builder-01".to_owned(),
            delegation_chain: vec![
                "spiffe://unsigned.gg/agent/estate/coordinator-01".to_owned(),
                "user:ctodie".to_owned(),
            ],
            scopes: vec!["gate:approve:secret-rotate".to_owned()],
        };
        let (tier, _) = classify_with_principal(&red_ctx, &delegated_agent, "unsigned.gg");
        assert_eq!(tier, GateTier::Green);

        // Sealed action remains Never even with operator delegation and admin scope.
        let sealed_ctx = ActionContext::safe_internal("force-push-main", "repo-gs");
        let (tier, _) = classify_with_principal(&sealed_ctx, &delegated_agent, "unsigned.gg");
        assert_eq!(tier, GateTier::Never);

        // Foreign trust domain or cyclic delegation is sealed (Never).
        let (tier, _) = classify_with_principal(&red_ctx, &delegated_agent, "other.domain");
        assert_eq!(tier, GateTier::Never);

        let cyclic = GatePrincipalClaims {
            subject: "spiffe://unsigned.gg/agent/estate/builder-01".to_owned(),
            delegation_chain: vec!["spiffe://unsigned.gg/agent/estate/builder-01".to_owned()],
            scopes: vec![],
        };
        let (tier, _) = classify_with_principal(&red_ctx, &cyclic, "unsigned.gg");
        assert_eq!(tier, GateTier::Never);
    }
}
