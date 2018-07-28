
use std::net::IpAddr;
use std::sync::Mutex;

use maxminddb::Reader;
use maxminddb::geoip2::City;

lazy_static! {
    static ref CITY_READER: Mutex<MaxmindReader> = Mutex::new(MaxmindReader { reader: None });
}

struct MaxmindReader { reader: Option<Reader> }

pub fn load_city_reader(mm_file: String) {
    CITY_READER.lock().unwrap().reader = Some(Reader::open(&mm_file).unwrap());
}

pub fn lookup_city(ip: IpAddr) -> Option<City> {
    CITY_READER.lock().unwrap().reader.as_ref().and_then(|r| r.lookup::<City>(ip).ok())
}
