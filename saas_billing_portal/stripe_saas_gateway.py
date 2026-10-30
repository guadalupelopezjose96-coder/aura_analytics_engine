#!/usr/bin/env python3
import http.server
import json
import os
import subprocess

PORT = int(os.environ.get("AURA_SAAS_PORT", "8080"))
CATALOG = {
    "1_vpp_pro_49_99": "https://buy.stripe.com/5kQ14n1zcgKQ96v0UGcjS00",
    "2_der_node_addon_99": "https://buy.stripe.com/5kQaEX4LocuAaazdHscjS02",
    "3_enterprise_core_499": "https://buy.stripe.com/5kQ28r91E9io1E30UGcjS01",
    "4_standard1979_audit_1500": "https://buy.stripe.com/8x28wPcdQ3Y44Qf9rccjS05",
    "5_dedicated_cloud_2500": "https://buy.stripe.com/3cIdR9fq266cfuT5aWcjS03",
    "6_omni_group_10000": "https://buy.stripe.com/9B614n7XA66cfuTgTEcjS04"
}
ENGINE_BIN = os.path.expanduser("~/aura_analytics_engine/target/release/aura_analytics_engine")
SUBS_DB = os.path.expanduser("~/aura_analytics_engine/audit_logs/active_subscriptions.json")
DOCS_DIR = os.path.expanduser("~/aura_analytics_engine/docs")

os.makedirs(os.path.dirname(SUBS_DB), exist_ok=True)
if not os.path.exists(SUBS_DB):
    with open(SUBS_DB, "w") as f:
        json.dump({"AURA_LIVE_SUB_KEY_2026": {"plan": "6_omni_group_10000", "active": True}}, f, indent=2)

class AuraSaaSHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path in ("/", "/index.html"):
            with open(os.path.join(DOCS_DIR, "index.html"), "rb") as f:
                content = f.read()
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(content)
        elif self.path in ("/standard1979", "/standard1979.html"):
            with open(os.path.join(DOCS_DIR, "standard1979.html"), "rb") as f:
                content = f.read()
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(content)
        elif self.path == "/api/status":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({
                "status": "ONLINE_100_PERCENT_MONETIZED",
                "org_id": 320717657398,
                "portals": {
                    "auraintelcore.com": 188680247367,
                    "standard1979.com": 282230018778
                },
                "stripe_live_links": CATALOG
            }, indent=2).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length).decode("utf-8")
        if self.path == "/api/v1/dispatch":
            api_key = self.headers.get("X-API-Key", "")
            with open(SUBS_DB, "r") as f:
                subs = json.load(f)
            if api_key not in subs or not subs[api_key].get("active"):
                self.send_response(402)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({
                    "error": "402 Payment Required",
                    "stripe_live_links": CATALOG
                }).encode())
                return
            proc = subprocess.run([ENGINE_BIN, "--ingest-json", body], capture_output=True, text=True)
            self.send_response(200 if proc.returncode == 0 else 400)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write((proc.stdout if proc.returncode == 0 else json.dumps({"error": proc.stderr})).encode())

if __name__ == "__main__":
    server = http.server.HTTPServer(("0.0.0.0", PORT), AuraSaaSHandler)
    server.serve_forever()
