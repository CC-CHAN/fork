use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::{
    extract::{Extension, FromRequest},
    http::{self, header, HeaderMap, HeaderValue, Request, StatusCode},
    response::IntoResponse,
};

use crate::init::AppConnections;

use super::session::UserSession;

pub async fn login(user_session: UserSession) -> impl IntoResponse {
    let (header, _) = match user_session {
        UserSession::GetSession(body) => (HeaderMap::new(), body),
        UserSession::CreateNewSession { body, cookie } => {
            let mut header = HeaderMap::new();
            // more secure cookie
            let cookie = cookie + ";secure ;httpOnly";
            header.insert(
                http::header::SET_COOKIE,
                HeaderValue::from_str(cookie.as_str()).unwrap(),
            );
            (header, body)
        }
    };

    header
}

pub async fn logout(
    header: HeaderMap,
    Extension(app_connections): Extension<AppConnections>,
) -> impl IntoResponse {
    let store = app_connections.session_store;
    // get cookie
    if let Some(cookie) = header
        .get(http::header::COOKIE)
        .and_then(|x| x.to_str().ok())
    {
        let session = store
            .load_session(cookie.to_string())
            .await
            .unwrap()
            .unwrap();
        if store.destroy_session(session).await.is_err() {
            return StatusCode::SERVICE_UNAVAILABLE;
        } else {
            return StatusCode::ACCEPTED;
        }
    }
    StatusCode::UNAUTHORIZED
}

async fn create_session(user_id: String) {
    //
}
