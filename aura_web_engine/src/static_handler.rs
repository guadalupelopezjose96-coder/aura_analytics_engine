#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::panic,
    clippy::float_arithmetic
)]

use axum::{
    body::Body,
    http::{header, HeaderValue, Response, StatusCode},
    response::IntoResponse,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand::RngCore;

pub async fn serve_landing_with_nonce(html_template: &str) -> Response<Body> {
    let mut nonce_bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let csp_nonce = BASE64.encode(nonce_bytes);

    let processed_html = html_template
        .replace("<script", &format!("<script nonce=\"{csp_nonce}\""))
        .replace("<style", &format!("<style nonce=\"{csp_nonce}\""));

    let csp_header = format!(
        "default-src 'self'; script-src 'self' 'nonce-{csp_nonce}'; style-src 'self' 'nonce-{csp_nonce}'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'; connect-src 'self' https://api.stripe.com;"
    );

    let mut response = match Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::X_FRAME_OPTIONS, "DENY")
        .header(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=63_072_000; includeSubDomains; preload",
        )
        .body(Body::from(processed_html))
    {
        Ok(res) => res,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if let Ok(val) = HeaderValue::from_str(&csp_header) {
        response.headers_mut().insert(header::CONTENT_SECURITY_POLICY, val);
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nonce_injection_in_html() {
        let template = "<html><head><style>body{}</style></head><body><script>console.log(1);</script></body></html>";
        let res = serve_landing_with_nonce(template).await;

        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().contains_key(header::CONTENT_SECURITY_POLICY));
    }
}
