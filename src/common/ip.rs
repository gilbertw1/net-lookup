use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader, Result};
use std::collections::Bound::{Included, Unbounded};
use std::net::IpAddr;
use std::sync::Arc;

use asn::{AsnDatabase, AutonomousSystemNumber};
use cidr::AnyIpCidr;
 
pub fn load_ip_asn_database(file_path: &Path, asn_database: &AsnDatabase) -> Result<IpAsnDatabase> {
    let file = File::open(file_path)?;
    let mut ip_map = BTreeMap::new();

    for line_res in BufReader::new(file).lines() {
        if line_res.is_ok() {
            let line = line_res.unwrap();
            if !line.starts_with(';') {
                let ip_block = parse_ip_block(line, asn_database);
                ip_map.insert(ip_block.start, ip_block);
            }
        } else {
            println!("[IP] skipping non utf8");
        }
    }
    Ok(IpAsnDatabase { ip_asn_map: ip_map })
}

fn parse_ip_block(line: String, asn_database: &AsnDatabase) -> IpAsnRecord {
    let split: Vec<&str> = line.split('\t').collect();
    let (start, end) = get_cidr_start_end(split[0]);

    IpAsnRecord {
        start: start,
        end: end,
        asn: asn_database.lookup(split[1].parse::<u32>().unwrap()).map(|r| r.clone()),
    }
}

fn get_cidr_start_end(cidr: &str) -> (IpAddr, IpAddr) {
    let split: Vec<&str> = cidr.split('/').collect();
    let addr = split[0].parse::<IpAddr>().unwrap();
    let len = split[1].parse::<u8>().unwrap();
    let cidr = AnyIpCidr::new(addr, len).unwrap();
    (cidr.first_address().unwrap(), cidr.last_address().unwrap())
}


#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct IpAsnRecord {
    pub start: IpAddr,
    pub end: IpAddr,
    pub asn: Option<Arc<AutonomousSystemNumber>>,
}

#[derive(Debug, Clone)]
pub struct IpAsnDatabase {
    ip_asn_map: BTreeMap<IpAddr,IpAsnRecord>
}

impl IpAsnDatabase {
    pub fn lookup(&self, ip: IpAddr) -> Option<&IpAsnRecord> {
        let range = self.ip_asn_map.range((Unbounded, Included(ip)));
        range.last().map(|e| e.1)
    }
}
