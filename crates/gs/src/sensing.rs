// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Sensing and perception primitives for cluster and daemon states.

use std::path::PathBuf;

/// Locations checked for the dormancy / pause flag.
#[must_use]
pub fn pause_flag_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(p) = std::env::var_os("REVENANT_PAUSE_FLAG") {
        paths.push(PathBuf::from(p));
    }
    if let Some(dir) = std::env::var_os("REVENANT_STATE_DIR") {
        paths.push(PathBuf::from(dir).join("PAUSED"));
    }
    if let Some(xdg) = std::env::var_os("XDG_STATE_HOME") {
        paths.push(PathBuf::from(xdg).join("revenant").join("PAUSED"));
    } else if let Some(home) = std::env::var_os("HOME") {
        paths.push(PathBuf::from(home).join(".local/state/revenant/PAUSED"));
    }
    paths.push(PathBuf::from("memory/REVENANT_PAUSED"));

    paths
}

/// Checks if cluster mutation is paused (dormant mode).
#[must_use]
pub fn is_paused() -> bool {
    pause_flag_paths().iter().any(|p| p.exists())
}

/// Probes a TCP port on localhost with a brief timeout.
#[must_use]
pub fn probe_tcp_port(port: u16) -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok()
}
