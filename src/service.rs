use std::sync::Arc;
use std::net::{IpAddr,SocketAddr};
use std::time::Instant;

use futures::{Future,future};
use futures;
use futures::*;
use hyper::{Method, StatusCode};
use hyper::{Body, Request, Response};
use hyper::service::{Service, NewService};
use hyper::server::conn::Http;
use hyper;
use serde_json;
use maxminddb::geoip2::City;
use tokio;
use tokio::net::TcpListener;

use ip::IpAsnDatabase;
use asn::AutonomousSystemNumber;
use maxmind::MaxmindDatabase;
use dns::DnsResolverHandle;

pub struct LookupService {
    pub database: Arc<IpAsnDatabase>,
    pub maxmind_database: Arc<MaxmindDatabase>,
    pub dns_resolver_handle: DnsResolverHandle
}

impl NewService for LookupService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Service = LookupService;
    type Future = Box<Future<Item=Self::Service, Error=Self::InitError> + Send>;
    type InitError = hyper::Error;

    fn new_service(&self) -> Self::Future {
        Box::new(futures::future::ok(Self { database: self.database.clone(),
                                            maxmind_database: self.maxmind_database.clone(),
                                            dns_resolver_handle: self.dns_resolver_handle.clone() }))
    }
}

impl Service for LookupService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        match (req.method(), req.uri().path()) {
            (&Method::GET, path) => {
                let ip_result = path.trim_left_matches('/').parse::<IpAddr>();
                if ip_result.is_ok() {
                    self.handle_ip_lookup(ip_result.unwrap())
                } else {
                    Self::create_response(StatusCode::BAD_REQUEST, Body::from("Invalid IP Address."))
                }
            },
            _ => {
                Self::create_response(StatusCode::NOT_FOUND, Body::from(""))
            },
        }
    }
}

impl LookupService {
    pub fn start(host: IpAddr, port: u16, ip_asn_database: IpAsnDatabase, maxmind_database: MaxmindDatabase, resolver_handle: DnsResolverHandle) {
        let addr = SocketAddr::new(host, port);
        let listener = TcpListener::bind(&addr).unwrap();
        let atomic_ip_asn_db = Arc::new(ip_asn_database);
        let atomic_maxmind_db = Arc::new(maxmind_database);
        let server = listener.incoming()
                             .map_err(|e| println!("error = {:?}", e))
                             .for_each(move |stream| {
                                 let future = Http::new()
                                     .serve_connection(stream, LookupService { database: atomic_ip_asn_db.clone(),
                                                                               maxmind_database: atomic_maxmind_db.clone(),
                                                                               dns_resolver_handle: resolver_handle.clone() })
                                     .map_err(|e| eprintln!("server error: {}", e));
                                 tokio::spawn(future);
                                 Ok(())
                             });
        println!("Running Lookup Service at {}", addr);
        tokio::run(server);
    }

    fn handle_ip_lookup(&self, ip: IpAddr) -> <LookupService as Service>::Future {
        let start = Instant::now();
        let dns_names_future = self.dns_resolver_handle.reverse_dns_lookup(ip);
        let asn_lookup_result = self.database.lookup(ip).map(|r| r.clone());
        let city_lookup_result = self.maxmind_database.lookup_city(ip);

        if dns_names_future.is_some() {
            Box::new(
                dns_names_future.unwrap().then(move |result| {
                    let reverse_dns = result.unwrap_or(None);
                    let lookup_response = LookupResponse { asn: asn_lookup_result.and_then(|r| r.asn.clone()),
                                                           geo: city_lookup_result,
                                                           reverse_dns: reverse_dns };
                    let time = Instant::now() - start;
                    println!("RunTime: {:?}", time);
                    future::ok(
                        Response::builder()
                            .body(Body::from(serde_json::to_string(&lookup_response).unwrap()))
                            .unwrap())
                }))
        } else {
            let lookup_response = LookupResponse { asn: asn_lookup_result.and_then(|r| r.asn.clone()),
                                                   geo: city_lookup_result,
                                                   reverse_dns: None };
            Self::create_response(StatusCode::OK, Body::from(serde_json::to_string(&lookup_response).unwrap()))
        }
    }

    fn create_response(code: StatusCode, body: Body) -> <LookupService as Service>::Future {
        Box::new(
            future::ok(
                Response::builder()
                    .status(code)
                    .body(body)
                    .unwrap()))
    }
}

#[derive(Serialize, Debug)]
struct LookupResponse {
    asn: Option<Arc<AutonomousSystemNumber>>,
    geo: Option<City>,
    reverse_dns: Option<Vec<String>>,
}
