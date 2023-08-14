# Rust Template / Hexa

## What's the goal of this rewrite ?

The goal of this rewrite is to make the code more readable, more maintainable and more efficient. It's also a good way to learn how to write test in Rust and go further in the hexa archi.

## Step

### 1. Rewrite the code

- [ ] Bring back the authentification system
- [ ] Bring back the User/Password sys (Otp and register)
- [ ] Bring back the User/OIDC sys
- [ ] Dev the app without taking into account the database sys
- [ ] Include Postgres database
- [ ] Include InMemory database
- [ ] Versioning of the api (/api/v1/...)
- [ ] Include Tracing
- [ ] Include Metrics (use the metrics from OpenTelemetry?)

### 2. Write tests

- [ ] Write unit tests
- [ ] Include test in the CI/CD
- [ ] Write integration tests

### 3. New features

- [ ] Dev api for the new BackOffice
- [ ] Integrates the future VueTS Front

### 4. Good Practices

- [ ] Include Hooks (https://docs.cocogitto.io/)
- [ ] Follow the commit convention (https://www.conventionalcommits.org/en/v1.0.0/)

## Hexa WorkFlow

### Hexa Archi

### Domain

#### Auth && User

```mermaid
classDiagram
    class User{
        UUID id
        String email
        String password
        String nom
        String prenom
        Option< String > otp_secret
        Option< String > otp_url
        Bool otp_enabmed
        Bool is_oauth
        Option< String > one_time_token
        DateTime created_at
        DateTime updated_at
        +validate_password(password) Result< bool,PasswordError >
        +update_password(password) Result< bool,PasswordError >
        +gen_otp_secret() Result< bool,TotpError >
        +create_otp_url(app_name) Result< bool,TotpError >
    }
    class TokenExtractError{
        <<Enum>>
        InvalidToken(String)
        OidcDisabled()
    }
    class Token{
        <<Enum>>
        Access(String)
        Refresh(String)
        Oidc(String)

        +get_user_email() Result< String, TokenExtractError >
    }
    Token --> TokenExtractError
```

#### Auth Oidc

```mermaid
classDiagram
    class OidcConfig{
        +Option< OidcHandler > back
        +Option< OidcFront > front
        +bool oidc_disabled
        +new() OidcConfig,VarError
        +new_disabled() OidcConfig
        +new_back() OidcHandler
        +new_front() OidcFront
    }

    class OidcFront{
        +String client_id
        +String token_url
        +String auth_url
        +String redirect_url
        +String scopes
        +String issuer
        +get_scope() Vec<String>
    }

    class OidcTokenClaim{
        +String iss
        +String sub
        +String aud
        +Usize exp
        +Usize iat
        +new(client_id,issuer) OidcTokenClaim
        -new_header(key_id) Header
        +sign_token(key_id,private_key) String
    }

    class OidcHandler{
        +String client_id
        +String client_secret
        +String issuer
        +String redirect_url
        +String scopes
        +String userinfo_url
        +String introspection_url
        +String key_id
        +String client_assertion_type
    }
```

#### Auth Local

```mermaid
classDiagram
    class TokenError{
        <<Enum>>
        InvalidSignToken(String)
        InvalidToken(String)
        WrongTokenType(String)
    }
    class TokenClaims{
        +UUID sub
        +String Email
        +Usise exp
        +Usise iat
        +String iss
        +Bool refresh
        +new()
        gen_header()
        +get_key()
        +sign_token()
        +validate_token()
    }
    TokenClaims --> TokenError

    class PasswordError{
        <<Enum>>
        HashEngineError(String)
    }
    class Password{
        +hash(password) Result < String,PasswordError >
        +verify(password,hash) Result < bool,PasswordError >
    }
    Password --> PasswordError


    class TotpError{
        <<Enum>>
        InvalidSecret(String)
        ValidateSecret(String)
    }
    class Totp{
        +get_totp_obj(email, secret, app_name) Result< TOTP, TotpError >
        +get_otp_url(email, secret, app_name) Result< String, TotpError >
        +gen_otp_secret() Result< String, TotpError >
        +validate_otp(otp,email, secret, app_name) Result < bool,TotpError >
    }
    Totp --> TotpError
```

## Dev Good Practices

In order to have a clean code, we will follow the following rules:

- Use Cocogitto Hooks (https://docs.cocogitto.io/)
- Use the commit convention (https://www.conventionalcommits.org/en/v1.0.0/)
- Test the code
- Use the tracing lib (https://docs.rs/tracing/latest/tracing/) and follow the OpenTelemetry logic (https://opentelemetry.io/docs/concepts/)
- Follow the hexagonal archi

### Before commit

- Create a new branch from the dev branch
- Install the hooks (https://docs.cocogitto.io/) with `cog install-hook --all`
- Make sure that all the tests are passing (`cargo test`)
- Make sure that the code you wrote is tested
- Make sure that the code you wrote is formatted (`cargo fmt`)
- Make sure that the code you wrote is linted (`cargo clippy`)
- Make sure that the code you wrote is updated in the Readme.md
- Make sure that the code you wrote is documented
