# SPEC-0001: Godseat Control Surface & Policy Gate Tiers

## 1. Status & Metadata
- **Status:** Approved & Living
- **Authors:** Mimir <mimir@unsigned.gg>, Kratos <kratos@unsigned.gg>, Idunn <idunn@unsigned.gg>
- **Target Crate:** `gs` (`crates/gs`)
- **License:** Apache-2.0 OR MIT / CC0 1.0 Universal

---

## 2. Problem Statement
Autonomous agent swarms executing in production clusters must not possess unconstrained mutation authority. Irreversible actions—such as secret rotations, node draining, volume destruction, and production deployment—can cause cascading operational disruptions.

`gs` (Godseat) implements the fundamental principle of **Perception Before Action**: no agent may propose or execute a state mutation without first sensing, verifying, and classifying the action through an attested policy gate.

---

## 3. Operational Invariants & Gate Tiers

### Invariant 1: Four-Tier Gate Model
Every action $A$ submitted to the estate control plane is classified into exactly one tier:
$$\text{GateTier}(A) \in \{\text{Green}, \text{Yellow}, \text{Red}, \text{Never}\}$$

1. **Green (Autonomous):** Permitted to run autonomously when all invariant checks, CI pipelines, and jury thresholds are green. Blast radius is strictly confined.
2. **Yellow (Downgrade):** Permitted to execute only in degraded or read-only mode, or with strictly bounded concurrency ($N \le 2$).
3. **Red (Operator Sign-off):** Execution is halted until an explicit cryptographic signature or interactive CLI confirmation is provided by the human operator.
4. **Never (Sealed Refusal):** Permanently prohibited actions. No operator override, bypass flag, or token may authorize a Sealed action. Examples include:
   - Writing unencrypted secrets to disk or git
   - Force-pushing to `main`
   - Bypassing Linux Landlock LSM sandboxes in untrusted execution
   - Exfiltrating telemetry outside the local host

### Invariant 2: Perception Precedes Action
No mutation command may be queued without a fresh perception receipt ($t_{\text{probe}} < 30\text{s}$). Stale state invalidates proposed actions.

### Invariant 3: Statutory Duty Clocks (Article 73)
Security incidents follow strict statutory clocks:
$$\Delta t_{\text{report}} \le 72\text{ hours from link establishment}$$
Clocks are monotonic, persistent, and verifiable against the Reverie state ledger.
