// Neutrale Fassung der Rechtsschrift.
//
// build.rs kopiert diese Datei nach src/stellungnahme_inhalt.rs, wenn dort
// noch keine liegt. Die echte Fassung ist in .gitignore ausgeschlossen, weil
// sie Namen, Adressen, Liegenschaft und Verfahren nennt. Hier steht dasselbe
// Gerüst ohne diese Angaben.

const KURZTITEL: &str = "Stellungnahme";

const KOPF: Kopf = Kopf {
    absender: &["Vorname Name", "Strasse Nr.", "PLZ Ort"],
    adressat: &["Behörde", "Abteilung", "Strasse Nr.", "PLZ Ort"],
    ort_datum: "Ort, Datum",
    betreff: "Stellungnahme im Rahmen des rechtlichen Gehörs\n\
              Verfügungsentwurf vom — \n\
              Objekt —",
    anrede: "Sehr geehrte Damen und Herren",
};

const EINLEITUNG: &str = "\
Innert der uns angesetzten Frist nehmen wir wie folgt Stellung.

Frühere Eingaben bleiben aufrecht; die vorliegende ergänzt sie.";

const ANTRAEGE: &[&str] = &[
    "Der erste Antrag - was die Behörde tun oder unterlassen soll.",
    "Ein Eventualantrag für den Fall, dass dem ersten nicht gefolgt wird.",
];

const ABSCHNITTE: &[Abschnitt] = &[
    Abschnitt {
        titel: "Zum Verfahren",
        text: "",
        unterpunkte: &[(
            "Ein Unterpunkt",
            "Was verfahrensrechtlich zu beanstanden ist. Absätze werden durch \
             eine Leerzeile getrennt.",
        )],
        bilder: &[],
    },
    Abschnitt {
        titel: "Zur Sache",
        text: "Die materielle Begründung. Die Nummerierung der Abschnitte \
               vergibt der Satz, damit sie beim Einschieben nicht von Hand \
               nachgezogen werden muss.",
        unterpunkte: &[],
        bilder: &[("Bild-01.jpg", "Legende zum Beleg.")],
    },
];

const SCHLUSS: &str = "\
Wir ersuchen um Berücksichtigung dieser Stellungnahme vor Erlass der \
definitiven Verfügung.";

const GRUSS: &str = "Mit freundlichen Grüssen";

const UNTERSCHRIFTEN: &[&str] = &["Vorname Name"];

const BEILAGEN: &[&str] = &["Erste Beilage"];
