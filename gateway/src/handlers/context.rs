use axum::body::Body;
use axum::http::Uri;
use axum::middleware::Next;
use tower_request_id::RequestId;

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub uri: Uri,
    pub request_id: String,
}

pub async fn context_middleware(request: axum::http::Request<Body>, next: Next) -> axum::response::Response {
    let uri = request.uri().clone();
    let request_ext = request.extensions().clone();
    let request_id = request_ext.get::<RequestId>().map(|r| &r.0).unwrap();

    let mut response = next.run(request).await;

    response.extensions_mut().insert(RequestContext {
        uri,
        request_id: request_id.to_string(),
    });

    response
}
