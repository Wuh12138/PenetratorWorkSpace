pub mod server;
pub use common::control_flow;

#[cfg(test)]
mod test {
    use crate::server;
    use common::rule::Rule;

    #[test]
    fn test() {
        let mut local_server = server::LocalServer::new();
        let rule = Rule {
            protocol: "tcp".to_string(),
            port_to_pub: 12345,
            name: "test".to_string(),
            password: "test".to_string(),
        };
        local_server.add_tcpmap_with_rule(
            rule,
            "127.0.0.1".to_string(),
            5173,
            "127.0.0.1".to_string(),
            8080,
        );

        while true {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
