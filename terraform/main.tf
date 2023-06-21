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
  email              = "maxleriche.60@gmail.com"
  is_email_verified  = true
  initial_password   = "Rust_template1"
}

resource "zitadel_org_member" "batleforc_org" {
  org_id  = zitadel_org.dev.id
  user_id = zitadel_human_user.batleforc.id
  roles   = ["ORG_OWNER"]
}
