use std::path::PathBuf;

use clap::ArgMatches;
use cli;

pub fn load_config() -> UpdaterConfig {
    let cli_app = cli::create_cli_app();
    let matches = cli_app.get_matches();
    UpdaterConfig {
        target_directory: get_string_value(&matches, "target-directory").map(|d| PathBuf::from(d)),
        exclude_asn: matches.is_present("exclude_asn"),
        exclude_ip2asn: matches.is_present("exclude_ip2asn"),
        exclude_maxmind: matches.is_present("exclude_maxmind"),
        maxmind_key: get_string_value(&matches, "maxmind-key"),
        skip_optimize: matches.is_present("skip_optimize"),
    }
}

fn get_string_value(matches: &ArgMatches, key: &str) -> Option<String> {
    matches.value_of(key).map(|m| m.to_string())
}

#[derive(Debug)]
pub struct UpdaterConfig {
    pub target_directory: Option<PathBuf>,
    pub exclude_asn: bool,
    pub exclude_ip2asn: bool,
    pub exclude_maxmind: bool,
    pub maxmind_key: Option<String>,
    pub skip_optimize: bool,
}
