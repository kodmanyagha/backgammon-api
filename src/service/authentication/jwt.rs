use anyhow::anyhow;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use strum::Display;

use crate::{
    state::app_state::AppState, types::http::api_response::ApiResponse, utils::consts, CONFIG,
};

pub const ISSUER: &str = "app";

pub type Perms = Vec<String>;
pub type Roles = Vec<AppRole>;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AppRole {
    // IMPORTANT Never change ordering of role fields.
    Admin,
    Reseller,
    User,
    #[default]
    Guest,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AppPerm {
    User,
    Role,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PermOperation {
    Admin,
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub exp: u64,
    pub iat: u64,
    pub sub: u64,

    pub roles: Roles,
    pub perms: Perms,
}

impl Claims {
    pub fn new() -> Self {
        Self {
            iss: ISSUER.to_string(),
            iat: Utc::now().timestamp() as u64,
            ..Default::default()
        }
    }

    pub fn with_exp_hours(mut self, exp_hours: i64) -> Self {
        let dt = Utc::now() + Duration::hours(exp_hours);

        self.exp = dt.timestamp() as u64;
        self
    }

    pub fn with_exp_minutes(mut self, exp_minutes: i64) -> Self {
        let dt = Utc::now() + Duration::minutes(exp_minutes);

        self.exp = dt.timestamp() as u64;
        self
    }

    pub fn with_sub(mut self, sub: u64) -> Self {
        self.sub = sub;
        self
    }

    pub fn with_perms(mut self, perms: Perms) -> Self {
        self.perms = perms;
        self
    }

    pub fn with_roles(mut self, roles: Roles) -> Self {
        self.roles = roles.clone();
        self
    }
}

impl Default for Claims {
    fn default() -> Self {
        Self {
            iss: ISSUER.into(),
            exp: (Utc::now() + Duration::hours(8)).timestamp() as u64,
            iat: Utc::now().timestamp() as u64,
            sub: Default::default(),
            perms: Default::default(),
            roles: Default::default(),
        }
    }
}

fn claims_from_jwt(token: &str) -> Result<Claims, Response> {
    decode_jwt(token.to_string())
        .map(|token_data| token_data.claims)
        .map_err(|_| AuthError::InvalidToken.into_response())
}

impl FromRequestParts<AppState> for Claims {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Ok(TypedHeader(Authorization(bearer))) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
        {
            return claims_from_jwt(bearer.token());
        }

        Err(AuthError::MissingToken.into_response())
    }
}

#[derive(Debug, Display)]
enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (StatusCode::BAD_REQUEST, consts::errors::MISSING_TOKENS),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, consts::errors::INVALID_TOKEN),
        };

        let body = Json(json!(ApiResponse::new().with_global_error(error_message)));
        (status, body).into_response()
    }
}

pub fn encode_jwt(claim: Claims) -> anyhow::Result<String> {
    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(CONFIG.get_app_secret().as_ref()),
    )
    .map_err(|err| anyhow!("{err}"))
}

pub fn decode_jwt(jwt_token: String) -> anyhow::Result<TokenData<Claims>> {
    decode(
        &jwt_token,
        &DecodingKey::from_secret(CONFIG.get_app_secret().as_ref()),
        &Validation::default(),
    )
    .map_err(|err| anyhow!(err.to_string()))
}
