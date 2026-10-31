terraform {
  required_version = ">= 1.7.0"
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.40.0"
    }
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.35.0"
    }
  }
}

variable "holding_org_id" {
  type        = string
  default     = "320717657398"
  description = "guadalupelopezjose96-org Holding ID"
}

variable "production_folder_id" {
  type        = string
  default     = "978426653396"
  description = "02-Production Lifecycle Folder ID"
}

resource "google_org_policy_policy" "disable_sa_key_creation" {
  name   = "organizations/${var.holding_org_id}/policies/iam.disableServiceAccountKeyCreation"
  parent = "organizations/${var.holding_org_id}"

  spec {
    rules {
      enforce = "TRUE"
    }
  }
}

resource "google_org_policy_policy" "restrict_gemini_api_targets" {
  name   = "organizations/${var.holding_org_id}/policies/serviceusage.restrictApiTargets"
  parent = "organizations/${var.holding_org_id}"

  spec {
    rules {
      values {
        allowed_values = ["service:generativelanguage.googleapis.com"]
      }
    }
  }
}

resource "google_storage_bucket" "sovereign_worm_audit_vault" {
  name                        = "guadalupelopezjose96-org-worm-audit-320717657398"
  location                    = "US"
  storage_class               = "STANDARD"
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"

  retention_policy {
    is_locked        = true
    retention_period = 315360000
  }

  versioning {
    enabled = true
  }
}
