// Bewilligungslage - was bewilligt wurde, was nicht, und wie die Entscheide
// zugestellt worden sind. Setzt aus dem Inhalt in src/bewilligung_inhalt.rs
// ein PDF: Übersichtstabellen, die massgebende Norm im Wortlaut, danach je
// Seite eine Ablichtung aus den Originalakten als Beleg.
//
//     cargo run --release --bin bewilligung
//     cargo run --release --bin bewilligung -- --out /pfad/zum.pdf
//
// Schriften aus $FONT_DIR (Vorgabe: ./fonts), Belegseiten als JPEG aus
// $BELEG_DIR (Vorgabe: ./attachments/bauamt/bewilligung_belege).
//
// Wie beim Bildinventar werden die Bildströme nach dem Satz gegen die
// JPEG-Dateien getauscht (`jpegs_einsetzen`), sonst wächst das PDF auf
// Dutzende Megabyte. Die Zuordnung läuft über die Reihenfolge und bricht bei
// abweichender Anzahl ab.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use genpdf::elements::{Break, FrameCellDecorator, Image, PageBreak, Paragraph, TableLayout};
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Element};

const DEFAULT_FONT_DIR: &str = "fonts";
const DEFAULT_BELEG_DIR: &str = "attachments/bauamt/bewilligung_belege";
const DEFAULT_OUT: &str = "Bewilligungslage.pdf";

const INK: Color = Color::Rgb(0x1b, 0x1b, 0x1d);
const GOLD: Color = Color::Rgb(0xa0, 0x8b, 0x6a);
const SLATE: Color = Color::Rgb(0x3a, 0x3d, 0x44);
const MUTED: Color = Color::Rgb(0x8a, 0x8d, 0x94);

// Die Belegseiten sind A4-Scans mit rund 1850 Pixeln Höhe. Bei 215 dpi
// ergibt das etwa 218 mm - eine Seite samt Legende passt so auf ein Blatt,
// und genpdf muss nie ein Bild umbrechen, was es nicht kann.
const BELEG_DPI: f64 = 215.0;

// ---------------------------------------------------------------------------
// Inhalt
// ---------------------------------------------------------------------------

struct Objekt {
    titel: &'static str,
    untertitel: &'static str,
    angaben: &'static [&'static str],
    stand: &'static str,
}

/// Ein Verfahren: Einleitung, Tabelle der Bauvorhaben, Würdigung.
struct Verfahren {
    titel: &'static str,
    einleitung: &'static str,
    kopf: [&'static str; 5],
    zeilen: &'static [[&'static str; 5]],
    wuerdigung: &'static str,
}

/// Ein Schritt der Zustellung und Fristen.
struct Schritt {
    datum: &'static str,
    vorgang: &'static str,
    beleg: &'static str,
}

/// Eine Belegseite: Ablichtung aus den Originalakten.
struct Beleg {
    datei: &'static str,
    titel: &'static str,
    hinweis: &'static str,
}

include!("bewilligung_inhalt.rs");

// ---------------------------------------------------------------------------
// Satz
// ---------------------------------------------------------------------------

/// Stil zweimal setzen - siehe bildinventar.rs.
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

fn h2(doc: &mut genpdf::Document, titel: &str) {
    doc.push(Break::new(0.6));
    push_lines(
        doc,
        titel,
        Style::new().with_color(SLATE).with_font_size(12).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.4));
}

fn zelle(text: &str, style: Style) -> genpdf::elements::PaddedElement<genpdf::elements::StyledElement<Paragraph>> {
    let mut p = Paragraph::default();
    p.push_styled(text.to_string(), style);
    p.styled(style).padded(genpdf::Margins::trbl(1, 1.5, 1, 1.5))
}

fn push_tabelle(
    doc: &mut genpdf::Document,
    breiten: Vec<usize>,
    kopf: &[&str],
    zeilen: &[Vec<&str>],
) -> Result<()> {
    let mut t = TableLayout::new(breiten);
    t.set_cell_decorator(FrameCellDecorator::new(true, true, false));
    let kopfstil = Style::new().with_color(SLATE).with_font_size(9).bold();
    let mut r = t.row();
    for k in kopf {
        r = r.element(zelle(k, kopfstil));
    }
    r.push().map_err(|e| anyhow!("Tabellenkopf: {e}"))?;
    let stil = Style::new().with_color(INK).with_font_size(9);
    for z in zeilen {
        let mut r = t.row();
        for c in z {
            r = r.element(zelle(c, stil));
        }
        r.push().map_err(|e| anyhow!("Tabellenzeile: {e}"))?;
    }
    doc.push(t);
    Ok(())
}

fn push_titel(doc: &mut genpdf::Document) {
    push_lines(
        doc,
        OBJEKT.untertitel,
        Style::new().with_color(GOLD).with_font_size(9).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.3));
    push_lines(
        doc,
        OBJEKT.titel,
        Style::new().with_color(SLATE).with_font_size(18).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.5));
    for a in OBJEKT.angaben {
        push_lines(
            doc,
            a,
            Style::new().with_color(INK).with_font_size(10),
            Alignment::Left,
        );
    }
    doc.push(Break::new(0.2));
    push_lines(
        doc,
        OBJEKT.stand,
        Style::new().with_color(MUTED).with_font_size(9),
        Alignment::Left,
    );
}

fn push_verfahren(doc: &mut genpdf::Document, v: &Verfahren) -> Result<()> {
    h2(doc, v.titel);
    body(doc, v.einleitung);
    doc.push(Break::new(0.4));
    let zeilen: Vec<Vec<&str>> = v.zeilen.iter().map(|z| z.to_vec()).collect();
    push_tabelle(doc, vec![2, 12, 6, 7, 13], &v.kopf, &zeilen)?;
    doc.push(Break::new(0.4));
    body(doc, v.wuerdigung);
    Ok(())
}

fn push_zustellung(doc: &mut genpdf::Document) -> Result<()> {
    h2(doc, ZUSTELLUNG_TITEL);
    body(doc, ZUSTELLUNG_EINLEITUNG);
    doc.push(Break::new(0.4));
    let zeilen: Vec<Vec<&str>> = SCHRITTE
        .iter()
        .map(|s| vec![s.datum, s.vorgang, s.beleg])
        .collect();
    push_tabelle(doc, vec![6, 24, 10], &["Datum", "Vorgang", "Beleg"], &zeilen)?;
    doc.push(Break::new(0.4));
    body(doc, ZUSTELLUNG_FOLGE);
    Ok(())
}

fn push_norm(doc: &mut genpdf::Document) {
    h2(doc, NORM_TITEL);
    body(doc, NORM_EINLEITUNG);
    doc.push(Break::new(0.3));
    // Der Wortlaut eingerückt und etwas kleiner, damit er sich vom eigenen
    // Text abhebt, ohne ein Zitat zu simulieren, das es nicht ist.
    for line in NORM_TEXT.split('\n') {
        let style = Style::new().with_color(SLATE).with_font_size(9);
        let mut p = Paragraph::default();
        p.push_styled(line.to_string(), style);
        doc.push(
            p.styled(style)
                .padded(genpdf::Margins::trbl(0, 0, 0, 6)),
        );
    }
    doc.push(Break::new(0.3));
    body(doc, NORM_FOLGERUNG);
}

fn push_belege(doc: &mut genpdf::Document, beleg_dir: &Path) -> Result<()> {
    for (i, b) in BELEGE.iter().enumerate() {
        doc.push(PageBreak::new());
        push_lines(
            doc,
            &format!("Beleg {}   ·   {}", i + 1, b.titel),
            Style::new().with_color(GOLD).with_font_size(9).bold(),
            Alignment::Left,
        );
        doc.push(Break::new(0.15));
        push_lines(
            doc,
            b.hinweis,
            Style::new().with_color(MUTED).with_font_size(9),
            Alignment::Left,
        );
        doc.push(Break::new(0.3));
        let pfad = beleg_dir.join(b.datei);
        let bild = Image::from_path(&pfad)
            .with_context(|| format!("Beleg {} laden: {}", i + 1, pfad.display()))?
            .with_alignment(Alignment::Left)
            .with_dpi(BELEG_DPI);
        doc.push(bild);
    }
    Ok(())
}

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

fn render(out: &Path, font_dir: &str, beleg_dir: &Path) -> Result<()> {
    let family = load_font_family(font_dir)?;
    let mut doc = genpdf::Document::new(family);
    doc.set_title(OBJEKT.titel);
    doc.set_minimal_conformance();
    doc.set_font_size(10);
    doc.set_line_spacing(1.3);

    let kopf = OBJEKT.untertitel;
    let mut deco = genpdf::SimplePageDecorator::new();
    deco.set_margins(20);
    deco.set_header(move |page| {
        let mut p = Paragraph::default();
        if page > 1 {
            p.push_styled(
                format!("{kopf}          {page}"),
                Style::new().with_color(MUTED).with_font_size(7),
            );
        }
        p.aligned(Alignment::Right)
            .padded(genpdf::Margins::trbl(0, 0, 5, 0))
    });
    doc.set_page_decorator(deco);

    push_titel(&mut doc);
    for v in VERFAHREN {
        push_verfahren(&mut doc, v)?;
    }
    push_zustellung(&mut doc)?;
    push_norm(&mut doc);
    h2(&mut doc, VORGESCHICHTE_TITEL);
    body(&mut doc, VORGESCHICHTE);
    doc.push(PageBreak::new());
    h2(&mut doc, SCHLUSS_TITEL);
    body(&mut doc, SCHLUSS);
    doc.push(Break::new(0.6));
    push_lines(
        &mut doc,
        QUELLEN,
        Style::new().with_color(MUTED).with_font_size(8),
        Alignment::Left,
    );
    push_belege(&mut doc, beleg_dir)?;

    doc.render_to_file(out)
        .map_err(|e| anyhow!("PDF schreiben {}: {}", out.display(), e))?;
    Ok(())
}

/// Bildströme gegen die JPEG-Originale tauschen - siehe bildinventar.rs.
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
    let beleg_dir =
        PathBuf::from(env::var("BELEG_DIR").unwrap_or_else(|_| DEFAULT_BELEG_DIR.to_string()));

    let mut dateien = Vec::new();
    for (i, b) in BELEGE.iter().enumerate() {
        let p = beleg_dir.join(b.datei);
        if !p.exists() {
            return Err(anyhow!(
                "Beleg {} fehlt: {} (Verzeichnis über $BELEG_DIR setzen)",
                i + 1,
                p.display()
            ));
        }
        dateien.push(p);
    }

    render(&out, &font_dir, &beleg_dir)?;
    jpegs_einsetzen(&out, &dateien)?;
    let groesse = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
    println!(
        "{} geschrieben – {} Belege, {:.1} MB.",
        out.display(),
        dateien.len(),
        groesse as f64 / 1_048_576.0
    );
    Ok(())
}
