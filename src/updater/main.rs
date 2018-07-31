extern crate clap;
extern crate common;
extern crate reqwest;
extern crate libc;
extern crate glob;
extern crate tempdir;

use std::env;
use std::fs;
use std::fs::File;
use std::path::{Path,PathBuf};
use std::process::{Command, Stdio};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use glob::glob;
use tempdir::TempDir;

fn main() {
    let args: Vec<String> = env::args().collect();
    let default_dir = "./output".to_owned();
    let target_dir_path = Path::new(args.get(1).unwrap_or(&default_dir));
    let working_dir =  TempDir::new("net-lookup-updater-working").unwrap();

    // Create output target_dir if it doesn't exist
    fs::create_dir_all(target_dir_path).unwrap();
    let target_dir = fs::canonicalize(target_dir_path).unwrap();

    println!("Working Dir: {:?}", working_dir.path());
    env::set_current_dir(working_dir.path()).unwrap();

    println!("Downloading and cleaning asn file...");
    let dirty_asn_file = download_and_clean_asn_file(&target_dir);

    println!("Creating ip2asn file...");
    let ip2asn_file = create_ip2asn_file(&target_dir);

    println!("Downloading maxmind city database...");
    let maxmind_city_file = download_maxmind_city_database(&target_dir);

    working_dir.close().unwrap();
}

fn download_and_clean_asn_file(target_dir: &Path) -> PathBuf {
    let dirty_file_path = Path::new("asn-dirty.txt");
    let mut dirty_file = File::create(&dirty_file_path).unwrap();
    reqwest::get("https://ftp.ripe.net/ripe/asnames/asn.txt").unwrap().copy_to(&mut dirty_file).unwrap();

    let clean_file_path = target_dir.join("asn.txt");
    let clean_file_fd = File::create(&clean_file_path).unwrap().into_raw_fd();
    let file_out = unsafe { Stdio::from_raw_fd(clean_file_fd) };

    let mut cmd = Command::new("iconv")
        .args(&["-f", "utf-8", "-t", "utf-8", "-c", &dirty_file_path.to_string_lossy()])
        .stdout(file_out)
        .spawn()
        .unwrap();

    cmd.wait().unwrap();
    clean_file_path
}

fn create_ip2asn_file(target_dir: &Path) -> PathBuf {
    let ip2asn_file_path = target_dir.join("ip2asn.dat");
    Command::new("pyasn_util_download.py")
        .args(&["-46"])
        .output()
        .unwrap();
    let pyasn_file = glob("rib*.bz2").unwrap().next().unwrap().unwrap();
    Command::new("pyasn_util_convert.py")
        .args(&["--single", &pyasn_file.to_string_lossy(), &ip2asn_file_path.to_string_lossy()])
        .output()
        .unwrap();
    ip2asn_file_path
}

fn download_maxmind_city_database(target_dir: &Path) -> PathBuf {
    let maxmind_file_path = target_dir.join("maxmind-geolite2-city.mmdb");
    let maxmind_archive_path = Path::new("maxmind-geolite-city.tar.gz");
    let mut maxmind_archive_file = File::create(&maxmind_archive_path).unwrap();
    reqwest::get("http://geolite.maxmind.com/download/geoip/database/GeoLite2-City.tar.gz").unwrap().copy_to(&mut maxmind_archive_file).unwrap();
    Command::new("tar")
        .args(&["xzf", &maxmind_archive_path.to_string_lossy()])
        .output()
        .unwrap();
    let geolite_dir = glob("GeoLite2-City_*").unwrap().next().unwrap().unwrap();
    fs::copy(geolite_dir.join("GeoLite2-City.mmdb"), &maxmind_file_path).unwrap();
    maxmind_file_path
}
