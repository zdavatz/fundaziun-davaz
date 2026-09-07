// Neutrale Fassung der Bewilligungslage.
//
// build.rs kopiert diese Datei nach src/bewilligung_inhalt.rs, wenn dort
// noch keine liegt. Die echte Fassung ist in .gitignore ausgeschlossen, weil
// sie Liegenschaft, Verfahrensnummern, Namen und die Entscheide im Wortlaut
// nennt. Hier steht dasselbe Gerüst ohne diese Angaben.

const OBJEKT: Objekt = Objekt {
    titel: "Bewilligungslage",
    untertitel: "Übersicht aus den Akten",
    angaben: &["Liegenschaft Nr. —, Gebäude Nr. —, Gemeinde"],
    stand: "Stand: —",
};

const VERFAHREN: &[Verfahren] = &[Verfahren {
    titel: "1. Beispielverfahren",
    einleitung: "Wann das Gesuch eingereicht und publiziert wurde.",
    kopf: ["", "Bauvorhaben", "Kanton", "Gemeinde", "Grundlage"],
    zeilen: &[["a", "Beispiel", "bewilligt", "bewilligt", "Art. — RPG"]],
    wuerdigung: "Was aus dem Entscheid folgt.",
}];

const ZUSTELLUNG_TITEL: &str = "Zustellung und Fristen";
const ZUSTELLUNG_EINLEITUNG: &str = "Wer wem was eröffnet hat.";
const SCHRITTE: &[Schritt] = &[Schritt {
    datum: "—",
    vorgang: "Beispielvorgang",
    beleg: "Beispielbeleg",
}];
const ZUSTELLUNG_FOLGE: &str = "Was daraus folgt.";

const NORM_TITEL: &str = "Die massgebende Norm";
const NORM_EINLEITUNG: &str = "Warum sie hier entscheidend ist.";
const NORM_TEXT: &str = "Wortlaut der Bestimmung.";
const NORM_FOLGERUNG: &str = "Was daraus für den Fall folgt.";

const VORGESCHICHTE_TITEL: &str = "Vorgeschichte";
const VORGESCHICHTE: &str = "Was vor dem Verfahren war.";

const SCHLUSS_TITEL: &str = "Was jetzt läuft";
const SCHLUSS: &str = "Der Stand des Verfahrens.";
const QUELLEN: &str = "Quellen: —";

const BELEGE: &[Beleg] = &[];
