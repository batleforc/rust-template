# Function

[doc](https://towardsdev.com/hexagonal-architecture-in-rust-the-use-cases-7d5a88bd0a4)
[doc-driven](https://towardsdev.com/hexagonal-architecture-in-rust-driven-adapters-ab02ed335dc5)

## Auth

### Login

INPUT: username, password

OUTPUT: (enum[OTP,REFRESH], (user,refresh_token) if REFRESH, one time password if OTP (the one time password bind with the otp token allow to generate a refresh token))

ERROR:

- Invalid username or password
- User is Oauth user
- Server error

### Logout

INPUT: refresh_token

OUTPUT: (None)

ERROR:

- Invalid refresh token
- Server error

### Refresh

INPUT: refresh_token

OUTPUT: (access_token)

ERROR:

- Invalid refresh token
- Server error

## Auth/Register

### Register

INPUT: surname, name, password, email

OUTPUT: (bool)

ERROR:

- Invalid Parameter
- User already exist
- Server error

### RegisterOauth

INPUT: OIDC_token

OUTPUT: (User)

ERROR:

- Invalid OIDC_token
- OIDC not enabled
- Server error
- (This endpoint is called each time a user login with OIDC to update the user information so no user already exist error)

## Auth/OTP

### GenerateOTP

Start the otp activation process

INPUT: refresh_token

OUTPUT: (url, qr_code)

ERROR:

- Invalid refresh token
- Already activated
- Can't activate (user is oauth)
- Server error

### ActivateOTP

INPUT: refresh_token, otp_code

OUTPUT: (bool)

ERROR:

- Invalid refresh token
- Invalid otp code
- Server error

### ValidateOTP

INPUT: OneTimePassword, OTP_token

OUTPUT: (refresh_token, user)

ERROR:

- Invalid OTP_token
- Invalid OneTimePassword
- Server error
