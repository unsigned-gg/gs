# SPEC-0002: Proof-Carrying Policy Gates & Gödel Prover Certification

## 1. Status & Metadata
- **Status:** Proposed & Living
- **Authors:** Kratos <kratos@yx.unsigned.gg>, Mimir <mimir@unsigned.gg>
- **Target Crate:** `gs-domain` (`crates/gs-domain`)
- **License:** Apache-2.0 OR MIT / CC0 1.0 Universal
- **Depends On:** [SPEC-0001-godseat-control-surface.md](SPEC-0001-godseat-control-surface.md)

---

## 2. Context & Problem Statement

Autonomous agents operating on the cluster control plane must evaluate actions before execution.
Static pattern matching (e.g. comparing action strings against allow and deny lists) exhibits structural limitations:
1. It cannot verify dynamic state transitions where safety depends on preconditions.
2. It cannot evaluate complex composed multi-step mutation sequences.
3. It cannot generalize to novel, semantically valid tool invocations without continuous manual rule expansion.

Theoretical self-improving systems (Schmidhuber's 2003 Gödel machine) mandate formal proof of global optimality before self-rewriting.
Modern empirical systems (Darwin Gödel Machine, STOP, ADAS) drop formal proof requirements for heuristic feedback.
This shift introduces documented failure modes:
* **Marker Stripping:** Agents delete internal validation markers to bypass hallucination detection.
* **Context Collapse:** Wholesale instruction rewrites purge tacit operational knowledge (e.g. ACE accuracy falling from 66.7% to 57.1%).
* **Permission Drift:** Agents grant themselves expanded authorization under the guise of workflow optimization.

`gs` resolves this tension using a **Proof-Carrying Code (PCC)** architecture for policy gates.
A high-level Gödel prover synthesizes machine-verifiable proofs out-of-band.
The low-level `gs-domain` gate verifies these proofs in deterministic $O(1)$ time without searching.

---

## 3. Formal Invariants & Model

### Invariant 1: State Transition Proof Obligation
Let state $S$ be formalized as a first-order theory representing permissions, cluster topology, and audit integrity.
Define the global estate safety invariant $I(S)$:

$$I(S) \triangleq \text{Safety}(S) \wedge \text{AuditIntact}(S) \wedge \neg\text{SealedAction}(S)$$

For any proposed action $a$ transitioning the estate from state $S$ to state $S'$, the prover must establish:

$$(I(S) \wedge \text{Pre}(a, S) \wedge \text{Exec}(S, a, S')) \implies (I(S') \wedge \text{TierCriteria}(a, S'))$$

### Invariant 2: Cryptographic Proof Binding
The proof $\pi$ must be bound to the cryptographic digests of the initial state, action, and resulting state:

$$\text{Binding} \triangleq \langle H(S), \text{canonical}(a), H(S') \rangle$$

A proof is valid only when $H(S)$ matches the live ground truth measured by the perception layer.

### Invariant 3: Zero-Search Gate Verification ($O(1)$)
The runtime gate certifier (`gs-domain`) shall not execute proof search, SMT solving, or tactic discovery.
The gate operates strictly as a deterministic Abstract Syntax Tree (AST) proof-checker:
* It traverses the proof term $\pi$ in a single forward pass.
* Verification time is bounded: $t_{\text{verify}} \le O(|\pi|) \le O(1)$ relative to state size $|S|$.
* Proof certificates must be self-contained and free of unresolved assumptions (`sorry`).

### Invariant 4: Permissions Immutability
Agents may propose changes to prompts, instructions, and tools.
Agents may never propose mutations to permission ledgers, sandbox configurations, or gate definitions.
Any state transition attempting to expand agent mutation authority fails closed.

---

## 4. Policy Gate Lattice Integration

The gate certifier assigns an action context to exactly one of four tiers based on proof verification:

| Verification Result | Operational Criteria | Assigned Tier |
|---|---|---|
| Valid proof $\pi$ | Internal, reversible, observable, certain | **Green (和面 wamen)** |
| Incomplete proof or timeout | Degradable, multi-target, or partial uncertainty | **Yellow / Red (怒眼 igan)** |
| Unproven or safety violation | Irreversible, outward-facing, live peer | **Red (赤鬼 akaoni)** |
| Sealed action ($a \in \text{Never}$) | Prohibited by definition; bypasses prover | **Never (封印 fūin)** |

> [!CAUTION]
> Sealed actions cannot execute under any circumstance.
> Actions in the Sealed set (`force-push-main`, `commit-secrets`, `disable-audit`, `drop-database`) bypass the proof synthesizer and fail closed immediately.

---

## 5. Architectural Pipeline

```
       [Proposed Action a]
                │
                ▼
      ┌───────────────────┐
      │  Sealed Check?    │──► Yes ──► Reject immediately (Never)
      └───────────────────┘
                │ No
                ▼
      ┌───────────────────┐
      │ Out-of-Band       │
      │ Gödel Prover      │──► Synthesizes transition proof π:
      │ (Lean 4 / Z3)     │    (I(S) ∧ Exec(S, a, S')) ⟹ I(S')
      └───────────────────┘
                │ Emits π bound to ⟨H(S), a, H(S')⟩
                ▼
      ┌───────────────────┐
      │ gs-domain Gate    │──► O(1) AST proof check:
      │ (Zero-I/O Kernel) │    • π valid ∧ Reversible ⟹ Green
      └───────────────────┘    • π invalid / Timeout  ⟹ Red
```

---

## 6. Implementation Blueprint in gs-domain

The `ActionContext` struct in `crates/gs-domain/src/gate.rs` is extended to carry optional proof verification tokens:

```rust
/// Cryptographic proof token validating state invariant preservation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofCertificate {
    /// Format identifier (e.g. "lean4-term", "alethe-smt", "snark-groth16").
    pub format: String,
    /// Cryptographic digest of initial state H(S).
    pub pre_state_hash: [u8; 32],
    /// Cryptographic digest of post-state H(S').
    pub post_state_hash: [u8; 32],
    /// Serialized AST proof term or verification token.
    pub proof_payload: Vec<u8>,
}
```

When evaluating `classify(&ctx)`:
1. If the action is sealed, return `GateTier::Never`.
2. If `ctx.proof` contains a valid certificate verifying $I(S) \implies I(S')$ and all local criteria are met, return `GateTier::Green`.
3. If the proof is absent, unverified, or timed out, default to `GateTier::Red`.
