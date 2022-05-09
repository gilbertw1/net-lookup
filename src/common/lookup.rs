use std::net::IpAddr;
use std::sync::Arc;

use futures::{future, Future, FutureExt};
use maxminddb::geoip2::City;

use crate::asn::AutonomousSystemNumber;
use crate::dns::{DnsLookupResult, DnsLookupResultMx, DnsLookupResultSoa, DnsResolverHandle};
use crate::ip::IpAsnDatabase;
use crate::maxmind::MaxmindDatabase;

pub fn create_lookup_handler(
    ip_asn_database: IpAsnDatabase,
    maxmind_database: MaxmindDatabase,
    dns_resolver_handle: DnsResolverHandle,
) -> LookupHandler {
    LookupHandler {
        ip_asn_database: Arc::new(ip_asn_database),
        maxmind_database: Arc::new(maxmind_database),
        dns_resolver_handle,
    }
}

#[derive(Clone)]
pub struct LookupHandler {
    ip_asn_database: Arc<IpAsnDatabase>,
    maxmind_database: Arc<MaxmindDatabase>,
    dns_resolver_handle: DnsResolverHandle,
}

impl LookupHandler {
    pub async fn lookup_ip(&self, ip: IpAddr) -> IpLookupResult {
        let dns_names = self.dns_resolver_handle.reverse_dns_lookup(ip).await;
        let asn_lookup_result = self.ip_asn_database.lookup(ip).map(|r| r.clone());
        let city_lookup_result = self.maxmind_database.lookup_city(ip);

        IpLookupResult {
            ip: ip.clone(),
            asn: asn_lookup_result.and_then(|r| r.asn.clone()),
            geo: city_lookup_result,
            reverse_dns: Some(dns_names),
        }
    }

    pub async fn lookup_ip_sync(&self, ip: IpAddr) -> IpLookupResult {
        self.lookup_ip(ip).await
    }

    pub async fn lookup_domain(&self, domain: String) -> DomainLookupResult {
        let dns = self.dns_resolver_handle.dns_lookup(domain.clone()).await;
        let handler = &self.clone();

        let mut ipaddrs: Vec<IpAddr> = Vec::new();
        ipaddrs.append(&mut dns.a.clone().into_iter().map(IpAddr::from).collect());
        ipaddrs.append(&mut dns.aaaa.clone().into_iter().map(IpAddr::from).collect());

        let ips = future::join_all(ipaddrs.into_iter().map(move |ip| handler.lookup_ip(ip))).await;

        DomainLookupResult {
            domain: domain.clone(),
            ipv4: ips
                .iter()
                .map(|r| r.to_owned())
                .filter(|r| r.ip.is_ipv4())
                .collect(),
            ipv6: ips
                .iter()
                .map(|r| r.to_owned())
                .filter(|r| r.ip.is_ipv6())
                .collect(),
            cname: dns.cname,
            ns: dns.ns,
            mx: dns.mx,
            txt: dns.txt,
            soa: dns.soa,
        }
    }

    pub async fn lookup_domain_sync(&self, domain: String) -> DomainLookupResult {
        self.lookup_domain(domain).await
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
