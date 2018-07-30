#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate tokio_core;
extern crate tokio;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate cidr;
extern crate maxminddb;
extern crate domain;
extern crate clap;

mod cli;
mod config;
mod asn;
mod ip;
mod service;
mod maxmind;
mod dns;
mod lookup;

use std::net::IpAddr;
use futures::future::Future;
use config::LookupConfig;
use lookup::LookupHandler;
use service::LookupService;

fn main() {
    let conf = config::load_config();

    vlog(&conf, "loading asn database");
    let asn_database = asn::load_asn_database(&conf.asn_database_file).unwrap();

    vlog(&conf, "loading ip database");
    let ip_asn_database = ip::load_ip_asn_database(&conf.ip_asn_database_file, &asn_database).unwrap();

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
        let result_future = handler.lookup_ip(ip);
        println!("{:?}", Future::wait(result_future).unwrap());
    } else {
        println!("ERROR: IP address is not valid, stopping.");
        std::process::exit(1);
    }
}

fn vlog(conf: &LookupConfig, msg: &str) {
   if conf.verbose {
       println!("{}", msg);
   } 
}
