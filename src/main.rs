#[macro_use]
extern crate serde_derive;

extern crate hyper;
extern crate futures;
extern crate cidr;
extern crate serde;
extern crate serde_json;
extern crate maxminddb;
extern crate tokio_core;
extern crate domain;
extern crate tokio;

use std::env;

mod asn;
mod ip;
mod service;
mod maxmind;
mod dns;

use service::LookupService;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("loading asn database");
    let asn_map = asn::load_asn_file(args[1].clone()).unwrap();

    println!("loading ip database");
    let ip_map = ip::load_ip_asn_file(args[2].clone(), &asn_map).unwrap();

    println!("loading maxmind city database");
    let maxmind_database = maxmind::load_maxmind_database(args[3].clone());

    println!("creating dns resolver");
    let dns_resolver_handle = dns::create_dns_resolver();
    
    println!("Database Load Complete\n");
    LookupService::start(ip_map, maxmind_database, dns_resolver_handle);
}
