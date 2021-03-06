use aws_lambda_events::{
    encodings::Body,
    event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
};
use http::header::HeaderMap;
use lambda_runtime::{handler_fn, Context, Error};
use serde::Serialize;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

#[derive(Debug, sqlx::FromRow, Serialize)]
struct PokemonHp {
    name: String,
    hp: u16,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("cold start");
    let processor = handler_fn(handler);
    lambda_runtime::run(processor).await?;
    Ok(())
}

async fn handler(_: ApiGatewayProxyRequest, _: Context) -> Result<ApiGatewayProxyResponse, Error> {
    println!("handler");
    let database_url = env::var("DATABASE_URL")?;

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let result = sqlx::query_as!(
        PokemonHp,
        r#"SELECT name, hp from pokemon where slug = "bulbasaur""#
    )
    .fetch_one(&pool)
    .await?;

    let json_pokemon = serde_json::to_string(&result)?;
    let response = ApiGatewayProxyResponse {
        status_code: 200,
        headers: HeaderMap::new(),
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Text(json_pokemon)),
        is_base64_encoded: Some(false),
    };
    Ok(response)
}

/*
async fn handler(_: ApiGatewayProxyRequest, _: Context) -> Result<ApiGatewayProxyResponse, Error> {
    println!("handler");
    let response = ApiGatewayProxyResponse {
        status_code: 200,
        headers: HeaderMap::new(),
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Text("Boop".to_string())),
        is_base64_encoded: Some(false),
    };
    Ok(response)
}
*/

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use aws_lambda_events::event::apigw::{
        ApiGatewayProxyRequest, ApiGatewayProxyRequestContext, ApiGatewayRequestIdentity,
    };
    use http::Method;

    use super::*;

    #[tokio::test]
    async fn handler_handles() {
        let event = ApiGatewayProxyRequest {
            resource: None,
            path: Some("/.netlify/functions/pokemon-api".to_string()),
            http_method: Method::GET,
            headers: HeaderMap::new(),
            multi_value_headers: HeaderMap::new(),
            query_string_parameters: HashMap::new(),
            multi_value_query_string_parameters: HashMap::new(),
            path_parameters: HashMap::new(),
            stage_variables: HashMap::new(),
            request_context: ApiGatewayProxyRequestContext {
                account_id: None,
                resource_id: None,
                operation_name: None,
                stage: None,
                domain_name: None,
                domain_prefix: None,
                request_id: None,
                protocol: None,
                identity: ApiGatewayRequestIdentity {
                    cognito_identity_pool_id: None,
                    account_id: None,
                    cognito_identity_id: None,
                    caller: None,
                    api_key: None,
                    api_key_id: None,
                    access_key: None,
                    source_ip: None,
                    cognito_authentication_type: None,
                    cognito_authentication_provider: None,
                    user_arn: None,
                    user_agent: None,
                    user: None,
                },
                resource_path: None,
                authorizer: HashMap::new(),
                http_method: Method::GET,
                request_time: None,
                request_time_epoch: 0,
                apiid: None,
            },
            body: None,
            is_base64_encoded: Some(false),
        };

        assert_eq!(
            handler(event.clone(), Context::default()).await.unwrap(),
            ApiGatewayProxyResponse {
                status_code: 200,
                headers: HeaderMap::new(),
                multi_value_headers: HeaderMap::new(),
                body: Some(Body::Text(
                    serde_json::to_string(&PokemonHp {
                        name: String::from("Bulbasaur"),
                        hp: 45
                    },)
                    .unwrap()
                )),
                is_base64_encoded: Some(false),
            }
        )
    }
}
