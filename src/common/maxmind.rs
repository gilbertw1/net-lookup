use std::net::IpAddr;
use std::path::Path;

use maxminddb::geoip2::City;
use maxminddb::Reader;

pub fn load_maxmind_database(mm_file: &Path) -> MaxmindDatabase {
    let reader = Reader::open(&mm_file.to_string_lossy()).unwrap();
    MaxmindDatabase { reader: reader }
}

pub struct MaxmindDatabase {
    reader: Reader,
}

impl MaxmindDatabase {
    pub fn lookup_city(&self, ip: IpAddr) -> Option<City> {
        self.reader.lookup::<City>(ip).ok()
    }
}
