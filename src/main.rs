use pingora::prelude::*;
//use crate::config::config_loader;
use crate::consul::register_service;
use rs_consul::types::RegisterEntityPayload;
use std::collections::HashMap;
use futures::executor::block_on;
use core::fmt::Error;

pub mod consul;

fn main() {
  // initialize everything beforehand
  let _ = initialization();

  // this starts the server.
  let mut server = Server::new(None).unwrap();
  server.bootstrap();
  println!("Started projectlb");
  server.run_forever();
}

// all the stuff we need to do before starting load balancer
fn initialization() -> Result<(), Error>{
  // register entity (service), this is a stub - change this
  let register_entity = RegisterEntityPayload{ID: None, Node: "test".to_string(), Address: "127.0.0.1".to_string(), Datacenter: Some("dc1".to_string()), TaggedAddresses: HashMap::new(), NodeMeta: HashMap::new(), Service: None, Check: None, SkipNodeUpdate: None};

  let register_service = register_service(register_entity);
  let _ = block_on(register_service);

  // load loadbalancer configuration
  //let config = config_loader().unwrap();
  Ok(())
}
