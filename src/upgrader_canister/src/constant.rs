use std::time::Duration;

pub(crate) const PERMISSIONS_MAP_MEMORY_ID: u8 = 1;
pub(crate) const PROJECTS_MAP_MEMORY_ID: u8 = 2;
pub(crate) const POLLS_PENDING_MAP_MEMORY_ID: u8 = 3;
pub(crate) const POLLS_CLOSED_MAP_MEMORY_ID: u8 = 4;
pub(crate) const POLLS_ID_SEQUENCE_MEMORY_ID: u8 = 5;
pub(crate) const SETTINGS_MAP_MEMORY_ID: u8 = 6;

/// The interval at which the poll timer should run
pub const POLL_TIMER_INTERVAL: Duration = Duration::from_secs(600);
