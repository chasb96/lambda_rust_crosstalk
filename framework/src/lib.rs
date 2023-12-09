#![allow(async_fn_in_trait)]
mod router;

pub use router::Router;

use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use lambda_runtime::{Error, service_fn, LambdaEvent};

pub async fn run_lambda(router: Router) -> Result<(), Error> {
    lambda_runtime::run(
        service_fn(
            |event: LambdaEvent<ApiGatewayV2httpRequest>| async {
                let response = match &event.payload.raw_path {
                    Some(path) => router.invoke(path.to_string(), event).await,
                    None => not_found(),
                };

                Ok::<ApiGatewayV2httpResponse, Error>(response)
            }
        )
    ).await
}

pub fn no_content() -> ApiGatewayV2httpResponse { status_code_response(204) }

pub fn bad_request() -> ApiGatewayV2httpResponse { status_code_response(400) }

pub fn unauthorized() -> ApiGatewayV2httpResponse { status_code_response(401) }

pub fn forbidden() -> ApiGatewayV2httpResponse { status_code_response(403) }

pub fn not_found() -> ApiGatewayV2httpResponse { status_code_response(404) }

pub fn internal_server_error() -> ApiGatewayV2httpResponse { status_code_response(500) }

pub fn status_code_response(status_code: i64) -> ApiGatewayV2httpResponse {
    ApiGatewayV2httpResponse {
        status_code: status_code,
        ..Default::default()
    }
}