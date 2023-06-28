use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify,
};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "access_token",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
        components.add_security_scheme(
            "refresh_token",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
        components.add_security_scheme(
            "Oidc",
            SecurityScheme::Http(Http::new(HttpAuthScheme::OAuth)),
        )
    }
}
