use pingora::prelude::*;
use etcd_client::{Client, EventType};
use futures::executor::block_on;

#[tokio::main]
async fn main() {
  async_main().await;
}

async fn async_main() {
  dynamic_config("endpoints").await;
  run_server();
}

fn run_server() {
  println!("Server is running!");
  let mut lb_server = Server::new(None).unwrap();
  lb_server.bootstrap();
  lb_server.run_forever();
}

async fn dynamic_config(key: &str) -> () {
    println!("Loading dynamic configuration");
    let mut client = Client::connect(["localhost:2379"], None).await.unwrap();
    let Ok((mut watcher, mut stream)) = client.watch(key, None).await else {
      println!("Cannot connect to etcd, exiting");
      std::process::exit(1)
    };

    while let Some(resp) = stream.message().await.unwrap() {
        for event in resp.events() {
            println!("event type: {:?}", event.event_type());
            if let Some(kv) = event.kv() {
                println!("kv: {{{}: {}}}", kv.key_str().unwrap(), kv.value_str().unwrap());
            }

            if EventType::Delete == event.event_type() {
                watcher.cancel_by_id(resp.watch_id()).await.unwrap();
            }
        }
    }

    ()
}
