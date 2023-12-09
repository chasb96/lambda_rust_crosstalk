use std::{future::Future, collections::HashMap, pin::Pin};
use aws_lambda_events::apigw::{ApiGatewayV2httpResponse, ApiGatewayV2httpRequest};
use lambda_runtime::LambdaEvent;
use crate::not_found;

type HandlerArgs = LambdaEvent<ApiGatewayV2httpRequest>;
type Output = dyn Future<Output = ApiGatewayV2httpResponse> + Send + 'static;
type BoxedOutput = Pin<Box<Output>>;
type Handler = Box<dyn Fn(HandlerArgs) -> BoxedOutput + Send + 'static>;

pub struct Router
{
    routes: HashMap<String, Handler>,
}

impl Router
{
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub(crate) async fn invoke(&self, path: String, event: LambdaEvent<ApiGatewayV2httpRequest>) -> ApiGatewayV2httpResponse {
        for (route_template, handler) in &self.routes {
            if template_matches(&route_template, &path) {
                return (handler)(event).await
            }
        }

        not_found()
    }

    pub fn register<S, R>(mut self, route_template: S, handler: fn(LambdaEvent<ApiGatewayV2httpRequest>) -> R) -> Self 
    where
        S: Into<String>,
        R: Future<Output = ApiGatewayV2httpResponse> + Send + 'static
    {
        let handler = Box::new(move |request| {            
            Box::pin(handler(request)) as Pin<Box<Output>>
        }) as Handler;

        self.routes.insert(route_template.into(), handler);

        self
    }
}

fn template_matches(template: &str, path: &str) -> bool {
    let mut template_parts = template.trim_matches('/').split('/');
    let mut path_parts = path.trim_matches('/').split('/');

    while let (Some(template_part), Some(path_part)) = (template_parts.next(), path_parts.next()) {
        if !template_part.starts_with(':') && template_part != path_part {
            return false
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
    use lambda_runtime::{LambdaEvent, Context};
    use crate::router::{Router, template_matches};

    #[tokio::test]
    async fn test_route() {
        async fn test_fn(_: LambdaEvent<ApiGatewayV2httpRequest>) -> ApiGatewayV2httpResponse {
            ApiGatewayV2httpResponse {
                status_code: 500,
                ..Default::default()
            }
        }

        let router = Router::new()
            .register("/".to_string(), test_fn);

        let event = LambdaEvent { 
            payload: ApiGatewayV2httpRequest::default(), 
            context: Context::default(),
        };

        let output = router
            .invoke("/".to_string(), event)
            .await;

        assert_eq!(output.status_code, 500);
    }

    #[tokio::test]
    async fn test_multiple_route() {
        async fn test_fn(_: LambdaEvent<ApiGatewayV2httpRequest>) -> ApiGatewayV2httpResponse {
            ApiGatewayV2httpResponse {
                status_code: 1,
                ..Default::default()
            }
        }

        async fn test_fn_2(_: LambdaEvent<ApiGatewayV2httpRequest>) -> ApiGatewayV2httpResponse {
            ApiGatewayV2httpResponse {
                status_code: 2,
                ..Default::default()
            }
        }

        let router = Router::new()
            .register("/1".to_string(), test_fn)
            .register("/2".to_string(), test_fn_2);

        let event = LambdaEvent { 
            payload: ApiGatewayV2httpRequest::default(), 
            context: Context::default(),
        };

        let output_1 = router
            .invoke("/1".to_string(), event)
            .await;

        let event = LambdaEvent { 
            payload: ApiGatewayV2httpRequest::default(), 
            context: Context::default(),
        };

        let output_2 = router
            .invoke("/2".to_string(), event)
            .await;

        assert_eq!(output_1.status_code, 1);
        assert_eq!(output_2.status_code, 2);
    }
    
    #[tokio::test]
    async fn test_not_found() {
        let router = Router::new();

        let event = LambdaEvent { 
            payload: ApiGatewayV2httpRequest::default(), 
            context: Context::default(),
        };

        let output = router
            .invoke("/".to_string(), event)
            .await;

        assert_eq!(output.status_code, 404);
    }

    #[test]
    fn test_route_matches() {
        assert!(template_matches("/", "/"));
        assert!(template_matches("/test", "/test"));
        assert!(template_matches("/:id", "/value"));
        assert!(template_matches("/test/:input", "/test/value"));
        assert!(template_matches("/:var/input", "/test/input"));
        assert!(template_matches("/test/input/", "/test/input"));
        assert!(template_matches("/:var/input/", "/test/input"));

        assert!(!template_matches("/var/input", "/test/input"));
        assert!(!template_matches("/var", "/test/input"));
        assert!(!template_matches("/var/test", "/test"));
        assert!(!template_matches("/var/", "/test/input"));
    }

    #[tokio::test]
    async fn test_route_template() {
        async fn test_fn(_: LambdaEvent<ApiGatewayV2httpRequest>) -> ApiGatewayV2httpResponse {
            ApiGatewayV2httpResponse {
                status_code: 600,
                ..Default::default()
            }
        }

        let router = Router::new()
            .register("/:id".to_string(), test_fn);

        let event = LambdaEvent { 
            payload: ApiGatewayV2httpRequest::default(), 
            context: Context::default(),
        };

        let output = router
            .invoke("/value".to_string(), event)
            .await;

        assert_eq!(output.status_code, 600);
    }
}