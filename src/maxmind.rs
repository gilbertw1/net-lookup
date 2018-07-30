use std::net::IpAddr;

use maxminddb::Reader;
use maxminddb::geoip2::City;

pub fn load_maxmind_database(mm_file: &str) -> MaxmindDatabase {
    let reader = Reader::open(mm_file).unwrap();
    MaxmindDatabase { reader: reader }
}

pub struct MaxmindDatabase { reader: Reader }

impl MaxmindDatabase {
    pub fn lookup_city(&self, ip: IpAddr) -> Option<City> {
        self.reader.lookup::<City>(ip).ok()
    }
}
