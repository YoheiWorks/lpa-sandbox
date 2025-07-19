mod handler;

use axum::{routing::get, serve, Router};
use mongodb::Client;
use mongodb::options::ClientOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let uri = "mongodb://root:example@localhost:27017";
    let client_options = ClientOptions::parse(uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("local");

    let app = Router::new()
    .route("/jpass/{collection}",
        get(handler::get_all)
        .post(handler::create))
    .route("/jpass/{collection}/{id}",
        get(handler::get_one)
        .put(handler::update)
        .delete(handler::delete))
    .with_state(db);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    serve(listener, app).await.unwrap();
}