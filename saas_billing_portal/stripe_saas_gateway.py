import http.server
import json
import os
import subprocess
import hmac
import hashlib
import time
import sqlite3

PORT = 3000
SUBS_DB = "/data/data/com.termux/files/home/aura_analytics_engine/saas_billing_portal/subscriptions.json"
LEDGER_DB = "/data/data/com.termux/files/home/aura_analytics_engine/saas_billing_portal/ledger.db"
ENGINE_BIN = "/data/data/com.termux/files/home/aura_analytics_engine/target/release/aura_analytics_engine"
WEBHOOK_SECRET = "whsec_2L8lmTzgWHVV7wrgfya0abFoUllfGMFU"

def verify_stripe_signature(payload: bytes, sig_header: str, secret: str, tolerance: int = 300) -> bool:
    if not sig_header or not secret:
        return False
    try:
        pairs = dict(item.strip().split("=", 1) for item in sig_header.split(","))
        timestamp = pairs.get("t")
        expected_sig = pairs.get("v1")
        if not timestamp or not expected_sig:
            return False
        if abs(time.time() - int(timestamp)) > tolerance:
            return False
        signed_payload = f"{timestamp}.".encode("utf-8") + payload
        computed_sig = hmac.new(secret.encode("utf-8"), signed_payload, hashlib.sha256).hexdigest()
        return hmac.compare_digest(computed_sig, expected_sig)
    except Exception:
        return False

def activate_subscription(api_key: str, customer_id: str = ""):
    try:
        data = {}
        if os.path.exists(SUBS_DB):
            with open(SUBS_DB, "r") as f:
                data = json.load(f)
        data[api_key] = {
            "active": True,
            "customer_id": customer_id,
            "updated_at": int(time.time())
        }
        with open(SUBS_DB, "w") as f:
            json.dump(data, f, indent=2)
        print(f"[SUBSCRIPTION] Activada: {api_key}")
    except Exception as e:
        print(f"[ERROR SUBS] {e}")

def record_ledger_receipt(receipt: dict):
    try:
        conn = sqlite3.connect(LEDGER_DB)
        cursor = conn.cursor()
        assessment = receipt.get("assessment", {})
        cursor.execute("""
            INSERT INTO aaufsa_vpp_operational_ledger (
                cycle_id, org_id, domain_id, bus_mw, grid_freq_chz, dispatch_cmd,
                displaced_payroll_cents, total_tax_cents, oasi_trust_cents, reskilling_trust_cents,
                prev_hash, record_hash
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            receipt.get("cycle_id"),
            receipt.get("organization_id"),
            receipt.get("domain_id"),
            receipt.get("updated_bus_mw"),
            receipt.get("grid_frequency_chz"),
            receipt.get("dispatch_command"),
            assessment.get("displaced_payroll_cents", 0),
            assessment.get("total_tax_amount_cents", 0),
            assessment.get("social_security_allocation_cents", 0),
            assessment.get("workforce_reskilling_allocation_cents", 0),
            receipt.get("previous_ledger_hash"),
            receipt.get("current_ledger_hash")
        ))
        conn.commit()
        conn.close()
        print(f"[LEDGER AUTO-INSERT] Ciclo {receipt.get('cycle_id')} registrado en ledger.db")
    except Exception as e:
        print(f"[ERROR LEDGER AUTO-INSERT] {e}")

class AuraSaaSHandler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length)

        if self.path == "/api/v1/stripe/webhook":
            sig_header = self.headers.get("Stripe-Signature", "")
            if not verify_stripe_signature(body, sig_header, WEBHOOK_SECRET):
                self.send_response(400)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(b'{"error": "Invalid signature"}')
                return

            try:
                event = json.loads(body.decode("utf-8"))
                event_type = event.get("type", "")
                data_obj = event.get("data", {}).get("object", {})
                if event_type in ("checkout.session.completed", "payment_intent.succeeded"):
                    api_key = data_obj.get("metadata", {}).get("api_key") or data_obj.get("client_reference_id")
                    customer_id = data_obj.get("customer", "")
                    if api_key:
                        activate_subscription(api_key, customer_id)
            except Exception as e:
                print(f"[ERROR STRIPE EVENT] {e}")

            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(b'{"status": "processed"}')
            return

        if self.path == "/api/v1/dispatch":
            api_key = self.headers.get("X-API-Key", "")
            try:
                with open(SUBS_DB, "r") as f:
                    subs = json.load(f)
            except Exception:
                subs = {}

            if api_key not in subs or not subs[api_key].get("active"):
                self.send_response(402)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(b'{"error": "402 Payment Required"}')
                return

            proc = subprocess.run([ENGINE_BIN, "--ingest-json", body.decode("utf-8")], capture_output=True, text=True)
            if proc.returncode == 0:
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(proc.stdout.encode("utf-8"))
                
                try:
                    receipt = json.loads(proc.stdout)
                    record_ledger_receipt(receipt)
                except Exception:
                    pass
            else:
                self.send_response(400)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": proc.stderr}).encode("utf-8"))
            return

        self.send_response(404)
        self.end_headers()

if __name__ == "__main__":
    server = http.server.HTTPServer(("0.0.0.0", PORT), AuraSaaSHandler)
    server.serve_forever()
