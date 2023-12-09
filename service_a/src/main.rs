use aws_lambda_events::{apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse}, encodings::Body};
use framework::{Router, run_lambda};
use lambda_runtime::{Error, LambdaEvent};
use serde_json::json;
use lazy_static::lazy_static;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let router = Router::new()
        .register("/hello_world", hello_world)
        .register("/invoke", invoke);

    run_lambda(router).await
}

async fn hello_world(_request: LambdaEvent<ApiGatewayV2httpRequest>) -> ApiGatewayV2httpResponse {
    let body = Body::Text(
        serde_json::to_string(
            &json!({
                "message": "from service_a"
            })
        ).unwrap()
    );

    ApiGatewayV2httpResponse {
        status_code: 200,
        body: Some(body),
        ..Default::default()
    }
}

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = {
        reqwest::Client::new()
    };
}

async fn invoke(_request: LambdaEvent<ApiGatewayV2httpRequest>) -> ApiGatewayV2httpResponse {
    let client = &HTTP_CLIENT;
    
    let body_bytes = client
        .get("")
        .send()
        .await
        .unwrap()
        .bytes()
        .await;

    let body = Body::Text(
        String::from_utf8(
            body_bytes.unwrap().to_vec()
        ).unwrap()
    );

    ApiGatewayV2httpResponse {
        status_code: 200,
        body: Some(body),
        ..Default::default()
    }
}