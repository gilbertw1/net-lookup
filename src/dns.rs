use std::net::IpAddr;

use domain::resolv::Resolver;
use domain::resolv::lookup::lookup_addr;
use tokio_core::reactor::Core;

pub fn reverse_dns_lookup(ip: IpAddr) -> Vec<String> {
    let mut core = Core::new().unwrap();
    let resolv = Resolver::new(&core.handle());
    let addrs = lookup_addr(resolv, ip);
    let response = core.run(addrs).unwrap();
    response.iter().map(|n| n.to_string().trim_right_matches(".").to_owned()).collect::<Vec<String>>()
}
