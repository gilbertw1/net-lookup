#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

extern crate hyper;
extern crate futures;
extern crate cidr;
extern crate serde;
extern crate serde_json;
extern crate maxminddb;

use std::env;
use std::sync::Arc;

mod asn;
mod ip;
mod service;
mod maxmind;

use service::LookupService;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("loading asn database");
    let asn_map = asn::load_asn_file(args[1].clone()).unwrap();

    println!("loading ip database");
    let ip_map = ip::load_ip_asn_file(args[2].clone(), &asn_map).unwrap();

    println!("loading maxmind city database");
    maxmind::load_city_reader(args[3].clone());
    
    println!("Database Load Complete\n");
    let service = LookupService { database: Arc::new(ip_map) };
    service.start();
}
