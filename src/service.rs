use std::sync::Arc;
use std::net::IpAddr;
use std::thread;

use std::time::Instant;

use futures::future;
use futures;
use hyper::{Method, StatusCode};
use hyper::{Body, Request, Response, Server};
use hyper::service::{Service, NewService};
use hyper::rt::Future;
use hyper;
use serde_json;
use maxminddb::geoip2::City;

use ip;
use ip::IpAsnDatabase;
use asn::AutonomousSystemNumber;
use maxmind;
use dns;

pub struct LookupService { pub database: Arc<IpAsnDatabase> }

impl NewService for LookupService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Service = LookupService;
    type Future = Box<Future<Item=Self::Service, Error=Self::InitError> + Send>;
    type InitError = hyper::Error;

    fn new_service(&self) -> Self::Future {
        Box::new(futures::future::ok(Self { database: self.database.clone() }))
    }
}

impl Service for LookupService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let mut response = Response::new(Body::empty());

        let start = Instant::now();
        match (req.method(), req.uri().path()) {
            (&Method::GET, path) => {
                let ip_result = path.trim_left_matches('/').parse::<IpAddr>();
                if ip_result.is_ok() {
                    let ip = ip_result.unwrap();
                    let asn_lookup_result = ip::query_database(&self.database, ip);
                    let city_lookup_result = maxmind::lookup_city(ip);
                    let reverse_dns_result = thread::spawn(move || {
                         dns::reverse_dns_lookup(ip)
                    }).join().unwrap();
                    let lookup_response = LookupResponse { asn: asn_lookup_result.and_then(|r| r.asn.clone()), geo: city_lookup_result, reverse_dns: reverse_dns_result };
                    *response.body_mut() = Body::from(serde_json::to_string(&lookup_response).unwrap());
                } else {
                    *response.body_mut() = Body::from("Invalid IP Address.");
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                }
            },
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            },
        };

        let duration = Instant::now() - start;
        println!("duration: {:?}", duration);
        Box::new(future::ok(response))
    }
}

impl LookupService {
    pub fn start(&self) {
        println!("Starting Lookup Service\n");
        let address = "127.0.0.1:8080".parse().unwrap();
        let server = Server::bind(&address)
            .serve(LookupService { database: self.database.clone() })
            .map_err(|e| eprintln!("server error: {}", e));
        println!("Running Lookup Service at {}", address);
        hyper::rt::run(server);
    }
}

#[derive(Serialize, Debug)]
struct LookupResponse {
    asn: Option<Arc<AutonomousSystemNumber>>,
    geo: Option<City>,
    reverse_dns: Vec<String>,
}
