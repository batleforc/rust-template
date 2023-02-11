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

### POST /api/auth/login

### GET /api/auth/logout

### POST /api/auth/register

### GET /api/auth/refresh

### GET /api/auth/otp/generate

### POST /api/auth/otp/activate

### POST /api/auth/otp/validate

Secu : https://github.com/juhaku/utoipa/blob/master/examples/todo-actix/src/todo.rs#L135
