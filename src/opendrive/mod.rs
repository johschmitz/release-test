// OpenDRIVE checker bundle: constants, schema map, checker orchestration.

pub mod checks;
pub mod models;
pub mod utils;

pub const BUNDLE_NAME: &str = "xodrBundle";
pub const BUNDLE_VERSION: &str = "v1.0.0";

/// Map of OpenDRIVE schema version (e.g. "1.7.0") to the bundled XSD file path.
/// Mirrors qc_opendrive/schema/schema_files.py.
pub fn schema_files() -> std::collections::HashMap<&'static str, &'static str> {
    let mut m = std::collections::HashMap::new();
    m.insert("1.1.0", "1.1/OpenDRIVE_1.1.xsd");
    m.insert("1.2.0", "1.2/OpenDRIVE_1.2.xsd");
    m.insert("1.3.0", "1.3/OpenDRIVE_1.3.xsd");
    m.insert("1.4.0", "1.4/OpenDRIVE_1.4H.xsd");
    m.insert("1.5.0", "1.5/OpenDRIVE_1.5M.xsd");
    m.insert("1.6.0", "1.6.1/opendrive_16_core.xsd");
    m.insert("1.7.0", "1.7.0/opendrive_17_core.xsd");
    m.insert("1.8.0", "1.8.1/OpenDRIVE_Core.xsd");
    m
}
