use axum_macros::FromRef;
use slog::Logger;
use crate::{cache::CacheService, database::Database};

#[derive(Clone, FromRef)]
pub struct SharedState {
    pub(crate) database: Database,
    pub(crate) cache: CacheService,
    pub(crate) logger: Logger,
}