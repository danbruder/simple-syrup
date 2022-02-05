use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{ObjectType, SubscriptionType};
use async_graphql_warp::{GraphQLBadRequest, GraphQLResponse};

use dotenv::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::convert::Infallible;
use std::env;
use std::fs;
use std::str::FromStr;
use warp::{http::Response as HttpResponse, http::StatusCode, Filter, Rejection};

pub use anyhow;
pub use async_graphql::{
    self, EmptyMutation, EmptySubscription, Object, Schema, SchemaBuilder, SimpleObject,
};
pub use chrono;
pub use serde;
pub use serde_json;
pub use sqlx;
pub use tokio;
pub use uuid;

pub struct SimpleSyrup<
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
> {
    builder: SchemaBuilder<Q, M, S>,
}

impl<Q: ObjectType + 'static, M: ObjectType + 'static, S: SubscriptionType + 'static>
    SimpleSyrup<Q, M, S>
{
    pub fn new(builder: SchemaBuilder<Q, M, S>) -> Self {
        Self { builder }
    }

    pub async fn run(self) {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").unwrap_or("sqlite://data.db".to_string());

        let options = SqliteConnectOptions::from_str(&database_url)
            .unwrap()
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_lazy_with(options);

        fs::create_dir_all("migrations").expect("Couldn't create migrations dir");

        sqlx::migrate!().run(&pool).await.ok();

        let schema = self.builder.data(pool.clone()).finish();

        let graphql_post = async_graphql_warp::graphql(schema).and_then(
            |(schema, request): (Schema<Q, M, S>, async_graphql::Request)| async move {
                Ok::<_, Infallible>(GraphQLResponse::from(schema.execute(request).await))
            },
        );

        let graphql_playground = warp::path!("playground").and(warp::get()).map(|| {
            HttpResponse::builder()
                .header("content-type", "text/html")
                .body(playground_source(GraphQLPlaygroundConfig::new("/")))
        });

        let routes = graphql_playground
            .or(graphql_post)
            .recover(|err: Rejection| async move {
                if let Some(GraphQLBadRequest(err)) = err.find() {
                    return Ok::<_, Infallible>(warp::reply::with_status(
                        err.to_string(),
                        StatusCode::BAD_REQUEST,
                    ));
                }

                Ok(warp::reply::with_status(
                    "INTERNAL_SERVER_ERROR".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            });

        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
    }
}
