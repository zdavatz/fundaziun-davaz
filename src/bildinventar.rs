// Bildinventar zu einer behördlichen Fotodokumentation als PDF.
//
// Führt eine Behörde einen Augenschein durch, fotografiert und weigert sich
// dann, die Aufnahmen zu nummerieren und zu sagen, was sie daran beanstandet,
// so lässt sich zu ihrem Vorhalt nicht Stellung nehmen - man kennt ihn nicht.
// Der Ausweg ist, die Dokumentation selbst zu nummerieren, jede Aufnahme
// sachlich zu beschreiben und ihr die Angaben der Eigentümerschaft
// zuzuordnen. Dieses Programm setzt daraus eine Beilage, die der
// Stellungnahme beigelegt werden kann.
//
// Das PDF zeigt je Aufnahme das Bild selbst, darunter Nummer, Fundstelle in
// der behördlichen Vorlage, die Beschreibung und den Verweis auf die
// zugehörigen Angaben. So sieht die Behörde neben jedem ihrer eigenen Bilder,
// was die Eigentümerschaft dazu erklärt.
//
//   cargo run --release --bin bildinventar
//   cargo run --release --bin bildinventar -- --out /pfad/zum.pdf
//
// Schriftverzeichnis über $FONT_DIR (Vorgabe: ./fonts), Bildverzeichnis über
// $FOTO_DIR (Vorgabe: ./attachments/bauamt).
//
// Der Inhalt - Objektangaben, Bildbeschreibungen, Angaben der
// Eigentümerschaft - steht in src/inventar_inhalt.rs. Diese Datei ist in
// .gitignore ausgeschlossen, weil sie Liegenschaft, Adresse und die
// Bauvorgänge beim Namen nennt; build.rs legt bei Bedarf die neutrale Fassung
// aus src/inventar_inhalt.beispiel.rs aus.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use genpdf::elements::{Break, Image, PageBreak, Paragraph};
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Element};

const DEFAULT_FONT_DIR: &str = "fonts";
const DEFAULT_FOTO_DIR: &str = "attachments/bauamt";
const DEFAULT_OUT: &str = "Bildinventar.pdf";

// Dieselbe zurückhaltende Palette wie im Recherchebericht: Tinte für den
// Fliesstext, Gold als einziger Akzent, Schiefer für Titel, Grau für Beiwerk.
const INK: Color = Color::Rgb(0x1b, 0x1b, 0x1d);
const GOLD: Color = Color::Rgb(0xa0, 0x8b, 0x6a);
const SLATE: Color = Color::Rgb(0x3a, 0x3d, 0x44);
const MUTED: Color = Color::Rgb(0x8a, 0x8d, 0x94);

// Die Aufnahmen sind 1039 Pixel breit. Bei 240 dpi ergibt das rund 110 mm und
// damit zwei Bildblöcke je Seite - Bild und zugehöriger Text bleiben
// beieinander, ohne dass eine Seite halb leer bleibt.
const BILD_DPI: f64 = 240.0;

// ---------------------------------------------------------------------------
// Inhalt
// ---------------------------------------------------------------------------

/// Kopfdaten des Objekts, wie sie auf dem Deckblatt erscheinen.
struct Objekt {
    titel: &'static str,
    untertitel: &'static str,
    /// Zeilenweise: Liegenschaft, Gebäude, Adresse, Verfahrensnummer.
    angaben: &'static [&'static str],
    stand: &'static str,
}

/// Eine Aufnahme der behördlichen Dokumentation.
struct Bild {
    /// Fortlaufende Nummer, von der Eigentümerschaft vergeben.
    nr: u8,
    /// Fundstelle in der behördlichen Vorlage, etwa "Seite 4 oben".
    seite: &'static str,
    /// Dateiname im Bildverzeichnis.
    datei: &'static str,
    /// Was auf der Aufnahme sichtbar ist - ohne rechtliche Würdigung.
    beschreibung: &'static str,
    /// Welchen Punkt des Verfügungsentwurfs die Aufnahme stützen soll;
    /// leer, wo sich kein Bezug erkennen lässt.
    beanstandet: &'static str,
    /// Verweis auf die Angaben, etwa "A1, A2"; leer, solange offen.
    grundlage: &'static str,
}

/// Ein Punkt des behördlichen Verfügungsentwurfs, zu dem die Aufnahmen in
/// Beziehung gesetzt werden.
struct Vorwurf {
    schluessel: &'static str,
    fundstelle: &'static str,
    text: &'static str,
    bilder: &'static str,
}

/// Aufnahmen, die denselben Gegenstand betreffen.
struct Gruppe {
    titel: &'static str,
    einleitung: &'static str,
    bilder: &'static [Bild],
}

/// Eine Angabe der Eigentümerschaft, auf die sich mehrere Bilder berufen.
struct Angabe {
    schluessel: &'static str,
    titel: &'static str,
    text: &'static str,
    bilder: &'static str,
}

include!("inventar_inhalt.rs");

// ---------------------------------------------------------------------------
// Satz
// ---------------------------------------------------------------------------

/// Jede `\n`-getrennte Zeile wird ein eigener Absatz, damit gesetzte
/// Zeilenumbrüche erhalten bleiben; innerhalb einer Zeile bricht genpdf um.
///
/// Der Stil muss zweimal gesetzt werden: `push_styled` färbt nur den
/// Textlauf, während die Zeilenhöhe aus dem Stil des umgebenden Elements
/// berechnet wird. Ohne das zusätzliche `styled` bekäme eine 20-pt-Zeile die
/// Zeilenhöhe der 10-pt-Grundschrift, und mehrzeilige Titel fielen
/// übereinander.
fn push_lines(doc: &mut genpdf::Document, text: &str, style: Style, align: Alignment) {
    for line in text.split('\n') {
        let mut p = Paragraph::default();
        p.push_styled(line.to_string(), style);
        doc.push(p.aligned(align).styled(style));
    }
}

fn body(doc: &mut genpdf::Document, text: &str) {
    push_lines(
        doc,
        text,
        Style::new().with_color(INK).with_font_size(10),
        Alignment::Left,
    );
}

fn h1(doc: &mut genpdf::Document, kicker: &str, titel: &str) {
    push_lines(
        doc,
        kicker,
        Style::new().with_color(GOLD).with_font_size(9).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.3));
    push_lines(
        doc,
        titel,
        Style::new().with_color(SLATE).with_font_size(17).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.8));
}

fn h2(doc: &mut genpdf::Document, titel: &str) {
    push_lines(
        doc,
        titel,
        Style::new().with_color(SLATE).with_font_size(12).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.4));
}

fn push_cover(doc: &mut genpdf::Document) {
    doc.push(Break::new(2.0));
    push_lines(
        doc,
        OBJEKT.untertitel,
        Style::new().with_color(GOLD).with_font_size(10).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.6));
    push_lines(
        doc,
        OBJEKT.titel,
        Style::new().with_color(SLATE).with_font_size(24).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(1.2));
    for zeile in OBJEKT.angaben {
        push_lines(
            doc,
            zeile,
            Style::new().with_color(INK).with_font_size(11),
            Alignment::Left,
        );
    }
    doc.push(Break::new(1.0));
    push_lines(
        doc,
        OBJEKT.stand,
        Style::new().with_color(MUTED).with_font_size(10),
        Alignment::Left,
    );
    // Die Vorbemerkung trägt die Begründung, warum dieses Inventar überhaupt
    // besteht. Sie ist zu lang für den Rest des Deckblatts und bekäme sonst
    // eine halb leere Folgeseite - also gleich eine eigene.
    doc.push(PageBreak::new());
    body(doc, VORBEMERKUNG);
}

fn push_angaben(doc: &mut genpdf::Document) {
    doc.push(PageBreak::new());
    h1(doc, "Teil II", "Angaben der Eigentümerschaft");
    for a in ANGABEN {
        push_lines(
            doc,
            &format!("{} – {}", a.schluessel, a.titel),
            Style::new().with_color(GOLD).with_font_size(10).bold(),
            Alignment::Left,
        );
        doc.push(Break::new(0.2));
        body(doc, a.text);
        doc.push(Break::new(0.2));
        push_lines(
            doc,
            a.bilder,
            Style::new().with_color(MUTED).with_font_size(9).italic(),
            Alignment::Left,
        );
        doc.push(Break::new(0.8));
    }
}

fn push_vorwuerfe(doc: &mut genpdf::Document) {
    doc.push(PageBreak::new());
    h1(doc, "Teil I", "Die beanstandeten Punkte");
    body(doc, VORWURF_EINLEITUNG);
    doc.push(Break::new(1.0));
    for v in VORWUERFE {
        push_lines(
            doc,
            &format!("{} – {}", v.schluessel, v.fundstelle),
            Style::new().with_color(GOLD).with_font_size(10).bold(),
            Alignment::Left,
        );
        doc.push(Break::new(0.2));
        body(doc, v.text);
        doc.push(Break::new(0.2));
        push_lines(
            doc,
            v.bilder,
            Style::new().with_color(SLATE).with_font_size(9).italic(),
            Alignment::Left,
        );
        doc.push(Break::new(0.8));
    }
}

/// Ein Bildblock: die Aufnahme, darunter Nummer und Fundstelle, die
/// Beschreibung und der Verweis auf die Angaben.
fn push_bild(doc: &mut genpdf::Document, foto_dir: &Path, b: &Bild) -> Result<()> {
    let pfad = foto_dir.join(b.datei);
    let bild = Image::from_path(&pfad)
        .with_context(|| format!("Aufnahme {} laden: {}", b.nr, pfad.display()))?
        .with_alignment(Alignment::Left)
        .with_dpi(BILD_DPI);
    doc.push(bild);
    doc.push(Break::new(0.3));
    push_lines(
        doc,
        &format!("Bild {}   ·   {}", b.nr, b.seite),
        Style::new().with_color(GOLD).with_font_size(9).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.15));
    push_lines(
        doc,
        b.beschreibung,
        Style::new().with_color(INK).with_font_size(9),
        Alignment::Left,
    );
    doc.push(Break::new(0.15));
    let (text, farbe) = if b.beanstandet.is_empty() {
        (
            "Beanstandet: kein Bezug zu einem Punkt des Entwurfs erkennbar".to_string(),
            MUTED,
        )
    } else {
        (format!("Beanstandet als: {}", b.beanstandet), SLATE)
    };
    push_lines(
        doc,
        &text,
        Style::new().with_color(farbe).with_font_size(9).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.1));
    // Wo die Eigentümerschaft nichts zu erklären hat, wird die Lücke nicht
    // stehen gelassen, sondern in eine Frage verwandelt. Bleibt sie
    // unbeantwortet, ist aktenkundig, dass an dieser Aufnahme nichts
    // beanstandet wird - das ist mehr wert als ein leeres Feld.
    let (text, farbe) = if b.grundlage.is_empty() {
        (
            "Rückfrage an das Bauamt: Was wird hieran beanstandet?".to_string(),
            GOLD,
        )
    } else {
        (
            format!("Angaben der Eigentümerschaft: {}", b.grundlage),
            SLATE,
        )
    };
    push_lines(
        doc,
        &text,
        Style::new().with_color(farbe).with_font_size(9).italic(),
        Alignment::Left,
    );
    doc.push(Break::new(1.0));
    Ok(())
}

fn push_gruppen(doc: &mut genpdf::Document, foto_dir: &Path) -> Result<()> {
    for (i, g) in GRUPPEN.iter().enumerate() {
        doc.push(PageBreak::new());
        h1(doc, &format!("Teil III.{}", i + 1), g.titel);
        // Nur Aufnahmen, die einem Punkt des Schreibens vom 25. Juni 2026
        // zuzuordnen sind. Die übrigen zeigen nichts, wozu Stellung zu nehmen
        // wäre; sie werden in Teil IV genannt, aber nicht abgebildet.
        let gezeigt: Vec<&Bild> = g
            .bilder
            .iter()
            .filter(|b| !b.beanstandet.is_empty())
            .collect();
        push_lines(
            doc,
            &format!("{} Aufnahmen.", gezeigt.len()),
            Style::new().with_color(SLATE).with_font_size(10).bold(),
            Alignment::Left,
        );
        doc.push(Break::new(0.3));
        body(doc, g.einleitung);
        doc.push(Break::new(1.0));
        // Ein Bildblock misst rund 117 mm, der Satzspiegel 253 mm: zwei Blöcke
        // gehen auf eine Seite, mit Gruppentitel und Einleitung nur einer.
        // genpdf kennt kein "zusammenhalten", und ein Umbruch zwischen
        // Aufnahme und Legende wäre in einer Beilage der schlimmste Fehler -
        // die Behörde läse die Beschreibung zum falschen Bild. Deshalb wird
        // der Umbruch hier von Hand gesetzt statt dem Satz überlassen.
        for (i, b) in gezeigt.iter().enumerate() {
            if i % 2 == 1 {
                doc.push(PageBreak::new());
            }
            push_bild(doc, foto_dir, b)?;
        }
    }
    Ok(())
}

/// Sammelt die Aufnahmen, zu denen die Eigentümerschaft nichts zu erklären
/// hat, und stellt sie dem Bauamt als Frage. Die Liste wird aus den Bilddaten
/// erzeugt und kann deshalb nicht von ihnen abweichen.
fn push_rueckfragen(doc: &mut genpdf::Document) {
    // Nur abgebildete Aufnahmen: die übrigen stehen weiter unten unter "Nicht
    // abgebildete Aufnahmen" und würden hier ein zweites Mal erscheinen. Und
    // nach Nummer sortiert, nicht nach Gruppenreihenfolge - eine Liste, die
    // "23, 26, 28, 5, 6" sagt, liest sich wie ein Versehen.
    let mut nummern: Vec<u8> = GRUPPEN
        .iter()
        .flat_map(|g| g.bilder.iter())
        .filter(|b| b.grundlage.is_empty() && !b.beanstandet.is_empty())
        .map(|b| b.nr)
        .collect();
    nummern.sort_unstable();
    let offen: Vec<String> = nummern.iter().map(|n| n.to_string()).collect();
    if offen.is_empty() {
        return;
    }
    doc.push(PageBreak::new());
    h1(doc, "Teil IV", "Rückfragen an das Bauamt");
    body(
        doc,
        &format!(
            "Zu den nachstehenden {} Aufnahmen hat die Eigentümerschaft nichts \
             zu erklären, weil sich ihr nicht erschliesst, was daran nach dem \
             Schreiben vom 25. Juni 2026 beanstandet sein soll. Sie ersucht das \
             Bauamt, zu jeder dieser Aufnahmen mitzuteilen, ob es daran etwas \
             beanstandet und, wenn ja, was.",
            offen.len()
        ),
    );
    doc.push(Break::new(0.6));
    push_lines(
        doc,
        &format!("Aufnahmen: {}.", offen.join(", ")),
        Style::new().with_color(SLATE).with_font_size(10).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.6));
    body(
        doc,
        "Bleibt diese Rückfrage unbeantwortet, ist davon auszugehen, dass an \
         diesen Aufnahmen nichts beanstandet wird und dass sie der beabsichtigten \
         Verfügung nicht zugrunde gelegt werden.",
    );

    // Aufnahmen, denen sich kein Punkt des Schreibens vom 25. Juni 2026
    // zuordnen lässt, werden nicht abgebildet - sie zeigten nichts, wozu
    // Stellung zu nehmen wäre. Verschwiegen werden sie aber nicht: eine
    // Beilage, die Bilder der Gegenseite stillschweigend weglässt, wäre
    // angreifbar, und zwar zu Recht.
    let ohne: Vec<&Bild> = GRUPPEN
        .iter()
        .flat_map(|g| g.bilder.iter())
        .filter(|b| b.beanstandet.is_empty())
        .collect();
    if ohne.is_empty() {
        return;
    }
    doc.push(Break::new(1.2));
    h2(doc, "Nicht abgebildete Aufnahmen");
    body(
        doc,
        "Die nachstehenden Aufnahmen sind in Teil III nicht wiedergegeben, weil \
         sich an ihnen kein Bezug zu einem der im Schreiben vom 25. Juni 2026 \
         beanstandeten Punkte erkennen lässt. Sie sind der Vollständigkeit halber \
         genannt. Sieht das Bauamt dies anders, wird es ersucht, dies \
         mitzuteilen.",
    );
    doc.push(Break::new(0.4));
    for b in &ohne {
        body(
            doc,
            &format!("Bild {} ({}): {}", b.nr, b.seite, b.beschreibung),
        );
        doc.push(Break::new(0.2));
    }
}

fn push_feststellungen(doc: &mut genpdf::Document) {
    doc.push(PageBreak::new());
    h1(doc, "Teil V", "Feststellungen zur Dokumentation");
    for (i, f) in FESTSTELLUNGEN.iter().enumerate() {
        h2(doc, &format!("{}. {}", i + 1, f.0));
        body(doc, f.1);
        doc.push(Break::new(0.8));
    }
}

// ---------------------------------------------------------------------------
// Schrift, Aufruf
// ---------------------------------------------------------------------------

/// Wie im Recherchebericht: die vier DejaVu-Schnitte einzeln laden. genpdf
/// erwartet bei `from_files` die Endung `-Regular`, unsere Datei heisst aber
/// schlicht `DejaVuSans.ttf`.
fn load_font_family(font_dir: &str) -> Result<genpdf::fonts::FontFamily<genpdf::fonts::FontData>> {
    let load = |file: &str| -> Result<genpdf::fonts::FontData> {
        let path = Path::new(font_dir).join(file);
        let data = std::fs::read(&path).map_err(|e| anyhow!("Schrift {}: {}", path.display(), e))?;
        genpdf::fonts::FontData::new(data, None).map_err(|e| anyhow!("Schrift {}: {}", file, e))
    };
    Ok(genpdf::fonts::FontFamily {
        regular: load("DejaVuSans.ttf")?,
        bold: load("DejaVuSans-Bold.ttf")?,
        italic: load("DejaVuSans-Oblique.ttf")?,
        bold_italic: load("DejaVuSans-BoldOblique.ttf")?,
    })
}

fn render(out: &Path, font_dir: &str, foto_dir: &Path) -> Result<()> {
    let family = load_font_family(font_dir)?;
    let mut doc = genpdf::Document::new(family);
    doc.set_title(OBJEKT.titel);
    doc.set_minimal_conformance();
    doc.set_font_size(10);
    doc.set_line_spacing(1.35);

    let kopf = OBJEKT.untertitel;
    let mut deco = genpdf::SimplePageDecorator::new();
    deco.set_margins(22);
    deco.set_header(move |page| {
        let mut p = Paragraph::default();
        if page > 1 {
            p.push_styled(
                format!("{kopf}          {page}"),
                Style::new().with_color(MUTED).with_font_size(7),
            );
        }
        p.aligned(Alignment::Right)
            .padded(genpdf::Margins::trbl(0, 0, 6, 0))
    });
    doc.set_page_decorator(deco);

    push_cover(&mut doc);
    push_vorwuerfe(&mut doc);
    push_angaben(&mut doc);
    push_gruppen(&mut doc, foto_dir)?;
    push_rueckfragen(&mut doc);
    push_feststellungen(&mut doc);

    doc.render_to_file(out)
        .map_err(|e| anyhow!("PDF schreiben {}: {}", out.display(), e))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Nachbearbeitung: die Aufnahmen als JPEG einsetzen
// ---------------------------------------------------------------------------

/// genpdf legt Bilder als entpackte RGB-Pixel ab. Neunundzwanzig Aufnahmen zu
/// 1039 x 779 ergeben so ein PDF von rund 47 MB - unversendbar. Die Vorlagen
/// sind aber bereits JPEG (baseline, drei Kanäle) und damit unmittelbar als
/// `DCTDecode`-Strom einsetzbar. Wir tauschen deshalb nach dem Satz jeden
/// Bildstrom gegen die Originaldatei; das PDF fällt auf wenige Megabyte, und
/// die Aufnahmen bleiben bitgleich die der Behörde - was bei einer Beilage in
/// einem Bauverfahren der eigentliche Punkt ist.
///
/// Die Zuordnung läuft über die Reihenfolge: printpdf vergibt die
/// Objektnummern aufsteigend in der Reihenfolge, in der die Bilder gesetzt
/// werden. Damit sich das nicht stillschweigend verschieben kann, vergleicht
/// die Funktion die Anzahl gefundener Bildobjekte mit der Anzahl Aufnahmen und
/// bricht bei Abweichung ab - dasselbe Sicherungsmuster wie bei den Links im
/// Recherchebericht.
fn jpegs_einsetzen(pdf: &Path, dateien: &[PathBuf]) -> Result<()> {
    use lopdf::{Document as LoDoc, Object};

    let mut doc = LoDoc::load(pdf).map_err(|e| anyhow!("PDF {} lesen: {}", pdf.display(), e))?;

    let mut ids: Vec<_> = doc
        .objects
        .iter()
        .filter(|(_, obj)| match obj {
            Object::Stream(s) => s
                .dict
                .get(b"Subtype")
                .ok()
                .and_then(|o| o.as_name().ok())
                .map(|n| n == b"Image")
                .unwrap_or(false),
            _ => false,
        })
        .map(|(id, _)| *id)
        .collect();
    ids.sort_unstable();

    if ids.len() != dateien.len() {
        return Err(anyhow!(
            "{} Bildobjekte im PDF, aber {} Aufnahmen - die Zuordnung über die \
             Reihenfolge wäre nicht mehr verlässlich",
            ids.len(),
            dateien.len()
        ));
    }

    for (id, datei) in ids.iter().zip(dateien) {
        let roh = std::fs::read(datei)
            .with_context(|| format!("Aufnahme lesen: {}", datei.display()))?;
        let laenge = roh.len() as i64;
        match doc.get_object_mut(*id) {
            Ok(Object::Stream(s)) => {
                s.set_plain_content(roh);
                s.dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
                s.dict.set("Length", Object::Integer(laenge));
                s.dict.remove(b"DecodeParms");
            }
            _ => return Err(anyhow!("Bildobjekt {:?} unerwartet verändert", id)),
        }
    }

    doc.save(pdf)
        .map_err(|e| anyhow!("PDF {} schreiben: {}", pdf.display(), e))?;
    Ok(())
}

fn main() -> Result<()> {
    let mut out = PathBuf::from(DEFAULT_OUT);
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => {
                out = PathBuf::from(
                    args.next()
                        .ok_or_else(|| anyhow!("--out erwartet einen Pfad"))?,
                )
            }
            other => return Err(anyhow!("unbekanntes Argument: {other}")),
        }
    }

    let font_dir = env::var("FONT_DIR").unwrap_or_else(|_| DEFAULT_FONT_DIR.to_string());
    let foto_dir =
        PathBuf::from(env::var("FOTO_DIR").unwrap_or_else(|_| DEFAULT_FOTO_DIR.to_string()));

    // Lieber ein Abbruch als ein PDF, in dem eine Aufnahme fehlt: eine Beilage
    // mit Lücken wäre schlimmer als gar keine. Die Reihenfolge dieser Liste ist
    // zugleich die, in der die Bilder gesetzt werden - `jpegs_einsetzen` stützt
    // sich darauf.
    let mut dateien = Vec::new();
    for g in GRUPPEN {
        for b in g.bilder.iter().filter(|b| !b.beanstandet.is_empty()) {
            let p = foto_dir.join(b.datei);
            if !p.exists() {
                return Err(anyhow!(
                    "Aufnahme {} fehlt: {} (Bildverzeichnis über $FOTO_DIR setzen)",
                    b.nr,
                    p.display()
                ));
            }
            dateien.push(p);
        }
    }

    render(&out, &font_dir, &foto_dir)?;
    jpegs_einsetzen(&out, &dateien)?;

    let groesse = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
    println!(
        "{} geschrieben – {} Aufnahmen, {:.1} MB.",
        out.display(),
        dateien.len(),
        groesse as f64 / 1_048_576.0
    );
    Ok(())
}
