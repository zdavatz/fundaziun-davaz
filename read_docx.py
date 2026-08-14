#!/usr/bin/env python3
"""Text, Kommentare und Änderungsverfolgung aus den .docx extrahieren.

python-docx zeigt weder Kommentare noch Tracked Changes - deshalb hier
direkt über das OOXML-Paket.
"""
import re
import sys
import zipfile

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"


def para_text(p):
    """Absatztext; Einfügungen/Löschungen aus der Änderungsverfolgung markieren."""
    import xml.etree.ElementTree as ET
    out = []
    for node in p.iter():
        tag = node.tag
        if tag == f"{W}t" and node.text:
            parent = None
            # w:t innerhalb w:ins / w:del kennzeichnen
            out.append(node.text)
        elif tag == f"{W}delText" and node.text:
            out.append(f"[GELÖSCHT: {node.text}]")
        elif tag == f"{W}tab":
            out.append("\t")
        elif tag == f"{W}br":
            out.append("\n")
    return "".join(out)


def dump(path):
    import xml.etree.ElementTree as ET
    print("=" * 78)
    print(path.split("/")[-1])
    print("=" * 78)

    with zipfile.ZipFile(path) as z:
        names = z.namelist()

        # --- Haupttext ---
        root = ET.fromstring(z.read("word/document.xml"))
        body = root.find(f"{W}body")

        # Markierung von Einfügungen aus der Änderungsverfolgung
        ins_ids = set()
        for ins in body.iter(f"{W}ins"):
            for t in ins.iter(f"{W}t"):
                ins_ids.add(id(t))

        for p in body.iter(f"{W}p"):
            parts = []
            for node in p.iter():
                if node.tag == f"{W}t" and node.text:
                    if id(node) in ins_ids:
                        parts.append(f"[NEU: {node.text}]")
                    else:
                        parts.append(node.text)
                elif node.tag == f"{W}delText" and node.text:
                    parts.append(f"[GELÖSCHT: {node.text}]")
                elif node.tag == f"{W}tab":
                    parts.append("\t")
            line = "".join(parts).rstrip()
            if line:
                print(line)

        # --- Kommentare ---
        if "word/comments.xml" in names:
            croot = ET.fromstring(z.read("word/comments.xml"))
            comments = list(croot.iter(f"{W}comment"))
            if comments:
                print("\n" + "-" * 78)
                print(f"KOMMENTARE ({len(comments)})")
                print("-" * 78)
                for c in comments:
                    author = c.get(f"{W}author", "?")
                    date = c.get(f"{W}date", "?")
                    txt = " ".join(
                        t.text for t in c.iter(f"{W}t") if t.text
                    ).strip()
                    print(f"\n[{author} · {date}]\n{txt}")


if __name__ == "__main__":
    for p in sys.argv[1:]:
        dump(p)
        print()
