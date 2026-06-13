//! HTTP layer: routing, request/response shapes and error mapping.
//!
//! The actual icon lookup lives in the `iconator` library crate; this module
//! is a thin adapter that translates HTTP requests into library calls and
//! library results (or errors) back into HTTP responses.

use axum::{
    Json, Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

use iconator::{IconError, get_icon_for_file, get_icon_for_folder};

/// Build the application router.
///
/// Endpoints:
/// - `GET /v1/file?path=…`
/// - `GET /v1/folder?path=…`
pub fn router() -> Router {
    Router::new()
        .route("/v1/file", get(file_icon))
        .route("/v1/folder", get(folder_icon))
        .layer(TraceLayer::new_for_http())
}

#[derive(Deserialize)]
struct PathQuery {
    path: String,
}

#[derive(Serialize)]
struct IconResponse {
    path: String,
    icon_id: u64,
}

async fn file_icon(Query(query): Query<PathQuery>) -> Result<Json<IconResponse>, ApiError> {
    let icon_id = get_icon_for_file(&query.path)?;
    Ok(Json(IconResponse {
        path: query.path,
        icon_id,
    }))
}

async fn folder_icon(Query(query): Query<PathQuery>) -> Result<Json<IconResponse>, ApiError> {
    let icon_id = get_icon_for_folder(&query.path)?;
    Ok(Json(IconResponse {
        path: query.path,
        icon_id,
    }))
}

/// HTTP-level Error type, wrapping SVG lookup and anything else that can go wrong
enum ApiError {
    Icon(IconError),
}

impl From<IconError> for ApiError {
    fn from(err: IconError) -> Self {
        ApiError::Icon(err)
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Icon(IconError::InvalidPath) => {
                (StatusCode::BAD_REQUEST, "invalid path".to_string())
            }
            ApiError::Icon(err @ IconError::IconNotFound(_)) => {
                (StatusCode::NOT_FOUND, err.to_string())
            }
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

#[cfg(test)]
mod property_tests {
    use super::router;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use hegel::generators;
    use iconator::get_icon_for_file;
    use tower::ServiceExt;

    /// Helper to make requests to Router we build in http module
    fn get(uri: &str) -> (StatusCode, Vec<u8>) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
            let response = router().oneshot(request).await.unwrap();
            let status = response.status();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            (status, body.to_vec())
        })
    }

    #[test]
    fn file_icon_returns_400_for_invalid_path() {
        let (status, body) = get("/v1/file?path=..");

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"].as_str().unwrap(), "invalid path");
    }

    /// For any path whose extension the library *does* know, the `/v1/file` endpoint must answer 200
    #[hegel::test]
    fn file_icon_forwards_library_result_for_known_extensions(tc: hegel::TestCase) {
        // URL-safe stem (no percent-encoding needed) + a known-resolving extension.
        let stem = tc.draw(generators::from_regex(r"[A-Za-z0-9_-]{1,12}").fullmatch(true));
        let ext = tc.draw(generators::sampled_from(vec![
            "js", "ts", "jsx", "tsx", "json", "yaml", "yml", "md", "css", "html", "rs",
        ]));
        let path = format!("{stem}.{ext}");

        let expected = get_icon_for_file(&path).expect("known extension must resolve");

        let (status, body) = get(&format!("/v1/file?path={path}"));

        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["icon_id"].as_u64().unwrap(), expected);
        assert_eq!(json["path"].as_str().unwrap(), path);
    }

    /// For any path the library cannot resolve, the `/v1/file` endpoint must answer 404
    #[hegel::test]
    fn file_icon_returns_404_for_unknown_paths(tc: hegel::TestCase) {
        let stem = tc.draw(generators::from_regex(r"[a-z]{1,8}").fullmatch(true));
        // A long random alphabetic extension is effectively never a real one;
        // `assume` discards the vanishingly rare collision with a known icon.
        let ext = tc.draw(generators::from_regex(r"[a-z]{8,16}").fullmatch(true));
        let path = format!("{stem}.{ext}");
        tc.assume(get_icon_for_file(&path).is_err());

        let (status, body) = get(&format!("/v1/file?path={path}"));

        assert_eq!(status, StatusCode::NOT_FOUND);
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("error").and_then(|e| e.as_str()).is_some());
    }
}
