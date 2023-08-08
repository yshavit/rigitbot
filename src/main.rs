use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::collections::HashMap;

async fn handler(event: LambdaEvent<LambdaFunctionUrlRequest>) -> Result<Value, Error> {
    let path = &event.payload.raw_path;
    let headers = &event.payload.headers;
    let mut headers_map = HashMap::with_capacity(headers.len());
    for (k, v) in headers {
        if let Ok(val) = v.to_str() {
            headers_map.insert(k.as_str(), val);
        }
    }
    Ok(json!({
        "message": format!("Hello, world!"),
        "headers-echo": headers_map,
        "path": path,
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(handler)).await
}
