// xodr-qcr — OpenDRIVE Quality Checker (Rust). CLI entry point.

use std::path::Path;

use clap::Parser;
use roxmltree::Document;

use xodr_qcr::opendrive;
use xodr_qcr::opendrive::checks::{CheckerData, run_checks};
use xodr_qcr::result::Result;

/// OpenDRIVE Quality Checker (Rust)
#[derive(Parser, Debug)]
#[command(name = "xodr-qcr", version, about = "Checks .xodr files for OpenDRIVE quality issues")]
struct Cli {
    /// Path to the .xodr file to check
    input: String,

    /// Optional path to write the .xqar report (XML)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Suppress the text report on stdout
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,
}

fn main() {
    let cli = Cli::parse();
    let path = Path::new(&cli.input);

    if !path.exists() {
        eprintln!("Error: input file does not exist: {}", cli.input);
        std::process::exit(2);
    }

    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            std::process::exit(2);
        }
    };

    // Parse the document once. If it fails to parse, we still register the
    // bundle and let the basic valid_xml_document checker report the location.
    let doc = Document::parse(&text).ok();

    let mut result = Result::new();
    result.register_checker_bundle(
        opendrive::BUNDLE_NAME,
        "OpenDrive checker bundle",
        opendrive::BUNDLE_VERSION,
        "",
    );
    result.set_result_version(opendrive::BUNDLE_VERSION);

    let mut cd = CheckerData {
        xml_file_path: path,
        doc: doc.as_ref(),
        schema_version: None,
        result: &mut result,
    };

    run_checks(&mut cd);

    result.generate_summaries();

    if !cli.quiet {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        if let Err(e) = xodr_qcr::report::text::write_text_report(&result, &mut handle) {
            eprintln!("Error writing text report: {}", e);
        }
    }

    if let Some(out) = &cli.output {
        let mut file = match std::fs::File::create(out) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error creating output file {}: {}", out, e);
                std::process::exit(2);
            }
        };
        if let Err(e) = xodr_qcr::report::xqar::write_xqar(&result, &mut file) {
            eprintln!("Error writing xqar report: {}", e);
            std::process::exit(2);
        }
    }

    // Exit code: 1 if any ERROR-level issue exists, else 0.
    if result.has_errors() {
        std::process::exit(1);
    }
}
