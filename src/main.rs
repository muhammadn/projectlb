use pingora::prelude::*;
use crate::consul::get_service_address_ports;
use core::fmt::Error;
use std::net::{SocketAddr, ToSocketAddrs};

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

pub mod config;
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

  // read command line arguments
  let opt = Opt::from_args();
  let mut server = Server::new(Some(opt)).unwrap();
  server.bootstrap();

  // Note that upstreams needs to be declared as `mut` now
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
async fn initialization() -> Result<Vec<SocketAddr>, Error>{
  // get address and ports for specific service
  let services = get_service_address_ports("<consul service name>").await;
  let services = services.unwrap();

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
  println!("initialization has finished!");

  // load loadbalancer configuration
  //let config = config_loader().unwrap();
  Ok(upstreams)
}
