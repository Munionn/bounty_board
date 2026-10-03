use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn root_returns_ok() {
    dotenvy::dotenv().ok();
    let config = server::AppConfig::from_env();
    let state = server::init_state(&config)
        .await
        .expect("failed to init app state (is postgres running?)");
    let app = server::build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}
