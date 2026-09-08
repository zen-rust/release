/// Client builder using the zen-release user agent, used
/// to identify zen-release to external http servers,
/// such as GitHub and crates.io.
pub fn http_client_builder() -> reqwest::ClientBuilder {
  let user_agent = format!("zen-release/{}", env!("CARGO_PKG_VERSION"));
  reqwest::Client::builder().user_agent(user_agent)
}
