extern crate hyper;
extern crate futures;
extern crate cidr;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::env;
use std::sync::Arc;

mod asn;
mod ip;
mod service;

use service::LookupService;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("loading asn database");
    let asn_map = asn::load_asn_file(args[1].clone()).unwrap();

    println!("loading ip database");
    let ip_map = ip::load_ip_asn_file(args[2].clone(), &asn_map).unwrap();
    
    println!("Database Load Complete\n");
    let service = LookupService { database: Arc::new(ip_map) };
    service.start();
}

