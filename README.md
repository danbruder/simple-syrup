# Simple Syrup

The fastest way to get a GraphQL server up and running in Rust

Add to `Cargo.toml`:

```toml
[dependencies]
simple-syrup = "0.3.0"
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
Running on 0.0.0.0:3000
    /playground
    /graphql
```

With [sqlx](https://crates.io/crates/sqlx) and a sqlite database: 

```rust
use simple_syrup::*;

#[tokio::main]
async fn main() {
    let db = SimpleSqlite::new("foo.db");
    db.migrate().await;

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription);

    SimpleGraphql::new(schema).with_sqlite(db).run().await
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn zero(&self) -> u32 {
        0
    }
}
```

