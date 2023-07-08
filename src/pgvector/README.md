## SQLx

Create a vector from a `Vec<f32>`

```rust
let embedding = pgvector::Vector::from(vec![1.0, 2.0, 3.0]);
```

Insert a vector

```rust
sqlx::query("INSERT INTO items (embedding) VALUES ($1)").bind(embedding).execute(&pool).await?;
```

Get the nearest neighbors

```rust
let rows = sqlx::query("SELECT * FROM items ORDER BY embedding <-> $1 LIMIT 1")
    .bind(embedding).fetch_all(&pool).await?;
```

Retrieve a vector

```rust
let row = sqlx::query("SELECT embedding FROM items LIMIT 1").fetch_one(&pool).await?;
let embedding: pgvector::Vector = row.try_get("embedding")?;
```