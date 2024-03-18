use pingora::prelude::*;

fn main() {
    println!("Starting <name here>");
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();
    my_server.run_forever();
}
