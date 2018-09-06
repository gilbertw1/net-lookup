use std::sync::Arc;
use std::net::IpAddr;

use futures::{future, Future};
use maxminddb::geoip2::City;

use ip::IpAsnDatabase;
use asn::AutonomousSystemNumber;
use maxmind::MaxmindDatabase;
use dns::{DnsResolverHandle, DnsLookupResult, DnsLookupResultMx, DnsLookupResultSoa};

pub fn create_lookup_handler(ip_asn_database: IpAsnDatabase, maxmind_database: MaxmindDatabase, dns_resolver_handle: DnsResolverHandle) -> LookupHandler {
    LookupHandler { ip_asn_database: Arc::new(ip_asn_database),
                    maxmind_database: Arc::new(maxmind_database),
                    dns_resolver_handle: dns_resolver_handle }
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

        dns_names_future.then(move |dns_names| {
            future::ok(
                IpLookupResult { ip: ip.clone(),
                                 asn: asn_lookup_result.and_then(|r| r.asn.clone()),
                                 geo: city_lookup_result,
                                 reverse_dns: dns_names.ok() })
        })
    }

    pub fn lookup_ip_sync(&self, ip: IpAddr) -> IpLookupResult {
        Future::wait(self.lookup_ip(ip)).unwrap()
    }

    pub fn lookup_domain(&self, domain: String) -> impl Future<Item=DomainLookupResult, Error=()> {
        let future = self.dns_resolver_handle.dns_lookup(domain.clone());
        let handler = self.clone();

        future.then(move |dns_result| {
            let dns = dns_result.unwrap_or(DnsLookupResult::empty());
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
    }

    pub fn lookup_domain_sync(&self, domain: String) -> DomainLookupResult {
        Future::wait(self.lookup_domain(domain)).unwrap()
    }
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
    pub mx: Vec<DnsLookupResultMx>,
    pub txt: Vec<String>,
    pub soa: Option<DnsLookupResultSoa>,
}
