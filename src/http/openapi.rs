use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{
    http::controllers,
    service::{
        authentication::{
            guest::GuestInputDto, login::LoginInputDto, register::RegisterInputDto,
            update_profile::UpdateProfileInputDto, AuthResultDto,
        },
        entity_service::users_entity_service::{CreateUserInputDto, UpdateUserInputDto},
        game::{
            create_invite::InviteResultDto, get_history::GameHistoryItemDto,
            get_score::ScoreDto, preview_invite::InvitePreviewDto,
            quick_match::QuickMatchResultDto,
        },
    },
    types::http::api_response::ApiResponse,
    utils::datatable::Datatable,
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().expect("components must exist");
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Tavla API",
        description = "Backend for the Tavla (backgammon) game.",
        version = "0.1.0",
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Home", description = "Service info"),
        (name = "Auth", description = "Register, login and guest sessions"),
        (name = "Profile", description = "Current user's profile"),
        (name = "Games", description = "Matchmaking, invites and match lookup"),
        (name = "Settings", description = "Roles and permissions"),
    ),
    paths(
        controllers::home::index::get_index,
        controllers::v1::auth::register::post_register,
        controllers::v1::auth::login::post_login,
        controllers::v1::auth::guest::post_guest,
        controllers::v1::profile::index::get_index,
        controllers::v1::profile::update::patch_update,
        controllers::v1::profile::score::get_score,
        controllers::v1::games::quick_match::post_join,
        controllers::v1::games::quick_match::delete_leave,
        controllers::v1::games::invites::post_create,
        controllers::v1::games::invites::get_preview,
        controllers::v1::games::invites::post_accept,
        controllers::v1::games::history::get_history,
        controllers::v1::games::show::get_show,
        controllers::v1::games::ws::handle_upgrade,
        controllers::v1::settings::permissions::get_index,
        controllers::v1::settings::roles::get_index,
    ),
    components(schemas(
        ApiResponse,
        Datatable<entity::permissions::Model>,
        CreateUserInputDto,
        UpdateUserInputDto,
        RegisterInputDto,
        LoginInputDto,
        GuestInputDto,
        UpdateProfileInputDto,
        AuthResultDto,
        QuickMatchResultDto,
        InviteResultDto,
        InvitePreviewDto,
        GameHistoryItemDto,
        ScoreDto,
        entity::users::Model,
        entity::games::Model,
        entity::games::GameStatus,
        entity::game_invites::GameInviteStatus,
        entity::permissions::Model,
        entity::utils::enums::active_passive_status::ActivePassiveStatus,
    ))
)]
pub struct ApiDoc;
