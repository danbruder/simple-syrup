use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{ObjectType, SubscriptionType};
use async_graphql_warp::{GraphQLBadRequest, GraphQLResponse};

use dotenv::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::convert::Infallible;
use std::str::FromStr;
use warp::{http::Response as HttpResponse, http::StatusCode, Filter, Rejection};

pub use anyhow;
pub use async_graphql::{
    self, Context, EmptyMutation, EmptySubscription, Object, Result, Schema, SchemaBuilder,
    SimpleObject,
};
pub use chrono;
pub use serde;
pub use serde_json;
pub use sqlx::{self, sqlite::SqlitePool};
pub use tokio;
pub use uuid;

/// SimpleGraphql
pub struct SimpleGraphql<
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
> {
    builder: SchemaBuilder<Q, M, S>,
    sqlite: Option<SimpleSqlite>,
}

impl<Q: ObjectType + 'static, M: ObjectType + 'static, S: SubscriptionType + 'static>
    SimpleGraphql<Q, M, S>
{
    pub fn new(builder: SchemaBuilder<Q, M, S>) -> Self {
        Self {
            builder,
            sqlite: None,
        }
    }

    pub fn with_sqlite(self, sqlite: SimpleSqlite) -> Self {
        Self {
            sqlite: Some(sqlite),
            ..self
        }
    }

    pub async fn run(self) {
        dotenv().ok();

        let schema = if let Some(sqlite) = self.sqlite {
            self.builder.data(sqlite.pool()).finish()
        } else {
            self.builder.finish()
        };

        let graphql_post = warp::path!("graphql")
            .and(async_graphql_warp::graphql(schema))
            .and_then(
                |(schema, request): (Schema<Q, M, S>, async_graphql::Request)| async move {
                    Ok::<_, Infallible>(GraphQLResponse::from(schema.execute(request).await))
                },
            );

        let graphql_playground = warp::path!("playground").and(warp::get()).map(|| {
            HttpResponse::builder()
                .header("content-type", "text/html")
                .body(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
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

        println!("Running at http://0.0.0.0:3030");
        println!("\nRoutes:");
        println!("\t/graphql");
        println!("\t/playground");
        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
    }
}

/// SimpleSqlite
pub struct SimpleSqlite {
    pool: SqlitePool,
}

impl SimpleSqlite {
    pub fn new(filename: &str) -> Self {
        let url = format!("sqlite://{}", filename);
        let options = SqliteConnectOptions::from_str(&url)
            .unwrap()
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_lazy_with(options);

        Self { pool }
    }

    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }
}
