use pingora::prelude::*;
use crate::consul::get_service_address_ports;
use crate::consul::get_service_names;
//use crate::consul::register_service;
use core::fmt::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use regex::Regex;
//use std::collections::HashMap;
//use rs_consul::RegisterEntityPayload;
//use rs_consul::RegisterEntityService;

use async_trait::async_trait;
use pingora_core::services::background::background_service;
use std::{sync::Arc};
use structopt::StructOpt;

use pingora_core::server::configuration::Opt;
use pingora_core::server::Server;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_core::Result;
use pingora_load_balancing::{selection::RoundRobin, LoadBalancer};
use pingora_proxy::{ProxyHttp, Session};

pub mod consul;

pub struct LB(Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self
            .0
            .select(b"", 256) // hash doesn't matter
            .unwrap();

        println!("upstream peer is: {:?}", upstream);

        let peer = Box::new(HttpPeer::new(upstream, false, "one.one.one.one".to_string()));
        Ok(peer)
    }
}

fn main() {
  // initialize everything beforehand
  let upstream_servers = initialization();
  //let upstream_servers = generate_upstreams("taqisystems-site".to_string());

  // read command line arguments
  let opt = Opt::from_args();
  let mut server = Server::new(Some(opt)).unwrap();
  server.bootstrap();

  // upstreams
  let upstream_vector = &upstream_servers.unwrap();
  let upstream_iter = (&upstream_vector[..]).to_socket_addrs().unwrap();
  let mut upstreams =
      LoadBalancer::try_from_iter(upstream_iter).unwrap();

  let hc = TcpHealthCheck::new();
  upstreams.set_health_check(hc);
  upstreams.health_check_frequency = Some(std::time::Duration::from_secs(1));

  let background = background_service("health check", upstreams);
  let upstreams = background.task();

  // `upstreams` no longer need to be wrapped in an arc
  let mut lb = http_proxy_service(&server.configuration, LB(upstreams));
  lb.add_tcp("0.0.0.0:6188"); 

  server.add_service(lb);
  server.add_service(background);
  println!("Started projectlb");
  server.run_forever();
}

// all the stuff we need to do before starting load balancer
#[tokio::main]
async fn initialization() -> Result<Vec<SocketAddr>, Error> {

  // register service
  //let _ = register().await;
  
  // get all service that needs to setup for loadbalancer
  let upstreams = Vec::new();
  let service_names = get_service_names().await;
  let service_names = service_names.unwrap();
  for service_name in service_names.response.iter() {
    if service_name.1.contains(&"traefik.enable=true".to_string()) {
      println!("{:?}", service_name.0);
      for sd in service_name.1 {
        if !service_name.1.is_empty() {
          let re = Regex::new(r"(`([^)]+)`)").unwrap(); 
          if re.is_match(&sd) {
            let caps = match re.captures(&sd) {
              Some(host) => host.get(2).unwrap().as_str(),
              None => "",
            };

            println!("Host is {:?}", caps);

            let upstreams = generate_upstreams(service_name.0.to_string(), caps.to_string());
            let upstreams = upstreams.await;
            return upstreams;
          }
        }
      }
    }
  }

  Ok(upstreams)
}

// generate upstreams
async fn generate_upstreams(service_name: String, _host: String) -> Result<Vec<SocketAddr>, Error> {
  // get address and ports for specific service
  let services = get_service_address_ports(service_name);
  let services = services.await.unwrap();

  println!("{:?}", services);

  // initialize new vector for upstreams
  let mut upstreams: Vec<SocketAddr> = Vec::new();
  for service in services.iter() {

      let ip = &service.0; // ip address
      let port = &service.1.to_string(); // port number to string
      let host = format!("{ip}:{port}");
      let service: SocketAddr = host
          .parse()
          .expect("Cannot parse host address");
      upstreams.push(service);
  }

  println!("upstreams from consul: {:?}", upstreams);
  Ok(upstreams)
}

// register ourselves to consul
/*async fn register() -> Result<(), Error> {
  let service = RegisterEntityService{ID: Some("123".to_string()), Service: "projectlb".to_string(), Tags: Vec::new(), TaggedAddresses: HashMap::new(), Meta: HashMap::new(), Port: None, Namespace: None};

  let payload = RegisterEntityPayload{ID: None, Node: "test".to_string(), Address: "127.0.0.1".to_string(), Datacenter: Some("kbr1".to_string()), TaggedAddresses: HashMap::new(), NodeMeta: HashMap::new(), Service: Some(service), Check: None, SkipNodeUpdate: None};
  let _ = register_service(payload).await;

  Ok(())
}*/
