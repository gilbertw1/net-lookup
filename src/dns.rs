use std::net::IpAddr;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use domain::resolv::Resolver;
use domain::resolv::lookup::lookup_addr;
use tokio_core::reactor::Core;

pub fn start_dns_lookup_loop() -> DnsLookupHandle {
    let (sender, receiver) = channel();
    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        let resolv = Resolver::new(&core.handle());
        for request in receiver {
            match request {
                DnsLookupRequest::ReverseDnsLookupRequest { ip, channel } => {
                    let names = reverse_dns_lookup(&mut core, resolv.clone(), ip);
                    let result = channel.send(ReverseDnsLookupResponse { names: names });
                    if result.is_err() {
                        println!("Error looking up reverse dns: {:?}", result);
                    }
                }
            }
        }
    });
    DnsLookupHandle { sender: sender }
}

fn reverse_dns_lookup(core: &mut Core, resolv: Resolver, ip: IpAddr) -> Vec<String> {
    let addrs = lookup_addr(resolv, ip);
    let response = core.run(addrs).unwrap();
    response.iter().map(|n| n.to_string().trim_right_matches(".").to_owned()).collect::<Vec<String>>()
}

#[derive(Debug)]
pub struct DnsLookupHandle {
    sender: Sender<DnsLookupRequest>,
}

impl DnsLookupHandle {
    pub fn reverse_dns_lookup(&self, ip: IpAddr) -> Option<Vec<String>> {
        let (sender, receiver) = channel();
        let result = self.sender.send(DnsLookupRequest::ReverseDnsLookupRequest { ip: ip, channel: sender });
        if result.is_ok() {
            receiver.recv().ok().map(|r| r.names)
        } else {
            println!("Error looking up reverse dns: {:?}", result);
            None
        }
    }
}

#[derive(Debug)]
pub enum DnsLookupRequest {
    ReverseDnsLookupRequest { ip: IpAddr, channel: Sender<ReverseDnsLookupResponse>}
}

#[derive(PartialEq, Debug)]
pub struct ReverseDnsLookupResponse {
    names: Vec<String>,
}
