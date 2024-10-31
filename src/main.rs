mod server;
mod threading;
mod model;

use std::net::{SocketAddr, TcpListener};

use server::handle_connection;
use threading::ThreadPool;

fn main() {
    println!("Logs :");

    let address = SocketAddr::from(([127, 0, 0, 1], 4221));
    let listener = TcpListener::bind(&address).unwrap();
    let thread_pool = ThreadPool::new(5);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread_pool.execute(move || {
                    handle_connection(stream)
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}