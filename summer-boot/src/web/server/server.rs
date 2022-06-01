use std::{
    cmp, io,
    net,
    sync::{Arc, Mutex},
    time::Duration,
};

use socket2::{Domain, Protocol, Socket, Type};

struct Config {
    host: Option<String>,
    client_request_timeout: Duration,
    client_disconnect_timeout: Duration,
}

pub struct SummerApplication {
    // config: Arc<Mutex<Config>>,
    pub backlog: u32,
}

impl SummerApplication {
    pub fn run<T: net::ToSocketAddrs>(mut self, address: T) -> io::Result<Self> {
        let sockets = self.bind(address).unwrap();

        for lst in sockets {
            // self = self.listen(lst).unwrap();
        }

        Ok(self)
    }

    fn bind<T: net::ToSocketAddrs>(&self, address: T) -> io::Result<Vec<net::TcpListener>> {
        let mut error = None;
        let mut success = false;
        let mut sockets = Vec::new();

        for address in address.to_socket_addrs().unwrap() {
            match create_tcp_listener(address, self.backlog) {
                Ok(lst) => {
                    success = true;
                    sockets.push(lst);
                }
                Err(e) => error = Some(e),
            }
        }

        if success {
            Ok(sockets)
        } else if let Some(e) = error.take() {
            Err(e)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "无法绑定地址"))
        }
    }
}

fn create_tcp_listener(address: net::SocketAddr, backlog: u32) -> io::Result<net::TcpListener> {
    let domain = Domain::for_address(address);
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).unwrap();

    socket.set_reuse_address(true).unwrap();
    socket.bind(&address.into()).unwrap();

    let backlog = cmp::min(backlog, i32::MAX as u32) as i32;
    socket.listen(backlog).unwrap();
    Ok(net::TcpListener::from(socket))
}

