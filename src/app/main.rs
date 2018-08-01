extern crate common;
extern crate clap;
extern crate serde_json;

use common::asn;
use common::ip;
use common::maxmind;
use common::dns;
use common::lookup;

use std::net::IpAddr;
use std::time::Instant;
use common::lookup::LookupHandler;
use common::service::LookupService;

mod cli;
mod config;

use config::LookupConfig;

fn main() {
    let conf = config::load_config();

    vlog(&conf, "loading asn database");
    let asn_start = Instant::now();
    let asn_database = asn::load_asn_database(&conf.asn_database_file).unwrap();
    println!("Asn Load Time: {:?}", Instant::now() - asn_start);

    vlog(&conf, "loading ip database");
    let ip_start = Instant::now();
    let ip_asn_database = ip::load_ip_asn_database(&conf.ip_asn_database_file, &asn_database).unwrap();
    println!("Ip Load Time: {:?}", Instant::now() - ip_start);

    vlog(&conf, "loading maxmind city database");
    let maxmind_database = maxmind::load_maxmind_database(&conf.maxmind_city_database_file);

    vlog(&conf, "Creating dns resolver");
    let dns_resolver_handle = dns::create_dns_resolver();

    vlog(&conf, "Creating lookup handler");
    let lookup_handler = lookup::create_lookup_handler(ip_asn_database, maxmind_database, dns_resolver_handle);
    
    if conf.daemon {
        vlog(&conf, "Starting lookup daemon");
        LookupService::start(conf.host, conf.port, lookup_handler);
    } else if conf.query.is_some() {
        execute_query(lookup_handler, conf.query.unwrap());
    } else {
        println!("ERROR: No query provided, stopping.");
        std::process::exit(1);
    }
}

fn execute_query(handler: LookupHandler, query: String) {
    let ip_result = query.parse::<IpAddr>();
    if ip_result.is_ok() {
        let ip = ip_result.unwrap();
        let result = handler.lookup_ip_sync(ip);
        println!("{}", serde_json::to_string(&result).unwrap());
    } else {
        let result = handler.lookup_domain_sync(query);
        println!("{}", serde_json::to_string(&result).unwrap());
    }
}

fn vlog(conf: &LookupConfig, msg: &str) {
   if conf.verbose {
       println!("{}", msg);
   } 
}
