use std::net::IpAddr;
use std::thread;

use futures::{Stream, Future};
use futures::sync::oneshot;
use futures::sync::mpsc;
use domain::resolv::Resolver;
use domain::resolv::lookup::lookup_addr;
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
        DnsLookupRequest::ReverseLookup { ip, sender } => {
            lookup_addr(resolv.clone(), ip).then(|result| {
                match result {
                    Ok(addrs) => {
                        let names = addrs.iter().map(|n| n.to_string().trim_right_matches(".").to_owned()).collect::<Vec<String>>();
                        let _unused = sender.send(ReverseDnsLookupResponse { names: Some(names) });
                        Ok(())
                    },
                    Err(_) => {
                        let _unused = sender.send(ReverseDnsLookupResponse { names: None });
                        Ok(())
                    },
                }
            })
        },
    }
}


#[derive(Clone)]
pub struct DnsResolverHandle {
    request_sender: mpsc::UnboundedSender<DnsLookupRequest>,
}

enum DnsLookupRequest {
    ReverseLookup { ip: IpAddr, sender: oneshot::Sender<ReverseDnsLookupResponse>}
}

#[derive(Debug, Clone)]
struct ReverseDnsLookupResponse {
    names: Option<Vec<String>>,
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
}
