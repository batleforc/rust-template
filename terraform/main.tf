terraform {
  required_providers {
    zitadel = {
      source  = "zitadel/zitadel"
      version = "1.0.0-alpha.18"
    }
  }
}

variable "path" {
  type    = string
  default = "../machinekey"
}

variable "domain" {
  type    = string
  default = "localhost"
}

#provider "zitadel" {
#  domain           = "localhost"
#  insecure         = "true"
#  port             = "8080"
#  jwt_profile_file = "/machinekey/zitadel-admin-sa.json"
#}

provider "zitadel" {
  domain           = var.domain
  insecure         = "true"
  port             = "8080"
  jwt_profile_file = "${var.path}/zitadel-admin-sa.json"
}

resource "zitadel_org" "dev" {
  name = "rust_template"
}

resource "zitadel_human_user" "batleforc" {
  org_id             = zitadel_org.dev.id
  user_name          = "batleforc@localhost.com"
  first_name         = "max"
  last_name          = "batleforc"
  nick_name          = "batleforc"
  display_name       = "Maxime"
  preferred_language = "en"
  gender             = "GENDER_MALE"
  email              = "max@weebo.fr"
  is_email_verified  = true
  initial_password   = "Rust_template1"
}

resource "zitadel_org_member" "batleforc_org" {
  org_id  = zitadel_org.dev.id
  user_id = zitadel_human_user.batleforc.id
  roles   = ["ORG_OWNER"]
}

resource "zitadel_instance_member" "batleforc_instance" {
  user_id = zitadel_human_user.batleforc.id
  roles   = ["IAM_OWNER"]
}

resource "zitadel_project" "rust_template" {
  org_id                 = zitadel_org.dev.id
  name                   = "rust_template"
  project_role_assertion = true
}

resource "zitadel_project_role" "admin" {
  org_id       = zitadel_org.dev.id
  project_id   = zitadel_project.rust_template.id
  role_key     = "ADMIN"
  display_name = "Admin"
  group        = "ADMIN"
}

resource "zitadel_project_role" "moderator" {
  org_id       = zitadel_org.dev.id
  project_id   = zitadel_project.rust_template.id
  role_key     = "MODERATOR"
  display_name = "Moderator"
  group        = "MODERATOR"
}
resource "zitadel_project_role" "member" {
  org_id       = zitadel_org.dev.id
  project_id   = zitadel_project.rust_template.id
  role_key     = "MEMBER"
  display_name = "Member"
  group        = "MEMBER"
}


# resource "zitadel_application_api" "backend" {
#   org_id           = zitadel_org.dev.id
#   project_id       = zitadel_project.rust_template.id
#   name             = "backend"
#   auth_method_type = "API_AUTH_METHOD_TYPE_PRIVATE_KEY_JWT"
# }

# resource "zitadel_application_key" "backend_key" {
#   org_id          = zitadel_org.dev.id
#   project_id      = zitadel_project.rust_template.id
#   app_id          = zitadel_application_api.backend.id
#   key_type        = "KEY_TYPE_JSON"
#   expiration_date = "2500-12-31T23:59:59Z"
# }

# resource "local_file" "rust_template_key" {
#   content  = zitadel_application_key.backend_key.key_details
#   filename = "/machinekey/rust_template_key.json"
# }

# We are possibly not in the case of an api but in a Web case (and the vue side should be an user agent)
# https://registry.terraform.io/providers/zitadel/zitadel/latest/docs/resources/application_oidc

resource "zitadel_application_oidc" "backend" {
  project_id                  = zitadel_project.rust_template.id
  org_id                      = zitadel_org.dev.id
  name                        = "backend"
  redirect_uris               = ["http://localhost:5437", "http://localhost:5437/oauth2/callback", "http://localhost:5437/api/oauth2/callback"]
  response_types              = ["OIDC_RESPONSE_TYPE_CODE"]
  grant_types                 = ["OIDC_GRANT_TYPE_AUTHORIZATION_CODE"]
  post_logout_redirect_uris   = ["http://localhost:5437/api/oauth2/logout"]
  app_type                    = "OIDC_APP_TYPE_WEB"
  auth_method_type            = "OIDC_AUTH_METHOD_TYPE_NONE"
  dev_mode                    = true
  access_token_type           = "OIDC_TOKEN_TYPE_BEARER"
  access_token_role_assertion = true
  id_token_role_assertion     = true
  id_token_userinfo_assertion = true
  additional_origins          = ["http://localhost:5437"]
}

resource "local_file" "rust_template_key" {
  content  = "{\"id\":\"${zitadel_application_oidc.backend.client_id}\",\"secret\":\"${zitadel_application_oidc.backend.client_secret}\"}"
  filename = "${var.path}/rust_template_key.json"
}
