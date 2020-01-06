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
    let cidr_slash_idx = line.find('/').unwrap();
    let tab_idx = line.find('\t').unwrap();

    let cidr_addr = unsafe { line.get_unchecked(0..cidr_slash_idx) };
    let cidr_len = unsafe { line.get_unchecked(cidr_slash_idx+1..tab_idx).parse::<u8>().unwrap() };
    let cidr = AnyIpCidr::new(cidr_addr.parse::<IpAddr>().unwrap(), cidr_len).unwrap();

    let asn_id = unsafe { line.get_unchecked(tab_idx+1..line.len()).parse::<u32>().unwrap() };

    IpAsnRecord {
        start: cidr.first_address().unwrap(),
        end: cidr.last_address().unwrap(),
        asn: asn_database.lookup(asn_id).map(|r| r.clone()),
    }
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
