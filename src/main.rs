use std::str::FromStr;
use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventPayload};
use rocket::http::{Header, Method, Status};
use rocket::local::asynchronous::Client;
use rocket::outcome::Outcome::{Failure, Success};
use rocket::{Request};
use rocket::request::{FromRequest, Outcome};
use serde::Serialize;

#[macro_use]
extern crate rocket;

#[tracing::instrument(skip(event), fields(req_id = %event.context.request_id))]
async fn handler(
    client: &Client,
    event: LambdaEvent<LambdaFunctionUrlRequest>,
) -> Result<Response, Error> {
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
        _ => {
            return Err("couldn't determine method".into());
        },
    };
    let mut local_handler = builder(&client, path);
    for (name, value) in event.payload.headers {
        let Some(name) = name else {
            continue;
        };
        let h = Header::new(name.to_string(), value.to_str().unwrap().to_string());
        local_handler.add_header(h);
    }
    local_handler.set_body(event.payload.body.unwrap());
    let result = local_handler.dispatch().await;
    let status_code = *(&result.status().code);
    let body = result.into_string().await.unwrap_or_default();

    return Ok(Response{
        status_code,
        body,
    });
}

#[post("/github/event", data = "<body>")]
fn gh_event(event: GhEventType<'_>, body: Vec<u8>) {
    let event = WebhookEvent::try_from_header_and_body(event.header_value, &body).unwrap();
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
    pub header_value: &'r str,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GhEventType<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("X-GitHub-Event") {
            Some(header_value) => Success(GhEventType { header_value }),
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
    tracing_subscriber::fmt().json()
        .with_max_level(tracing::Level::INFO)
        .with_current_span(false) // remove duplicated information in the log.
        .without_time() // CloudWatch will add the ingestion time.
        .with_target(false) // remove the name of the function from every entry
        .init();

    let rocket = rocket::build().mount("/", routes![gh_event, hello]);
    let client = Client::untracked(rocket).await?;
    let client_ref = &client;
    lambda_runtime::run(service_fn(move |event| {
        handler(client_ref, event)
    })).await
}

/// From [the AWS docs][1]:
///
/// [1]: https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html#http-api-develop-integrations-lambda.proxy-format
///
/// ```
/// {
///     "isBase64Encoded": true|false,
///     "statusCode": httpStatusCode,
///     "headers": { "headername": "headervalue", ... },
///     "multiValueHeaders": { "headername": ["headervalue", "headervalue2", ...], ... },
///     "body": "..."
/// }
/// ```
///
/// where:
///
/// * `isBase64Encoded` defaults to false
/// * header `content-type` defaults to `application/json`
#[derive(Serialize)]
struct Response {
    #[serde(rename = "statusCode")]
    status_code: u16,
    body: String,
}
