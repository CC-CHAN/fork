use crate::config::CONFIG;
use crate::init::AppConnections;
use crate::{model::User, schema::user::dsl::*};
use async_session::{async_trait, Session, SessionStore};
use axum::{
    extract::{Extension, FromRequest},
    http::{self, StatusCode},
    BoxError, Json,
};
use diesel::prelude::*;
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionBody {
    user_id: i32,
    username: String,
}

#[derive(Debug)]
pub enum UserSession {
    CreateNewSession {
        cookie: String,
        body: UserSessionBody,
    },
    GetSession(UserSessionBody),
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
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

    debug!("password: {}", hashed_password);

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
    session.insert("body", session_body.clone()).unwrap();

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

        let headers = req.headers().expect("headers not found");

        let cookie = if let Some(cookie) = headers
            .get(http::header::COOKIE)
            .and_then(|x| x.to_str().ok())
            .map(|x| x.to_string())
        {
            cookie
        } else {
            let login_payload = Json::<LoginPayload>::from_request(req).await;

            if login_payload.is_err() {
                return Err((StatusCode::UNAUTHORIZED, "invalid input"));
            }

            let (session, body) = create_session(&app_connections, login_payload.unwrap())
                .await
                .unwrap();
            let cookie = app_connections
                .session_store
                .store_session(session)
                .await
                .unwrap()
                .unwrap();

            return Ok(Self::CreateNewSession { cookie, body });
        };
        let store = app_connections.session_store;

        let session = if let Some(session) = store.load_session(cookie).await.unwrap() {
            session
        } else {
            return Err((StatusCode::UNAUTHORIZED, "invalid user"));
        };

        let body = session.get::<UserSessionBody>("body").unwrap();

        Ok(Self::GetSession(body))
    }
}

// trait DestroySession {
//     fn destory_session(&self, session_store: RedisSessionStore) -> Result<StatusCode, StatusCode>;
// }

// impl DestroySession for UserSession {
//     fn destory_session(&self, session_store: RedisSessionStore) -> Result<StatusCode, StatusCode> {
//         let session_body = if let UserSession::GetSession(body) = self {
//             body.to_owned()
//         } else {
//             return Err(StatusCode::UNAUTHORIZED);
//         };

//         session_store.destroy_session(session_body);
//         Ok(StatusCode::ACCEPTED)
//     }
// }
