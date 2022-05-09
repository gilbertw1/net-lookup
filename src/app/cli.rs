use clap::{App, Arg};

pub fn create_cli_app() -> App<'static, 'static> {
    App::new("net-lookup")
        .version("0.1.0")
        .author("Bryan G. <bryan@bryan.sh>")
        .about("A utility that can be used to lookup information about IP addresses and domains.")
        .arg(
            Arg::with_name("query")
                .value_name("QUERY")
                .help("Optional query to run if not in daemon mode")
                .index(1),
        )
        .arg(
            Arg::with_name("daemon")
                .long("daemon")
                .short("-d")
                .help("Runs net-lookup in daemon mode."),
        )
        .arg(
            Arg::with_name("action")
                .short("-a")
                .long("action")
                .help("Runs the specified utility action.")
                .possible_values(&["optimize"])
                .takes_value(true),
        )
        .arg(
            Arg::with_name("host")
                .long("host")
                .help("Host to bind service to.")
                .default_value("0.0.0.0")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("-p")
                .long("port")
                .help("Port to bind service to.")
                .default_value("8080")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("resolver-host")
                .long("resolver-host")
                .help("Resolver host to use for dns lookups.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("resolver-port")
                .long("resolver-port")
                .help("Resolver port to use for dns lookups.")
                .default_value("53")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("maxmind-city-database")
                .long("maxmind-city-database")
                .help("Specify maxmind city database file.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("asn-database")
                .long("asn-database")
                .help("Specify asn database file.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ip2asn-database")
                .long("ip2asn-database")
                .help("Specify ip2asn database file.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("-v")
                .long("verbose")
                .help("Enables verbose logging"),
        )
}
