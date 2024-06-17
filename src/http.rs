use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
};
use axum::{routing, Router};

use std::collections::HashMap;

use super::storage::Storage;

lazy_static::lazy_static! {
    pub static ref AUTH_KEY: String = std::env::var("AUTH_KEY").expect("AUTH_KEY unset");
}

pub struct Handler;

impl Handler {
    async fn auth_middleware(
        req: Request,
        next: Next,
    ) -> Result<impl IntoResponse, (StatusCode, String)> {
        let (parts, body) = req.into_parts();
        let is_auth = parts
            .headers
            .get("X-Auth-Key")
            .map(|v| v == &**AUTH_KEY)
            .unwrap_or(false);
        if !is_auth {
            return Err((StatusCode::UNAUTHORIZED, "{}".into()));
        }
        Ok(next.run(Request::from_parts(parts, body)).await)
    }

    async fn block_address(
        Query(params): Query<HashMap<String, String>>,
        State(state): State<Storage>,
    ) -> Result<String, StatusCode> {
        if let Some(addr) = params.get("addr") {
            if let Err(e) = state.block_address(addr) {
                log::error!("failed to block address: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            log::info!("blocked address: {}", addr);
            Ok("{}".into())
        } else {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    async fn unblock_address(
        Query(params): Query<HashMap<String, String>>,
        State(state): State<Storage>,
    ) -> Result<String, StatusCode> {
        if let Some(addr) = params.get("addr") {
            if let Err(e) = state.unblock_address(addr) {
                log::error!("failed to unblock address: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            log::info!("unblocked address: {}", addr);
            Ok("{}".into())
        } else {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    pub async fn serve(addr: &str, storage: Storage) {
        let app = Router::new()
            .route("/block", routing::put(Self::block_address))
            .route("/block", routing::delete(Self::unblock_address))
            .layer(middleware::from_fn(Self::auth_middleware))
            .with_state(storage);
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("binding address");
        log::info!("Listening to {}", addr);
        axum::serve(listener, app).await.expect("running server");
    }
}
