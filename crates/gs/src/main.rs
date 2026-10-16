// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Command-line interface for gs (Godseat Control Surface).

use std::env;
use std::process::ExitCode;

use gs::{
    DutyWindow, GatePrincipalClaims, IncidentRecord, evaluate_gate, evaluate_gate_with_principal,
    is_paused, probe_tcp_port,
};

fn print_usage() {
    eprintln!(
        "gs — Godseat Control Surface (v{})\n\
        \n\
        USAGE:\n\
            gs status                Sense estate daemons, pause flags, and health\n\
            gs gate <action> [target] [--principal <uri>] [--act <uri>...] [--scope <s >...] [--trust-domain <td >]\n\
                                     Classify an action against the 4-tier policy gate\n\
            gs incident demo         Demonstrate Article 73 duty clock lifecycle\n\
            gs --help                Print this help text\n\
            gs --version             Print version information",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        return ExitCode::FAILURE;
    }

    match args[0].as_str() {
        "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        "--version" | "-V" => {
            println!("gs {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "status" | "probe" => {
            let paused = is_paused();
            let reverie_up = probe_tcp_port(7437);
            let metrics_up = probe_tcp_port(9100);

            println!("Godseat Estate Perception:");
            println!(
                "  Dormancy / Paused: {}",
                if paused {
                    "YES (dormant)"
                } else {
                    "NO (active)"
                }
            );
            println!(
                "  Reverie Daemon (:7437): {}",
                if reverie_up { "ONLINE" } else { "OFFLINE" }
            );
            println!(
                "  Revenant Metrics (:9100): {}",
                if metrics_up { "ONLINE" } else { "OFFLINE" }
            );
            ExitCode::SUCCESS
        }
        "gate" => {
            if args.len() < 2 {
                eprintln!(
                    "error: 'gs gate' requires an action name (e.g. 'gs gate merge-to-main')"
                );
                return ExitCode::FAILURE;
            }
            let action = &args[1];
            let mut target = "cluster";
            let mut idx = 2;
            if let Some(arg) = args.get(2) {
                if !arg.starts_with("--") {
                    target = arg.as_str();
                    idx = 3;
                }
            }

            let mut principal: Option<String> = None;
            let mut delegation_chain: Vec<String> = Vec::new();
            let mut scopes: Vec<String> = Vec::new();
            let mut trust_domain = "unsigned.gg".to_owned();

            while idx < args.len() {
                match args[idx].as_str() {
                    "--principal" => {
                        let Some(val) = args.get(idx + 1) else {
                            eprintln!("error: '--principal' requires a value");
                            return ExitCode::FAILURE;
                        };
                        principal = Some(val.clone());
                        idx += 2;
                    }
                    "--act" => {
                        let Some(val) = args.get(idx + 1) else {
                            eprintln!("error: '--act' requires a value");
                            return ExitCode::FAILURE;
                        };
                        delegation_chain.push(val.clone());
                        idx += 2;
                    }
                    "--scope" => {
                        let Some(val) = args.get(idx + 1) else {
                            eprintln!("error: '--scope' requires a value");
                            return ExitCode::FAILURE;
                        };
                        scopes.push(val.clone());
                        idx += 2;
                    }
                    "--trust-domain" => {
                        let Some(val) = args.get(idx + 1) else {
                            eprintln!("error: '--trust-domain' requires a value");
                            return ExitCode::FAILURE;
                        };
                        trust_domain = val.clone();
                        idx += 2;
                    }
                    flag => {
                        eprintln!("error: unknown gate option '{flag}'");
                        return ExitCode::FAILURE;
                    }
                }
            }

            let (tier, reason) = if let Some(subject) = principal {
                let claims = GatePrincipalClaims {
                    subject,
                    delegation_chain,
                    scopes,
                };
                evaluate_gate_with_principal(action, target, &claims, &trust_domain)
            } else {
                evaluate_gate(action, target)
            };
            println!("Action: {action}");
            println!("Target: {target}");
            println!("Gate Tier: {tier}");
            println!("Reason: {reason}");
            ExitCode::SUCCESS
        }
        "incident" => {
            println!("Article 73 Incident Clock Lifecycle Demo:");
            let now = 1_700_000_000;
            let mut record = IncidentRecord::new(
                "INC-2026-001",
                "Synthetic frontier anomaly",
                DutyWindow::Widespread2d,
                now,
            );
            println!(
                "  1. State: {:?}, Duty Window: {:?}",
                record.state, record.duty_window
            );
            println!(
                "  2. Deadline before link: {:?}",
                record.deadline_unix_secs()
            );

            if let Err(err) = record.establish_link(now + 100) {
                eprintln!("gs: failed to establish link: {err}");
                return ExitCode::FAILURE;
            }
            println!("  3. Link established at: {}", now + 100);
            println!(
                "  4. Derived deadline: {}",
                record.deadline_unix_secs().unwrap_or_default()
            );
            println!(
                "  5. Is breached at now + 500s? {}",
                record.is_deadline_breached(now + 500)
            );
            println!(
                "  6. Is breached after 2 days? {}",
                record.is_deadline_breached(now + 100 + 2 * 86_400 + 1)
            );
            ExitCode::SUCCESS
        }
        unknown => {
            eprintln!("gs: unknown command '{unknown}'");
            print_usage();
            ExitCode::FAILURE
        }
    }
}
