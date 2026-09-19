use std::sync::Arc;

use entity::repository::{
    game_invites_repo::GameInvitesRepository, game_moves_repo::GameMovesRepository,
    games_repo::GamesRepository, roles_repo::RolesRepository, scores_repo::ScoresRepository,
    user_throttles_repo::UserThrottlesRepository, users_repo::UsersRepository,
};
use sea_orm::DatabaseConnection;

use crate::{
    service::{
        access_control::{
            ip_rate_limiter::IpRateLimiter, rate_limiter::RateLimiter, throttle_cache::ThrottleCache,
        },
        entity_service::users_entity_service::UsersEntityService,
        game::{matchmaking_queue::MatchmakingQueue, sessions::GameSessions},
        redis_service::redis_service::RedisService,
    },
    utils::db_helpers::create_db_conn_pool,
};

#[derive(Clone)]
pub struct AppState {
    pub db_conn: Arc<DatabaseConnection>,
    pub redis_service: RedisService,

    pub users_repo: UsersRepository,
    pub user_throttles_repo: UserThrottlesRepository,
    pub games_repo: GamesRepository,
    pub game_moves_repo: GameMovesRepository,
    pub game_invites_repo: GameInvitesRepository,
    pub scores_repo: ScoresRepository,

    pub users_entity_service: UsersEntityService,

    pub throttle_cache: ThrottleCache,
    pub rate_limiter: RateLimiter,
    pub ip_rate_limiter: IpRateLimiter,
    pub matchmaking: MatchmakingQueue,
    pub game_sessions: GameSessions,
}

impl AppState {
    pub async fn try_new(db_conn: Option<DatabaseConnection>) -> anyhow::Result<Self> {
        let db_conn = match db_conn {
            Some(db_conn) => db_conn,
            None => create_db_conn_pool().await?,
        };

        Self::new_with_db(db_conn).await
    }

    pub async fn new_with_db(db_conn: DatabaseConnection) -> anyhow::Result<Self> {
        let db_conn = Arc::new(db_conn);

        let users_repo = UsersRepository::new(db_conn.clone());
        let roles_repo = RolesRepository::new(db_conn.clone());
        let user_throttles_repo = UserThrottlesRepository::new(db_conn.clone());
        let games_repo = GamesRepository::new(db_conn.clone());
        let game_moves_repo = GameMovesRepository::new(db_conn.clone());
        let game_invites_repo = GameInvitesRepository::new(db_conn.clone());
        let scores_repo = ScoresRepository::new(db_conn.clone());

        let throttle_cache = ThrottleCache::load(&user_throttles_repo).await?;
        let rate_limiter = RateLimiter::new();
        let ip_rate_limiter = IpRateLimiter::new();

        let redis_service = RedisService::new().await?;

        Ok(Self {
            db_conn: db_conn.clone(),
            redis_service,

            users_entity_service: UsersEntityService::new(users_repo.clone(), roles_repo.clone()),

            users_repo,
            user_throttles_repo,
            games_repo,
            game_moves_repo,
            game_invites_repo,
            scores_repo,

            throttle_cache,
            rate_limiter,
            ip_rate_limiter,
            matchmaking: MatchmakingQueue::new(),
            game_sessions: GameSessions::new(),
        })
    }
}
