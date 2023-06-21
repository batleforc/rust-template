terraform {
  required_providers {
    zitadel = {
      source  = "zitadel/zitadel"
      version = "1.0.0-alpha.18"
    }
  }
}

provider "zitadel" {
  domain           = "zitadel"
  insecure         = "true"
  port             = "8080"
  jwt_profile_file = "/machinekey/zitadel-admin-sa.json"
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
  preferred_language = "fr"
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

resource "zitadel_project_member" "batleforc_membership" {
  org_id     = zitadel_org.dev.id
  project_id = zitadel_project.rust_template.id
  user_id    = zitadel_human_user.batleforc.id
  roles      = ["ADMIN"]
}
