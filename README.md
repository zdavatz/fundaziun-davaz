# fundaziun-davaz

Arbeitswerkzeuge für die Errichtung der **FUNDAZIUN DA VAZ – VAL MÜSTAIR**,
einer Stiftung nach Art. 80 ff. ZGB mit Sitz in Sta. Maria, Val Müstair,
Kanton Graubünden.

Die Stiftung verfolgt den Zweck «Förderung der Kreativität und des
Eigensinns» und steht auf zwei Strängen: dem bildnerischen Werk von Jürg
Davatz und der Arbeit von Dr. med. Ursula Davatz zu ADHS und
Neurodiversität.

> Dieses Repository enthält **Werkzeuge**, keine Dokumente. Die
> Stiftungsurkunde, das Konzept, die Finanzübersicht, die Folgerungen aus
> der Recherche und die Baurechtsakten sind vertraulich und liegen im
> zugriffsgeschützten Google Doc beziehungsweise auf Drive – nicht in einer
> Datei, die bloss von `.gitignore` verdeckt würde. Was hier eingecheckt
> ist, darf öffentlich sein.
>
> Jedes der drei Satzprogramme trennt deshalb Satz und Inhalt: der Satz ist
> eingecheckt, der Inhalt liegt in einer `*_inhalt.rs` beziehungsweise
> `befunde.rs` daneben und ist ausgeschlossen. `build.rs` legt jeweils eine
> neutrale Beispielfassung aus, damit ein frischer Klon baut.

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

Zwei weitere Grenzen von genpdf 0.2, beide in den Bilddokumenten
aufgetreten und im Code begründet. **Bilder legt es als entpackte
RGB-Pixel ab** – neunundzwanzig Aufnahmen ergaben ein PDF von 47 MB. Die
Vorlagen sind baseline-JPEG und damit unmittelbar als `DCTDecode`-Strom
einsetzbar; sie werden nach dem Satz mit `lopdf` eingetauscht, was auf
gut 6 MB führt und die Aufnahmen bitgleich denen der Behörde belässt. Die
Zuordnung läuft über die Reihenfolge und bricht bei abweichender Anzahl ab,
wie bei den Links. **Und es kennt kein «zusammenhalten»** – weder
`LinearLayout` noch `TableLayout`, beide melden `has_more` und laufen auf
der nächsten Seite weiter, und die Resthöhe einer Seite lässt sich von
aussen nicht abfragen. Umbrüche zwischen Bild und Legende, zwischen Titel
und Text und zwischen Antragsnummer und Antrag werden deshalb von Hand
gesetzt beziehungsweise die Teile in denselben Absatz gezogen.

## Rust: Behördenfotos beschriften

`src/bildinventar.rs` entstand aus einer Verlegenheit. Führt eine Behörde
einen Augenschein durch, fotografiert und weigert sich dann, die Aufnahmen
zu nummerieren und zu sagen, was sie daran beanstandet, so lässt sich zu
ihrem Vorhalt nicht Stellung nehmen – man kennt ihn nicht. Der Ausweg ist,
die Dokumentation selbst zu nummerieren, jede Aufnahme sachlich zu
beschreiben, sie den Punkten der behördlichen Verfügung zuzuordnen und die
Angaben der Eigentümerschaft danebenzustellen.

```sh
cargo run --release --bin bildinventar
cargo run --release --bin bildinventar -- --out /pfad/zum.pdf
```

Je Aufnahme das Bild selbst, darunter Nummer, Fundstelle in der
behördlichen Vorlage, Beschreibung, «Beanstandet als …» und «Angaben der
Eigentümerschaft …». Wo die Eigentümerschaft nichts zu erklären hat, wird
die Lücke nicht stehen gelassen, sondern in eine Rückfrage verwandelt –
bleibt sie unbeantwortet, ist aktenkundig, dass dort nichts beanstandet
wird. Aufnahmen ohne zuordenbaren Vorwurf werden nicht abgebildet, aber
genannt: eine Beilage, die Bilder der Gegenseite stillschweigend weglässt,
wäre angreifbar, und zwar zu Recht.

Bildverzeichnis über `$FOTO_DIR` (Vorgabe `attachments/bauamt`).

## Rust: Rechtsschrift als PDF

`src/stellungnahme.rs` setzt einen Brief an eine Behörde – Stellungnahme,
Gesuch, Einsprache: Absender, Adressat, Ort und Datum, Betreff, Anträge
vorn, dann die Begründung in nummerierten Ziffern, am Schluss
Unterschriften und Beilagen.

```sh
cargo run --release --bin stellungnahme
cargo run --release --bin stellungnahme -- --out /pfad/zum.pdf
INHALT=zweite_inhalt.rs FOTO_DIR=/pfad/zu/belegen cargo run --release --bin stellungnahme
```

Derselbe Satz kann mehrere Rechtsschriften erzeugen: `INHALT` wählt die
Inhaltsdatei in `src/` (Vorgabe `stellungnahme_inhalt.rs`), `FOTO_DIR`
das Verzeichnis der Belegbilder. Jede `src/*_inhalt.rs` ist vom
Repository ausgeschlossen, weil sie Liegenschaft, Namen und Verfahren
nennt.

Die Anträge stehen vor der Begründung, weil die Behörde auf der ersten
Seite sehen soll, was verlangt wird, und erst danach, warum. Die Nummern
der Ziffern vergibt der Satz – eine falsch nummerierte Rechtsschrift
verweist ins Leere, und beim Einschieben eines Abschnitts geht das von Hand
jedes Mal schief. Belegbilder stehen im Text, wo ein Argument auf einer
Aufnahme beruht; über eine Baubewilligungssache entscheidet der
Gemeindevorstand, nicht das Bauamt, und ein Argument neben der Aufnahme
wirkt anders als eine blosse Bildnummer.

Die Seitenumbrüche setzt das Programm, nicht genpdf. Jede Hauptziffer
beginnt auf einer eigenen Seite, Antragsnummer und Antragstext stehen im
selben Absatz, Bild und Legende ebenfalls. Wo ein Zwischentitel trotzdem
allein ans Seitenende fiele, beginnt sein Titel im Inhalt mit `@`; das
Zeichen erzwingt eine neue Seite und wird beim Satz entfernt. Das kostet
Weissraum – aber eine Rechtsschrift, auf deren Ziffern später verwiesen
wird, verträgt keinen Titel ohne Text darunter.

Die Belegbilder bettet genpdf als entpackte Pixel ein; nach dem Satz
werden die Bildströme gegen die Original-JPEGs getauscht, damit die
Aufnahmen der Behörde bitgleich bleiben. Zugeordnet wird über Breite und
Höhe, nicht über die Objektfolge im PDF – die folgt bei vielen Bildern
nicht der Seitenfolge.

## Rust: Aktendossier aus Fotos einer Akteneinsicht

`src/aktendossier.rs` macht aus einem Ordner Handyfotos ein einziges PDF:
je Aufnahme eine Seite, aufrecht gedreht, mit Seitenzahl und Dateiname in
der Fusszeile, damit ein Aktenverzeichnis darauf verweisen kann.

```sh
cargo run --release --bin aktendossier -- \
    --dir /pfad/zu/den/fotos --out Dossier.pdf \
    --titel "Baudossier …, Akteneinsicht vom …" \
    --orient lagen.txt --reihenfolge reihenfolge.txt --hoehe 1300
```

`--reihenfolge` legt die Seitenfolge fest, je Zeile «dateiname<TAB>
beschriftung», etwa chronologisch nach einem Aktenverzeichnis; die
Beschriftung erscheint in der Fusszeile. Was in der Liste fehlt, folgt am
Schluss in Aufnahmefolge.

Das PDF wird direkt mit lopdf geschrieben, nicht mit genpdf: genpdf bettet
Bilder als entpackte Pixel ein, und vierhundert Aufnahmen ergäben damit
mehrere Gigabyte. Hier werden die Bilder verkleinert, als JPEG neu kodiert
und mit DCTDecode eingebettet; 409 Seiten ergeben rund 70 MB.

Gedreht wird zweimal: zuerst nach der EXIF-Ausrichtung des Handys, die das
`image`-Paket nicht liest und deshalb aus dem APP1-Segment selbst gelesen
wird, dann nach der Lage aus `--orient` – eine Textdatei mit je Zeile
«dateiname up|right|left|down», wie sie eine Texterkennung liefert, die
die Leserichtung aus der Geometrie der Textzeilen bestimmt. Die Blätter
eines Dossiers liegen oft quer oder kopfstehend auf dem Tisch; ohne diesen
zweiten Schritt wären sie es auch im PDF. Vorsicht bei der Lagebestimmung:
Apples Vision liest Text auch kopfstehend und seitlich und liefert dann
dieselbe Zeichenzahl – die Zeichenzahl je Drehung taugt nicht als Mass.
Verlässlich ist der Vektor von der linken zur rechten oberen Ecke jeder
erkannten Zeile; sein Winkel sagt, wie das Blatt liegt.

## Rust: Bewilligungslage mit Belegen

`src/bewilligung.rs` fasst zusammen, was in einem Bauverfahren bewilligt
wurde und was nicht, wie die Entscheide zugestellt worden sind und welche
Fristen daraus liefen. Vorn Übersichtstabellen je Verfahren, die massgebende
Norm im Wortlaut, dann je Seite eine Ablichtung aus den Originalakten als
Beleg – die Leserin soll die Zusammenfassung an der Quelle prüfen können,
ohne die Akten daneben zu legen.

```sh
cargo run --release --bin bewilligung
cargo run --release --bin bewilligung -- --out /pfad/zum.pdf
```

Die Belegseiten sind JPEG-Ablichtungen aus `$BELEG_DIR` (Vorgabe
`attachments/bauamt/bewilligung_belege`), erzeugt mit `pdftoppm -jpeg`.
Wie beim Bildinventar werden die Bildströme nach dem Satz gegen die
Originaldateien getauscht, sonst wüchse das PDF auf Dutzende Megabyte.
Jeder Beleg steht allein auf einer Seite, weil genpdf ein Bild nicht
umbrechen kann; die Auflösung ist so gewählt, dass eine A4-Ablichtung samt
Legende auf ein Blatt passt.

Der Inhalt steht in `src/bewilligung_inhalt.rs` und ist ausgeschlossen –
er nennt Liegenschaft, Verfahrensnummern, Namen und die Entscheide im
Wortlaut. `build.rs` legt die neutrale Fassung aus
`src/bewilligung_inhalt.beispiel.rs` aus.

## Python: Google-Workspace-Werkzeuge

Alle Skripte sprechen die Google-APIs über OAuth an. Zugangsdaten liegen
ausserhalb des Repositorys unter `~/.config/fundaziun-davaz/`, mit zwei
Trennungen: je Scope ein eigenes Token, damit ein Lesezugriff nie zum
Versandrecht wird – und je Konto ein eigenes Verzeichnis, damit kein Skript
aus dem falschen Postfach sendet.

```
~/.config/fundaziun-davaz/
├── client_secret_<projekt>.json
└── <konto>/
    ├── gmail.readonly.json      Mails und Anhänge lesen
    ├── gmail.send.json          Mails versenden
    ├── gmail.compose.json       Entwürfe anlegen
    ├── documents.json           Google Docs bearbeiten
    └── drive.file.json          eigene Drive-Dateien
```

Der Pfad nennt das Konto, nicht der Dateiname. Ein Token, das nur
`token_gmail_send.json` heisst, sagt nicht, aus welchem Postfach es sendet.

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
