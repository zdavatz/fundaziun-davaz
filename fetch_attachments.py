#!/usr/bin/env python3
"""Lade die .docx-Anhänge des Stiftungs-Threads via Gmail API herunter.

Aufruf:
    uv run --with google-auth-oauthlib --with google-api-python-client \
        python fetch_attachments.py

Beim ersten Lauf öffnet sich der Browser zur Freigabe des eigenen Kontos.
Das Token wird in token.json zwischengespeichert.
"""
import base64
import glob
import os
import pathlib
import sys

from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build

SCOPES = ["https://www.googleapis.com/auth/gmail.readonly"]
# Default: Thread mit der Stiftungsaufsicht GR. Weitere Thread-IDs per argv.
THREAD_IDS = [
    "19cd2db5bdd62aba",  # Justiz/FIVE GR, 09.03.2026
    "19bd6b80cc48671e",  # Eidg. Stiftungsaufsicht ESA, 19.01.2026
]
HERE = pathlib.Path(__file__).parent
OUTDIR = HERE / "attachments"


def get_service():
    creds = None
    token = HERE / "token.json"
    if token.exists():
        creds = Credentials.from_authorized_user_file(str(token), SCOPES)
    if not creds or not creds.valid:
        if creds and creds.expired and creds.refresh_token:
            creds.refresh(Request())
        else:
            secret = glob.glob(str(HERE / "client_secret_*.json"))[0]
            flow = InstalledAppFlow.from_client_secrets_file(secret, SCOPES)
            creds = flow.run_local_server(port=0)
        token.write_text(creds.to_json())
        os.chmod(token, 0o600)
    return build("gmail", "v1", credentials=creds)


def walk_parts(part):
    """Rekursiv alle Parts durchlaufen (Anhänge stecken oft verschachtelt)."""
    yield part
    for sub in part.get("parts", []) or []:
        yield from walk_parts(sub)


def main():
    svc = get_service()
    thread_ids = sys.argv[1:] or THREAD_IDS

    OUTDIR.mkdir(exist_ok=True)
    seen = set()
    msgs = []
    for tid in thread_ids:
        t = svc.users().threads().get(userId="me", id=tid, format="full").execute()
        msgs.extend(t["messages"])

    for msg in msgs:
        hdrs = {h["name"].lower(): h["value"] for h in msg["payload"].get("headers", [])}
        sender, date = hdrs.get("from", "?"), hdrs.get("date", "?")
        for part in walk_parts(msg["payload"]):
            fn = part.get("filename") or ""
            if not fn.lower().endswith(".docx"):
                continue
            att_id = part["body"].get("attachmentId")
            if not att_id:
                continue
            data = (
                svc.users().messages().attachments()
                .get(userId="me", messageId=msg["id"], id=att_id).execute()
            )
            raw = base64.urlsafe_b64decode(data["data"])

            # Gleicher Anhang taucht in Original + Weiterleitung auf -> deduplizieren
            import hashlib
            digest = hashlib.sha256(raw).hexdigest()[:12]
            if digest in seen:
                print(f"  (übersprungen, Duplikat) {fn}")
                continue
            seen.add(digest)

            safe = fn.replace("/", "_").strip()
            dest = OUTDIR / safe
            dest.write_bytes(raw)
            print(f"[{digest}] {len(raw):>7} B  {safe}")
            print(f"          von: {sender}")
            print(f"          am : {date}")

    print(f"\n{len(seen)} eindeutige .docx in {OUTDIR}")


if __name__ == "__main__":
    main()
