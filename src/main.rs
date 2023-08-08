use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::str::FromStr;
use rocket::http::Method;
use rocket::local::asynchronous::Client;

#[macro_use] extern crate rocket;

async fn handler(client: &Client, event: LambdaEvent<LambdaFunctionUrlRequest>) -> Result<Value, Error> {
    let Some(method) = event.payload.request_context.http.method else {
        return Err("couldn't determine HTTP method".into());
    };
    let Ok(method) = Method::from_str(&method) else {
        return Err("couldn't determine HTTP method".into());
    };
    let Some(path) = &event.payload.raw_path else {
        return Err("couldn't determine path".into());
    };
    let builder = match method {
        Method::Get => Client::get,
        Method::Put => Client::put,
        Method::Post => Client::post,
        Method::Delete => Client::delete,
        Method::Options => Client::options,
        Method::Head => Client::head,
        Method::Patch => Client::patch,
        _ => panic!("TODO")
    };
    let result = builder(&client, path).dispatch().await;
    let result_status = (&result.status()).to_string();
    let result_body = result.into_string().await.unwrap();


    let path = &event.payload.raw_path;
    Ok(json!({
        "message": format!("Hello, world!"),
        "method": method,
        "path": path,
        "result":     result_status,
        "result_msg": result_body,
    }))
}

#[get("/")]
fn hello() -> &'static str {
    "Hello, world! It's me!"
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let rocket = rocket::build().mount("/", routes![hello]);
    let client = Client::untracked(rocket).await?;
    lambda_runtime::run(service_fn(|event| {
        handler(&client, event)
    })).await
}
