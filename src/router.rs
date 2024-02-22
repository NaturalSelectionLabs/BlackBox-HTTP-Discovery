use std::collections::HashMap;
use axum::{routing::{get, MethodRouter}, Router, Json};
use axum::http::StatusCode;
use crate::model::{FileConfig, Response};
use crate::config::CONFIG;

fn route(path: &str, method_router: MethodRouter<()>) -> Router {
    Router::new().route(path, method_router)
}

pub fn healthcheck() -> Router {
    async fn handler() -> &'static str {
        "ok"
    }
    route("/healthcheck", get(handler))
}

pub fn root() -> Router {
    async fn handler() -> (StatusCode, Json<Response>) {
        let mut resp : Response = Vec::new();

        for _endpoint in CONFIG.endpoint.iter() {
            resp.push(FileConfig {
                targets: CONFIG.target.iter().map(|x| x.url.clone()).collect(),
                labels: HashMap::from([
                    ("__endpoint__url".to_string(), _endpoint.address.clone()),
                    ("__endpoint__name".to_string(), _endpoint.name.clone()),
                    ("__endpoint__geohash".to_string(), _endpoint.geohash.clone())
                ]),
            })
        }

        (StatusCode::OK, Json(resp))
    }

    route("/", get(handler))
}