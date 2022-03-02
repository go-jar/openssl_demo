// Why choose openssl? Because connecting via an IP address isn't supported currently in the rustls or webpki crates
// https://github.com/rustls/hyper-rustls/issues/84

use bytes::Bytes;
use futures_util::stream::StreamExt;
use hyper::client::HttpConnector;
use hyper::{Body, Client};
use hyper_openssl::HttpsConnector;
use openssl::ssl::{SslConnector, SslMethod};
use std::time::Duration;

pub struct Connection {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl Connection {
    pub async fn new_for_client(connect_timeout: Duration, verify: bool) -> Self {
        let mut connector = HttpConnector::new();
        connector.enforce_http(false);
        connector.set_connect_timeout(Some(connect_timeout));

        let mut ssl = SslConnector::builder(SslMethod::tls()).unwrap();
        if !verify {
            ssl.set_verify(openssl::ssl::SslVerifyMode::NONE);
        }

        let ssl = hyper_openssl::HttpsConnector::with_connector(connector, ssl).unwrap();
        let client = Client::builder().build::<_, Body>(ssl);
        Connection { client }
    }

    #[inline]
    pub async fn get(&self, uri: &str) -> Option<Result<Bytes, hyper::Error>> {
        match uri.parse() {
            Ok(uri) => {
                match self.client.get(uri).await {
                    Err(e) => Some(e),
                    Ok(body) => {
                        let mut body = body.into_body();
                        return body.next().await;
                    }
                };
            }
            Err(e) => log::error!("parse uri error: {:?} {:?}", uri, e),
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn req_xglink() {
        let timeout = std::time::Duration::from_millis(500);
        let conn = Connection::new_for_client(timeout, false).await;
        if let Some(res) = conn.get("https://192.168.11.122:2555/getSomething").await {
            match res {
                Err(e) => println!("get error: {:?}", e),
                Ok(body) => println!("get res: {:?}", std::str::from_utf8(&body).unwrap()),
            }
        }
    }
}
