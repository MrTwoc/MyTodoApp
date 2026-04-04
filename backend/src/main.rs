use salvo::http::Method;
use salvo::prelude::*;
use salvo::{cors::Cors, oapi::extract::*};

mod db;
mod models;
use db::migrations::init_database;
use db::pool::create_pool;

mod middleware;
use middleware::logging::{logger, request_logger};
mod ws;

mod utils;

mod handlers;
mod routes;
mod services;

use routes::{task_routes, user_routes};

use crate::routes::{dashboard_routes, sub_task_routes, sub_team_routes, team_routes, ws_routes};

#[endpoint]
async fn hello(name: QueryParam<String, false>) -> String {
    format!("Hello, {}!", name.as_deref().unwrap_or("World"))
}

#[tokio::main]
async fn main() {
    let _guard = utils::log::init_logger();

    if let Err(e) = run_database().await {
        tracing::error!("数据库初始化失败: {:?}", e);
        return;
    }

    if let Err(e) = utils::id_generator::test_sonyflake_id() {
        tracing::error!("测试sonflake id失败: {:?}", e);
        return;
    }

    let cors = Cors::new()
        .allow_origin([
            "http://127.0.0.1:8698",
            "http://localhost:8698",
            "http://127.0.0.1:8080",
            "http://localhost:8080",
        ])
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec!["authorization", "content-type"])
        .allow_credentials(true)
        .into_handler();

    let router = Router::new()
        // .hoop(cors)
        .push(Router::with_path("hello").post(hello))
        .push(user_routes::user_router())
        .push(task_routes::task_router())
        .push(team_routes::team_router())
        .push(dashboard_routes::dashboard_router())
        .push(sub_team_routes::sub_team_router())
        .push(sub_team_routes::sub_team_single_router())
        .push(sub_task_routes::sub_task_router())
        .push(ws_routes::ws_router());

    let doc = OpenApi::new("test api", "0.0.1").merge_router(&router);

    let router = router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger-ui"));

    let acceptor = TcpListener::new("localhost:8698").bind().await;
    let service = Service::new(router).hoop(cors);
    Server::new(acceptor).serve(service).await;
}

async fn run_database() -> Result<(), Box<dyn std::error::Error>> {
    let _pool = init_database().await?;
    create_pool().await?;
    Ok(())
}
