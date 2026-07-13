// xodr-qcr — OpenDRIVE Quality Checker (Rust). Library root.
//
// Exposes the engine so integration tests in `tests/` can call the checkers
// directly (mirroring the Python `test_setup.py` pattern of invoking the
// engine rather than shelling out to the binary).

pub mod opendrive;
pub mod report;
pub mod result;
pub mod version;

pub use opendrive::checks::{CheckerData, run_checks};
pub use result::{IssueSeverity, Result, StatusType};
