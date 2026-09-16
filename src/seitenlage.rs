// Seitenlage: in welcher Drehung liegt ein fotografiertes Blatt?
//
// Wer ein Baudossier fotografiert, legt die Blätter, wie sie kommen: quer,
// kopfstehend, halb aus dem Ordner. Für ein lesbares PDF muss jede Aufnahme
// aufrecht stehen. Dieses Programm bestimmt je Foto die nötige Drehung und
// schreibt «dateiname lage» je Zeile – die Lagedatei für `aktendossier
// --orient`.
//
//   cargo run --release --bin seitenlage -- FOTO... > lagen.txt
//
// Erkennung über Apples Vision-Framework (objc2-Bindings), also nur auf
// macOS. Vision liest Text in jeder Drehung und liefert die Ecken jeder
// erkannten Zeile in normierten Bildkoordinaten. Der Vektor von der linken
// zur rechten oberen Ecke zeigt die Leserichtung; sein Winkel sagt, wie das
// Blatt liegt: 0° aufrecht (up), 90° Kopf nach links, im Uhrzeigersinn zu
// drehen (right), 180° kopfstehend (down), 270° (left). Die Zeichen jeder
// Zeile gewichten die Stimme.
//
// Was nicht geht: die Zeichenzahl je Drehung zu vergleichen. Vision
// erkennt kopfstehenden und seitlichen Text praktisch gleich gut, und
// dieser Ansatz hat jede vierte Seite falsch gelegt. Seiten fast ohne Text
// (Pläne, Fotos, Couverts) bleiben «up» und tragen die Zeichenzahl 0; sie
// sind von Hand zu prüfen.
//
// Die EXIF-Ausrichtung des Fotos wird Vision mitgegeben, damit die Lage
// relativ zum Bild gilt, wie es ein Betrachter zeigt; `aktendossier`
// wendet zuerst EXIF und dann diese Lage an.

use std::env;
use std::path::Path;
use std::process::ExitCode;

use objc2::rc::Retained;
use objc2::AnyThread;
use objc2_core_foundation::{CFDictionary, CFNumber, CFString, CFURL};
use objc2_foundation::{NSArray, NSDictionary, NSString};
use objc2_image_io::{kCGImagePropertyOrientation, CGImagePropertyOrientation, CGImageSource};
use objc2_vision::{
    VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
};

const LAGEN: [&str; 4] = ["up", "right", "down", "left"];

/// EXIF-Ausrichtung als Vision-Orientierung; 1 = up, wenn keine gesetzt.
fn exif_orientierung(quelle: &CGImageSource) -> CGImagePropertyOrientation {
    let wert = unsafe { quelle.properties_at_index(0, None) }
        .and_then(|d: objc2_core_foundation::CFRetained<CFDictionary>| {
            let d: &CFDictionary<CFString, CFNumber> = unsafe { d.cast_unchecked() };
            d.get(unsafe { kCGImagePropertyOrientation })
        })
        .and_then(|n| n.as_i32())
        .unwrap_or(1);
    match wert {
        2 => CGImagePropertyOrientation::UpMirrored,
        3 => CGImagePropertyOrientation::Down,
        4 => CGImagePropertyOrientation::DownMirrored,
        5 => CGImagePropertyOrientation::LeftMirrored,
        6 => CGImagePropertyOrientation::Right,
        7 => CGImagePropertyOrientation::RightMirrored,
        8 => CGImagePropertyOrientation::Left,
        _ => CGImagePropertyOrientation::Up,
    }
}

/// Bestimmt Lage, Zeichenzahl und die vier Stimmen für ein Foto.
fn lage(pfad: &Path) -> Option<(usize, usize, [usize; 4])> {
    let url = CFURL::from_file_path(pfad)?;
    let quelle = unsafe { CGImageSource::with_url(&url, None) }?;
    let bild = unsafe { quelle.image_at_index(0, None) }?;
    let orientierung = exif_orientierung(&quelle);

    let request = VNRecognizeTextRequest::new();
    request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
    request.setUsesLanguageCorrection(true);
    let sprachen = NSArray::from_retained_slice(&[NSString::from_str("de-DE")]);
    request.setRecognitionLanguages(&sprachen);

    let handler = unsafe {
        VNImageRequestHandler::initWithCGImage_orientation_options(
            VNImageRequestHandler::alloc(),
            &bild,
            orientierung,
            &NSDictionary::new(),
        )
    };
    let requests: Retained<NSArray<VNRequest>> =
        NSArray::from_retained_slice(&[Retained::into_super(Retained::into_super(request.clone()))]);
    if handler.performRequests_error(&requests).is_err() {
        return None;
    }

    let mut stimmen = [0usize; 4];
    let mut zeichen = 0usize;
    if let Some(ergebnisse) = request.results() {
        for beobachtung in ergebnisse.iter() {
            let kandidaten = beobachtung.topCandidates(1);
            let Some(bester) = kandidaten.iter().next() else { continue };
            if bester.confidence() < 0.5 {
                continue;
            }
            let n = bester.string().len();
            let (tl, tr) = unsafe { (beobachtung.topLeft(), beobachtung.topRight()) };
            let winkel = (tr.y - tl.y).atan2(tr.x - tl.x).to_degrees();
            let k = (((winkel + 360.0 + 45.0) % 360.0) / 90.0) as usize % 4;
            stimmen[k] += n;
            zeichen += n;
        }
    }
    // Bei Gleichstand (etwa keinerlei Text) gilt «up»: die erste Lage gewinnt.
    let beste = (0..4).fold(0, |b, k| if stimmen[k] > stimmen[b] { k } else { b });
    Some((beste, zeichen, stimmen))
}

fn main() -> ExitCode {
    let pfade: Vec<String> = env::args().skip(1).collect();
    if pfade.is_empty() {
        eprintln!("Verwendung: seitenlage FOTO... > lagen.txt");
        return ExitCode::FAILURE;
    }
    let mut fehler = 0;
    for p in &pfade {
        let pfad = Path::new(p);
        let name = pfad.file_name().and_then(|n| n.to_str()).unwrap_or(p);
        match lage(pfad) {
            Some((k, zeichen, stimmen)) => {
                println!("{} {} {} {:?}", name, LAGEN[k], zeichen, stimmen)
            }
            None => {
                eprintln!("{}: nicht lesbar", name);
                println!("{} up 0 [0, 0, 0, 0]", name);
                fehler += 1;
            }
        }
    }
    if fehler > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
