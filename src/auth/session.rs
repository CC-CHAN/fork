use crate::config::CONFIG;
use crate::init::AppConnections;
use crate::{model::User, schema::user::dsl::*};
use async_session::{async_trait, Session, SessionStore};
use axum::http::HeaderMap;
use axum::{
    extract::{Extension, FromRequest},
    http::{self, StatusCode},
    BoxError, Json,
};
use cookie::Cookie;
use diesel::prelude::*;
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionBody {
    pub user_id: i32,
    pub username: String,
}

#[derive(Debug)]
pub enum UserSession {
    CreateNewSession {
        cookie: Cookie<'static>,
        body: UserSessionBody,
    },
    GetSession(UserSessionBody),
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

pub fn get_cookie_from_header(header: &HeaderMap) -> Option<Cookie<'_>> {
    header
        .get(http::header::COOKIE)
        .and_then(|x| x.to_str().ok())
        .map(|x| Cookie::parse(x).unwrap())
}

async fn create_session(
    app_connections: &AppConnections,
    login_payload: Json<LoginPayload>,
) -> Result<(Session, UserSessionBody), &'static str> {
    let auth_user = user
        .filter(username.eq(&login_payload.username))
        .limit(1)
        .first::<User>(&app_connections.db_connections.get().unwrap())
        .unwrap();

    // check password
    let hashed_password = argon2::hash_encoded(
        auth_user.password.as_bytes(),
        CONFIG.auth.salt.as_bytes(),
        &argon2::Config::default(),
    )
    .expect("invalid password");

    let is_valid = argon2::verify_encoded(&hashed_password, auth_user.password.as_bytes())
        .expect("invalid password");

    if !is_valid {
        panic!("invalid password");
    }

    let session_body = UserSessionBody {
        user_id: auth_user.id,
        username: auth_user.username,
    };
    debug!("create new sesion for user: {:?}", session_body);
    let mut session = Session::new();
    session
        .insert(CONFIG.session.key.as_str(), session_body.clone())
        .unwrap();

    Ok((session, session_body))
}

#[async_trait]
impl<T> FromRequest<T> for UserSession
where
    T: http_body::Body + Send,
    T::Data: Send,
    T::Error: Into<BoxError>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(
        req: &mut axum::extract::RequestParts<T>,
    ) -> Result<Self, Self::Rejection> {
        let Extension(app_connections) = Extension::<AppConnections>::from_request(req)
            .await
            .expect("AppConnections not found");

        let header = req.headers().expect("headers not found");

        let cookie = if let Some(cookie) = get_cookie_from_header(header) {
            cookie.to_owned()
        } else {
            debug!("cookie not found, create new session base on username and password");
            let login_payload = Json::<LoginPayload>::from_request(req).await;

            if login_payload.is_err() {
                return Err((StatusCode::UNAUTHORIZED, "invalid input"));
            }

            let (session, body) = create_session(&app_connections, login_payload.unwrap())
                .await
                .unwrap();
            let cookie_value = app_connections
                .session_store
                .store_session(session)
                .await
                .unwrap()
                .unwrap();
            let cookie = Cookie::build(CONFIG.session.key.as_str(), cookie_value)
                .path("/")
                // .secure(true)
                .http_only(true)
                .max_age(time::Duration::days(1))
                .finish();

            return Ok(Self::CreateNewSession { cookie, body });
        };
        let store = app_connections.session_store;

        let session = if let Some(session) = store
            .load_session(cookie.value().to_string())
            .await
            .unwrap()
        {
            session
        } else {
            return Err((StatusCode::UNAUTHORIZED, "invalid user"));
        };

        let body = session
            .get::<UserSessionBody>(CONFIG.session.key.as_str())
            .unwrap();

        Ok(Self::GetSession(body))
    }
}
