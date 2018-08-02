use std::sync::Arc;
use std::net::IpAddr;

use futures::{future, Future};
use futures::future::Either;
use maxminddb::geoip2::City;

use ip::IpAsnDatabase;
use asn::AutonomousSystemNumber;
use maxmind::MaxmindDatabase;
use dns::{DnsResolverHandle, DnsLookupResponseMx, DnsLookupResponseSoa};

pub fn create_lookup_handler(ip_asn_database: IpAsnDatabase, maxmind_database: MaxmindDatabase, dns_resolver_handle: DnsResolverHandle) -> LookupHandler {
    LookupHandler { ip_asn_database: Arc::new(ip_asn_database),
                    maxmind_database: Arc::new(maxmind_database),
                    dns_resolver_handle: dns_resolver_handle }
}

#[derive(Serialize, Debug, Clone)]
pub struct IpLookupResult {
    ip: IpAddr,
    asn: Option<Arc<AutonomousSystemNumber>>,
    geo: Option<City>,
    reverse_dns: Option<Vec<String>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct DomainLookupResult {
    pub domain: String,
    pub ipv4: Vec<IpLookupResult>,
    pub ipv6: Vec<IpLookupResult>,
    pub cname: Vec<String>,
    pub ns: Vec<String>,
    pub mx: Vec<DnsLookupResponseMx>,
    pub txt: Vec<String>,
    pub soa: Option<DnsLookupResponseSoa>,
}

#[derive(Clone)]
pub struct LookupHandler {
    ip_asn_database: Arc<IpAsnDatabase>,
    maxmind_database: Arc<MaxmindDatabase>,
    dns_resolver_handle: DnsResolverHandle
}

impl LookupHandler {
    pub fn lookup_ip(&self, ip: IpAddr) -> impl Future<Item=IpLookupResult, Error=()> {
        let dns_names_future = self.dns_resolver_handle.reverse_dns_lookup(ip);
        let asn_lookup_result = self.ip_asn_database.lookup(ip).map(|r| r.clone());
        let city_lookup_result = self.maxmind_database.lookup_city(ip);

        if dns_names_future.is_some() {
            Either::A (
                    dns_names_future.unwrap().then(move |result| {
                        future::ok(
                            IpLookupResult { ip: ip.clone(),
                                             asn: asn_lookup_result.and_then(|r| r.asn.clone()),
                                             geo: city_lookup_result,
                                             reverse_dns: result.unwrap_or(None) })
                    }))
        } else {
            let lookup_response = IpLookupResult { ip: ip,
                                                   asn: asn_lookup_result.and_then(|r| r.asn.clone()),
                                                   geo: city_lookup_result,
                                                   reverse_dns: None };
            Either::B(future::ok(lookup_response))
        }
    }

    pub fn lookup_ip_sync(&self, ip: IpAddr) -> IpLookupResult {
        Future::wait(self.lookup_ip(ip)).unwrap()
    }

    pub fn lookup_domain(&self, domain: String) -> impl Future<Item=DomainLookupResult, Error=()> {
        let future = self.dns_resolver_handle.dns_lookup(domain.clone());
        let handler = self.clone();

        if future.is_some() {
            Either::A(
                future.unwrap().then(move |dns_result| {
                    let dns = dns_result.unwrap();
                    let mut ipaddrs: Vec<IpAddr> = Vec::new();
                    ipaddrs.append(&mut dns.a.clone().into_iter().map(IpAddr::from).collect());
                    ipaddrs.append(&mut dns.aaaa.clone().into_iter().map(IpAddr::from).collect());
                    future::join_all(ipaddrs.into_iter().map(move |ip| handler.clone().lookup_ip(ip))).then(move |ip_results| {
                        let ips = ip_results.unwrap_or(Vec::new());
                        future::ok(DomainLookupResult {
                            domain: domain.clone(),
                            ipv4: ips.iter().map(|r| r.to_owned()).filter(|r| r.ip.is_ipv4()).collect(),
                            ipv6: ips.iter().map(|r| r.to_owned()).filter(|r| r.ip.is_ipv6()).collect(),
                            cname: dns.cname,
                            ns: dns.ns,
                            mx: dns.mx,
                            txt: dns.txt,
                            soa: dns.soa,
                        })
                    })
                })
            )
        } else {
            Either::B(future::ok(DomainLookupResult { domain: domain,
                                                      ipv4: Vec::new(),
                                                      ipv6: Vec::new(),
                                                      cname: Vec::new(),
                                                      ns: Vec::new(),
                                                      mx: Vec::new(),
                                                      txt: Vec::new(),
                                                      soa: None,
            }))
        }
    }

    pub fn lookup_domain_sync(&self, domain: String) -> DomainLookupResult {
        Future::wait(self.lookup_domain(domain)).unwrap()
    }
}
