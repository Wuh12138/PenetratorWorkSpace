use common::rule::Rule;
use penetrator::server;

fn main() {
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
        "47.92.175.152".to_string(),
        21212u16,
    );

    while true {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
