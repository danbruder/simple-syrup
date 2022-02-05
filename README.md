# Simple Syrup

The fastest way to get a GraphQL server up and running in Rust

Add to `Cargo.toml`:

```toml
[dependencies]
simple-syrup = "0.5.2"
```

```rust
use simple_syrup::*;

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription);

    SimpleGraphql::new(schema).run().await
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn zero(&self) -> u32 {
        0
    }
}
```

```bash
Running at http://0.0.0.0:3030

Routes:
    /graphql
    /playground
```

With [sqlx](https://crates.io/crates/sqlx) and a sqlite database: 

```rust
use simple_syrup::*;

#[tokio::main]
async fn main() {
    let db = SimpleSqlite::new("foo.db");
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription);

    SimpleGraphql::new(schema).with_sqlite(db).run().await
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn two(&self, ctx: &Context<'_>) -> Result<i64> {
        let pool = ctx.data::<SqlitePool>()?;

        let result: (i64,) = sqlx::query_as("SELECT 1 + 1").fetch_one(&*pool).await?;
        Ok(result.0)
    }
}
```
