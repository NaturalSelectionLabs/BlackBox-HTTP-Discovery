use crate::config::Config;
use crate::model::{FileConfig, Response};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::get, Json, Router};
use std::collections::HashMap;

const ENDPOINT_URL: &str = "__endpoint__url";
const ENDPOINT_NAME: &str = "__endpoint__name";
const ENDPOINT_GEOHASH: &str = "__endpoint__geohash";
const TARGET_TAG: &str = "__target_tag";

async fn handler(State(config): State<Config>) -> (StatusCode, Json<Response>) {
    // Group targets by tags
    // Map: tag -> list of targets
    let grouped_targets: HashMap<String, Vec<String>> = config
        .target
        .iter()
        .map(|target| {
            target
                .tags
                .iter()
                .map(|tag| (tag.clone(), target.url.clone()))
                .collect::<Vec<(String, String)>>()
        })
        .flatten()
        .fold(HashMap::new(), |mut acc, (tag, url)| {
            acc.entry(tag).or_insert_with(Vec::new).push(url);
            acc
        });

    // generate config for each endpoint and tag combination
    let resp: Vec<FileConfig> = config
        .endpoint
        .iter()
        .map(|endpoint| {
            grouped_targets.iter().map(|(tag, targets)| FileConfig {
                targets: targets.clone(),
                labels: HashMap::from([
                    (ENDPOINT_URL.to_string(), endpoint.address.clone()),
                    (ENDPOINT_NAME.to_string(), endpoint.name.clone()),
                    (ENDPOINT_GEOHASH.to_string(), endpoint.geohash.clone()),
                    (TARGET_TAG.to_string(), tag.clone()),
                ]),
            })
        })
        .flatten()
        .collect();

    (StatusCode::OK, Json(resp))
}

pub fn routes() -> Router<Config> {
    Router::new()
        .route("/healthcheck", get(|| async { "OK" }))
        .route("/healthz", get(|| async { "OK" }))
        .route("/", get(handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use std::collections::HashMap;

    // 创建测试用的模拟配置
    fn create_test_config() -> crate::config::Config {
        crate::config::Config {
            target: vec![
                crate::config::Target {
                    module: "http_2xx".to_string(),
                    url: "https://example1.com".to_string(),
                    tags: vec!["web".to_string(), "api".to_string()],
                },
                crate::config::Target {
                    module: "http_2xx".to_string(),
                    url: "https://example2.com".to_string(),
                    tags: vec!["web".to_string()],
                },
                crate::config::Target {
                    module: "http_2xx".to_string(),
                    url: "https://api.example.com".to_string(),
                    tags: vec!["api".to_string()],
                },
            ],
            endpoint: vec![
                crate::config::Endpoint {
                    address: "test1.example.com:443".to_string(),
                    geohash: "test_hash_1".to_string(),
                    name: "TestEndpoint1".to_string(),
                },
                crate::config::Endpoint {
                    address: "test2.example.com:443".to_string(),
                    geohash: "test_hash_2".to_string(),
                    name: "TestEndpoint2".to_string(),
                },
            ],
        }
    }

    #[tokio::test]
    async fn test_healthcheck_route() {
        let test_config = create_test_config();
        let app = routes().with_state(test_config);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/healthcheck").await;
        response.assert_status(StatusCode::OK);
        response.assert_text("OK");
    }

    #[tokio::test]
    async fn test_healthz_route() {
        let test_config = create_test_config();
        let app = routes().with_state(test_config);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/healthz").await;
        response.assert_status(StatusCode::OK);
        response.assert_text("OK");
    }

    #[tokio::test]
    async fn test_root_route_with_state() {
        let test_config = create_test_config();
        let app = routes().with_state(test_config);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/").await;
        response.assert_status(StatusCode::OK);

        // 验证响应是 JSON 格式
        let json_response: Response = response.json();
        assert!(!json_response.is_empty());

        // 验证每个配置项都有正确的标签
        for config in &json_response {
            assert!(config.labels.contains_key(ENDPOINT_URL));
            assert!(config.labels.contains_key(ENDPOINT_NAME));
            assert!(config.labels.contains_key(ENDPOINT_GEOHASH));
            assert!(config.labels.contains_key(TARGET_TAG));
            assert!(!config.targets.is_empty());
        }
    }

    #[tokio::test]
    async fn test_root_route_response_structure() {
        let test_config = create_test_config();
        let app = routes().with_state(test_config);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/").await;
        response.assert_status(StatusCode::OK);

        let json_response: Response = response.json();

        // 验证响应结构
        assert!(!json_response.is_empty());

        // 验证标签分组逻辑
        let web_configs: Vec<_> = json_response
            .iter()
            .filter(|config| config.labels.get(TARGET_TAG) == Some(&"web".to_string()))
            .collect();
        let api_configs: Vec<_> = json_response
            .iter()
            .filter(|config| config.labels.get(TARGET_TAG) == Some(&"api".to_string()))
            .collect();

        // web tag 应该包含 2 个 targets (example1.com 和 example2.com)
        assert_eq!(web_configs.len(), 2); // 2 endpoints * 1 web tag group
        assert!(web_configs.iter().any(|config| config.targets.len() == 2));

        // api tag 应该包含 2 个 targets (example1.com 和 api.example.com)
        assert_eq!(api_configs.len(), 2); // 2 endpoints * 1 api tag group
        assert!(api_configs.iter().any(|config| config.targets.len() == 2));
    }

    #[tokio::test]
    async fn test_handler_with_different_configs() {
        // 测试不同的配置组合
        let mut test_config = create_test_config();

        // 添加更多测试数据
        test_config.target.push(crate::config::Target {
            module: "http_2xx".to_string(),
            url: "https://test3.com".to_string(),
            tags: vec!["monitoring".to_string()],
        });

        test_config.endpoint.push(crate::config::Endpoint {
            address: "test3.example.com:443".to_string(),
            geohash: "test_hash_3".to_string(),
            name: "TestEndpoint3".to_string(),
        });

        let app = routes().with_state(test_config);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/").await;
        response.assert_status(StatusCode::OK);

        let json_response: Response = response.json();

        // 验证有监控标签的配置
        let monitoring_configs: Vec<_> = json_response
            .iter()
            .filter(|config| config.labels.get(TARGET_TAG) == Some(&"monitoring".to_string()))
            .collect();

        assert_eq!(monitoring_configs.len(), 3); // 3 endpoints * 1 monitoring tag group
        assert!(monitoring_configs
            .iter()
            .any(|config| config.targets.len() == 1));
    }

    #[tokio::test]
    async fn test_empty_config() {
        // 测试空配置的情况
        let empty_config = crate::config::Config {
            target: vec![],
            endpoint: vec![],
        };

        let app = routes().with_state(empty_config);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/").await;
        response.assert_status(StatusCode::OK);

        let json_response: Response = response.json();
        assert!(json_response.is_empty());
    }

    #[tokio::test]
    async fn test_single_target_multiple_tags() {
        // 测试单个 target 有多个 tags 的情况
        let config = crate::config::Config {
            target: vec![crate::config::Target {
                module: "http_2xx".to_string(),
                url: "https://multi-tag.com".to_string(),
                tags: vec![
                    "web".to_string(),
                    "api".to_string(),
                    "monitoring".to_string(),
                ],
            }],
            endpoint: vec![crate::config::Endpoint {
                address: "test.example.com:443".to_string(),
                geohash: "test_hash".to_string(),
                name: "TestEndpoint".to_string(),
            }],
        };

        let app = routes().with_state(config);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/").await;
        response.assert_status(StatusCode::OK);

        let json_response: Response = response.json();

        // 应该有 3 个配置项（每个 tag 一个）
        assert_eq!(json_response.len(), 3);

        // 验证每个配置项都包含相同的 target
        for config in &json_response {
            assert_eq!(config.targets.len(), 1);
            assert_eq!(config.targets[0], "https://multi-tag.com");
        }
    }

    #[test]
    fn test_target_grouping_logic() {
        let test_config = create_test_config();
        let mut grouped_targets: HashMap<String, Vec<String>> = HashMap::new();

        // 测试分组逻辑
        for target in test_config.target.iter() {
            for tag in &target.tags {
                grouped_targets
                    .entry(tag.clone())
                    .or_default()
                    .push(target.url.clone());
            }
        }

        // 验证分组结果
        assert_eq!(grouped_targets.len(), 2); // web 和 api 两个标签

        let web_targets = grouped_targets.get("web").unwrap();
        assert_eq!(web_targets.len(), 2);
        assert!(web_targets.contains(&"https://example1.com".to_string()));
        assert!(web_targets.contains(&"https://example2.com".to_string()));

        let api_targets = grouped_targets.get("api").unwrap();
        assert_eq!(api_targets.len(), 2);
        assert!(api_targets.contains(&"https://example1.com".to_string()));
        assert!(api_targets.contains(&"https://api.example.com".to_string()));
    }

    #[test]
    fn test_file_config_creation() {
        let targets = vec![
            "https://test1.com".to_string(),
            "https://test2.com".to_string(),
        ];
        let mut labels = HashMap::new();
        labels.insert(ENDPOINT_URL.to_string(), "test.example.com:443".to_string());
        labels.insert(ENDPOINT_NAME.to_string(), "TestEndpoint".to_string());
        labels.insert(ENDPOINT_GEOHASH.to_string(), "test_hash".to_string());
        labels.insert(TARGET_TAG.to_string(), "test_tag".to_string());

        let file_config = FileConfig { targets, labels };

        assert_eq!(file_config.targets.len(), 2);
        assert_eq!(file_config.labels.len(), 4);
        assert_eq!(
            file_config.labels.get(ENDPOINT_NAME),
            Some(&"TestEndpoint".to_string())
        );
        assert_eq!(
            file_config.labels.get(TARGET_TAG),
            Some(&"test_tag".to_string())
        );
    }

    #[tokio::test]
    async fn test_routes_integration() {
        // 集成测试：测试所有路由是否都能正常工作
        let test_config = create_test_config();
        let app = routes().with_state(test_config);
        let server = TestServer::new(app).unwrap();

        // 测试健康检查路由
        let health_response = server.get("/healthcheck").await;
        health_response.assert_status(StatusCode::OK);
        health_response.assert_text("OK");

        let healthz_response = server.get("/healthz").await;
        healthz_response.assert_status(StatusCode::OK);
        healthz_response.assert_text("OK");

        // 测试主路由
        let root_response = server.get("/").await;
        root_response.assert_status(StatusCode::OK);

        let json_response: Response = root_response.json();
        assert!(!json_response.is_empty());
    }

    #[tokio::test]
    async fn test_handler_with_connect_info() {
        // 测试 handler 函数是否正确处理 ConnectInfo
        let test_config = create_test_config();
        let app = routes().with_state(test_config);
        let server = TestServer::new(app).unwrap();

        // 这个测试主要验证 handler 函数能正常执行，不会因为 ConnectInfo 而崩溃
        let response = server.get("/").await;
        response.assert_status(StatusCode::OK);

        // 验证响应格式正确
        let json_response: Response = response.json();
        assert!(!json_response.is_empty());
    }
}
