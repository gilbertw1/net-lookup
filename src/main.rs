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

use service::LookupService;

fn main() {
    let conf = config::load_config();

    println!("loading asn database");
    let asn_database = asn::load_asn_database(conf.asn_database_file).unwrap();

    println!("loading ip database");
    let ip_asn_database = ip::load_ip_asn_database(conf.ip_asn_database_file, &asn_database).unwrap();

    println!("loading maxmind city database");
    let maxmind_database = maxmind::load_maxmind_database(conf.maxmind_city_database_file);

    println!("creating dns resolver");
    let dns_resolver_handle = dns::create_dns_resolver();
    
    println!("Database Load Complete\n");
    LookupService::start(conf.host, conf.port, ip_asn_database, maxmind_database, dns_resolver_handle);
}
