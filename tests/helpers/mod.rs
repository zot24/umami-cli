use umami_cli::api::UmamiClient;
use wiremock::MockServer;

pub fn test_client(server: &MockServer, token: Option<&str>) -> UmamiClient {
    UmamiClient::new(&server.uri(), token.map(|t| t.to_string()))
}
