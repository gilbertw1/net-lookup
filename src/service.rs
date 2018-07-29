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

use ip;
use ip::IpAsnDatabase;
use asn::AutonomousSystemNumber;
use maxmind;
use dns::DnsResolverHandle;

pub struct LookupService { pub database: Arc<IpAsnDatabase>, pub dns_resolver_handle: DnsResolverHandle }

impl NewService for LookupService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Service = LookupService;
    type Future = Box<Future<Item=Self::Service, Error=Self::InitError> + Send>;
    type InitError = hyper::Error;

    fn new_service(&self) -> Self::Future {
        Box::new(futures::future::ok(Self { database: self.database.clone(), dns_resolver_handle: self.dns_resolver_handle.clone() }))
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
                let start = Instant::now();
                let ip_result = path.trim_left_matches('/').parse::<IpAddr>();
                if ip_result.is_ok() {
                    let ip = ip_result.unwrap();
                    
                    let dns_names_future = self.dns_resolver_handle.reverse_dns_lookup(ip);
                    let asn_lookup_result = ip::query_database(&self.database, ip).map(|r| r.clone());
                    let city_lookup_result = maxmind::lookup_city(ip).clone();

                    Box::new(
                        dns_names_future.then(move |result| {
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
                    Box::new(
                        future::ok(
                            Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from("Invalid IP Address."))
                                .unwrap()))
                }
            },
            _ => {
                Box::new(
                    future::ok(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from(""))
                            .unwrap()))
            },
        }
    }
}

impl LookupService {
    pub fn start(database: IpAsnDatabase, resolver_handle: DnsResolverHandle) {
        let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        let shared_db = Arc::new(database);
        let server = listener.incoming()
                             .map_err(|e| println!("error = {:?}", e))
                             .for_each(move |stream| {
                                 let future = Http::new()
                                     .serve_connection(stream, LookupService { database: shared_db.clone(), dns_resolver_handle: resolver_handle.clone() })
                                     .map_err(|e| eprintln!("server error: {}", e));
                                 tokio::spawn(future);
                                 Ok(())
                             });
        println!("Running Lookup Service at {}", addr);
        tokio::run(server);
    }
}

#[derive(Serialize, Debug)]
struct LookupResponse {
    asn: Option<Arc<AutonomousSystemNumber>>,
    geo: Option<City>,
    reverse_dns: Option<Vec<String>>,
}
