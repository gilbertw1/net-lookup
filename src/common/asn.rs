use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::collections::HashMap;
use std::sync::Arc;

pub fn load_asn_database(file: &str) -> Result<AsnDatabase> {
    let file = File::open(file)?;
    let mut asn_map = HashMap::new();

    for line in BufReader::new(file).lines() {
        if line.is_ok() {
            let asn = parse_autonomous_system_number(line.unwrap());
            asn_map.insert(asn.id, Arc::new(asn));
        } else {
            println!("[ASN] skipping non utf8");
        }
    }
    
    Ok(AsnDatabase { asn_map })
}

fn parse_autonomous_system_number(line: String) -> AutonomousSystemNumber {
    let id_split: Vec<&str> = line.splitn(2, ' ').collect();
    let idstr = id_split[0].to_owned();
    let id = idstr.parse::<u32>().unwrap();
    let rest = id_split[1];
    let country_split: Vec<&str> = rest.rsplitn(2, ' ').collect();
    let country = country_split[0].to_owned();
    let (handle, name) = parse_handle_and_name(country_split[1]);
    AutonomousSystemNumber { id: id, handle: handle, name: name, country: country }
}

fn parse_handle_and_name(value: &str) -> (String, Option<String>) {
    if value.contains(" - ") {
        let split: Vec<&str> = value.splitn(2, " - ").collect();
        (split[0].to_owned(), Some(split[1].trim_right_matches(",").to_owned()))
    } else {
        (value.to_owned(), None)
    }
}

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct AutonomousSystemNumber {
    id: u32,
    handle: String,
    name: Option<String>,
    country: String,
}

#[derive(Debug, Clone)]
pub struct AsnDatabase {
    asn_map: HashMap<u32, Arc<AutonomousSystemNumber>>,
}

impl AsnDatabase {
    pub fn lookup(&self, id: u32) -> Option<Arc<AutonomousSystemNumber>> {
        self.asn_map.get(&id).map(|r| r.clone())
    }
}
