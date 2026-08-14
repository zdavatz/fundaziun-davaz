#!/usr/bin/env python3
"""Sucht in Google Drive nach Dokumenten zur Fundaziun Da Vaz.

Braucht einen zusätzlichen Scope gegenüber fetch_attachments.py, daher
ein eigenes Token (token_drive.json) - so bleibt das Gmail-Token gültig.
"""
import glob
import os
import pathlib

from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build

SCOPES = ["https://www.googleapis.com/auth/drive.readonly"]
HERE = pathlib.Path(__file__).parent
TOKEN = HERE / "token_drive.json"

TERMS = ["Fundaziun", "Da Vaz", "DaVaz", "Stiftungsurkunde", "Müstair", "Muestair"]


def get_service():
    creds = None
    if TOKEN.exists():
        creds = Credentials.from_authorized_user_file(str(TOKEN), SCOPES)
    if not creds or not creds.valid:
        if creds and creds.expired and creds.refresh_token:
            creds.refresh(Request())
        else:
            secret = glob.glob(str(HERE / "client_secret_*.json"))[0]
            creds = InstalledAppFlow.from_client_secrets_file(
                secret, SCOPES
            ).run_local_server(port=0)
        TOKEN.write_text(creds.to_json())
        os.chmod(TOKEN, 0o600)
    return build("drive", "v3", credentials=creds)


def main():
    svc = get_service()
    seen = {}
    for term in TERMS:
        esc = term.replace("'", "\\'")
        q = f"(name contains '{esc}' or fullText contains '{esc}') and trashed = false"
        res = svc.files().list(
            q=q,
            fields="files(id,name,mimeType,modifiedTime,owners(emailAddress),webViewLink)",
            pageSize=50,
            includeItemsFromAllDrives=True,
            supportsAllDrives=True,
        ).execute()
        for f in res.get("files", []):
            seen.setdefault(f["id"], (f, set()))[1].add(term)

    if not seen:
        print("Keine Treffer in Drive.")
        return

    print(f"{len(seen)} Treffer:\n")
    for f, terms in sorted(seen.values(), key=lambda x: x[0]["modifiedTime"], reverse=True):
        kind = f["mimeType"].split(".")[-1]
        owner = (f.get("owners") or [{}])[0].get("emailAddress", "?")
        print(f"{f['modifiedTime'][:10]}  [{kind}]  {f['name']}")
        print(f"    Owner: {owner}   Treffer via: {', '.join(sorted(terms))}")
        print(f"    {f['webViewLink']}\n")


if __name__ == "__main__":
    main()
