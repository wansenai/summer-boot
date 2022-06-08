use std::{
    cmp, io,
    net,
    sync::{Arc, Mutex},
    time::Duration, pin::Pin,
};

use std::net::{SocketAddr, TcpListener as StdTcpListener};
use socket2::{Domain, Protocol, Socket, Type};

use tokio::net::TcpListener;
use tokio::time::Sleep;

struct Config {
    host: Option<String>,
    client_request_timeout: Duration,
    client_disconnect_timeout: Duration,
}

pub struct SummerApplication {
    // config: Arc<Mutex<Config>>,
    pub backlog: u32,
    pub addr: SocketAddr,
    pub listener: TcpListener,
    pub sleep_on_errors: bool,
    pub tcp_keepalive_timeout: Option<Duration>,
    pub tcp_nodelay: bool,
    pub timeout: Option<Pin<Box<Sleep>>>,
}

impl SummerApplication {
    pub fn run<T: net::ToSocketAddrs>(mut self, address: T) -> io::Result<Self> {
        let sockets = self.bind(address).unwrap();

        for lst in sockets {
            self = self.listen(lst).unwrap();
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
            println!("success");
            Ok(sockets)
        } else if let Some(e) = error.take() {
            Err(e)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "无法绑定地址"))
        }
    }

    pub fn listen(mut self, lst: net::TcpListener) -> io::Result<Self> {
        Ok(self)
    }

    pub fn new(addr: &SocketAddr) -> Self{
        let std_listener = StdTcpListener::bind(addr).unwrap();

        SummerApplication::from_std(std_listener)
    }

    pub fn from_std(std_listener: StdTcpListener) -> Self{
        std_listener
            .set_nonblocking(true).unwrap();
        let listener = TcpListener::from_std(std_listener).unwrap();
        SummerApplication::from_listener(listener)
    }

    pub fn bind_two(addr: &SocketAddr) -> Self{
        SummerApplication::new(addr)
    }

    pub fn from_listener(listener: TcpListener) -> Self{
        let addr = listener.local_addr().unwrap();
        SummerApplication {
            backlog: 5,
            listener,
            addr,
            sleep_on_errors: true,
            tcp_keepalive_timeout: None,
            tcp_nodelay: false,
            timeout: None,
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn set_keepalive(&mut self, keepalive: Option<Duration>) -> &mut Self {
        self.tcp_keepalive_timeout = keepalive;
        self
    }

    pub fn set_nodelay(&mut self, enabled: bool) -> &mut Self {
        self.tcp_nodelay = enabled;
        self
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