use clap::{App, Arg};

pub fn create_cli_app() -> App<'static, 'static> {
    App::new("net-lookup-updater")
        .version("0.1.0")
        .author("Bryan G. <bryan@bryan.sh>")
        .about("A utiltiy that downloads data required to run net-lookup.")
        .arg(
            Arg::with_name("target-directory")
                .long("target-directory")
                .value_name("DIR")
                .help("Target directory to download files to.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("exclude-asn")
                .long("exclude-asn")
                .help("Exclude asn data file from update process."),
        )
        .arg(
            Arg::with_name("exclude-ip2asn")
                .long("exclude-ip2asn")
                .help("Exclude ip2asn file from update process."),
        )
        .arg(
            Arg::with_name("exclude-maxmind")
                .long("exclude-maxmind")
                .help("Exclude maxmind database from update process."),
        )
        .arg(
            Arg::with_name("maxmind-key")
                .long("maxmind-key")
                .value_name("KEY")
                .help("Key used to fetch maxmind data files.")
                .takes_value(true),
        )
}
