use crate::thread_pool::FixedThreadPool;
use std::io::Write;
use std::net::{SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

mod thread_pool;

fn main() {
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 2020))).unwrap();

    let pool = FixedThreadPool::new(4);

    for stream in listener.incoming().take(10) {
        pool.execute(move || {
            handle_connection(stream.unwrap());
        })
        .unwrap();
    }
}

fn handle_connection(mut stream: TcpStream) {
    sleep(Duration::from_secs(3));
    stream
        .write_all(
            format!(
                "HTTP/1.1 200 OK
Content-Type: text/html;charset=UTF-8

<html>
<h1>hello, world</h1>{:?}
</html>",
                SystemTime::now()
            )
            .as_bytes(),
        )
        .unwrap();
}
