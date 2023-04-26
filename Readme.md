# Api

## Description

This is a template of an api to have a quick start.

This api is linked to a [vue3 template](https://github.com/batleforc/vue-template).

This api also help me learn how to use:

- tracing/metrics/open telemetry
- Prometheus
- swagger
- Otp (with rust, already done in golang)
- actix-web deeper (middleware, guards, wrap, etc)

What this api does:

- Auth module (Login,Logout,Register,Refresh(token), Otp)
- User module (Read,Update,Delete) the create is done by the auth module
- Expose a swagger doc
- Expose Metrics and Tracing
- Handle Postgres
- And some more handsome stuff

## Auth Endpoint

Secu : <https://github.com/juhaku/utoipa/blob/master/examples/todo-actix/src/todo.rs#L135>

### GET /api/auth => Return the auth model : DONE

### POST /api/auth/login => Include the otp code if the user has activated it : DONE

### GET /api/auth/logout : DONE

### POST /api/auth/register : DONE

### GET /api/auth/refresh : DONE

### GET /api/auth/logout-all

### GET /api/auth/otp/activate => Gen QRCODE string : DONE

### POST /api/auth/otp/activate => Activate the otp : Done

### POST /api/auth/otp/validate => Validate the otp (2FA login) : DONE

<https://crates.io/crates/totp-rs>

<https://qoomon.github.io/otp-authenticator-webapp/>

## User Endpoint

### GET /api/user : DONE

### GET /api/user/{id} : DONE

### PUT /api/user : DONE

### DELETE /api/user : DONE

## Asset Endpoint

### GET /api/asset/{id}/download

### GET /api/asset/{id}

### POST /api/asset

### PUT /api/asset

### DELETE /api/asset

## TODO

- [ ] Create a container for the api
- [ ] Create a helm chart
- [ ] Integrate the CI/CD pipeline
- [ ] Add support for OIDC
- [ ] Create a vue3 template with model auto generation
- [ ] Integrate the api with S3 (asset endpoint)
- [ ] Make the code more KISS (Keep it simple stupid)
- [ ] WRITE SOME TESTS

### TODO : Kiss

- [ ] Factorize the login and validate_otp (refresh token handling)

### TODO : Ci/Cd

- [ ] Sonarqube
- [ ] Dependency check
- [ ] Build the docker image and push it to the registry (handle the multi env)
- [ ] Deploy the helm chart (handle the multi env)

### TODO : OIDC

- [ ] Enable/Disable OIDC
- [ ] Add the OIDC config in the helm chart
- [ ] Add endpoint to authenticate user with OIDC
- [ ] Handle case where the user log for the first time
- [ ] Handle case where the email is already used

## Pre-requisites

- Rust
- OpenSSL
- Make
- Docker (for the database and the tracing/metrics)

## Test de charge

<https://github.com/fcsonline/drill>

## Configuring Jaeger collector

- <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/sdk-environment-variables.md#jaeger-exporter>

## Generate swagger.json

```bash
cargo run --bin generate
```
