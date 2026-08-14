// src/befunde.rs trägt die Folgerungen mit den konkreten Angaben der
// Stiftung - Anfangskapital, Lohn, Familienregelung - und ist deshalb in
// .gitignore ausgeschlossen. Damit ein frischer Klon trotzdem baut, wird
// hier die neutrale Fassung ausgelegt, sofern noch keine Datei da ist.
// Eine vorhandene Datei wird nie überschrieben.
use std::path::Path;

fn main() {
    let ziel = Path::new("src/befunde.rs");
    if !ziel.exists() {
        std::fs::copy("src/befunde.beispiel.rs", ziel)
            .expect("neutrale Fassung der Befunde konnte nicht ausgelegt werden");
        println!(
            "cargo:warning=src/befunde.rs fehlte - neutrale Fassung aus \
             src/befunde.beispiel.rs ausgelegt."
        );
    }
    println!("cargo:rerun-if-changed=src/befunde.rs");
    println!("cargo:rerun-if-changed=src/befunde.beispiel.rs");
}
