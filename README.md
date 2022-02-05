# Simple Syrup

The fastest way to get a GraphQL server up and running in Rust

```rust
use simple_syrup::*;

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription);

    SimpleSyrup::new(schema).run().await
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn zero(&self) -> u32 {
        0
    }
}
```

With [sqlx](https://crates.io/crates/sqlx) and a sqlite database: 

```rust
use simple_syrup::*;

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription);

    SimpleSyrup::new(schema)
      .with_sqlite("sqlite://data.db")
      .run()
      .await
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn zero(&self) -> u32 {
        0
    }
}
```

