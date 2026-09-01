# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Werkzeuge für die Errichtung der **FUNDAZIUN DA VAZ – VAL MÜSTAIR**
(Art. 80 ff. ZGB, Sitz Sta. Maria, Val Müstair, Kanton Graubünden). Zwei
Sprachen, zwei Aufgaben:

- **Rust** – `src/stiftungen.rs` erzeugt den Recherchebericht als PDF.
- **Python** – Google-Workspace-Skripte für Gmail, Drive und Docs sowie
  die versionierte Bearbeitung der Stiftungsurkunde.

Kommentare, Dokumententexte und Commit-Messages sind auf Deutsch
(Schweizer Rechtschreibung: **ss statt ß**). Bezeichner im Code bleiben
gemischt; der bestehende Stil in der jeweiligen Datei gibt den Ausschlag.

## Build und Ausführung

```sh
cargo build --release --bin stiftungen
cargo run --release --bin stiftungen              # beide Stränge
cargo run --release --bin stiftungen -- --k       # nur Kunst
cargo run --release --bin stiftungen -- --g       # nur Gesundheit
cargo run --release --bin stiftungen -- --out /pfad/zum.pdf
```

Es gibt keine Tests und kein CI. Prüfen heisst hier: PDF erzeugen, mit
`pdftotext` den Text und mit `pdftoppm -png` einzelne Seiten ansehen.

Python-Skripte laufen **nie** gegen das System-Python – venv oder
`uv run --with google-auth-oauthlib --with google-api-python-client python …`.

## Architektur

### PDF-Erzeugung (`src/stiftungen.rs`)

Der Inhalt steht als Konstanten am Kopf der Datei (`PORTRAITS`, `PSYCHE`,
`BEFUNDE_*`, `QUELLEN_*`); darunter folgt nur noch der Satz. Inhaltliche
Änderungen gehören in die Konstanten, nicht in die Satzfunktionen.

Drei Fallstricke, die schon zugeschlagen haben:

1. **Stil zweimal setzen.** `push_styled` färbt nur den Textlauf; die
   Zeilenhöhe kommt vom Stil des umgebenden Elements. Ohne das
   zusätzliche `.styled(style)` in `push_lines` bekommt eine 30-pt-Zeile
   die Zeilenhöhe der 10-pt-Grundschrift, und mehrzeilige Titel fallen
   übereinander.

2. **URLs können nicht umbrechen.** genpdf bricht nur an Leerzeichen und
   lässt ein Wort, das nicht in die Zeile passt, ersatzlos weg. Deshalb
   kürzt `link_text` die Anzeigeform auf `MAX_LINK_CHARS`; verlinkt wird
   die vollständige Adresse.

3. **`LINK_FONT_SIZE` ist ein Anker, keine Geschmacksfrage.** genpdf kennt
   keine Hyperlinks. `add_links` findet die URL-Zeilen im Inhaltsstrom
   daran wieder, dass sie die einzigen in dieser Schriftgrösse sind, und
   ordnet ihnen `Auswahl::urls()` der Reihe nach zu. Sobald eine andere
   Stelle im Satz dieselbe Grösse verwendet, verschiebt sich die
   Zuordnung. `render` vergleicht deshalb die Anzahl gefundener Zeilen mit
   der Anzahl URLs und bricht bei Abweichung ab – dieser Abgleich darf
   nicht entschärft werden.

Die `--k`/`--g`-Auswahl steuert über `enum Auswahl` Deckblatt, Kapitelfolge,
Nummerierung und Vorgabedateiname. Ohne Argument enthält das PDF beide
Stränge.

Die Folgerungen liegen in `src/befunde.rs`, das per `include!` eingezogen
wird. Diese Datei ist ein **Bauartefakt**: `build.rs` legt sie aus
`src/befunde.beispiel.rs` an, wenn sie fehlt. Die Fassung mit den konkreten
Angaben – Anfangskapital, Lohn, Familienregelung – steht im Konzept-Google-Doc
unter «11. Recherche: Was vergleichbare Stiftungen für unsere Urkunde
bedeuten», nicht hier. Wer den Bericht mit den echten Zahlen setzen will,
holt sie von dort und legt sie lokal ab; eingecheckt wird nur die neutrale
Fassung.

### Urkunden-Skripte (`make_v1*.py`) – lokal, nicht im Repository

Diese Skripte, ebenso `send_mail*.py` und `edit_konzept.py`, sind in
`.gitignore` ausgeschlossen: sie enthalten den Urkundentext im Klartext.
Sie existieren im Arbeitsverzeichnis und sind dort zu bearbeiten – aber
**niemals einchecken**, auch nicht mit `git add -f`.

Je Fassung ein Skript, das die vorige liest. Sie ersetzen Zeichenketten
direkt in `word/document.xml` und schreiben das ZIP neu, alle übrigen
Einträge bitgleich – kein `python-docx`, weil das Formatierung und
Kommentare zerstört. Jedes Skript prüft, dass sein Anker **genau einmal**
vorkommt, und bricht sonst ab. Dieses Muster beibehalten: lieber ein
Abbruch als eine stillschweigend falsche Ersetzung.

Offene Angaben stehen im Dokument gelb hinterlegt
(`w:highlight w:val="yellow"`).

### Google-APIs

GCP-Projekt `fundaziun-davaz`, OAuth-Client `mail-cli` (Desktop).
Getrennte Scopes mit je eigenem Token: `token.json` (gmail.readonly),
`token_drive.json` (drive.readonly), `token_docs.json` (documents),
`token_send.json` (gmail.send). Ein Lesezugriff soll nie zum Versandrecht
werden.

Hängt der OAuth-Flow scheinbar, liegt es an gepuffertem stdout – mit
`python -u` starten, damit die Consent-URL erscheint.

## Vertraulichkeit

**Dieses Repository ist öffentlich.** Ausserhalb bleiben und in
`.gitignore` geführt:

- `client_secret_*.json`, `token*.json`
- `Stiftungsurkunde_*.docx`, `Finanzuebersicht_*.md`
- `attachments/`, `drive_docs/`
- `make_v*.py`, `send_mail*.py`, `edit_konzept.py`, `konzept_befunde.py` –
  sie tragen den Urkundentext im Klartext: Liegenschaftsadressen,
  Mailadressen der Familie und der Aufsichtsbehörde, Lohn- und
  Hypothekenangaben
- die erzeugten `*_Recherche.pdf` (jederzeit reproduzierbar)
- `src/befunde.rs` – Bauartefakt, das `build.rs` anlegt

**Der Grundsatz:** vertrauliche Inhalte gehören ins zugriffsgeschützte
Google Doc, nicht in eine Datei, die bloss von `.gitignore` verdeckt wird.
Ein `.gitignore`-Eintrag ist eine Vorsichtsmassnahme, kein Schutz – ein
`git add -f` genügt. Wo ein Inhalt geheim bleiben soll, ist der richtige
Ort das Doc; das Repository bekommt allenfalls eine neutrale Fassung.

Das Stiftungskonzept bleibt bewusst als Google Doc und kommt nicht ins
Repository. Vor jedem Commit `git status` prüfen; niemals Adressen,
Hypothekenzahlen, Namen von Familienmitgliedern oder Zugangsdaten in
eingecheckte Dateien schreiben.

## Fachliches, das im Code nicht steht

- Aufsichtsbehörde ist die Stiftungsaufsicht des Kantons Graubünden,
  solange der Zweck **mehrheitlich im Sitzkanton** erfüllt wird; bei
  gesamtschweizerischer oder internationaler Tätigkeit ginge die Aufsicht
  an die Eidgenössische Stiftungsaufsicht (ESA) in Bern über. Art. 2 der
  Urkunde öffnet das Tätigkeitsgebiet – dieser Punkt ist offen.
  Massgebend ist, **wo der Zweck erfüllt wird, nicht wen er erreicht**:
  die Fondation Beyeler bleibt trotz Weltrang bei der kantonalen BVG- und
  Stiftungsaufsicht beider Basel, weil ihr Zweck an das Museum in Riehen
  gebunden ist.
- Wo Stifter, Eigentümer der Liegenschaft, Stiftungsrat und Urheber
  **dieselbe Person** sind, prüft die Aufsicht zuerst Interessenkonflikt,
  Selbstkontrahierung und Ausstand. Die Urkunde braucht dann dreierlei:
  eine Ausstandsregelung für Geschäfte zwischen Stiftung und Stifter,
  eine nicht der Familie angehörende Mehrheit im Stiftungsrat – sonst
  wirkt der Ausstand nicht –, und eine ausdrückliche Regel, ob und
  wieviel die Stiftung für die Nutzung der Liegenschaft bezahlt. Ein zu
  hoher Zins wäre eine verdeckte Zuwendung an den Stifter, ein zu tiefer
  eine Dauersubvention, auf die sich die Stiftung nicht verlassen kann.
  Vergleichsfall im Bericht: Fundaziun Chastè da Tarasp (Portrait 9).
- Urheberpersönlichkeitsrechte sind nach URG nicht übertragbar. Gewidmet
  werden können nur die **übertragbaren** Urheberrechte; eine Widmung
  «sämtlicher Urheberrechte» wäre insoweit unwirksam.
- Eine Schuldübernahme wirkt gegenüber der Gläubigerin erst mit deren
  Zustimmung (Art. 176 OR).

## Lizenz

GPL-3.0. Neue Quelldateien tragen einen GPL-3.0-verträglichen Kopf, und
jede Abhängigkeit muss mit GPL-3.0 vereinbar sein.

## Übergeordnete Anweisungen

`~/software/CLAUDE.md` (Arbeitsbereich) gilt hier ebenfalls.
