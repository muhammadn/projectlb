use rs_consul::Config;
use rs_consul::Consul;
use rs_consul::types::RegisterEntityPayload;
use rs_consul::types::ResponseMeta;
use rs_consul::ConsulError;
use hyper::Client;
use std::collections::HashMap;

// consul configuration
pub fn consul_client() -> Consul {
  // new client builder
  let client = Client::builder();
  // set configuration for consul
  let config = Config{address: "http://consul.server:8500".to_string(), token: None, hyper_builder: client};

  // initialise consul
  let consul = Consul::new(config);
  consul
}

// register service
pub async fn register_service(register_entity_payload: RegisterEntityPayload) -> Result<(), ConsulError> {
  let consul = consul_client();
  let register = consul.register_entity(&register_entity_payload).await;

  Ok(register?)
}

pub async fn get_service_address_ports(service: String) -> Result<Vec<(String, u16)>, ConsulError> {
  let consul = consul_client();
  let service = consul.get_service_addresses_and_ports(&service, None).await;

  Ok(service?)
}

pub async fn get_service_names() -> Result<ResponseMeta<HashMap<std::string::String, Vec<String>>>, ConsulError>{
  let consul = consul_client();
  let service_names = consul.get_all_registered_service_names(None).await;

  Ok(service_names?)
}
