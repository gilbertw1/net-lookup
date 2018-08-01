use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::thread;
use std::str;

use futures::{Stream, Future};
use futures::sync::oneshot;
use futures::sync::mpsc;
use futures::future::Either;
use futures::future;
use domain::resolv::Resolver;
use domain::resolv::lookup::lookup_addr;
use domain::iana::{Rtype,Class};
use domain::bits::{DNameBuf, ParsedDName};
use domain::rdata;
use tokio_core::reactor::Core;

pub fn create_dns_resolver() -> DnsResolverHandle {
    let request_sender = start_dns_resolver_loop();
    DnsResolverHandle { request_sender }
}

fn start_dns_resolver_loop() -> mpsc::UnboundedSender<DnsLookupRequest> {
    let (req_tx, req_rx) = mpsc::unbounded::<DnsLookupRequest>();

    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        let core_handle = core.handle();
        let resolv = Resolver::new(&core.handle());        
        let resolver_loop =
            req_rx.map_err(|e| println!("error = {:?}", e))
                  .for_each(move |request| {
                      let future = handle_dns_lookup_request(request, resolv.clone());
                      core_handle.spawn(future);
                      Ok(())
                  });

        core.run(resolver_loop).expect("[dns] Failed to start reactor core loop.");
    });

    req_tx
}

fn handle_dns_lookup_request(request: DnsLookupRequest, resolv: Resolver) -> impl Future<Item=(), Error=()> {
    match request {
        DnsLookupRequest::Lookup { domain, sender } => {
            let mut domain = domain.clone();
            if  !domain.ends_with('.') {
                domain.push('.')
            }
            Either::A(
                create_lookup_future_maybe(resolv.clone(), domain).then(|response| {

                    let _result = sender.send(response.unwrap());
                    Ok(())
                }))
        },
        DnsLookupRequest::ReverseLookup { ip, sender } => {
            Either::B(
                create_reverse_ip_lookup_future(resolv.clone(), ip).then(|names| {
                    let _result = sender.send(ReverseDnsLookupResponse { names: names.unwrap() });
                    Ok(())
                }))
        },
    }
}

fn create_lookup_future_maybe(resolv: Resolver, domain: String) -> impl Future<Item=DnsLookupResponse, Error=()> {
    let dname_result = domain.parse::<DNameBuf>();
    if dname_result.is_err() {
        Either::A(future::ok(DnsLookupResponse::default()))
    } else {
        Either::B(create_lookup_future(resolv.clone(), &dname_result.unwrap()))
    }
}

fn create_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=DnsLookupResponse, Error=()> {
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
                Ok(DnsLookupResponse {a, aaaa, cname, ns, mx, txt, soa}),
            Err(_) =>
                Ok(DnsLookupResponse::default())
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

fn create_mx_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=Vec<DnsLookupResponseMx>, Error=()> {
    resolv.query((dname, Rtype::Mx, Class::In)).then(|result| {
        match result {
            Ok(response) => {
                let mut mxs = Vec::new();
                for record in response.answer().unwrap().limit_to::<rdata::Mx<ParsedDName>>() {
                    if record.is_ok() {
                        let mx = record.unwrap().into_data();
                        mxs.push(DnsLookupResponseMx { preference: mx.preference(),
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

fn create_soa_lookup_future(resolv: Resolver, dname: &DNameBuf) -> impl Future<Item=Option<DnsLookupResponseSoa>, Error=()> {
    resolv.query((dname, Rtype::Soa, Class::In)).then(|result| {
        match result {
            Ok(response) => {
                Ok(response.answer().unwrap().limit_to::<rdata::Soa<ParsedDName>>().next().and_then(|record| {
                    if record.is_ok() {
                        let soa = record.unwrap().into_data();
                        Some(DnsLookupResponseSoa { mname: format!("{}", soa.mname()),
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

fn create_reverse_ip_lookup_future(resolv: Resolver, ip: IpAddr) -> impl Future<Item=Option<Vec<String>>, Error=()> {
    lookup_addr(resolv, ip).then(|result| {
        match result {
            Ok(addrs) => {
                let names = addrs.iter().map(|n| n.to_string().trim_right_matches(".").to_owned()).collect::<Vec<String>>();
                Ok(Some(names))
            },
            Err(_) => Ok(None),
        }
    })
}


#[derive(Clone)]
pub struct DnsResolverHandle {
    request_sender: mpsc::UnboundedSender<DnsLookupRequest>,
}

enum DnsLookupRequest {
    ReverseLookup { ip: IpAddr, sender: oneshot::Sender<ReverseDnsLookupResponse>},
    Lookup { domain: String, sender: oneshot::Sender<DnsLookupResponse> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseDnsLookupResponse {
    names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsLookupResponse {
    a: Vec<Ipv4Addr>,
    aaaa: Vec<Ipv6Addr>,
    cname: Vec<String>,
    ns: Vec<String>,
    mx: Vec<DnsLookupResponseMx>,
    txt: Vec<String>,
    soa: Option<DnsLookupResponseSoa>,
}

impl DnsLookupResponse {
    pub fn default() -> DnsLookupResponse {
        DnsLookupResponse {
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
pub struct DnsLookupResponseSoa {
    mname: String,
    rname: String,
    serial: u32,
    refresh: u32,
    retry: u32,
    expire: u32,
    minimum: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsLookupResponseMx {
    preference: u16,
    exchange: String,
}

impl DnsResolverHandle {
    pub fn reverse_dns_lookup(&self, ip: IpAddr) -> Option<impl Future<Item=Option<Vec<String>>, Error=oneshot::Canceled>> {
        let (resp_tx, resp_rx) = oneshot::channel::<ReverseDnsLookupResponse>();
        let result = self.request_sender.unbounded_send(DnsLookupRequest::ReverseLookup { ip: ip, sender: resp_tx });
        if result.is_ok() {
            Some(resp_rx.map(|res| res.names))
        } else {
            None
        }
    }

    pub fn dns_lookup(&self, domain: String) -> Option<impl Future<Item=DnsLookupResponse, Error=oneshot::Canceled>> {
        let (resp_tx, resp_rx) = oneshot::channel::<DnsLookupResponse>();
        let result = self.request_sender.unbounded_send(DnsLookupRequest::Lookup { domain: domain, sender: resp_tx });
        if result.is_ok() {
            Some(resp_rx)
        } else {
            None
        }
    }
}
