use rs_consul::Config;
use rs_consul::Consul;
use rs_consul::types::RegisterEntityPayload;
use rs_consul::ConsulError;
use hyper::Client;

// consul configuration
pub fn consul_client() -> Consul {
  // new client builder
  let client = Client::builder();
  // set configuration for consul
  let config = Config{address: "localhost".to_string(), token: None, hyper_builder: client};

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
