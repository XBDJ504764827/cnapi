//! Health state of an external dependency (PostgreSQL, Redis, ...).

/// Identifier of a dependency that the API relies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyName {
    Database,
    Redis,
}

impl DependencyName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Database => "postgresql",
            Self::Redis => "redis",
        }
    }
}

/// Result of a single dependency probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyStatus {
    Up,
    Down,
}

impl DependencyStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }

    pub const fn is_up(self) -> bool {
        matches!(self, Self::Up)
    }
}

/// One sanitized dependency probe result.
///
/// `detail` never contains SQL, connection strings or stack traces: it is a
/// stable, low cardinality reason (for example `timeout`) that is safe to show
/// to clients. Full errors are logged server side.
#[derive(Debug, Clone)]
pub struct DependencyCheck {
    pub name: DependencyName,
    pub status: DependencyStatus,
    pub latency_ms: u64,
    pub detail: Option<String>,
}

impl DependencyCheck {
    pub fn up(name: DependencyName, latency_ms: u64) -> Self {
        Self {
            name,
            status: DependencyStatus::Up,
            latency_ms,
            detail: None,
        }
    }

    pub fn down(name: DependencyName, latency_ms: u64, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: DependencyStatus::Down,
            latency_ms,
            detail: Some(detail.into()),
        }
    }
}
