use async_session::{async_trait, MemoryStore, Session, SessionStore};
use axum::{
    extract::{Extension, FromRequest},
    http::{self, StatusCode},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSessionBody {
    id: String,
    name: String,
}

#[derive(Debug)]
pub enum UserSession {
    CreateNewSession {
        cookie: String,
        body: UserSessionBody,
    },
    GetSession(UserSessionBody),
}

#[async_trait]
impl<T> FromRequest<T> for UserSession
where
    T: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(
        req: &mut axum::extract::RequestParts<T>,
    ) -> Result<Self, Self::Rejection> {
        let Extension(store) = Extension::<MemoryStore>::from_request(req)
            .await
            .expect("MemoryStore not found");

        let headers = req.headers().expect("headers not found");

        let cookie = if let Some(cookie) = headers
            .get(http::header::COOKIE)
            .and_then(|x| x.to_str().ok())
            .map(|x| x.to_string())
        {
            cookie
        } else {
            let mut session = Session::new();
            let session_body = UserSessionBody {
                id: "dummy_id".to_string(),
                name: "dummy_name".to_string(),
            };

            session.insert("body", session_body.clone()).unwrap();
            let cookie = store.store_session(session).await.unwrap().unwrap();

            return Ok(Self::CreateNewSession {
                cookie,
                body: session_body,
            });
        };

        let session = if let Some(session) = store.load_session(cookie).await.unwrap() {
            session
        } else {
            return Err((StatusCode::BAD_REQUEST, "invalid user"));
        };

        let body = session.get::<UserSessionBody>("body").unwrap();

        Ok(Self::GetSession(body))
    }
}
