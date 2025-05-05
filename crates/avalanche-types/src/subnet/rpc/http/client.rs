use std::io::{self, Error, ErrorKind};

use crate::{proto::pb, subnet};
use prost::bytes::Bytes;
use tonic::transport::Channel;

/// Client which interacts with gRPC HTTP service
pub struct Client {
    /// The inner gRPC HTTP client
    inner: pb::http::http_client::HttpClient<Channel>,
}

impl Client {
    /// Creates a new HTTP handler from a channel connection
    #[must_use]
    pub fn new_handler(client_conn: Channel) -> Box<dyn subnet::rpc::http::Handler + Send + Sync> {
        Box::new(Self {
            inner: pb::http::http_client::HttpClient::new(client_conn)
                .max_decoding_message_size(usize::MAX)
                .max_encoding_message_size(usize::MAX),
        })
    }
}

#[tonic::async_trait]
impl subnet::rpc::http::Handler for Client {
    async fn serve_http(
        &mut self,
        _req: http::Request<Vec<u8>>,
    ) -> io::Result<http::Response<Vec<u8>>> {
        Err(Error::new(ErrorKind::Other, "not implemented"))
    }

    /// HTTP client takes an HTTP request and sends to server. Does not support websockets.
    async fn serve_http_simple(
        &mut self,
        req: http::Request<Vec<u8>>,
    ) -> io::Result<http::Response<Vec<u8>>> {
        let req = get_http_simple_request(&req);

        let resp = self.inner.handle_simple(req).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("handle simple request failed: {e:?}"),
            )
        })?;

        Ok(get_http_response(resp.into_inner()))
    }
}

/// Convert from [`http::Request`] to [`pb::http::HandleSimpleHttpRequest`]
fn get_http_simple_request(req: &http::Request<Vec<u8>>) -> pb::http::HandleSimpleHttpRequest {
    let headers = convert_to_proto_headers(req.headers());

    pb::http::HandleSimpleHttpRequest {
        method: req.method().to_string(),
        url: req.uri().to_string(),
        body: Bytes::from(req.body().to_owned()),
        headers,
    }
}

/// Convert from [`pb::http::HandleSimpleHttpResponse`] to [`http::Response`]
///
/// # Panics
///
/// Panics if the response builder fails to build a valid HTTP response.
fn get_http_response(resp: pb::http::HandleSimpleHttpResponse) -> http::Response<Vec<u8>> {
    // Use try_from to safely convert i32 to u16
    let status_code = u16::try_from(resp.code).unwrap_or(500);
    let mut http_resp = http::Response::builder().status(status_code);

    for header in resp.headers {
        http_resp = http_resp.header(header.key, header.values.concat());
    }

    http_resp.body(resp.body.to_vec()).unwrap_or_else(|e| {
        // If we can't build the response, create a 500 error response
        http::Response::builder()
            .status(500)
            .body(format!("failed to generate http response: {e:?}").into_bytes())
            .unwrap()
    })
}

/// Converts [`http::HeaderMap`] to a vec of elements that avalanche proto can use
fn convert_to_proto_headers(
    headers: &http::HeaderMap<http::HeaderValue>,
) -> Vec<pb::http::Element> {
    let mut vec_headers: Vec<pb::http::Element> = Vec::with_capacity(headers.keys_len());
    for (key, value) in headers {
        let element = pb::http::Element {
            key: key.to_string(),
            values: vec![String::from_utf8_lossy(value.as_bytes()).to_string()],
        };
        vec_headers.push(element);
    }
    vec_headers
}
