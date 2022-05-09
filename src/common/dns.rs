use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str;

use domain::base::iana::Class;
use domain::base::{Dname, ParsedDname, Question, Rtype};
use domain::rdata;
use domain::resolv::stub::conf::ServerConf;
use domain::resolv::stub::conf::{ResolvConf, Transport};
use domain::resolv::StubResolver;

pub fn create_dns_resolver_handle(host: Option<IpAddr>, port: u16) -> DnsResolverHandle {
    return DnsResolverHandle {
        resolv: create_resolver(host, port),
    };
}

fn create_resolver(resolver_host: Option<IpAddr>, resolver_port: u16) -> StubResolver {
    match resolver_host {
        Some(addr) => {
            let server_conf = ServerConf::new(SocketAddr::new(addr, resolver_port), Transport::Udp);
            let mut resolv_conf = ResolvConf::new();
            resolv_conf.servers = vec![server_conf];
            StubResolver::from_conf(resolv_conf)
        }
        None => StubResolver::new(),
    }
}

#[derive(Clone)]
pub struct DnsResolverHandle {
    resolv: StubResolver,
}

impl DnsResolverHandle {
    pub async fn reverse_dns_lookup(&self, ip: IpAddr) -> Vec<String> {
        match self.resolv.lookup_addr(ip).await {
            Ok(addrs) => addrs.iter().map(|n| n.to_string()).collect(),
            Err(err) => {
                println!("error = {:?}", err);
                Vec::new()
            }
        }
    }

    pub async fn dns_lookup(&self, domain: String) -> DnsLookupResult {
        let mut domain = domain.clone();
        if !domain.ends_with('.') {
            domain.push('.')
        }
        match Dname::<Vec<_>>::from_chars(domain.chars()) {
            Ok(dname) => create_lookup_future(self.resolv.clone(), &dname).await,
            Err(_) => DnsLookupResult::empty(),
        }
    }
}

async fn create_lookup_future(resolv: StubResolver, dname: &Dname<Vec<u8>>) -> DnsLookupResult {
    let a = create_a_lookup_future(resolv.clone(), &dname).await;
    let aaaa = create_aaaa_lookup_future(resolv.clone(), &dname).await;
    let cname = create_cname_lookup_future(resolv.clone(), &dname).await;
    let ns = create_ns_lookup_future(resolv.clone(), &dname).await;
    let mx = create_mx_lookup_future(resolv.clone(), &dname).await;
    let txt = create_txt_lookup_future(resolv.clone(), &dname).await;
    let soa = create_soa_lookup_future(resolv.clone(), &dname).await;
    DnsLookupResult {
        a,
        aaaa,
        cname,
        ns,
        mx,
        txt,
        soa,
    }
}

async fn create_a_lookup_future(resolv: StubResolver, dname: &Dname<Vec<u8>>) -> Vec<Ipv4Addr> {
    match resolv.query((dname, Rtype::A, Class::In)).await {
        Ok(response) => {
            let mut addrs = Vec::new();
            for record in response.answer().unwrap().limit_to::<rdata::A>() {
                if record.is_ok() {
                    addrs.push(record.unwrap().into_data().addr());
                }
            }
            addrs
        }
        Err(_) => Vec::new(),
    }
}

async fn create_aaaa_lookup_future(resolv: StubResolver, dname: &Dname<Vec<u8>>) -> Vec<Ipv6Addr> {
    match resolv.query((dname, Rtype::Aaaa, Class::In)).await {
        Ok(response) => {
            let mut addrs = Vec::new();
            for record in response.answer().unwrap().limit_to::<rdata::Aaaa>() {
                if record.is_ok() {
                    addrs.push(record.unwrap().into_data().addr());
                }
            }
            addrs
        }
        Err(_) => Vec::new(),
    }
}

async fn create_cname_lookup_future(resolv: StubResolver, dname: &Dname<Vec<u8>>) -> Vec<String> {
    match resolv.query((dname, Rtype::Cname, Class::In)).await {
        Ok(response) => {
            let mut cnames = Vec::new();
            for record in response
                .answer()
                .unwrap()
                .limit_to::<rdata::Cname<ParsedDname<_>>>()
            {
                if record.is_ok() {
                    let cname = record.unwrap().into_data();
                    cnames.push(format!("{}", cname));
                }
            }
            cnames
        }
        Err(_) => Vec::new(),
    }
}

async fn create_ns_lookup_future(resolv: StubResolver, dname: &Dname<Vec<u8>>) -> Vec<String> {
    match resolv.query((dname, Rtype::Ns, Class::In)).await {
        Ok(response) => {
            let mut nss = Vec::new();
            for record in response
                .answer()
                .unwrap()
                .limit_to::<rdata::Ns<ParsedDname<_>>>()
            {
                if record.is_ok() {
                    let cname = record.unwrap().into_data();
                    nss.push(format!("{}", cname));
                }
            }
            nss
        }
        Err(_) => Vec::new(),
    }
}

async fn create_mx_lookup_future(
    resolv: StubResolver,
    dname: &Dname<Vec<u8>>,
) -> Vec<DnsLookupResultMx> {
    match resolv.query((dname, Rtype::Mx, Class::In)).await {
        Ok(response) => {
            let mut mxs = Vec::new();
            for record in response
                .answer()
                .unwrap()
                .limit_to::<rdata::Mx<ParsedDname<_>>>()
            {
                if record.is_ok() {
                    let mx = record.unwrap().into_data();
                    mxs.push(DnsLookupResultMx {
                        preference: mx.preference(),
                        exchange: format!("{}", mx.exchange()),
                    });
                }
            }
            mxs
        }
        Err(_) => Vec::new(),
    }
}

async fn create_txt_lookup_future(resolv: StubResolver, dname: &Dname<Vec<u8>>) -> Vec<String> {
    match resolv.query((dname, Rtype::Txt, Class::In)).await {
        Ok(response) => {
            let mut txts = Vec::new();
            for record in response.answer().unwrap().limit_to::<rdata::Txt<_>>() {
                if record.is_ok() {
                    let txt = record.unwrap().into_data();
                    txts.push(txt.to_string())
                }
            }
            txts
        }
        Err(_) => Vec::new(),
    }
}

async fn create_soa_lookup_future(
    resolv: StubResolver,
    dname: &Dname<Vec<u8>>,
) -> Option<DnsLookupResultSoa> {
    match resolv.query((dname, Rtype::Soa, Class::In)).await {
        Ok(response) => response
            .answer()
            .unwrap()
            .limit_to::<rdata::Soa<ParsedDname<_>>>()
            .next()
            .and_then(|record| {
                if record.is_ok() {
                    let soa = record.unwrap().into_data();
                    Some(DnsLookupResultSoa {
                        mname: format!("{}", soa.mname()),
                        rname: format!("{}", soa.rname()),
                        serial: soa.serial().into_int(),
                        refresh: soa.refresh(),
                        retry: soa.retry(),
                        expire: soa.expire(),
                        minimum: soa.minimum(),
                    })
                } else {
                    None
                }
            }),
        Err(_) => None,
    }
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
            soa: None,
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
