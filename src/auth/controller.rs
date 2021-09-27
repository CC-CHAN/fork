use super::session::UserSession;
use crate::auth::session::get_cookie_from_header;
use crate::init::AppConnections;
use async_session::SessionStore;
use axum::{
    extract::Extension,
    http::{self, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use tracing::info;

pub async fn login(user_session: UserSession) -> impl IntoResponse {
    let (header, session_body) = match user_session {
        UserSession::GetSession(body) => (HeaderMap::new(), body),
        UserSession::CreateNewSession { body, cookie } => {
            let mut header = HeaderMap::new();
            header.insert(
                http::header::SET_COOKIE,
                HeaderValue::from_str(cookie.to_string().as_str()).unwrap(),
            );
            (header, body)
        }
    };
    info!("current user: {}", session_body.username);
    header
}

pub async fn logout(
    header: HeaderMap,
    Extension(app_connections): Extension<AppConnections>,
) -> impl IntoResponse {
    let store = app_connections.session_store;
    // get cookie
    if let Some(mut cookie) = get_cookie_from_header(&header) {
        if let Some(session) = store
            .load_session(cookie.value().to_string())
            .await
            .unwrap()
        {
            if store.destroy_session(session).await.is_ok() {
                cookie.set_expires(time::OffsetDateTime::now_utc());

                let mut header = HeaderMap::new();
                header.insert(
                    http::header::SET_COOKIE,
                    HeaderValue::from_str(cookie.to_string().as_str()).unwrap(),
                );
                return (StatusCode::OK, header);
                // return StatusCode::ACCEPTED;
            }
        }
    }
    (StatusCode::UNAUTHORIZED, HeaderMap::new())
}
