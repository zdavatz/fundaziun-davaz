// Neutrale Fassung des Bildinventars.
//
// build.rs kopiert diese Datei nach src/inventar_inhalt.rs, wenn dort noch
// keine liegt. Die echte Fassung ist in .gitignore ausgeschlossen, weil sie
// Liegenschaft, Adresse, Verfahrensnummern und die Bauvorgänge am Gebäude
// beim Namen nennt. Hier steht dasselbe Gerüst ohne diese Angaben - das
// Programm baut und läuft damit, auch wenn es jemand ohne die privaten Daten
// übersetzt.

const OBJEKT: Objekt = Objekt {
    titel: "Bildinventar zur Fotodokumentation\neines behördlichen Augenscheins",
    untertitel: "Beilage zur Stellungnahme",
    angaben: &[
        "Gebäude Nr. —",
        "Liegenschaft Nr. —, Ort",
        "Verfahrens-Nr. —",
        "Bauamt",
    ],
    stand: "Stand: —",
};

const VORBEMERKUNG: &str = "\
Führt eine Behörde einen Augenschein durch und stellt die Aufnahmen ohne \
Nummerierung, ohne Beschriftung und ohne Angabe dessen zu, was sie daran \
beanstandet, so lässt sich zu ihrem Vorhalt nicht Stellung nehmen - man \
kennt ihn nicht.

Zu einem Vorhalt kann sich nur äussern, wer weiss, worin er besteht. Wo die \
Behörde ihn nicht bezeichnet, nummeriert und beschreibt die Eigentümerschaft \
die Dokumentation selbst und ordnet jeder Aufnahme ihre Angaben zu.

Die Beschreibung gibt wieder, was auf der Aufnahme sichtbar ist; sie enthält \
keine rechtliche Würdigung.";

const VORWURF_EINLEITUNG: &str = "\
Was die Behörde beanstandet, steht in ihrem Verfügungsentwurf. Zu jedem Punkt \
ist angegeben, welche Aufnahmen ihn nach dem Sichtbaren betreffen können.";

const VORWUERFE: &[Vorwurf] = &[Vorwurf {
    schluessel: "lit. a",
    fundstelle: "Ziff. — lit. a – Beispielvorwurf",
    text: "Was die Behörde an dieser Stelle verlangt.",
    bilder: "Aufnahmen: 1.",
}];

const ANGABEN: &[Angabe] = &[Angabe {
    schluessel: "A1",
    titel: "Beispielangabe",
    text: "Hier steht, was die Eigentümerschaft zu einem Bauteil erklärt: seit \
           wann der Zustand besteht und worauf er beruht.",
    bilder: "Betrifft Bild 1.",
}];

const GRUPPEN: &[Gruppe] = &[Gruppe {
    titel: "Beispielgruppe",
    einleitung: "Aufnahmen, die denselben Gegenstand betreffen, stehen \
                 zusammen.",
    bilder: &[Bild {
        nr: 1,
        seite: "Seite 1",
        datei: "Bild-01.jpg",
        beschreibung: "Was auf der Aufnahme sichtbar ist.",
        beanstandet: "lit. a",
        grundlage: "A1",
    }],
}];

const FESTSTELLUNGEN: &[(&str, &str)] = &[(
    "Feststellung zur Dokumentation",
    "Was sich aus der Dokumentation als Ganzem ergibt - etwa welche Aufnahmen \
     unveränderten Bestand zeigen, welche einen anderen Gegenstand betreffen \
     und ob die Aufnahmen datiert sind.",
)];
