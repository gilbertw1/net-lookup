use std::net::IpAddr;
use std::str::FromStr;
use clap::ArgMatches;
use cli;

pub fn load_config() -> LookupConfig {
    let cli_app = cli::create_cli_app();
    let matches = cli_app.get_matches();
    LookupConfig {
        host: get_string_value(&matches, "host").unwrap_or("0.0.0.0".to_owned()).parse::<IpAddr>().unwrap(),
        port: get_value::<u16>(&matches, "port").unwrap_or(8080),
        maxmind_city_database_file: get_string_value(&matches, "maxmind-city-database").unwrap(),
        asn_database_file: get_string_value(&matches, "asn-database").unwrap(),
        ip_asn_database_file: get_string_value(&matches, "ip2asn-database").unwrap(),
        daemon: matches.is_present("daemon"),
        query: get_string_value(&matches, "query"),
        verbose: matches.is_present("verbose"),
    }
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
    pub maxmind_city_database_file: String,
    pub asn_database_file: String,
    pub ip_asn_database_file: String,
    pub daemon: bool,
    pub query: Option<String>,
    pub verbose: bool,
}

