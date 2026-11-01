import http.server
import sqlite3
import json

DB_PATH = "/data/data/com.termux/files/home/aura_analytics_engine/saas_billing_portal/ledger.db"

class MetricsHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()
        cursor.execute("SELECT cycle_id, bus_mw, grid_freq_chz, dispatch_cmd, prev_hash, record_hash FROM aaufsa_vpp_operational_ledger ORDER BY cycle_id DESC LIMIT 10")
        rows = cursor.fetchall()
        conn.close()

        html = "<html><head><title>Aura Telemetry</title><meta http-equiv='refresh' content='3'><style>body{font-family:monospace;background:#111;color:#0f0;padding:20px;}table{border-collapse:collapse;width:100%;}th,td{border:1px solid #333;padding:8px;text-align:left;}th{background:#222;color:#fff;}</style></head><body>"
        html += "<h2>AURA ENGINE - OPERATIONAL LEDGER (LIVE)</h2><table><tr><th>Cycle</th><th>Bus MW</th><th>Freq (cHz)</th><th>Command</th><th>Prev Hash</th><th>Record Hash</th></tr>"
        for r in rows:
            html += f"<tr><td>{r[0]}</td><td>{r[1]}</td><td>{r[2]}</td><td>{r[3]}</td><td>{r[4][:10]}...</td><td>{r[5][:10]}...</td></tr>"
        html += "</table></body></html>"

        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.end_headers()
        self.wfile.write(html.encode("utf-8"))

if __name__ == "__main__":
    server = http.server.HTTPServer(("0.0.0.0", 8080), MetricsHandler)
    server.serve_forever()
