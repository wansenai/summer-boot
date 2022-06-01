#[cfg(test)]
mod tests {
    use summer_boot::web::server::server;


    #[test]
    fn test() {
        assert_eq!(4 + 4, 8);
    }

    #[test]
    fn test_socket() {
        let sa = server::SummerApplication {
            backlog: 5,
        };

        server::SummerApplication::run(sa, "127.0.0.1:8080");
    }
}
