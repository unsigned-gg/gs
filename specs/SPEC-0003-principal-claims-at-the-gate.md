# SPEC-0003: Principal Claims at the Policy Gate (retro spec)

## 1. Status & Metadata
- **Status:** Implemented (retro spec for PR 6, merged 2026-10-16 as `109b289`)
- **Target Crates:** `gs-domain` (`crates/gs-domain/src/gate.rs`), `gs` (`crates/gs/src/main.rs`)
- **Depends On:** [SPEC-0001-godseat-control-surface.md](SPEC-0001-godseat-control-surface.md)
- **Defect fixed:** SEC-179 (elevation from unverified claims; see section 5)

This spec records the behaviour that shipped, and the changes SEC-179 made to
it (sections 4 and 5). Where the PR 6 description and the code differ, this
spec follows the code.

## 2. Principals

`WimsePrincipalKind::parse` (`crates/gs-domain/src/gate.rs`) accepts three forms:

- `spiffe://<trust_domain>/agent/<team>/<agent_id>`: an autonomous agent.
- `spiffe://<trust_domain>/conclave/<mesh>/elf/<elf_id>`: a conclave elf.
- `user:<id>` or `operator:<id>`: a human operator.

Each segment must be non-empty and contain no whitespace, control character
or `:`. Any other form is an error. The PR 6 description names `wimse://` URIs; those
never parsed. The only workload form is `spiffe://`, and docs and CLI help say
so.

## 3. Claims and validation

`GatePrincipalClaims` holds:
- `subject`;
- an ordered RFC 8693 `act` delegation chain, outermost actor first;
- `scopes`.

`validate(expected_trust_domain)` fails when:
- the chain is longer than `MAX_DELEGATION_DEPTH` (8);
- the subject or any actor does not parse;
- a workload trust domain differs from the expected one;
- the subject appears in its own chain;
- an actor appears twice.

## 4. Classification

Two entry points share one rule set. Only the first can raise a tier.

- `classify_with_principal(ctx, &VerifiedPrincipalClaims, trust_domain)`: for
  claims a verifier has authenticated. `VerifiedPrincipalClaims::from_verified`
  takes the claims and the way they were verified (`Introspection` or
  `JwtSvid`). `gs-domain` does no I/O and cannot check a token, so that call is
  the caller's assertion; it belongs next to the verification and never on
  command-line input.
- `classify_with_claimed_principal(ctx, &GatePrincipalClaims, trust_domain)`:
  for claims nobody has verified, such as `gs gate --principal`. It never
  returns a tier above the one `classify(ctx)` gives.

Rules:

1. Claims that fail validation give `Never` ("the mask refuses").
2. The base tier comes from `classify(ctx)`. A base `Never` stays `Never`,
   whatever the principal or scopes.
3. A base `Red` or `Yellow` becomes `Green` only for verified claims, and only
   when either:
   - the subject is an operator; or
   - an operator appears in the delegation chain and the claims carry the
     scope `gate:approve:<canonical action>` or `estate:admin`.

   Otherwise it is `Red`. For unverified claims it is always `Red`, and the
   reason says unverified claims cannot raise a tier.
4. A base `Green` stays `Green` when the claims carry the scope
   `gate:<canonical action>` or `gate:*`. Without one it is `Yellow`. This is
   the downgrade the PR 6 description named; it applies to verified and
   unverified claims alike.

Tests: `gate::tests::test_wimse_principal_and_rfc8693_delegation_at_gate` (the
verified path), `unverified_claims_never_raise_a_tier`,
`green_without_a_gate_scope_is_yellow`, and `crates/gs/tests/gate_cli.rs` (the
flags never turn a Red action Green).

## 5. Trust assumption (SEC-179, fixed)

Before SEC-179, `classify_with_principal` trusted whatever claims it was given
and `gs gate --principal --act --scope --trust-domain` built them from
command-line flags, so `--principal user:anyone` turned a `Red` action `Green`.
The verified/unverified split above closes it: the CLI can only call the
unverified entry point, so its flags can lower a result but never raise it.

The CLI's output is still not an authorization decision: it has no verifier.
Feed `classify_with_principal` claims only from OAuth2 introspection or a
verified JWT-SVID.
