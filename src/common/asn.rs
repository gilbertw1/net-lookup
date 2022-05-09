use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;
use std::sync::Arc;

pub fn load_asn_database(file_path: &Path) -> Result<AsnDatabase> {
    let file = File::open(file_path)?;
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
    let first_space_idx = line.find(' ').unwrap();
    let last_space_idx = line.rfind(' ').unwrap();
    let sep_idx = line.find(" - ");

    let id = unsafe {
        line.get_unchecked(0..first_space_idx)
            .parse::<u32>()
            .unwrap()
    };
    let country = unsafe { line.get_unchecked(last_space_idx + 1..line.len()) };
    let (handle, name) = if sep_idx.is_some() && sep_idx.unwrap() > first_space_idx {
        let hdl = unsafe { line.get_unchecked(first_space_idx + 1..sep_idx.unwrap()) };
        let nme = unsafe { line.get_unchecked(sep_idx.unwrap() + 3..last_space_idx) };
        (hdl, Some(nme))
    } else {
        let hdl = unsafe { line.get_unchecked(first_space_idx + 1..last_space_idx) };
        (hdl, None)
    };

    AutonomousSystemNumber {
        id: id,
        handle: handle.to_owned(),
        name: name.map(|n| n.to_owned()),
        country: country.to_owned(),
    }
}

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct AutonomousSystemNumber {
    id: u32,
    handle: String,
    name: Option<String>,
    country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsnDatabase {
    asn_map: HashMap<u32, Arc<AutonomousSystemNumber>>,
}

impl AsnDatabase {
    pub fn lookup(&self, id: u32) -> Option<Arc<AutonomousSystemNumber>> {
        self.asn_map.get(&id).map(|r| r.clone())
    }
}
