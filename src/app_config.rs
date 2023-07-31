use crate::api::{get_vault_items, post_vault, put_vault_item};
use crate::key_extractor::PerPathBearerTokenKeyExtractor;
use actix_governor::governor::middleware::StateInformationMiddleware;
use actix_governor::{Governor, GovernorConfig, GovernorConfigBuilder};
use actix_web::middleware;
use actix_web::web::{self, get, post, put, ServiceConfig};
use once_cell::sync::Lazy;

type PerPathBearerTokenKeyExtractorGovernorConfig =
    GovernorConfig<PerPathBearerTokenKeyExtractor, StateInformationMiddleware>;

// Use Lazy for initializing the Arc<Governor::RateLimiter> inside the config to prevent independent rate limiting state
// 3 req per minute, 1 req replenish in 20s
/// A lazy initialized governor config for POST /vault.
static GOVERNOR_CONFIG_POST_VAULT: Lazy<PerPathBearerTokenKeyExtractorGovernorConfig> =
    Lazy::new(|| {
        GovernorConfigBuilder::default()
            .per_second(20)
            .burst_size(3)
            .key_extractor(PerPathBearerTokenKeyExtractor)
            .use_headers()
            .finish()
            .unwrap()
    });

// 1200 req per minute = 1 req replenish in 50ms
/// A lazy initialized governor config for GET /vault/items.
static GOVERNOR_CONFIG_GET_VAULT_ITEMS: Lazy<PerPathBearerTokenKeyExtractorGovernorConfig> =
    Lazy::new(|| {
        GovernorConfigBuilder::default()
            .per_millisecond(50)
            .burst_size(1200)
            .key_extractor(PerPathBearerTokenKeyExtractor)
            .use_headers()
            .finish()
            .unwrap()
    });

// 60 request per minute per item = 1 req replenish in 1s
/// A lazy initialized governor config for PUT /vault/items/{id}.
static GOVERNOR_CONFIG_PUT_VAULT_ITEM_WITH_ID: Lazy<PerPathBearerTokenKeyExtractorGovernorConfig> =
    Lazy::new(|| {
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(60)
            .key_extractor(PerPathBearerTokenKeyExtractor)
            .use_headers()
            .finish()
            .unwrap()
    });

/// An actix service configuration for the API.
pub fn config_app(cfg: &mut ServiceConfig) {
    // The governor configuration need to be lazily initialize to ensure they are shared among threads.
    cfg.service(
        web::scope("")
            .route(
                "/vault",
                post()
                    .to(post_vault)
                    .wrap(Governor::new(&GOVERNOR_CONFIG_POST_VAULT)),
            )
            .route(
                "/vault/items",
                get()
                    .to(get_vault_items)
                    .wrap(Governor::new(&GOVERNOR_CONFIG_GET_VAULT_ITEMS)),
            )
            .route(
                "/vault/items/{id}",
                put()
                    .to(put_vault_item)
                    .wrap(Governor::new(&GOVERNOR_CONFIG_PUT_VAULT_ITEM_WITH_ID)),
            )
            .wrap(middleware::Logger::default()),
    );
}
