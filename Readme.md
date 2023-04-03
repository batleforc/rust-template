# Api

## Description

This is a template of an api to have a quick start.

This api also help me learn how to use:

- tracing/metrics/open telemetry
- Prometheus
- swagger
- Otp
- actix-web deeper (middleware, guards, etc)
- Rate limiting

What this api does:

- Auth module (Login,Logout,Register,Refresh(token), Otp)
- User module (Read,Update,Delete) the create is done by the auth module
- Asset module (CRUD) though S3 (minio)
- Expose a swagger doc
- Expose Metrics and Tracing
- Handle Postgres

## Auth Endpoint

Secu : <https://github.com/juhaku/utoipa/blob/master/examples/todo-actix/src/todo.rs#L135>

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

## Pre-requisites

- Rust
- OpenSSL
- Make
- Docker (for the database and the tracing/metrics)

## Test de charge

<https://github.com/fcsonline/drill>
