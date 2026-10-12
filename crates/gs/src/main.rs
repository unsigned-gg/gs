// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Command-line interface for gs (Godseat Control Surface).

use std::env;
use std::process::ExitCode;

use gs::{evaluate_gate, is_paused, probe_tcp_port, DutyWindow, IncidentRecord};

fn print_usage() {
    eprintln!(
        "gs — Godseat Control Surface (v{})\n\
        \n\
        USAGE:\n\
            gs status                Sense estate daemons, pause flags, and health\n\
            gs gate <action> [target] Classify an action against the 4-tier policy gate\n\
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
            let target = args.get(2).map(String::as_str).unwrap_or("cluster");

            let (tier, reason) = evaluate_gate(action, target);
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

            record.establish_link(now + 100).unwrap();
            println!("  3. Link established at: {}", now + 100);
            println!(
                "  4. Derived deadline: {}",
                record.deadline_unix_secs().unwrap()
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
