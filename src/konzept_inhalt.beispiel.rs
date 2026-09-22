// Neutrale Fassung des Konzeptpapiers.
//
// build.rs kopiert diese Datei nach src/konzept_inhalt.rs, wenn dort noch
// keine liegt. Die echte Fassung ist in .gitignore ausgeschlossen, weil sie
// Familienmitglieder, Liegenschaften und Vermögenswerte nennt.

const KOPF: Kopf = Kopf {
    titel: "Konzeptpapier",
    untertitel: "Untertitel",
    einleitung: "Wozu das Papier dient und was es nicht ist.",
    kopfzeile: "Konzeptpapier · vertraulich",
};

const BLOECKE: &[Block] = &[
    Block::H2("1. Ziel"),
    Block::P("Ein Absatz mit **fettem** Lauf.\n\nEin zweiter Absatz."),
    Block::H3("1.1 Zwischentitel"),
    Block::Liste(&["Erster Punkt.", "Zweiter Punkt mit **Hervorhebung**."]),
    Block::Tabelle {
        breiten: &[3, 2, 1],
        kopf: &["Objekt", "Ort", "Wert"],
        zeilen: &[&["Beispiel", "Ort", "1'000"], &["Total", "", "1'000"]],
        rechts: &[2],
    },
    Block::Hinweis("**Empfehlung:** Ein hervorgehobener Kasten."),
    Block::Umbruch,
    Block::H2("2. Offene Fragen"),
    Block::Liste(&["Frage eins?", "Frage zwei?"]),
];
