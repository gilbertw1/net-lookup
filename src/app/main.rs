extern crate clap;
extern crate common;
extern crate serde_json;

use common::asn;
use common::dns;
use common::ip;
use common::lookup;
use common::maxmind;

use common::lookup::LookupHandler;
use common::service::LookupService;
use std::net::IpAddr;

mod cli;
mod config;

use config::LookupConfig;

#[tokio::main]
async fn main() {
    let conf = config::load_config();

    vlog(&conf, "loading asn database");
    let asn_database = asn::load_asn_database(&conf.asn_database_file).unwrap();

    vlog(&conf, "loading ip database");
    let ip_asn_database =
        ip::load_ip_asn_database(&conf.ip_asn_database_file, &asn_database).unwrap();

    vlog(&conf, "loading maxmind city database");
    let maxmind_database = maxmind::load_maxmind_database(&conf.maxmind_city_database_file);

    vlog(&conf, "Creating dns resolver");
    let dns_resolver_handle =
        dns::create_dns_resolver_handle(conf.resolver_host, conf.resolver_port);

    vlog(&conf, "Creating lookup handler");
    let lookup_handler =
        lookup::create_lookup_handler(ip_asn_database, maxmind_database, dns_resolver_handle);

    if conf.daemon {
        vlog(&conf, "Starting lookup daemon");
        let service = LookupService {
            handler: lookup_handler,
        };
        service.start(conf.host, conf.port).await;
    } else if conf.query.is_some() {
        execute_query(lookup_handler, conf.query.unwrap()).await;
    } else {
        println!("ERROR: No query provided, stopping.");
        std::process::exit(1);
    }
}

async fn execute_query(handler: LookupHandler, query: String) {
    let ip_result = query.parse::<IpAddr>();
    if ip_result.is_ok() {
        let ip = ip_result.unwrap();
        let result = handler.lookup_ip_sync(ip).await;
        println!("{}", serde_json::to_string(&result).unwrap());
    } else {
        let result = handler.lookup_domain_sync(query).await;
        println!("{}", serde_json::to_string(&result).unwrap());
    }
}

fn vlog(conf: &LookupConfig, msg: &str) {
    if conf.verbose {
        println!("{}", msg);
    }
}
