// Eine Rechtsschrift als PDF - Stellungnahme, Gesuch, Einsprache.
//
// Anders als der Recherchebericht und das Bildinventar ist dies ein Brief an
// eine Behörde: Absender oben, Adressat darunter, Ort und Datum, Betreff,
// dann Anträge und Begründung, am Schluss Unterschriften und Beilagen. Die
// Form folgt dem, was eine Bündner Gemeinde und die Rechtsmittelinstanz
// erwarten - nummerierte Abschnitte, damit sich später darauf verweisen
// lässt.
//
//   cargo run --release --bin stellungnahme
//   cargo run --release --bin stellungnahme -- --out /pfad/zum.pdf
//
// Schriftverzeichnis über $FONT_DIR (Vorgabe: ./fonts).
//
// Der Inhalt steht in src/stellungnahme_inhalt.rs und ist in .gitignore
// ausgeschlossen: er nennt Namen, Adressen, Liegenschaft und Verfahren.
// build.rs legt bei Bedarf die neutrale Fassung aus
// src/stellungnahme_inhalt.beispiel.rs aus.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use genpdf::elements::{Break, Image, PageBreak, Paragraph};
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Element};

const DEFAULT_FONT_DIR: &str = "fonts";
const DEFAULT_FOTO_DIR: &str = "attachments/bauamt";
const DEFAULT_OUT: &str = "Stellungnahme.pdf";

// Kleiner als im Bildinventar: hier trägt der Text, das Bild belegt ihn nur.
// 1039 Pixel bei 300 dpi ergeben rund 88 mm.
const BILD_DPI: f64 = 300.0;

const INK: Color = Color::Rgb(0x1b, 0x1b, 0x1d);
const GOLD: Color = Color::Rgb(0xa0, 0x8b, 0x6a);
const SLATE: Color = Color::Rgb(0x3a, 0x3d, 0x44);
const MUTED: Color = Color::Rgb(0x8a, 0x8d, 0x94);

// ---------------------------------------------------------------------------
// Inhalt
// ---------------------------------------------------------------------------

/// Kopf des Briefs: wer schreibt, an wen, von wo und wann.
struct Kopf {
    absender: &'static [&'static str],
    adressat: &'static [&'static str],
    ort_datum: &'static str,
    betreff: &'static str,
    /// Anrede, etwa "Sehr geehrte Frau Gemeindepräsidentin".
    anrede: &'static str,
}

/// Ein nummerierter Abschnitt der Begründung. Die Nummer vergibt der Satz,
/// damit sie beim Einschieben eines Abschnitts nicht von Hand nachgezogen
/// werden muss - eine falsch nummerierte Rechtsschrift ist peinlich und
/// verweist ins Leere.
struct Abschnitt {
    titel: &'static str,
    /// Absätze; leere Zeilen trennen sie im Satz.
    text: &'static str,
    /// Untergeordnete Punkte, je Titel und Text; leer, wenn keine.
    unterpunkte: &'static [(&'static str, &'static str)],
    /// Belegbilder, je Dateiname und Legende. Über die Verfügung entscheidet
    /// der Gemeindevorstand, nicht das Bauamt - Laien also, die den Bau nicht
    /// Bild für Bild im Kopf haben. Ein Argument neben der Aufnahme, auf der
    /// es beruht, wirkt anders als eine blosse Bildnummer. Aufgenommen wird
    /// deshalb nicht die ganze Dokumentation - die ist Beilage -, sondern nur,
    /// was ein Argument trägt.
    bilder: &'static [(&'static str, &'static str)],
}

include!("stellungnahme_inhalt.rs");

// ---------------------------------------------------------------------------
// Satz
// ---------------------------------------------------------------------------

/// Wie im Recherchebericht: der Stil muss zweimal gesetzt werden, weil
/// `push_styled` nur den Textlauf färbt, die Zeilenhöhe aber vom Stil des
/// umgebenden Elements kommt.
fn push_lines(doc: &mut genpdf::Document, text: &str, style: Style, align: Alignment) {
    for line in text.split('\n') {
        let mut p = Paragraph::default();
        p.push_styled(line.to_string(), style);
        doc.push(p.aligned(align).styled(style));
    }
}

/// Absätze werden durch Leerzeilen getrennt und mit Abstand gesetzt, damit
/// der Fliesstext nicht als Blockmauer erscheint.
fn body(doc: &mut genpdf::Document, text: &str) {
    let stil = Style::new().with_color(INK).with_font_size(10);
    for (i, absatz) in text.split("\n\n").enumerate() {
        if i > 0 {
            doc.push(Break::new(0.5));
        }
        push_lines(doc, absatz, stil, Alignment::Left);
    }
}

fn push_kopf(doc: &mut genpdf::Document) {
    for zeile in KOPF.absender {
        push_lines(
            doc,
            zeile,
            Style::new().with_color(INK).with_font_size(10),
            Alignment::Left,
        );
    }
    doc.push(Break::new(1.6));
    for zeile in KOPF.adressat {
        push_lines(
            doc,
            zeile,
            Style::new().with_color(INK).with_font_size(10),
            Alignment::Left,
        );
    }
    doc.push(Break::new(1.6));
    push_lines(
        doc,
        KOPF.ort_datum,
        Style::new().with_color(MUTED).with_font_size(10),
        Alignment::Right,
    );
    doc.push(Break::new(1.2));
    push_lines(
        doc,
        KOPF.betreff,
        Style::new().with_color(SLATE).with_font_size(12).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(1.2));
    push_lines(
        doc,
        KOPF.anrede,
        Style::new().with_color(INK).with_font_size(10),
        Alignment::Left,
    );
    doc.push(Break::new(0.6));
    body(doc, EINLEITUNG);
    doc.push(Break::new(1.0));
}

/// Die Anträge stehen vorn, nicht hinten: die Behörde soll auf der ersten
/// Seite sehen, was verlangt wird, und erst danach, warum.
fn push_antraege(doc: &mut genpdf::Document) {
    push_lines(
        doc,
        "Anträge",
        Style::new().with_color(SLATE).with_font_size(12).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.5));
    for (i, a) in ANTRAEGE.iter().enumerate() {
        // Nummer und Text in denselben Absatz. Als eigenes Element gesetzt,
        // könnte die Nummer am Seitenfuss hängenbleiben, während ihr Antrag
        // auf der nächsten Seite beginnt - genpdf bricht zwischen Elementen
        // um, ohne zu fragen, ob sie zusammengehören.
        body(doc, &format!("{}.  {}", i + 1, a));
        doc.push(Break::new(0.6));
    }
    doc.push(Break::new(0.6));
}

/// Belegbilder mit Legende, am Schluss des Abschnitts.
///
/// genpdf 0.2 kennt kein "zusammenhalten" - auch `LinearLayout` bricht um -,
/// und die Resthöhe einer Seite lässt sich von aussen nicht abfragen. Bleibt
/// ein Bild am Seitenfuss hängen, landet seine Legende auf der Folgeseite,
/// direkt über dem nächsten Bild. Wer die Seite überfliegt, ordnet die Legende
/// dann dem falschen Bild zu - in einer Rechtsschrift der schlimmste Fehler,
/// den ein Satzprogramm machen kann.
///
/// Deshalb beginnt der Belegteil auf einer frischen Seite, und es stehen zwei
/// Blöcke darauf: 88 mm Bild und rund 15 mm Legende ergeben gut 100 mm, zwei
/// davon bleiben unter dem Satzspiegel von 249 mm.
fn push_belege(
    doc: &mut genpdf::Document,
    foto_dir: &Path,
    bilder: &[(&str, &str)],
) -> Result<()> {
    for (i, (datei, legende)) in bilder.iter().enumerate() {
        if i % 2 == 0 {
            doc.push(PageBreak::new());
        }
        let pfad = foto_dir.join(datei);
        let bild = Image::from_path(&pfad)
            .with_context(|| format!("Beleg laden: {}", pfad.display()))?
            .with_alignment(Alignment::Left)
            .with_dpi(BILD_DPI);
        doc.push(bild);
        doc.push(Break::new(0.25));
        push_lines(
            doc,
            legende,
            Style::new().with_color(SLATE).with_font_size(9).italic(),
            Alignment::Left,
        );
        doc.push(Break::new(0.6));
    }
    Ok(())
}

/// Ein Unterpunkt, dessen Titel mit diesem Zeichen beginnt, wird auf einer
/// neuen Seite begonnen.
///
/// Für die Hauptziffern erzwingt `push_begruendung` den Umbruch ohnehin. Bei
/// den Zwischentiteln geht das nicht: sie sollen im Fluss stehen. Fällt einer
/// von ihnen ans Seitenende, steht er dort allein und sein Text beginnt erst
/// auf der nächsten Seite - genau der Satzfehler, den genpdf 0.2 nicht
/// verhindern kann, weil es weder eine Schusterjungen-Regel kennt noch die
/// Resthöhe einer Seite preisgibt. Dann wird der Umbruch hier von Hand
/// gesetzt, indem der Titel im Inhalt mit diesem Zeichen beginnt.
const UMBRUCH: char = '@';

fn push_begruendung(doc: &mut genpdf::Document, foto_dir: &Path) -> Result<()> {
    push_lines(
        doc,
        "Begründung",
        Style::new().with_color(SLATE).with_font_size(12).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.8));
    for (i, a) in ABSCHNITTE.iter().enumerate() {
        // Jede Hauptziffer auf eine eigene Seite. genpdf 0.2 kennt keine
        // Schusterjungen-Regel: ein Titel landet dort, wo er hinfällt, und
        // steht dann allein am Seitenfuss, während sein Text erst auf der
        // nächsten Seite beginnt. Da sich die Resthöhe einer Seite von aussen
        // nicht abfragen lässt, wird der Umbruch hier erzwungen. Das kostet
        // Weissraum, macht die Gliederung im Ausdruck aber sofort lesbar - und
        // in einer Rechtsschrift, auf deren Ziffern später verwiesen wird, ist
        // das mehr wert als eine dichte Seite.
        if i > 0 {
            doc.push(PageBreak::new());
        }
        push_lines(
            doc,
            &format!("{}. {}", i + 1, a.titel),
            Style::new().with_color(GOLD).with_font_size(10).bold(),
            Alignment::Left,
        );
        doc.push(Break::new(0.35));
        if !a.text.is_empty() {
            body(doc, a.text);
        }
        for (j, (titel, text)) in a.unterpunkte.iter().enumerate() {
            match titel.strip_prefix(UMBRUCH) {
                Some(_) => doc.push(PageBreak::new()),
                None => doc.push(Break::new(0.5)),
            }
            let titel = titel.strip_prefix(UMBRUCH).unwrap_or(titel);
            push_lines(
                doc,
                &format!("{}.{} {}", i + 1, j + 1, titel),
                Style::new().with_color(SLATE).with_font_size(10).bold(),
                Alignment::Left,
            );
            doc.push(Break::new(0.25));
            body(doc, text);
        }
        push_belege(doc, foto_dir, a.bilder)?;
        doc.push(Break::new(1.0));
    }
    Ok(())
}

fn push_schluss(doc: &mut genpdf::Document) {
    body(doc, SCHLUSS);
    doc.push(Break::new(1.2));
    push_lines(
        doc,
        GRUSS,
        Style::new().with_color(INK).with_font_size(10),
        Alignment::Left,
    );
    doc.push(Break::new(2.5));
    for zeile in UNTERSCHRIFTEN {
        push_lines(
            doc,
            zeile,
            Style::new().with_color(INK).with_font_size(10),
            Alignment::Left,
        );
    }
    if !BEILAGEN.is_empty() {
        doc.push(Break::new(1.6));
        push_lines(
            doc,
            "Beilagen",
            Style::new().with_color(SLATE).with_font_size(10).bold(),
            Alignment::Left,
        );
        doc.push(Break::new(0.3));
        for (i, b) in BEILAGEN.iter().enumerate() {
            push_lines(
                doc,
                &format!("{}. {}", i + 1, b),
                Style::new().with_color(INK).with_font_size(10),
                Alignment::Left,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Schrift, Aufruf
// ---------------------------------------------------------------------------

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
    doc.set_title(KOPF.betreff);
    doc.set_minimal_conformance();
    doc.set_font_size(10);
    doc.set_line_spacing(1.4);

    let kopf = KURZTITEL;
    let mut deco = genpdf::SimplePageDecorator::new();
    deco.set_margins(24);
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

    push_kopf(&mut doc);
    push_antraege(&mut doc);
    push_begruendung(&mut doc, foto_dir)?;
    push_schluss(&mut doc);

    doc.render_to_file(out)
        .map_err(|e| anyhow!("PDF schreiben {}: {}", out.display(), e))?;
    Ok(())
}

/// Wie im Bildinventar: genpdf legt Bilder als entpackte RGB-Pixel ab. Die
/// Vorlagen sind baseline-JPEG und lassen sich unmittelbar als
/// `DCTDecode`-Strom einsetzen. Die Zuordnung läuft über die Reihenfolge und
/// bricht bei abweichender Anzahl ab.
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
            "{} Bildobjekte im PDF, aber {} Belege - die Zuordnung über die \
             Reihenfolge wäre nicht mehr verlässlich",
            ids.len(),
            dateien.len()
        ));
    }
    for (id, datei) in ids.iter().zip(dateien) {
        let roh = std::fs::read(datei)
            .with_context(|| format!("Beleg lesen: {}", datei.display()))?;
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

    let mut dateien = Vec::new();
    for a in ABSCHNITTE {
        for (datei, _) in a.bilder {
            let p = foto_dir.join(datei);
            if !p.exists() {
                return Err(anyhow!(
                    "Beleg fehlt: {} (Bildverzeichnis über $FOTO_DIR setzen)",
                    p.display()
                ));
            }
            dateien.push(p);
        }
    }

    render(&out, &font_dir, &foto_dir)?;
    if !dateien.is_empty() {
        jpegs_einsetzen(&out, &dateien)?;
    }
    let groesse = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
    println!(
        "{} geschrieben – {} Anträge, {} Abschnitte, {} Belegbilder, {:.1} MB.",
        out.display(),
        ANTRAEGE.len(),
        ABSCHNITTE.len(),
        dateien.len(),
        groesse as f64 / 1_048_576.0
    );
    Ok(())
}
