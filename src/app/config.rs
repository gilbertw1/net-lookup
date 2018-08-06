use std::net::IpAddr;
use std::str::FromStr;
use std::path::PathBuf;
use clap::ArgMatches;

use cli;
use common::files;

pub fn load_config() -> LookupConfig {
    let cli_app = cli::create_cli_app();
    let matches = cli_app.get_matches();
    LookupConfig {
        host: get_string_value(&matches, "host").unwrap_or("0.0.0.0".to_owned()).parse::<IpAddr>().unwrap(),
        port: get_value::<u16>(&matches, "port").unwrap_or(8080),
        resolver_host: get_string_value(&matches, "resolver-host").map(|s| s.parse::<IpAddr>().unwrap()),
        resolver_port: get_value::<u16>(&matches, "resolver-port").unwrap_or(53),
        maxmind_city_database_file: get_file_path_or(&matches, "maxmind-city-database", files::get_default_maxmind_path()),
        asn_database_file: get_file_path_or(&matches, "asn-database", files::get_default_asn_path()),
        ip_asn_database_file: get_file_path_or(&matches, "ip2asn-database", files::get_default_ip2asn_path()),
        daemon: matches.is_present("daemon"),
        query: get_string_value(&matches, "query"),
        verbose: matches.is_present("verbose"),
    }
}

fn get_file_path_or(matches: &ArgMatches, key: &str, path: PathBuf) -> PathBuf {
    get_string_value(matches, key).map(|m| PathBuf::from(m)).unwrap_or(path)
}

fn get_string_value(matches: &ArgMatches, key: &str) -> Option<String> {
    matches.value_of(key).map(|m| m.to_string())
}

fn get_value<T: FromStr>(matches: &ArgMatches, key: &str) -> Result<T, T::Err> {
    matches.value_of(key).unwrap().parse::<T>()
}


#[derive(Debug)]
pub struct LookupConfig {
    pub host: IpAddr,
    pub port: u16,
    pub resolver_host: Option<IpAddr>,
    pub resolver_port: u16,
    pub maxmind_city_database_file: PathBuf,
    pub asn_database_file: PathBuf,
    pub ip_asn_database_file: PathBuf,
    pub daemon: bool,
    pub query: Option<String>,
    pub verbose: bool,
}

