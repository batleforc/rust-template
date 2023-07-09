use utoipa::{
    openapi::security::{
        AuthorizationCode, Flow, Http, HttpAuthScheme, OAuth2, Scopes, SecurityScheme,
    },
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
            "oidc",
            SecurityScheme::OAuth2(OAuth2::with_description(
                [Flow::AuthorizationCode(AuthorizationCode::new(
                    "http://localhost:8080/oauth/v2/authorize",
                    "http://localhost:8080/oauth/v2/token",
                    Scopes::from_iter([
                        ("openid", "Access auth (needed)"),
                        ("profile", "Access profile (needed)"),
                        ("email", "Access email (needed)"),
                        ("offline_access", "Access offline (needed)"),
                    ]),
                ))],
                "Zitadel dev",
            )),
        )
    }
}
