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
    self, Context, EmptyMutation, EmptySubscription, Enum, InputObject, Interface, Object, Result,
    Scalar, Schema, SchemaBuilder, SimpleObject, Union,
};
pub use chrono;
pub use serde;
pub use serde_json;
pub use sqlx::{self, sqlite::SqlitePool};
pub use tokio;
pub use uuid::{self, Uuid};

/// SimpleGraphql
pub struct SimpleGraphql<
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
> {
    builder: SchemaBuilder<Q, M, S>,
    sqlite: Option<SimpleSqlite>,
    spa: Option<SpaConfig>,
}

pub struct SpaConfig {
    assets: String,
    index: String,
}

impl<Q: ObjectType + 'static, M: ObjectType + 'static, S: SubscriptionType + 'static>
    SimpleGraphql<Q, M, S>
{
    pub fn new(builder: SchemaBuilder<Q, M, S>) -> Self {
        Self {
            builder,
            sqlite: None,
            spa: None,
        }
    }

    pub fn with_sqlite(self, sqlite: SimpleSqlite) -> Self {
        Self {
            sqlite: Some(sqlite),
            ..self
        }
    }

    pub fn with_spa(self, assets_dir: &str, index_file: &str) -> Self {
        Self {
            spa: Some(SpaConfig {
                assets: assets_dir.to_string(),
                index: index_file.to_string(),
            }),
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

        let recover = |err: Rejection| async move {
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
        };

        // Conditional routes here is not great
        if let Some(spa) = self.spa {
            let assets = warp::fs::dir(spa.assets);
            let spa_index = warp::fs::file(spa.index);

            let routes = graphql_playground
                .or(graphql_post)
                .or(assets)
                .or(spa_index)
                .recover(recover);
            print_start();
            warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
        } else {
            let routes = graphql_playground.or(graphql_post).recover(recover);
            print_start();
            warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
        }
    }
}

fn print_start() {
    println!("Running at http://0.0.0.0:3030");
    println!("\nRoutes:");
    println!("\t/graphql");
    println!("\t/playground");
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
