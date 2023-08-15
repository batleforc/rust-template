# Function

## Auth

### Login

INPUT: username, password

OUTPUT: enum[OTP,REFRESH], (user,refresh_token) if REFRESH, one time password if OTP (the one time password bind with the otp token allow to generate a refresh token)

ERROR:

- Invalid username or password
- User is Oauth user
- Server error
