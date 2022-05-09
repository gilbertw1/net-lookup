use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};

use hyper;
use hyper::http::Result;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use serde_json;

use crate::lookup;
use lookup::LookupHandler;

#[derive(Clone)]
pub struct LookupContext {
    handler: LookupHandler,
}

async fn handle_lookup(
    context: LookupContext,
    addr: SocketAddr,
    req: Request<Body>,
) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, path) => {
            let ip_result = path.trim_start_matches('/').parse::<IpAddr>();
            if ip_result.is_ok() {
                let result = context.handler.lookup_ip(ip_result.unwrap()).await;
                Response::builder()
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&result).unwrap()))
            } else {
                let result = context
                    .handler
                    .lookup_domain(path.trim_start_matches('/').to_owned())
                    .await;
                Response::builder()
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&result).unwrap()))
            }
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub struct LookupService {
    pub handler: LookupHandler,
}

impl LookupService {
    pub async fn start(&self, host: IpAddr, port: u16) {
        let addr = SocketAddr::new(host, port);
        let make_service = make_service_fn(move |conn: &AddrStream| {
            let context = LookupContext {
                handler: self.handler.clone(),
            };
            let addr = conn.remote_addr();
            let service = service_fn(move |req| handle_lookup(context.clone(), addr, req));
            async move { Ok::<_, Infallible>(service) }
        });

        let server = Server::bind(&addr).serve(make_service);
        println!("Running Lookup Service at {}", addr);
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}
