// Aktendossier: ein Ordner Fotos einer Akteneinsicht als ein PDF.
//
// Wer ein Baudossier bei der Gemeinde nur fotografieren darf, hat danach
// einige hundert Handyaufnahmen: unsortiert, teils gedreht, mehrere Megabyte
// je Bild. Für eine Beilage im Verfahren braucht es ein einziges PDF, in dem
// jede Aufnahme aufrecht steht, eine Seitenzahl trägt und ihren Dateinamen
// nennt, damit sich ein Aktenverzeichnis darauf beziehen kann.
//
//   cargo run --release --bin aktendossier -- \
//       --dir /pfad/zu/den/fotos --out Dossier.pdf \
//       --titel "Baudossier …, Akteneinsicht 10.9.2026" \
//       [--orient orientierungen.txt] [--reihenfolge reihenfolge.txt] [--hoehe 1600]
//
// Die Bilder werden verkleinert (Vorgabe: 1600 Pixel Höhe), als JPEG neu
// kodiert und mit DCTDecode ins PDF gelegt. Das PDF wird direkt mit lopdf
// geschrieben, nicht mit genpdf: genpdf bettet Bilder als entpackte Pixel
// ein, und vierhundert Aufnahmen ergäben damit mehrere Gigabyte.
//
// Drehung: Zuerst wird die EXIF-Ausrichtung des Handys angewendet, die das
// `image`-Paket 0.23 nicht liest. Die Blätter eines Dossiers liegen aber oft
// zusätzlich quer oder kopfstehend auf dem Tisch. Deshalb nimmt --orient eine Textdatei mit je Zeile «dateiname up|right|
// left|down», wie sie die Texterkennung liefert, die alle vier Lagen
// durchprobiert und die mit dem meisten Text behält. Fehlt die Datei oder
// der Eintrag, bleibt das Bild, wie es ist.
//
// Reihenfolge: Ohne --reihenfolge kommen die Dateien alphabetisch, also in
// Aufnahmefolge. Mit --reihenfolge bestimmt eine Textdatei die Seitenfolge –
// je Zeile «dateiname<TAB>beschriftung», etwa chronologisch nach dem
// Aktenverzeichnis; die Beschriftung erscheint in der Fusszeile. Dateien, die
// in der Liste fehlen, folgen am Schluss in Aufnahmefolge.
//
// Die Fusszeile nutzt die PDF-Standardschrift Helvetica in WinAnsi-Kodierung;
// sie braucht keine eingebettete Schrift, und Umlaute sind darin enthalten.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, GenericImageView};
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};

const A4_BREIT: f64 = 595.0;
const A4_HOCH: f64 = 842.0;
const RAND: f64 = 20.0;
const FUSS: f64 = 30.0;
const SCHRIFT: f64 = 7.0;

struct Optionen {
    dir: PathBuf,
    out: PathBuf,
    titel: String,
    orient: Option<PathBuf>,
    reihenfolge: Option<PathBuf>,
    hoehe: u32,
}

fn optionen() -> Result<Optionen> {
    let mut o = Optionen {
        dir: PathBuf::new(),
        out: PathBuf::from("Aktendossier.pdf"),
        titel: String::from("Aktendossier"),
        orient: None,
        reihenfolge: None,
        hoehe: 1600,
    };
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        let mut wert = || args.next().ok_or_else(|| anyhow!("{a} erwartet einen Wert"));
        match a.as_str() {
            "--dir" => o.dir = PathBuf::from(wert()?),
            "--out" => o.out = PathBuf::from(wert()?),
            "--titel" => o.titel = wert()?,
            "--orient" => o.orient = Some(PathBuf::from(wert()?)),
            "--reihenfolge" => o.reihenfolge = Some(PathBuf::from(wert()?)),
            "--hoehe" => o.hoehe = wert()?.parse().context("--hoehe: ganze Zahl")?,
            other => return Err(anyhow!("unbekanntes Argument: {other}")),
        }
    }
    if o.dir.as_os_str().is_empty() {
        return Err(anyhow!("--dir fehlt"));
    }
    Ok(o)
}

/// Liest «dateiname lage» je Zeile; unbekannte Lagen werden ignoriert.
fn orientierungen(pfad: &Path) -> Result<HashMap<String, &'static str>> {
    let text = fs::read_to_string(pfad)
        .with_context(|| format!("Orientierungen lesen: {}", pfad.display()))?;
    let mut map = HashMap::new();
    for zeile in text.lines() {
        let mut teile = zeile.split_whitespace();
        if let (Some(name), Some(lage)) = (teile.next(), teile.next()) {
            let lage = match lage {
                "up" => "up",
                "right" => "right",
                "left" => "left",
                "down" => "down",
                _ => continue,
            };
            map.insert(name.to_string(), lage);
        }
    }
    Ok(map)
}

/// Liest «dateiname<TAB>beschriftung» je Zeile; ohne Tabulator bleibt die
/// Beschriftung leer.
fn reihenfolge(pfad: &Path) -> Result<Vec<(String, String)>> {
    let text = fs::read_to_string(pfad)
        .with_context(|| format!("Reihenfolge lesen: {}", pfad.display()))?;
    Ok(text
        .lines()
        .filter(|z| !z.trim().is_empty())
        .map(|z| {
            let mut t = z.splitn(2, '\t');
            (
                t.next().unwrap_or("").trim().to_string(),
                t.next().unwrap_or("").trim().to_string(),
            )
        })
        .collect())
}

/// EXIF-Ausrichtung (Tag 0x0112) aus dem APP1-Segment eines JPEG; 1, wenn
/// keine vorhanden. Handys speichern die Sensorlage so, statt die Pixel zu
/// drehen; `image` 0.23 ignoriert das Tag, die Texterkennung wendet es an.
/// Damit beide vom selben Bild sprechen, wird es hier nachvollzogen.
fn exif_ausrichtung(roh: &[u8]) -> u32 {
    let mut i = 2;
    while i + 4 <= roh.len() && roh[i] == 0xFF {
        let marker = roh[i + 1];
        let laenge = u16::from_be_bytes([roh[i + 2], roh[i + 3]]) as usize;
        if marker == 0xE1 && roh.len() >= i + 4 + laenge && &roh[i + 4..i + 10] == b"Exif\0\0" {
            let t = &roh[i + 10..i + 2 + laenge];
            if t.len() < 8 {
                return 1;
            }
            let le = &t[0..2] == b"II";
            let u16at = |p: usize| -> u32 {
                if p + 2 > t.len() {
                    return 0;
                }
                let b = [t[p], t[p + 1]];
                (if le { u16::from_le_bytes(b) } else { u16::from_be_bytes(b) }) as u32
            };
            let u32at = |p: usize| -> usize {
                if p + 4 > t.len() {
                    return 0;
                }
                let b = [t[p], t[p + 1], t[p + 2], t[p + 3]];
                (if le { u32::from_le_bytes(b) } else { u32::from_be_bytes(b) }) as usize
            };
            let ifd = u32at(4);
            let n = u16at(ifd) as usize;
            for k in 0..n {
                let e = ifd + 2 + k * 12;
                if u16at(e) == 0x0112 {
                    return u16at(e + 8);
                }
            }
            return 1;
        }
        if marker == 0xDA {
            break;
        }
        i += 2 + laenge;
    }
    1
}

/// Lädt ein Foto, dreht es aufrecht, verkleinert es und kodiert es als JPEG.
fn jpeg_aufbereiten(pfad: &Path, lage: &str, hoehe: u32) -> Result<(Vec<u8>, u32, u32)> {
    let roh = fs::read(pfad).with_context(|| format!("Bild lesen: {}", pfad.display()))?;
    let bild = image::load_from_memory(&roh)
        .with_context(|| format!("Bild dekodieren: {}", pfad.display()))?;
    // Zuerst die EXIF-Lage, so wie es jeder Bildbetrachter tut.
    let bild = match exif_ausrichtung(&roh) {
        3 => bild.rotate180(),
        6 => bild.rotate90(),
        8 => bild.rotate270(),
        _ => bild,
    };
    // Dann die Lage aus der Texterkennung: «right» heisst, das Bild musste
    // um 90° im Uhrzeigersinn gedreht werden, damit der Text lesbar war.
    let bild = match lage {
        "right" => bild.rotate90(),
        "left" => bild.rotate270(),
        "down" => bild.rotate180(),
        _ => bild,
    };
    let (b, h) = bild.dimensions();
    let bild = if h > hoehe {
        let nb = (b as f64 * hoehe as f64 / h as f64).round() as u32;
        bild.resize_exact(nb, hoehe, image::imageops::FilterType::Lanczos3)
    } else {
        bild
    };
    let rgb = bild.to_rgb8();
    let (b, h) = rgb.dimensions();
    let mut roh = Vec::new();
    JpegEncoder::new_with_quality(&mut roh, 72)
        .encode(rgb.as_raw(), b, h, ColorType::Rgb8)
        .context("JPEG kodieren")?;
    Ok((roh, b, h))
}

/// Text für die Fusszeile in WinAnsi (Latin-1); Zeichen ausserhalb werden
/// durch «?» ersetzt, damit die Zeile nie leer bleibt.
fn winansi(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| {
            let n = c as u32;
            if n < 256 {
                n as u8
            } else {
                b'?'
            }
        })
        .collect()
}

/// PDF-String mit maskierten Klammern und Backslashes.
fn pdf_string(bytes: Vec<u8>) -> Object {
    Object::String(bytes, lopdf::StringFormat::Literal)
}

fn main() -> Result<()> {
    let o = optionen()?;
    let lagen = match &o.orient {
        Some(p) => orientierungen(p)?,
        None => HashMap::new(),
    };

    let mut dateien: Vec<PathBuf> = fs::read_dir(&o.dir)
        .with_context(|| format!("Verzeichnis lesen: {}", o.dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e.to_ascii_lowercase().as_str(), "jpg" | "jpeg"))
                .unwrap_or(false)
        })
        .collect();
    dateien.sort();
    if dateien.is_empty() {
        return Err(anyhow!("keine JPEG-Dateien in {}", o.dir.display()));
    }
    // Seitenfolge und Beschriftung: aus der Liste, Rest hinten anfügen.
    let mut seiten: Vec<(PathBuf, String)> = Vec::with_capacity(dateien.len());
    if let Some(p) = &o.reihenfolge {
        let mut uebrig: Vec<PathBuf> = dateien.clone();
        for (name, text) in reihenfolge(p)? {
            if let Some(i) = uebrig.iter().position(|d| {
                d.file_name().and_then(|n| n.to_str()) == Some(name.as_str())
            }) {
                seiten.push((uebrig.remove(i), text));
            } else {
                eprintln!("Reihenfolge: {} nicht im Verzeichnis, übersprungen", name);
            }
        }
        for d in uebrig {
            seiten.push((d, String::new()));
        }
    } else {
        seiten = dateien.into_iter().map(|d| (d, String::new())).collect();
    }
    let anzahl = seiten.len();

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding",
    });
    let mut kids = Vec::with_capacity(anzahl);

    for (i, (pfad, text)) in seiten.iter().enumerate() {
        let name = pfad
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let lage = lagen.get(&name).copied().unwrap_or("up");
        let (roh, b, h) = jpeg_aufbereiten(pfad, lage, o.hoehe)?;

        let bild_id = doc.add_object(Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => b as i64,
                "Height" => h as i64,
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => 8,
                "Filter" => "DCTDecode",
            },
            roh,
        ));

        // Hochformat für stehende, Querformat für liegende Aufnahmen.
        let (sb, sh) = if b <= h { (A4_BREIT, A4_HOCH) } else { (A4_HOCH, A4_BREIT) };
        let skala = ((sb - 2.0 * RAND) / b as f64).min((sh - RAND - FUSS - 10.0) / h as f64);
        let zb = b as f64 * skala;
        let zh = h as f64 * skala;
        let x = (sb - zb) / 2.0;
        let y = sh - RAND - zh;

        let fuss = if text.is_empty() {
            format!("{} – Seite {} von {} – {}", o.titel, i + 1, anzahl, name)
        } else {
            format!("{} – Seite {} von {} – {} – {}", o.titel, i + 1, anzahl, text, name)
        };
        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "cm",
                    vec![zb.into(), 0.into(), 0.into(), zh.into(), x.into(), y.into()],
                ),
                Operation::new("Do", vec![Object::Name(b"Im1".to_vec())]),
                Operation::new("Q", vec![]),
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), SCHRIFT.into()]),
                Operation::new("rg", vec![0.35.into(), 0.35.into(), 0.35.into()]),
                Operation::new("Td", vec![RAND.into(), (FUSS / 2.0).into()]),
                Operation::new("Tj", vec![pdf_string(winansi(&fuss))]),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode()?));
        let resources = dictionary! {
            "XObject" => dictionary! { "Im1" => bild_id },
            "Font" => dictionary! { "F1" => font_id },
        };
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), sb.into(), sh.into()],
            "Contents" => content_id,
            "Resources" => resources,
        });
        kids.push(Object::Reference(page_id));
        if (i + 1) % 50 == 0 {
            eprintln!("{} von {}", i + 1, anzahl);
        }
    }

    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => anzahl as i64,
        }),
    );
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc.compress();
    doc.save(&o.out)
        .map_err(|e| anyhow!("PDF {} schreiben: {}", o.out.display(), e))?;

    let mb = fs::metadata(&o.out)?.len() as f64 / 1_048_576.0;
    println!("{} geschrieben – {} Seiten, {:.1} MB.", o.out.display(), anzahl, mb);
    Ok(())
}
