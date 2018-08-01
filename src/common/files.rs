use std::env;
use std::path::{Path, PathBuf};

static DEFAULT_ASN_FILE_NAME: &'static str = "asn.dat";
static DEFAULT_IP2ASN_FILE_NAME: &'static str = "ip2asn.dat";
static DEFAULT_ENCODED_IP2ASN_FILE_NAME: &'static str = "ip2asn-encoded.dat";
static DEFAULT_MAXMIND_FILE_NAME: &'static str = "maxmind-geolite2-city.mmdb";

pub fn get_default_directory() -> PathBuf {
    env::home_dir().unwrap().join(".local/share/net-lookup")
}

pub fn get_asn_path(directory: &Path) -> PathBuf {
    directory.join(DEFAULT_ASN_FILE_NAME)
}

pub fn get_default_asn_path() -> PathBuf {
    get_asn_path(&get_default_directory())
}

pub fn get_ip2asn_path(directory: &Path) -> PathBuf {
    directory.join(DEFAULT_IP2ASN_FILE_NAME)
}

pub fn get_default_ip2asn_path() -> PathBuf {
    get_ip2asn_path(&get_default_directory())
}

pub fn get_maxmind_path(directory: &Path) -> PathBuf {
    directory.join(DEFAULT_MAXMIND_FILE_NAME)
}

pub fn get_default_maxmind_path() -> PathBuf {
    get_maxmind_path(&get_default_directory())
}

pub fn get_encoded_ip2asn_path(directory: &Path) -> PathBuf {
    directory.join(DEFAULT_ENCODED_IP2ASN_FILE_NAME)
}

pub fn get_default_encoded_ip2asn_path() -> PathBuf {
    get_encoded_ip2asn_path(&get_default_directory())
}
