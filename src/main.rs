use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::str::FromStr;
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventPayload};
use rocket::http::{Method, Status};
use rocket::local::asynchronous::Client;
use rocket::outcome::Outcome::{Failure, Success};
use rocket::Request;
use rocket::request::{FromRequest, Outcome};

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

#[post("/github/event", data = "<body>")]
fn gh_event(event: GhEventType<'_>, body: &[u8]) {
    let event = WebhookEvent::try_from_header_and_body(event.header_value, body).unwrap();
    match event.specific {
        WebhookEventPayload::Ping(p) => {
            let hook_id = p.hook_id;
            let zen = p.zen;
            println!("heard a ping with hook_id={hook_id:?}, zen={zen:?}")
        }
        _ => {
            let event_kind = event.kind;
            println!("couldn't handle event of type {event_kind:?}")
        }
    }
}

struct GhEventType<'r> {
    pub header_value: &'r str
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GhEventType<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("X-GitHub-Event") {
            Some(header_value) => Success(GhEventType{ header_value}),
            None => Failure((Status::BadRequest, ())),
        }
    }
}

#[get("/hello")]
fn hello() -> &'static str {
    "Hello, world! It's me!"
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let rocket = rocket::build().mount("/", routes![gh_event, hello]);
    let client = Client::untracked(rocket).await?;
    lambda_runtime::run(service_fn(move |event| {
        handler(&client, event)
    })).await
}
