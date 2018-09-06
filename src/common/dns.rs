use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::thread;
use std::str;

use futures::Future;
use std::sync::mpsc;
use futures::future::Either;
use futures::future;
use domain::resolv::Resolver;
use domain::resolv::conf::ServerConf;
use domain::resolv::conf::ResolvConf;
use domain::resolv::lookup::lookup_addr;
use domain::iana::{Rtype,Class};
use domain::bits::{DNameBuf, ParsedDName};
use domain::rdata;
use tokio_core::reactor::Core;

pub fn create_dns_resolver_handle(host: Option<IpAddr>, port: u16) -> DnsResolverHandle {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        let resolv = create_resolver(&core, host, port);
        tx.send(resolv).expect("[dns] Failed to respond with resolver.");
        loop { core.turn(None); }
    });
    return DnsResolverHandle { resolv: rx.recv().unwrap() }
}

fn create_resolver(core: &Core, resolver_host: Option<IpAddr>, resolver_port: u16) -> Resolver {
    match resolver_host {
        Some(addr) => {
            let server_conf = ServerConf::new(SocketAddr::new(addr, resolver_port));
            let mut resolv_conf = ResolvConf::new();
            resolv_conf.servers = vec![server_conf];
            Resolver::from_conf(&core.handle(), resolv_conf)
        },
        None => Resolver::new(&core.handle()),
    }
}

#[derive(Clone)]
pub struct DnsResolverHandle {
    resolv: Resolver,
}

impl DnsResolverHandle {
    pub fn reverse_dns_lookup(&self, ip: IpAddr) -> impl Future<Item=Vec<String>, Error=()> {
        lookup_addr(self.resolv.clone(), ip).map_err(|e| println!("error = {:?}", e))
                                            .map(|addrs| addrs.iter().map(|n| n.to_string()).collect())
    }

    pub fn dns_lookup(&self, domain: String) -> impl Future<Item=DnsLookupResult, Error=()> {
        let mut domain = domain.clone();
        if  !domain.ends_with('.') {
            domain.push('.')
        }
        match domain.parse::<DNameBuf>() {
            Ok(dname) =>
                Either::A(create_lookup_future(self.resolv.clone(), &dname)),
            Err(_) =>
                Either::B(future::ok(DnsLookupResult::empty())),
        }
    }
}

fn create_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=DnsLookupResult, Error=()> {
    let a_future = create_a_lookup_future(resolv.clone(), &dname);
    let aaaa_future = create_aaaa_lookup_future(resolv.clone(), &dname);
    let cname_future = create_cname_lookup_future(resolv.clone(), &dname);
    let ns_future = create_ns_lookup_future(resolv.clone(), &dname);
    let mx_future = create_mx_lookup_future(resolv.clone(), &dname);
    let txt_future = create_txt_lookup_future(resolv.clone(), &dname);
    let soa_future = create_soa_lookup_future(resolv.clone(), &dname);

    a_future.join(aaaa_future).join(cname_future).join(ns_future).join(mx_future).join(txt_future).join(soa_future).then(|result| {
        match result {
            Ok(((((((a, aaaa), cname), ns), mx), txt), soa)) =>
                Ok(DnsLookupResult {a, aaaa, cname, ns, mx, txt, soa}),
            Err(_) =>
                Ok(DnsLookupResult::empty())
        }
    })
}

fn create_a_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=Vec<Ipv4Addr>, Error=()> {
    resolv.query((dname, Rtype::A, Class::In)).then(|result| {
        match result {
            Ok(response) => {
                let mut addrs = Vec::new();
                for record in response.answer().unwrap().limit_to::<rdata::A>() {
                    if record.is_ok() {
                        addrs.push(record.unwrap().into_data().addr());
                    }
                }
                Ok(addrs)
            },
            Err(_) => Ok(Vec::new()),
        }
    })
}

fn create_aaaa_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=Vec<Ipv6Addr>, Error=()> {
    resolv.query((dname, Rtype::Aaaa, Class::In)).then(|result| {
        match result {
            Ok(response) => {
                let mut addrs = Vec::new();
                for record in response.answer().unwrap().limit_to::<rdata::Aaaa>() {
                    if record.is_ok() {
                        addrs.push(record.unwrap().into_data().addr());
                    }
                }
                Ok(addrs)
            },
            Err(_) => Ok(Vec::new()),
        }
    })
}

fn create_cname_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=Vec<String>, Error=()> {
    resolv.query((dname, Rtype::Cname, Class::In)).then(|result| {
        match result {
            Ok(response) => {
                let mut cnames = Vec::new();
                for record in response.answer().unwrap().limit_to::<rdata::Cname<ParsedDName>>() {
                    if record.is_ok() {
                        let cname = record.unwrap().into_data();
                        cnames.push(format!("{}", cname));
                    }
                }
                Ok(cnames)
            },
            Err(_) => Ok(Vec::new()),
        }
    })
}

fn create_ns_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=Vec<String>, Error=()> {
    resolv.query((dname, Rtype::Ns, Class::In)).then(|result| {
        match result {
            Ok(response) => {
                let mut nss = Vec::new();
                for record in response.answer().unwrap().limit_to::<rdata::Ns<ParsedDName>>() {
                    if record.is_ok() {
                        let cname = record.unwrap().into_data();
                        nss.push(format!("{}", cname));
                    }
                }
                Ok(nss)
            },
            Err(_) => Ok(Vec::new()),
        }
    })
}

fn create_mx_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=Vec<DnsLookupResultMx>, Error=()> {
    resolv.query((dname, Rtype::Mx, Class::In)).then(|result| {
        match result {
            Ok(response) => {
                let mut mxs = Vec::new();
                for record in response.answer().unwrap().limit_to::<rdata::Mx<ParsedDName>>() {
                    if record.is_ok() {
                        let mx = record.unwrap().into_data();
                        mxs.push(DnsLookupResultMx { preference: mx.preference(),
                                                     exchange: format!("{}", mx.exchange()) });
                    }
                }
                Ok(mxs)
            },
            Err(_) => Ok(Vec::new()),
        }
    })
}

fn create_txt_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=Vec<String>, Error=()> {
    resolv.query((dname, Rtype::Txt, Class::In)).then(|result| {
        match result {
            Ok(response) => {
                let mut txts = Vec::new();
                for record in response.answer().unwrap().limit_to::<rdata::Txt<&[u8]>>() {
                    if record.is_ok() {
                        let txt = record.unwrap().into_data();
                        txts.push(String::from_utf8_lossy(&txt.text().to_vec()).into_owned())
                    }
                }
                Ok(txts)
            },
            Err(_) => Ok(Vec::new()),
        }
    })
}

fn create_soa_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=Option<DnsLookupResultSoa>, Error=()> {
    resolv.query((dname, Rtype::Soa, Class::In)).then(|result| {
        match result {
            Ok(response) => {
                Ok(response.answer().unwrap().limit_to::<rdata::Soa<ParsedDName>>().next().and_then(|record| {
                    if record.is_ok() {
                        let soa = record.unwrap().into_data();
                        Some(DnsLookupResultSoa { mname: format!("{}", soa.mname()),
                                                  rname: format!("{}", soa.rname()),
                                                  serial: soa.serial(),
                                                  refresh: soa.refresh(),
                                                  retry: soa.retry(),
                                                  expire: soa.expire(),
                                                  minimum: soa.minimum() })
                    } else {
                        None
                    }
                }))
            },
            Err(_) => Ok(None),
        }
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseDnsLookupResponse {
    ip: IpAddr,
    names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsLookupResult {
    pub a: Vec<Ipv4Addr>,
    pub aaaa: Vec<Ipv6Addr>,
    pub cname: Vec<String>,
    pub ns: Vec<String>,
    pub mx: Vec<DnsLookupResultMx>,
    pub txt: Vec<String>,
    pub soa: Option<DnsLookupResultSoa>,
}

impl DnsLookupResult {
    pub fn empty() -> DnsLookupResult {
        DnsLookupResult {
            a: Vec::new(),
            aaaa: Vec::new(),
            cname: Vec::new(),
            ns: Vec::new(),
            mx: Vec::new(),
            txt: Vec::new(),
            soa: None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsLookupResultSoa {
    pub mname: String,
    pub rname: String,
    pub serial: u32,
    pub refresh: u32,
    pub retry: u32,
    pub expire: u32,
    pub minimum: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsLookupResultMx {
    pub preference: u16,
    pub exchange: String,
}
