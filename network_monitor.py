import threading
import base64
import json
import socket
import time
import random

# ==========================================
# 1. Configuration & Key Management
# ==========================================
# محاولة تحميل مفاتيح التشفير من ملفات الـ JSON
try:
    with open("decrypted_artifacts.json", "r") as f:
        artifacts = json.load(f)
    XOR_KEY = artifacts.get("xorkey", 0x5B)
    SIGNATURE = artifacts.get("signature", "Qkoko")
    print("[INIT] Keys loaded successfully from artifacts.")
except FileNotFoundError:
    # حماية: قيم افتراضية للاختبار المعزول في حال عدم وجود الملف
    XOR_KEY = 0x5B
    SIGNATURE = "Qkoko"
    print("[INIT] Warning: decrypted_artifacts.json not found. Using default keys.")

# ==========================================
# 2. Core Engine (Decryption & Protocol)
# ==========================================
def decrypt_payload(payload):
    """دالة لفك التشفير وإزالة الغلاف الأمني (Signature -> Base64 -> XOR)"""
    try:
        if not payload.startswith(SIGNATURE + ":"):
            return None
        encoded = payload.split(":", 1)[1]
        raw = base64.b64decode(encoded)
        decrypted = bytes([b ^ XOR_KEY for b in raw]).decode(errors="ignore")
        return decrypted
    except Exception as e:
        return f"[Error Decoding] {e}"

def smart_inject(user_cmd, udp_sock, tcp_sock):
    """دالة الحقن الذكي: تقوم بتمويه الأمر وبناء حزمة مطابقة لمعايير النظام"""
    # 1. بناء الهيكل المتوافق مع بصمة الـ DNA
    payload_struct = {
        "node": 46,
        "command": user_cmd,
        "meta": {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "session": "diag" # Session وهمية للتشخيص
        }
    }

    # 2. التشفير (XOR -> Base64)
    raw_json = json.dumps(payload_struct)
    raw = bytes([ord(c) ^ XOR_KEY for c in raw_json])
    encoded = base64.b64encode(raw).decode()

    # 3. توقيع مزدوج + Timestamp لمنع هجمات الـ Replay
    packet = f"{SIGNATURE}:{encoded}:{int(time.time())}"
    handshake = f"{SIGNATURE}:HELLO:{int(time.time())}"

    # 4. إرسال Handshake للتمويه (فتح الباب)
    udp_sock.sendto(handshake.encode(), ("127.0.0.1", 3702))
    try:
        tcp_sock.send(handshake.encode())
    except Exception:
        pass # تجاهل الخطأ بصمت إذا لم يكن الـ TCP جاهزاً
    
    # تأخير زمني بشري (Jitter)
    time.sleep(random.uniform(0.2, 0.5))

    # 5. إرسال الـ Payload مقسماً (Chunking) للهروب من الفحص
    for chunk in [packet[i:i+50] for i in range(0, len(packet), 50)]:
        udp_sock.sendto(chunk.encode(), ("127.0.0.1", 3702))
        try:
            tcp_sock.send(chunk.encode())
        except Exception:
            pass
        time.sleep(random.uniform(0.1, 0.3)) # Whisper delay

    print(f"[Smart Inject] Command '{user_cmd}' injected successfully.")

    # ==========================================
    # 3. Feedback Loop (المراقبة العكسية)
    # ==========================================
    udp_sock.settimeout(1.5) # وقت كافٍ للـ Agent ليرد
    try:
        data, addr = udp_sock.recvfrom(4096)
        decoded = decrypt_payload(data.decode(errors="ignore"))
        if decoded:
            print(f"[Feedback] Response from {addr}: {decoded}")
    except socket.timeout:
        print("[Feedback] Timeout: No response from tunnel within 1.5 seconds.")
    except Exception as e:
        print(f"[Feedback Error] {e}")

# ==========================================
# 4. Interactive Console (واجهة التحكم)
# ==========================================
def interactive_mode():
    """واجهة سطر الأوامر لتلقي أوامر المشغل وحقنها"""
    udp_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    tcp_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    tcp_sock.settimeout(1.0)
    
    try:
        tcp_sock.connect(("127.0.0.1", 8080))
    except Exception:
        pass # تجاهل إذا لم يكن المنفذ مفتوحاً بعد

    print("\n" + "="*50)
    print("[*] INTERACTIVE CONTROL TERMINAL ACTIVE [*]")
    print("[*] Type your command (e.g., PING, START). Type EXIT to abort.")
    print("="*50 + "\n")

    while True:
        try:
            user_cmd = input("Command >> ").strip()
            if user_cmd.upper() == "EXIT":
                print("[*] Exiting Interactive Mode...")
                break
            if user_cmd:
                smart_inject(user_cmd, udp_sock, tcp_sock)
        except KeyboardInterrupt: # حماية في حالة الضغط على Ctrl+C
            break
            
    udp_sock.close()
    tcp_sock.close()

# ==========================================
# 5. Background Monitors (عيون الرادار)
# ==========================================
def monitor_p2p():
    """مراقبة حركة الـ UDP (P2P Traffic)"""
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        sock.bind(("0.0.0.0", 3702))
        print("[P2P Monitor] Listening actively on UDP/3702...")
        while True:
            data, addr = sock.recvfrom(4096)
            decoded = decrypt_payload(data.decode(errors="ignore"))
            if decoded:
                print(f"[P2P-Traffic] {addr}: {decoded}")
    except Exception as e:
        print(f"[P2P Monitor Error] Failed to bind: {e}")

def monitor_ghost():
    """مراقبة حركة الـ TCP (Ghost Traffic)"""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.bind(("0.0.0.0", 8080))
        sock.listen(5)
        print("[Ghost Monitor] Listening actively on TCP/8080...")
        while True:
            conn, addr = sock.accept()
            data = conn.recv(4096)
            decoded = decrypt_payload(data.decode(errors="ignore"))
            if decoded:
                print(f"[Ghost-Traffic] {addr}: {decoded}")
            conn.close()
    except Exception as e:
        print(f"[Ghost Monitor Error] Failed to bind: {e}")

# ==========================================
# Main Execution Entry Point
# ==========================================
if __name__ == "__main__":
    # تشغيل عيون الرادار في الخلفية
    t1 = threading.Thread(target=monitor_p2p, daemon=True)
    t2 = threading.Thread(target=monitor_ghost, daemon=True)
    t1.start()
    t2.start()

    # انتظار بسيط لترتيب الشاشة
    time.sleep(0.5)
    
    print("[SYSTEM] Network Diagnostic Tool is fully operational.")
    # تشغيل واجهة التحكم
    interactive_mode()