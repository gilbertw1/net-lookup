use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::collections::Bound::{Included, Unbounded};
use std::net::IpAddr;

use asn::{AsnDatabase, AutonomousSystemNumber};

pub type IpAsnDatabase = BTreeMap<IpAddr,IpBlock>;

pub fn query_database(ip_map: &IpAsnDatabase, ip: IpAddr) -> Option<&IpBlock> {
    let range = ip_map.range((Unbounded, Included(ip)));
    range.last().map(|e| e.1)
}

pub fn load_ip_asn_file(file: String, asn_map: &AsnDatabase) -> Result<IpAsnDatabase> {
    let file = File::open(file)?;
    let mut ip_map = BTreeMap::new();

    for line in BufReader::new(file).lines() {
        if line.is_ok() {
            let ip_block = parse_ip_block(line.unwrap(), asn_map);
            ip_map.insert(ip_block.start, ip_block);
        } else {
            println!("[IP] skipping non utf8");
        }
    }
    Ok(ip_map)
}

fn parse_ip_block(line: String, asn_map: &AsnDatabase) -> IpBlock {
    let split: Vec<&str> = line.split('\t').collect();

    IpBlock {
        start: split[0].parse::<IpAddr>().unwrap(),
        end: split[1].parse::<IpAddr>().unwrap(),
        asn: asn_map.get(&split[2].parse::<u32>().unwrap()).map(|r| r.clone()),
    }
}


#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct IpBlock {
    start: IpAddr,
    end: IpAddr,
    asn: Option<AutonomousSystemNumber>,
}
