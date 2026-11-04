// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `gs gate` builds principal claims from command-line flags. Nobody has
//! verified those, so they may lower a result but never raise it (SEC-179,
//! SPEC-0003 section 4).

#![allow(clippy::expect_used)]

use std::process::Command;

fn gate(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_gs"))
        .arg("gate")
        .args(args)
        .output()
        .expect("run gs");
    assert!(out.status.success(), "gs gate {args:?} failed: {out:?}");
    String::from_utf8(out.stdout).expect("utf-8 output")
}

fn tier(output: &str) -> &str {
    output
        .lines()
        .find_map(|l| l.strip_prefix("Gate Tier: "))
        .expect("a Gate Tier line")
}

#[test]
fn flags_never_turn_a_red_action_green() {
    let operator = "user:anyone";
    let agent = "spiffe://unsigned.gg/agent/estate/builder-01";
    for args in [
        vec!["secret-rotate", "openbao/prod", "--principal", operator],
        vec![
            "secret-rotate",
            "openbao/prod",
            "--principal",
            operator,
            "--scope",
            "estate:admin",
        ],
        vec![
            "secret-rotate",
            "openbao/prod",
            "--principal",
            agent,
            "--act",
            operator,
            "--scope",
            "gate:approve:secret-rotate",
        ],
        vec![
            "secret-rotate",
            "openbao/prod",
            "--principal",
            agent,
            "--act",
            "user:ctodie",
            "--scope",
            "estate:admin",
        ],
    ] {
        let out = gate(&args);
        assert!(tier(&out).contains("RED"), "{args:?} gave: {out}");
        assert!(!out.contains("GREEN"), "{args:?} gave: {out}");
    }
}

#[test]
fn a_foreign_trust_domain_is_still_sealed() {
    let out = gate(&[
        "secret-rotate",
        "openbao/prod",
        "--principal",
        "spiffe://other.example/agent/estate/builder-01",
    ]);
    assert!(tier(&out).contains("NEVER"), "{out}");
}
