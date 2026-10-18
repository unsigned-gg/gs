# SPEC-0003: Principal Claims at the Policy Gate (retro spec)

## 1. Status & Metadata
- **Status:** Implemented (retro spec for PR 6, merged 2026-10-16 as `109b289`)
- **Target Crates:** `gs-domain` (`crates/gs-domain/src/gate.rs`), `gs` (`crates/gs/src/main.rs`)
- **Depends On:** [SPEC-0001-godseat-control-surface.md](SPEC-0001-godseat-control-surface.md)
- **Open defect:** SEC-179 (elevation from unverified claims; see section 5)

This spec records the behaviour that shipped. Where the PR 6 description and
the code differ, this spec follows the code.

## 2. Principals

`WimsePrincipalKind::parse` (`crates/gs-domain/src/gate.rs`) accepts three forms:

- `spiffe://<trust_domain>/agent/<team>/<agent_id>`: an autonomous agent.
- `spiffe://<trust_domain>/conclave/<mesh>/elf/<elf_id>`: a conclave elf.
- `user:<id>` or `operator:<id>`: a human operator.

Each segment must be non-empty and contain no whitespace, control character
or `:`. Any other form is an error. The PR 6 description names `wimse://` URIs;
the parser accepts `spiffe://` only.

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

## 4. Classification (`classify_with_principal`)

1. Claims that fail validation give `Never` ("the mask refuses").
2. The base tier comes from `classify(ctx)`. A base `Never` stays `Never`,
   whatever the principal or scopes.
3. A base `Red` or `Yellow` becomes `Green` when either:
   - the subject is an operator; or
   - an operator appears in the delegation chain and the claims carry the
     scope `gate:approve:<canonical action>` or `estate:admin`.

   Otherwise it is `Red`, and the reason names the missing delegation and
   scope.
4. A base `Green` stays `Green` whatever the scopes. The reason text says
   "verified WIMSE principal", but nothing is verified at this point. The PR 6
   description says a missing `gate:<action>` or `gate:*` scope downgrades
   `Green` to `Yellow`. That downgrade is not implemented.

The test `test_wimse_principal_and_rfc8693_delegation_at_gate` asserts rule 3.
It upgrades `secret-rotate` on `openbao/prod` from Red to Green, using an
operator in the chain and the approve scope.

## 5. Trust assumption (open defect)

`classify_with_principal` trusts the claims it is given and verifies no token.
`gs gate --principal --act --scope --trust-domain` builds the claims directly
from command-line flags. A caller can therefore obtain `Green` for a `Red`
action by passing an operator subject. Until SEC-179 is fixed:

- Do not use `gs gate` output as an authorization decision.
- Feed `classify_with_principal` claims only from verified tokens: OAuth2
  introspection or a verified JWT-SVID.
