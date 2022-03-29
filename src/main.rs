//! ```not_rust
//! cargo run
//! ```

mod orderbook;
use crate::orderbook::run_reporter_poll;
use tokio::time::Duration;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Request, Response, Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use crate::orderbook::{OrderBookSchema, QueryRoot, SubscriptionRoot};

//async fn graphql_handler(schema: Extension<OrderBookSchema>, req: GraphQLRequest) -> GraphQLResponse {
async fn graphql_handler(schema: Extension<OrderBookSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
//
async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws")))
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot)
        .finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema));

    println!("Playground: http://localhost:3000");

    tokio::join!(run_reporter_poll(), axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service()));

}
// async fn graphql_handler(schema: Extension<BooksSchema>, req: GraphQLRequest) -> GraphQLResponse {
//     schema.execute(req.into_inner()).await.into()
// }

// async fn graphql_playground() -> impl IntoResponse {
//     response::Html(playground_source(
//         GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
//     ))
// }
