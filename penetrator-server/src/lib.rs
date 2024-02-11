pub mod authentification;
pub mod config;
pub mod server;
pub use common::control_flow;

#[cfg(test)]
mod tests {
    use crate::server;

    #[test]
    fn it_works() {
        let server = server::Server::new();
        server.run();
    }
}
