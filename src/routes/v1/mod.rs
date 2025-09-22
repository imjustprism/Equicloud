use axum::{Router, middleware, routing::head};

pub mod oauth;
pub mod settings;

pub fn register() -> Router {
    let oauth_routes = oauth::register();

    let auth_routes = Router::new()
        .route(
            "/settings",
            head(crate::lib::settings_handlers::handle_head_settings)
                .get(crate::lib::settings_handlers::handle_get_settings)
                .put(crate::lib::settings_handlers::handle_put_settings)
                .delete(crate::lib::settings_handlers::handle_delete_settings),
        )
        .route_layer(middleware::from_fn(
            crate::middleware::auth::auth_middleware,
        ));

    Router::new().merge(oauth_routes).merge(auth_routes)
}
