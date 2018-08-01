extern crate common;
extern crate clap;
extern crate reqwest;
extern crate libc;
extern crate glob;
extern crate tempdir;

mod cli;
mod config;

use std::env;
use std::fs;
use std::fs::File;
use std::path::{Path,PathBuf};
use std::process::{Command, Stdio};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use glob::glob;
use tempdir::TempDir;

use common::files;
use config::UpdaterConfig;

fn main() {
    let conf = config::load_config();
    let target_dir_path = create_target_path(&conf);
    let working_dir =  TempDir::new("net-lookup-updater-working").unwrap();

    let asn_target_file = files::get_asn_path(&target_dir_path);
    let ip2asn_target_file = files::get_ip2asn_path(&target_dir_path);
    let maxmind_target_file = files::get_maxmind_path(&target_dir_path);

    env::set_current_dir(working_dir.path()).unwrap();

    if !conf.exclude_asn {
        println!("Downloading and cleaning asn file...");
        download_and_clean_asn_file(&asn_target_file);
    }

    if !conf.exclude_ip2asn {
        println!("Creating ip2asn file...");
        create_ip2asn_file(&ip2asn_target_file);
    }

    if !conf.exclude_maxmind {
        println!("Downloading maxmind city database...");
        download_maxmind_city_database(&maxmind_target_file);
    }

    working_dir.close().unwrap();
}

fn create_target_path(conf: &UpdaterConfig) -> PathBuf {
    let default_dir = files::get_default_directory();
    let raw_target_dir_path = conf.target_directory.clone().unwrap_or(default_dir);
    fs::create_dir_all(&raw_target_dir_path).unwrap();
    fs::canonicalize(raw_target_dir_path).unwrap()
}

fn download_and_clean_asn_file(target_file: &Path) {
    let dirty_file_path = Path::new("asn-dirty.txt");
    let mut dirty_file = File::create(&dirty_file_path).unwrap();
    reqwest::get("https://ftp.ripe.net/ripe/asnames/asn.txt").unwrap().copy_to(&mut dirty_file).unwrap();

    let target_file_fd = File::create(&target_file).unwrap().into_raw_fd();
    let file_out = unsafe { Stdio::from_raw_fd(target_file_fd) };

    let mut cmd = Command::new("iconv")
        .args(&["-f", "utf-8", "-t", "utf-8", "-c", &dirty_file_path.to_string_lossy()])
        .stdout(file_out)
        .spawn()
        .unwrap();

    cmd.wait().unwrap();
}

fn create_ip2asn_file(target_file: &Path) {
    Command::new("pyasn_util_download.py")
        .args(&["-46"])
        .output()
        .unwrap();
    let pyasn_file = glob("rib*.bz2").unwrap().next().unwrap().unwrap();
    Command::new("pyasn_util_convert.py")
        .args(&["--single", &pyasn_file.to_string_lossy(), &target_file.to_string_lossy()])
        .output()
        .unwrap();
}

fn download_maxmind_city_database(target_file: &Path) {
    let maxmind_archive_path = Path::new("maxmind-geolite-city.tar.gz");
    let mut maxmind_archive_file = File::create(&maxmind_archive_path).unwrap();
    reqwest::get("http://geolite.maxmind.com/download/geoip/database/GeoLite2-City.tar.gz").unwrap().copy_to(&mut maxmind_archive_file).unwrap();
    Command::new("tar")
        .args(&["xzf", &maxmind_archive_path.to_string_lossy()])
        .output()
        .unwrap();
    let geolite_dir = glob("GeoLite2-City_*").unwrap().next().unwrap().unwrap();
    fs::copy(geolite_dir.join("GeoLite2-City.mmdb"), &target_file).unwrap();
}
