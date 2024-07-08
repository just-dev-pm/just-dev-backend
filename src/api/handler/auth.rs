use axum::{http::StatusCode, response::IntoResponse, Json};
use axum_login::AuthSession;

use crate::{db::model::user::Credentials, usecase::util::auth_backend::AuthBackend};

pub async fn login(
    mut auth_session: AuthSession<AuthBackend>,
    Json(creds): Json<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (StatusCode::OK, "login success").into_response()
}
