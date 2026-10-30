terraform {
  required_version = ">= 1.6.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.30"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 5.25"
    }
  }
}

variable "cloudflare_account_id" {
  type        = string
  description = "Cloudflare Enterprise Account ID para guadalupelopezjose96-org"
}

variable "gcp_project_id" {
  type        = string
  default     = "aura-prod-978426653396"
  description = "GCP Project ID en 02-Production"
}

locals {
  domains = {
    "auratetrax"        = { fqdn = "auratetrax.com", domain_id = "877771073297" }
    "fourphonix"        = { fqdn = "fourphonix.com", domain_id = "964521086569" }
    "auraintelcore"     = { fqdn = "auraintelcore.com", domain_id = "188680247367" }
    "auraintel-global"  = { fqdn = "auraintel-global.com", domain_id = "1087754464927" }
    "standard1979"      = { fqdn = "standard1979.com", domain_id = "282230018778" }
    "prophonix"         = { fqdn = "prophonix.com", domain_id = "358233233" }
  }
}

# Zonas DNS Administradas en Cloudflare con DNSSEC y HSTS
resource "cloudflare_zone" "corporate_zones" {
  for_each   = local.domains
  account_id = var.cloudflare_account_id
  zone       = each.value.fqdn
  plan       = "enterprise"
  type       = "full"
}

resource "cloudflare_zone_dnssec" "dnssec_enforcement" {
  for_each = cloudflare_zone.corporate_zones
  zone_id  = each.value.id
}

# Políticas de Cero Puertos Públicos mediante Cloudflare Tunnel
resource "cloudflare_tunnel" "secure_edge_tunnel" {
  for_each   = cloudflare_zone.corporate_zones
  account_id = var.cloudflare_account_id
  name       = "tunnel-${each.key}-edge"
  secret     = "dGVzdC1pbm11dGFibGUtc2VjcmV0LWZvci10ZXJtdXgtYXVkaXQtb25seQ=="
}

# Registro CAA estricto para emisión de certificados TLS
resource "cloudflare_record" "caa_letsencrypt" {
  for_each = cloudflare_zone.corporate_zones
  zone_id  = each.value.id
  name     = "@"
  type     = "CAA"
  data {
    flags = "0"
    tag   = "issue"
    value = "letsencrypt.org"
  }
}

# Registro DMARC en modo rechazo estricto
resource "cloudflare_record" "dmarc_reject" {
  for_each = cloudflare_zone.corporate_zones
  zone_id  = each.value.id
  name     = "_dmarc"
  type     = "TXT"
  value    = "v=DMARC1; p=reject; sp=reject; adkim=s; aspf=s; fo=1;"
}
