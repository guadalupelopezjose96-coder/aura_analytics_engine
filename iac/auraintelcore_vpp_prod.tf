# Aprovisionamiento de Producción: AuraIntel VPP en AuraIntelCore.com (188680247367)
# Carpeta de Ciclo de Vida: 02-Production (978426653396)

variable "auraintelcore_project_id" {
  type        = string
  default     = "188680247367"
  description = "AuraIntelCore-com Production Tenant ID"
}

resource "google_project_service" "auraintelcore_gemini_api" {
  project            = var.auraintelcore_project_id
  service            = "generativelanguage.googleapis.com"
  disable_on_destroy = false
}

resource "cloudflare_record" "auraintel_vpp_core_mtls_endpoint" {
  zone_id = "auraintelcore-com-zone-188680247367"
  name    = "vpp.auraintelcore.com"
  content = "100.64.0.1"
  type    = "A"
  proxied = true
  comment = "Endpoint Zero-Trust mTLS + ML-KEM-768 para AuraIntel VPP en 02-Production (978426653396)"
}
