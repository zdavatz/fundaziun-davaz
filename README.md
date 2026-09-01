# fundaziun-davaz

Arbeitswerkzeuge für die Errichtung der **FUNDAZIUN DA VAZ – VAL MÜSTAIR**,
einer Stiftung nach Art. 80 ff. ZGB mit Sitz in Sta. Maria, Val Müstair,
Kanton Graubünden.

Die Stiftung verfolgt den Zweck «Förderung der Kreativität und des
Eigensinns» und steht auf zwei Strängen: dem bildnerischen Werk von Jürg
Davatz und der Arbeit von Dr. med. Ursula Davatz zu ADHS und
Neurodiversität.

> Dieses Repository enthält **Werkzeuge**, keine Dokumente. Die
> Stiftungsurkunde, das Konzept, die Finanzübersicht und die Folgerungen
> aus der Recherche sind vertraulich und liegen im zugriffsgeschützten
> Google Doc beziehungsweise als `.docx` daneben – nicht in einer Datei,
> die bloss von `.gitignore` verdeckt würde. Was hier eingecheckt ist,
> darf öffentlich sein.

## Rust: Recherchebericht als PDF

`src/stiftungen.rs` erzeugt den Bericht *Stiftungen in der Schweiz* – wie
vergleichbare Stiftungen betrieben, geführt, finanziert und beaufsichtigt
werden, und was daraus für unsere Urkunde folgt. Siebzehn Porträts – neun
Kunststiftungen, acht zu psychischer Gesundheit –, dazu die Folgerungen und
alle Quellen als anklickbare Links.

```sh
cargo run --release --bin stiftungen              # beide Stränge (28 Seiten)
cargo run --release --bin stiftungen -- --k       # nur Kunst (16 Seiten)
cargo run --release --bin stiftungen -- --g       # nur Gesundheit (14 Seiten)
cargo run --release --bin stiftungen -- --out /pfad/zum.pdf
```

Ohne Argument enthält das PDF beide Stränge. Die Schriften liegen in
`fonts/` (DejaVu Sans); ein anderes Verzeichnis lässt sich über `$FONT_DIR`
setzen.

Die Porträts, das Kapitel zu Handelsregister und Stiftungsaufsicht sowie
alle Quellen sind öffentlich recherchiert und liegen im Repository. Die
Folgerungen für unsere Urkunde nennen Anfangskapital, Lohn und die
Familienregelung – sie stehen deshalb im Konzept-Google-Doc. Eingecheckt
ist mit `src/befunde.beispiel.rs` eine neutrale Fassung, die `build.rs`
auslegt, wenn keine lokale vorhanden ist: ein frischer Klon baut also und
liefert einen vollständigen Bericht, nur ohne unsere Zahlen.

Gesetzt wird direkt mit [`genpdf`](https://crates.io/crates/genpdf) – kein
Browser, kein HTML-Zwischenschritt, dieselbe Pipeline wie in
`~/software/listingtracker`. Da genpdf keine Hyperlinks kennt, werden die
Link-Annotationen nachträglich mit `lopdf` über die URL-Zeilen gelegt; die
Zuordnung ist durch einen Abgleich der Anzahl abgesichert.

## Python: Google-Workspace-Werkzeuge

Alle Skripte sprechen die Google-APIs des GCP-Projekts `fundaziun-davaz`
über OAuth an. Die Scopes sind bewusst getrennt, jeder mit eigenem Token,
damit ein Lesezugriff nie zum Versandrecht wird:

| Datei | Scope | Zweck |
| --- | --- | --- |
| `token.json` | `gmail.readonly` | Mails und Anhänge lesen |
| `token_drive.json` | `drive.readonly` | Drive durchsuchen und herunterladen |
| `token_docs.json` | `documents` | Google Docs bearbeiten |
| `token_send.json` | `gmail.send` | Mails versenden |

- `fetch_attachments.py` – lädt `.docx`-Anhänge aus Gmail-Threads,
  entdoppelt über SHA-256 (dasselbe Dokument taucht in Original und
  Weiterleitungen auf) und steigt rekursiv durch verschachtelte
  MIME-Teile.
- `read_docx.py` – liest Text, Word-Kommentare (`word/comments.xml`) und
  Änderungsverfolgung direkt aus dem OOXML; `python-docx` kann beides
  nicht anzeigen.
- `search_drive.py`, `fetch_drive_docs.py` – Volltextsuche in Drive und
  Download; native Google Docs werden exportiert, hochgeladene `.docx`
  direkt geholt.
Python-Aufrufe laufen über ein venv oder `uv run --with …`, nie gegen das
System-Python.

### Nicht im Repository

`make_v*.py` (je eine Fassung der Stiftungsurkunde), `send_mail*.py` und
`edit_konzept.py` bleiben lokal. Nicht weil der Code geheim wäre, sondern
weil er den Urkundentext im Klartext enthält: Liegenschaftsadressen,
Mailadressen der Familie und der Aufsichtsbehörde, Lohn- und
Hypothekenangaben.

Das Muster der `make_v*`-Skripte sei hier trotzdem festgehalten, weil es
die einzige verlässliche Art ist, ein `.docx` zu ändern, ohne
Formatierung, Kommentare und Änderungsverfolgung zu verlieren: Jedes
Skript liest die vorige Fassung, ersetzt Zeichenketten direkt in
`word/document.xml` und schreibt das ZIP neu, alle übrigen Einträge
bitgleich – kein `python-docx`. Vorher wird geprüft, dass der Anker
**genau einmal** vorkommt; sonst bricht das Skript ab, statt zu raten.
Offene Angaben stehen gelb hinterlegt (`w:highlight w:val="yellow"`).

## Lizenz

GPL-3.0. Siehe [LICENSE](LICENSE).
