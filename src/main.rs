//! ```not_rust
//! cargo run
//! ```

mod orderbook;

use tokio::time::Duration;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Request, Response, Schema,
};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use crate::orderbook::{OrderBook, QueryRoot, OrderBookSchema, run_reporter_poll};

async fn graphql_handler(schema: Extension<OrderBookSchema>, req: Json<Request>) -> Json<Response> {
    schema.execute(req.0).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema));

    println!("Playground: http://localhost:3000");

    tokio::join!(run_reporter_poll(), axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service()));

}
