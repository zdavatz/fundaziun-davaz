// Konzeptpapier als PDF: Titel, Abschnitte, Absätze, Listen, Tabellen und
// hervorgehobene Hinweise.
//
// Gedacht für Arbeitspapiere der Familie - etwa das Konzept, den
// Immobilienbesitz in eine Aktiengesellschaft zu überführen -, die in
// mehreren Fassungen entstehen und jedes Mal sauber gesetzt sein sollen.
// Der Inhalt steht als Blockfolge in src/konzept_inhalt.rs; das Programm
// kennt nur den Satz.
//
//   cargo run --release --bin konzept
//   cargo run --release --bin konzept -- --out /pfad/zum.pdf
//
// Schriftverzeichnis über $FONT_DIR (Vorgabe: ./fonts).
//
// Die Inhaltsdatei ist in .gitignore ausgeschlossen (src/*_inhalt.rs): sie
// nennt Familienmitglieder, Liegenschaften und Vermögenswerte. build.rs
// legt die neutrale Fassung aus src/konzept_inhalt.beispiel.rs aus.
//
// Auszeichnung im Text: «**fett**» setzt einen Lauf fett. Mehr gibt es
// nicht; wer Kursives oder Links braucht, erweitert `push_markup`.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use genpdf::elements::{Break, FrameCellDecorator, PageBreak, Paragraph, TableLayout};
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Element};

const DEFAULT_FONT_DIR: &str = "fonts";
const DEFAULT_OUT: &str = "Konzept.pdf";

const INK: Color = Color::Rgb(0x1b, 0x1b, 0x1d);
const SLATE: Color = Color::Rgb(0x3a, 0x3d, 0x44);
const MUTED: Color = Color::Rgb(0x8a, 0x8d, 0x94);

/// Ein Baustein des Papiers. Die Reihenfolge in `BLOECKE` ist die
/// Reihenfolge im Dokument.
#[allow(dead_code)]
enum Block {
    /// Hauptziffer, z. B. «1. Ziel und Zweck».
    H2(&'static str),
    /// Zwischentitel, z. B. «2.1 Grundmodell».
    H3(&'static str),
    /// Fliesstext; Leerzeilen trennen Absätze, «**…**» setzt fett.
    P(&'static str),
    /// Aufzählung mit Punkt; je Eintrag ein Absatz.
    Liste(&'static [&'static str]),
    /// Tabelle: Spaltenbreiten (relativ), Kopfzeile, Zeilen; `rechts`
    /// nennt die rechtsbündigen Spalten (Zahlen).
    Tabelle {
        breiten: &'static [usize],
        kopf: &'static [&'static str],
        zeilen: &'static [&'static [&'static str]],
        rechts: &'static [usize],
    },
    /// Hervorgehobener Kasten.
    Hinweis(&'static str),
    /// Neue Seite.
    Umbruch,
}

struct Kopf {
    titel: &'static str,
    untertitel: &'static str,
    einleitung: &'static str,
    /// Kopfzeile auf jeder Seite ausser der ersten.
    kopfzeile: &'static str,
}

include!("konzept_inhalt.rs");

// ---------------------------------------------------------------------------
// Satz

/// Absatz mit «**fett**»-Auszeichnung. Stil zweimal setzen - siehe
/// bildinventar.rs.
fn push_markup(doc: &mut genpdf::Document, text: &str, style: Style, align: Alignment) {
    let mut p = Paragraph::default();
    for (i, teil) in text.split("**").enumerate() {
        if teil.is_empty() {
            continue;
        }
        if i % 2 == 1 {
            p.push_styled(teil.to_string(), style.bold());
        } else {
            p.push_styled(teil.to_string(), style);
        }
    }
    doc.push(p.aligned(align).styled(style));
}

fn absatz_stil() -> Style {
    Style::new().with_color(INK).with_font_size(10)
}

fn push_absaetze(doc: &mut genpdf::Document, text: &str) {
    for abs in text.split("\n\n") {
        let abs = abs.trim();
        if abs.is_empty() {
            continue;
        }
        push_markup(doc, abs, absatz_stil(), Alignment::Left);
        doc.push(Break::new(0.5));
    }
}

fn push_h2(doc: &mut genpdf::Document, titel: &str) {
    doc.push(Break::new(0.8));
    push_markup(
        doc,
        titel,
        Style::new().with_color(SLATE).with_font_size(13).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.5));
}

fn push_h3(doc: &mut genpdf::Document, titel: &str) {
    doc.push(Break::new(0.4));
    push_markup(
        doc,
        titel,
        Style::new().with_color(SLATE).with_font_size(10).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.2));
}

/// Aufzählung als zweispaltige Tabelle ohne Rahmen: Punkt links, Text
/// rechts. genpdf hat keine Listen; so bleibt der Einzug bei Umbrüchen
/// erhalten.
fn push_liste(doc: &mut genpdf::Document, eintraege: &[&str]) -> Result<()> {
    let mut t = TableLayout::new(vec![1, 30]);
    t.set_cell_decorator(FrameCellDecorator::new(false, false, false));
    for e in eintraege {
        let mut punkt = Paragraph::default();
        punkt.push_styled("•".to_string(), absatz_stil());
        let mut p = Paragraph::default();
        for (i, teil) in e.split("**").enumerate() {
            if teil.is_empty() {
                continue;
            }
            let s = if i % 2 == 1 { absatz_stil().bold() } else { absatz_stil() };
            p.push_styled(teil.to_string(), s);
        }
        t.row()
            .element(punkt.styled(absatz_stil()).padded(genpdf::Margins::trbl(0, 1, 1, 2)))
            .element(p.styled(absatz_stil()).padded(genpdf::Margins::trbl(0, 0, 1, 0)))
            .push()
            .map_err(|e| anyhow!("Listeneintrag: {e}"))?;
    }
    doc.push(t);
    doc.push(Break::new(0.4));
    Ok(())
}

fn zelle(
    text: &str,
    style: Style,
    align: Alignment,
) -> genpdf::elements::PaddedElement<genpdf::elements::StyledElement<Paragraph>> {
    let mut p = Paragraph::default();
    for (i, teil) in text.split("**").enumerate() {
        if teil.is_empty() {
            continue;
        }
        let s = if i % 2 == 1 { style.bold() } else { style };
        p.push_styled(teil.to_string(), s);
    }
    p.aligned(align)
        .styled(style)
        .padded(genpdf::Margins::trbl(1, 1.5, 1, 1.5))
}

fn push_tabelle(
    doc: &mut genpdf::Document,
    breiten: &[usize],
    kopf: &[&str],
    zeilen: &[&[&str]],
    rechts: &[usize],
) -> Result<()> {
    let mut t = TableLayout::new(breiten.to_vec());
    t.set_cell_decorator(FrameCellDecorator::new(true, true, false));
    let ausr = |i: usize| {
        if rechts.contains(&i) {
            Alignment::Right
        } else {
            Alignment::Left
        }
    };
    let kopfstil = Style::new().with_color(SLATE).with_font_size(9).bold();
    let mut r = t.row();
    for (i, k) in kopf.iter().enumerate() {
        r = r.element(zelle(k, kopfstil, ausr(i)));
    }
    r.push().map_err(|e| anyhow!("Tabellenkopf: {e}"))?;
    let stil = Style::new().with_color(INK).with_font_size(9);
    for z in zeilen {
        let mut r = t.row();
        for (i, c) in z.iter().enumerate() {
            r = r.element(zelle(c, stil, ausr(i)));
        }
        r.push().map_err(|e| anyhow!("Tabellenzeile: {e}"))?;
    }
    doc.push(t);
    doc.push(Break::new(0.6));
    Ok(())
}

/// Kasten: einzellige Tabelle mit Rahmen.
fn push_hinweis(doc: &mut genpdf::Document, text: &str) -> Result<()> {
    let mut t = TableLayout::new(vec![1]);
    t.set_cell_decorator(FrameCellDecorator::new(true, true, false));
    let mut p = Paragraph::default();
    let stil = Style::new().with_color(SLATE).with_font_size(9.5 as u8);
    for (i, teil) in text.split("**").enumerate() {
        if teil.is_empty() {
            continue;
        }
        let s = if i % 2 == 1 { stil.bold() } else { stil };
        p.push_styled(teil.to_string(), s);
    }
    t.row()
        .element(p.styled(stil).padded(genpdf::Margins::trbl(2, 3, 2, 3)))
        .push()
        .map_err(|e| anyhow!("Hinweis: {e}"))?;
    doc.push(t);
    doc.push(Break::new(0.6));
    Ok(())
}

fn push_titel(doc: &mut genpdf::Document) {
    push_markup(
        doc,
        KOPF.titel,
        Style::new().with_color(INK).with_font_size(20).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.3));
    push_markup(
        doc,
        KOPF.untertitel,
        Style::new().with_color(SLATE).with_font_size(12),
        Alignment::Left,
    );
    doc.push(Break::new(0.6));
    push_markup(
        doc,
        KOPF.einleitung,
        Style::new().with_color(MUTED).with_font_size(9),
        Alignment::Left,
    );
    doc.push(Break::new(0.8));
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

fn render(out: &Path, font_dir: &str) -> Result<usize> {
    let family = load_font_family(font_dir)?;
    let mut doc = genpdf::Document::new(family);
    doc.set_title(KOPF.titel);
    doc.set_minimal_conformance();
    doc.set_font_size(10);
    doc.set_line_spacing(1.3);

    let kopf = KOPF.kopfzeile;
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
    for b in BLOECKE {
        match b {
            Block::H2(t) => push_h2(&mut doc, t),
            Block::H3(t) => push_h3(&mut doc, t),
            Block::P(t) => push_absaetze(&mut doc, t),
            Block::Liste(e) => push_liste(&mut doc, e)?,
            Block::Tabelle { breiten, kopf, zeilen, rechts } => {
                push_tabelle(&mut doc, breiten, kopf, zeilen, rechts)?
            }
            Block::Hinweis(t) => push_hinweis(&mut doc, t)?,
            Block::Umbruch => doc.push(PageBreak::new()),
        }
    }

    doc.render_to_file(out)
        .map_err(|e| anyhow!("PDF schreiben {}: {}", out.display(), e))?;
    Ok(BLOECKE.len())
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
    let n = render(&out, &font_dir)?;
    let mb = std::fs::metadata(&out)?.len() as f64 / 1_048_576.0;
    println!("{} geschrieben – {} Blöcke, {:.2} MB.", out.display(), n, mb);
    Ok(())
}
