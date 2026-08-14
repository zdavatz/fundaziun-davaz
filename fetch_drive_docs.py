#!/usr/bin/env python3
"""Lädt die inhaltlich relevanten Fundaziun-Dokumente aus Drive.

Native Google Docs müssen exportiert werden (export_media), hochgeladene
.docx heruntergeladen (get_media) - der Anzeigename verrät den Unterschied
nicht, deshalb wird der echte mimeType abgefragt.
"""
import io
import pathlib

from googleapiclient.http import MediaIoBaseDownload

from search_drive import get_service

OUTDIR = pathlib.Path("drive_docs")

FILES = [
    ("1K9SZ6FNWgzSfPZWtCnwh7rL6skCwlivv", "Stiftungsurkunde_DaVaz (2026-03-09)"),
    ("1UpqUR38ONU65lcGEhlygxPPbWakJ2tRxK_iSLSv9HD0", "FunDaziun DaVaz (2026-01-07)"),
    ("1-Sv6CzzJEweMzaJcMTGVaZDMglYgxrnZ", "FUNDAZIUN DA VAZ (2021-03-11)"),
    ("1O_kCQkTGz3nvOsmw-FG435ICmpkMrBOb", "Fundaziun DaVaz Sinn und Zweck (2021-03-11)"),
    ("1rSNnBIiDWRYZS1gykdG6YPNtny8V5qn_", "Das Münstertal bietet Lebensqualität an (2021-01-14)"),
    ("14qiwdR_0bDJpJHyupQ3xKHkMJv2FWG1ysuHBwOPRM48", "Protokoll Familiensitzung 26.12.20"),
]

GOOGLE_DOC = "application/vnd.google-apps.document"


def main():
    svc = get_service()
    OUTDIR.mkdir(exist_ok=True)

    for fid, label in FILES:
        meta = svc.files().get(fileId=fid, fields="name,mimeType").execute()
        native = meta["mimeType"] == GOOGLE_DOC
        safe = label.replace("/", "_")

        if native:
            req = svc.files().export_media(fileId=fid, mimeType="text/plain")
            dest = OUTDIR / f"{safe}.txt"
        else:
            req = svc.files().get_media(fileId=fid)
            dest = OUTDIR / f"{safe}.docx"

        buf = io.BytesIO()
        dl = MediaIoBaseDownload(buf, req)
        done = False
        while not done:
            _, done = dl.next_chunk()
        dest.write_bytes(buf.getvalue())
        kind = "Google Doc (exportiert)" if native else "hochgeladene .docx"
        print(f"✓ {dest.name}  [{kind}, {dest.stat().st_size} B]")


if __name__ == "__main__":
    main()
