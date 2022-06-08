use summer_boot::web2::server::server;
    
    #[tokio::main]
    async fn main() {
        let addrs = "127.0.0.1:8081".parse().unwrap();
        server::SummerApplication::bind_two(&addrs);
    }