use axum::extract::Path;

use super::session::UserSession;

pub async fn login(session: UserSession) {
    println!("{:?}", session);
}

pub async fn logout() {
    //
}

async fn create_session(user_id: String) {
    //
}
