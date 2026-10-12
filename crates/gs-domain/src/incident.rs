// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Serious incident register and Article 73 statutory duty clocks.
//!
//! Pure zero-I/O domain model representing incident lifecycles:
//! `Suspected` -> `LinkEstablished` -> `Reported` -> `Closed`.
//! Deadlines are derived from `link_established_at` + `DutyWindow::duration_secs()`.

use serde::{Deserialize, Serialize};

/// Regulatory duty windows under EU AI Act Article 73.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DutyWindow {
    /// Art. 73(2) default: 15 days from link establishment.
    Standard15d,
    /// Art. 73(3): death / grave harm — 10 days.
    Death10d,
    /// Art. 73(2) widespread infringement or critical infrastructure disruption: 2 days.
    Widespread2d,
}

impl DutyWindow {
    /// Duration of the duty window in seconds.
    #[must_use]
    pub const fn duration_secs(self) -> u64 {
        match self {
            Self::Standard15d => 15 * 86_400,
            Self::Death10d => 10 * 86_400,
            Self::Widespread2d => 2 * 86_400,
        }
    }

    /// Parses a string into a duty window identifier.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "standard" | "standard-15d" | "15d" => Some(Self::Standard15d),
            "death" | "death-10d" | "10d" => Some(Self::Death10d),
            "widespread" | "widespread-2d" | "2d" | "critical" => Some(Self::Widespread2d),
            _ => None,
        }
    }
}

/// Incident lifecycle stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IncidentState {
    /// Initial suspicion of a causal link between model output and harm.
    Suspected,
    /// Causal link established; statutory duty clock begins ticking immediately.
    LinkEstablished,
    /// Formal regulatory authority notification dispatched.
    Reported,
    /// Resolved, mitigated, or closed as false alarm.
    Closed,
}

impl IncidentState {
    /// Validates whether a state transition from `self` to `target` is permitted.
    #[must_use]
    pub const fn is_valid_transition(self, target: Self) -> bool {
        matches!(
            (self, target),
            // Forward progression
            (Self::Suspected, Self::LinkEstablished | Self::Closed)
                | (Self::LinkEstablished, Self::Reported | Self::Closed)
                | (Self::Reported, Self::Closed)
        )
    }
}

/// First-class serious incident record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentRecord {
    /// Incident identifier (e.g. "INC-2026-001").
    pub id: String,
    /// Descriptive title of the incident.
    pub title: String,
    /// Current lifecycle state.
    pub state: IncidentState,
    /// Assigned statutory duty window.
    pub duty_window: DutyWindow,
    /// Timestamp when incident was recorded (unix seconds).
    pub opened_at_secs: u64,
    /// Timestamp when causal link was established (unix seconds).
    pub link_established_at_secs: Option<u64>,
    /// Timestamp when reported to authorities (unix seconds).
    pub reported_at_secs: Option<u64>,
    /// Timestamp when closed (unix seconds).
    pub closed_at_secs: Option<u64>,
}

impl IncidentRecord {
    /// Creates a new incident record in `Suspected` state.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        duty_window: DutyWindow,
        now_secs: u64,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            state: IncidentState::Suspected,
            duty_window,
            opened_at_secs: now_secs,
            link_established_at_secs: None,
            reported_at_secs: None,
            closed_at_secs: None,
        }
    }

    /// Establishes causal link, starting the duty clock.
    pub fn establish_link(&mut self, now_secs: u64) -> Result<(), &'static str> {
        if !self
            .state
            .is_valid_transition(IncidentState::LinkEstablished)
        {
            return Err("invalid transition: link already established or closed");
        }
        self.state = IncidentState::LinkEstablished;
        self.link_established_at_secs = Some(now_secs);
        Ok(())
    }

    /// Derives the reporting deadline timestamp in unix seconds.
    #[must_use]
    pub fn deadline_unix_secs(&self) -> Option<u64> {
        self.link_established_at_secs
            .map(|start| start.saturating_add(self.duty_window.duration_secs()))
    }

    /// Checks if the duty clock has expired without formal report.
    #[must_use]
    pub fn is_deadline_breached(&self, now_secs: u64) -> bool {
        if self.state == IncidentState::Reported || self.state == IncidentState::Closed {
            return false;
        }
        if let Some(deadline) = self.deadline_unix_secs() {
            now_secs > deadline
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duty_window_durations() {
        assert_eq!(DutyWindow::Standard15d.duration_secs(), 15 * 86_400);
        assert_eq!(DutyWindow::Death10d.duration_secs(), 10 * 86_400);
        assert_eq!(DutyWindow::Widespread2d.duration_secs(), 2 * 86_400);
    }

    #[test]
    fn test_incident_lifecycle_transitions() {
        assert!(IncidentState::Suspected.is_valid_transition(IncidentState::LinkEstablished));
        assert!(IncidentState::Suspected.is_valid_transition(IncidentState::Closed));
        assert!(!IncidentState::Suspected.is_valid_transition(IncidentState::Reported));

        assert!(IncidentState::LinkEstablished.is_valid_transition(IncidentState::Reported));
        assert!(IncidentState::LinkEstablished.is_valid_transition(IncidentState::Closed));

        assert!(IncidentState::Reported.is_valid_transition(IncidentState::Closed));
        assert!(!IncidentState::Closed.is_valid_transition(IncidentState::Suspected));
    }

    #[test]
    fn test_duty_clock_deadline_and_breach() {
        let mut inc = IncidentRecord::new("INC-01", "Test", DutyWindow::Widespread2d, 1000);
        assert_eq!(inc.deadline_unix_secs(), None);
        assert!(!inc.is_deadline_breached(500_000));

        inc.establish_link(2000).unwrap();
        let expected_deadline = 2000 + 2 * 86_400;
        assert_eq!(inc.deadline_unix_secs(), Some(expected_deadline));
        assert!(!inc.is_deadline_breached(expected_deadline - 1));
        assert!(inc.is_deadline_breached(expected_deadline + 1));
    }
}
