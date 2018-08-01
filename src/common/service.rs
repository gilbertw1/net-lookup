use std::net::{IpAddr,SocketAddr};

use futures::{Future,future};
use futures;
use futures::*;
use hyper::{Method, StatusCode};
use hyper::{Body, Request, Response};
use hyper::service::{Service, NewService};
use hyper::server::conn::Http;
use hyper;
use serde_json;
use tokio;
use tokio::net::TcpListener;

use lookup::LookupHandler;

pub struct LookupService {
    handler: LookupHandler
}

impl NewService for LookupService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Service = LookupService;
    type Future = Box<Future<Item=Self::Service, Error=Self::InitError> + Send>;
    type InitError = hyper::Error;

    fn new_service(&self) -> Self::Future {
        Box::new(futures::future::ok(Self { handler: self.handler.clone() }))
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
    pub fn start(host: IpAddr, port: u16, handler: LookupHandler) {
        let addr = SocketAddr::new(host, port);
        let listener = TcpListener::bind(&addr).unwrap();
        let server = listener.incoming()
                             .map_err(|e| println!("error = {:?}", e))
                             .for_each(move |stream| {
                                 let future = Http::new()
                                     .serve_connection(stream, LookupService { handler: handler.clone() })
                                     .map_err(|e| eprintln!("server error: {}", e));
                                 tokio::spawn(future);
                                 Ok(())
                             });
        println!("Running Lookup Service at {}", addr);
        tokio::run(server);
    }

    fn handle_ip_lookup(&self, ip: IpAddr) -> <LookupService as Service>::Future {
        Box::new(
            self.handler.lookup_ip(ip).then(move |result | {
                future::ok(
                    Response::builder()
                        .body(Body::from(serde_json::to_string(&result.unwrap()).unwrap()))
                        .unwrap())
            }))
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
