use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use tokio::time::Instant;

pub async fn add_timing(request: Request, next: Next) -> Response {
    let started = Instant::now();
    let mut response = next.run(request).await;

    // Only add timing info on generated HTML pages
    if let Some(content_type) = &response.headers().get("Content-Type")
        && content_type.as_bytes().starts_with(b"text/html")
    {
        let millis: f64 = started.elapsed().as_micros() as f64 / 1000.0;
        let formatted_time = format!("{:.2}ms", millis);
        response.headers_mut().insert(
            "x-generation-time",
            formatted_time.clone().try_into().unwrap(),
        );
        // response.body_mut().replace("__GENERATION_TIME__", formatted_time);

        // Change the request body
        // See https://stackoverflow.com/questions/76180158/axum-middleware-to-log-the-response-body
        let (mut res_parts, res_body) = response.into_parts();

        // Reparse the HTML answer as bytes.
        let res_body = axum::body::to_bytes(res_body, usize::MAX).await.unwrap();
        let res_body = match String::from_utf8(res_body.to_vec()) {
            Ok(mut body_str) => {
                body_str = body_str.replace("__GENERATION_TIME__", &formatted_time);
                // When your encoding is chunked there can be problems without removing the header
                res_parts.headers.remove("transfer-encoding");
                // Recreate modified body
                Body::from(body_str.into_bytes())
            }
            Err(e) => {
                log::error!("timing: Failed to parse response body as UTF-8:\n{e}");
                Body::from(res_body)
            }
        };

        return Response::from_parts(res_parts, res_body);
    }

    response
}
