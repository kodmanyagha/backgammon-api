pub const ROLE_MUST_BE_SET: &str = "error.role_must_be_set";
pub const UNAUTHORIZED: &str = "error.unauthorized";
pub const USER_NOT_FOUND: &str = "error.user_not_found";
pub const PARSE_ERROR: &str = "error.parse_error";
pub const FOLDER_NOT_FOUND: &str = "error.folder_not_found";
pub const INTERNAL_ERROR: &str = "error.internal";
pub const INVALID_REQUEST_BODY: &str = "error.invalid_request_body";
pub const INVALID_QUERY: &str = "error.invalid_query";

pub const MISSING_TOKENS: &str = "error.missing_tokens";
pub const INVALID_TOKEN: &str = "error.invalid_token";
pub const RATE_LIMIT_EXCEEDED: &str = "error.rate_limit_exceeded";
pub const PAYLOAD_TOO_LARGE: &str = "error.payload_too_large";
pub const GUEST_CREATION_PAUSED: &str = "error.guest_creation_paused";
pub const CAPTCHA_UNAVAILABLE: &str = "error.captcha_unavailable";

pub const EMAIL_ALREADY_TAKEN: &str = "error.email_already_taken";
pub const USERNAME_ALREADY_TAKEN: &str = "error.username_already_taken";
pub const PASSWORDS_DO_NOT_MATCH: &str = "error.passwords_do_not_match";
pub const INVALID_CREDENTIALS: &str = "error.invalid_credentials";

pub const GAME_NOT_FOUND: &str = "error.game_not_found";
pub const INVITE_NOT_FOUND: &str = "error.invite_not_found";
pub const INVITE_NOT_AVAILABLE: &str = "error.invite_not_available";
pub const INVITE_EXPIRED: &str = "error.invite_expired";
pub const CANNOT_ACCEPT_OWN_INVITE: &str = "error.cannot_accept_own_invite";

pub mod validation {
    pub const INVALID_PASSWORD_LEN: &str = "error.validation.invalid_password_len";
    pub const REQUIRED: &str = "error.validation.required";
    pub const INVALID_EMAIL: &str = "error.validation.invalid_email";
    pub const INVALID_GUEST_UNIQUE_ID: &str = "error.validation.invalid_guest_unique_id";
}
