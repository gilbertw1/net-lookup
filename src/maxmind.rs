use std::net::IpAddr;
use std::sync::Arc;

use maxminddb::Reader;
use maxminddb::geoip2::City;

pub fn load_maxmind_database(mm_file: String) -> MaxmindDatabase {
    let reader = Reader::open(&mm_file).unwrap();
    MaxmindDatabase { reader: Arc::new(reader) }
}

#[derive(Clone)]
pub struct MaxmindDatabase { reader: Arc<Reader> }

impl MaxmindDatabase {
    pub fn lookup_city(&self, ip: IpAddr) -> Option<City> {
        self.reader.lookup::<City>(ip).ok()
    }
}
