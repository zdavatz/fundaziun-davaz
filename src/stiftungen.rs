// Recherchebericht zu Schweizer Stiftungen als PDF - zwei Stränge, wie die
// FUNDAZIUN DA VAZ - VAL MÜSTAIR selbst zwei hat:
//
//   Teil I   Kunststiftungen (der Strang von Jürg Davatz)
//   Teil II  Stiftungen für psychische Gesundheit und ADHS (der Strang von
//            Dr. med. Ursula Davatz)
//
// Beide Teile beantworten dieselben vier Fragen - Trägerschaft, Führung,
// Finanzierung des Unterhalts, Anmeldung - und münden in die Folgerungen für
// unsere Stiftungsurkunde.
//
// Pure Rust, kein Chrome: das PDF entsteht direkt mit `genpdf` (das über
// printpdf schreibt) und bettet die DejaVu-Sans-Familie ein - dieselbe
// Pipeline wie in ~/software/listingtracker.
//
//   cargo run --release --bin stiftungen
//   cargo run --release --bin stiftungen -- --out /pfad/zum.pdf
//
// Schriftverzeichnis überschreibbar via $FONT_DIR (Vorgabe: ./fonts).

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use genpdf::elements::{Break, PageBreak, Paragraph};
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Element};

const DEFAULT_FONT_DIR: &str = "fonts";
const DEFAULT_OUT: &str = "Stiftungen_Schweiz_Recherche.pdf";

const STAND: &str = "Stand: 14. August 2026";

// Zurückhaltende Palette: Tinte für den Fliesstext, Gold als einziger Akzent,
// Schiefer für Zwischentitel, Grau für Beiwerk.
const INK: Color = Color::Rgb(0x1b, 0x1b, 0x1d);
const GOLD: Color = Color::Rgb(0xa0, 0x8b, 0x6a);
const SLATE: Color = Color::Rgb(0x3a, 0x3d, 0x44);
const MUTED: Color = Color::Rgb(0x8a, 0x8d, 0x94);
const LINK: Color = Color::Rgb(0x2c, 0x5a, 0x8a);

// genpdf 0.2 kennt keine Hyperlinks. Wir setzen die URL-Zeilen als einzige
// Zeilen des Dokuments in dieser Schriftgrösse und legen nach dem Rendern
// über jede solche Zeile eine Link-Annotation (siehe `add_links`). Sobald
// irgendwo sonst im Satz diese Grösse auftaucht, bricht die Zuordnung -
// deshalb laufen Kopfzeile und Schlussnotiz bewusst auf 7 pt.
const LINK_FONT_SIZE: u8 = 8;
const A4_WIDTH_PT: f64 = 595.276;
// Seitenrand in Punkt (22 mm), Rechtsanschlag der Klickfläche.
const MARGIN_PT: f64 = 22.0 * 72.0 / 25.4;
// Mittlere Vorschubbreite von DejaVu Sans für Kleinbuchstaben, in em.
const AVG_ADVANCE_EM: f64 = 0.55;
// Längste Anzeigeform einer URL. 88 Zeichen zu 8 pt ergeben rund 387 pt und
// bleiben damit klar unter der Satzbreite von 166 mm (rund 470 pt).
const MAX_LINK_CHARS: usize = 88;

// ---------------------------------------------------------------------------
// Inhalt
// ---------------------------------------------------------------------------

/// Steckbrief einer Stiftung. Die Reihenfolge der Felder ist die Reihenfolge
/// der Fragen des Auftrags: wer betreibt sie, wer führt sie, wovon lebt sie,
/// wo ist sie angemeldet.
struct Portrait {
    name: &'static str,
    ort: &'static str,
    url: &'static str,
    kurz: &'static str,
    gegruendet: &'static str,
    traegerschaft: &'static str,
    fuehrung: &'static str,
    finanzierung: &'static str,
    angemeldet: &'static str,
}

const PORTRAITS: &[Portrait] = &[
    Portrait {
        name: "Beyeler-Stiftung / Fondation Beyeler",
        ort: "Riehen BS",
        url: "https://www.fondationbeyeler.ch/museum/beyeler-stiftung",
        kurz: "Die betrieblich grösste Kunststiftung der Schweiz und der \
               einzige Fall mit einer ausgebauten zweistufigen Struktur.",
        gegruendet: "1982 vom Sammlerehepaar Ernst und Hildy Beyeler; das \
                     Museum von Renzo Piano öffnete 1997.",
        traegerschaft: "Zweistufig. Die Beyeler-Stiftung ist Eigentümerin der \
                        Sammlung und Rechtsträgerin des Hauses; die Beyeler \
                        Museum AG führt den Betrieb. Betriebsverluste der AG \
                        werden durch Einlagen der Stiftung gedeckt.",
        fuehrung: "Direktor des Museums ist seit 2008 Sam Keller. Den \
                   Stiftungsrat präsidiert Hansjörg Wyss; ihm gehören \
                   Gottfried Boehm, Edgar Fluri, James Koch, Eric Lohrer, \
                   Georg Schmid, Gili Fridland Svensson und Michael Willi an. \
                   Den Verwaltungsrat der Museum AG präsidiert Edgar Fluri.",
        finanzierung: "Eintritte, Kunstshop, Restaurant, Dienstleistungen, \
                       Sponsoring und Art Club ergaben einen Betriebsertrag \
                       von rund CHF 17.2 Mio. (Zahl aus einer Vorlage von \
                       2016). Rund die Hälfte des Budgets stammt aus den \
                       Eintritten. Dazu kommen Beiträge der Hansjörg Wyss \
                       Foundation, Einlagen der Beyeler-Stiftung, ein \
                       Betriebsbeitrag der Gemeinde Riehen von jährlich \
                       CHF 1.126 Mio. ab Herbst 2025 samt Erlass mehrerer \
                       Baurechtszinsen, Subventionen des Kantons Basel-Stadt \
                       und Projektbeiträge des Kantons Basel-Landschaft.",
        angemeldet: "Sitz und Handelsregister Basel-Stadt. Der Grosse Rat \
                     Basel-Stadt beschliesst die Staatsbeiträge an die Beyeler \
                     Museum AG periodenweise, zuletzt für 2024 bis 2027.",
    },
    Portrait {
        name: "Stiftung Sammlung E. G. Bührle",
        ort: "Zürich",
        url: "https://buehrle.ch/",
        kurz: "Das Auslagerungsmodell: die Stiftung besitzt die Werke, den \
               Betrieb besorgt eine bestehende Institution.",
        gegruendet: "Zweck ist, die Werke der Sammlung dauerhaft ins Eigentum \
                     der Stiftung zu überführen, sie als Ganzes zu erhalten \
                     und öffentlich zugänglich zu machen. Die Stiftung \
                     verfolgt weder Erwerbs- noch Selbsthilfezweck.",
        traegerschaft: "Mit Vertrag vom 28. Mai 2012 regelten die Zürcher \
                        Kunstgesellschaft und die Stiftung die langfristige \
                        Leihgabe von rund 190 Gemälden und Skulpturen an das \
                        Kunsthaus Zürich. Personelle Verschränkung in beide \
                        Richtungen: der Kunsthaus-Direktor sass im \
                        Stiftungsrat, der Stiftungsdirektor im Vorstand der \
                        Kunstgesellschaft.",
        fuehrung: "Lukas Gloor führte die Stiftung von 2002 bis Ende 2021. Er \
                   trat im November 2021 mitten in der Herkunftsdebatte \
                   zurück, mit der Begründung, seine Aufgabe sei mit dem \
                   Einzug der Werke ins Kunsthaus erfüllt.",
        finanzierung: "Die Kosten für Unterhalt, Erhaltung und Vermittlung \
                       sind im Businessplan der Kunstgesellschaft budgetiert, \
                       nicht bei der Stiftung. Die Stifterfamilie leistete \
                       zudem einen namhaften Beitrag an die Baukosten der \
                       Kunsthaus-Erweiterung.",
        angemeldet: "Sitz und Handelsregister Zürich.",
    },
    Portrait {
        name: "Fondation Pierre Gianadda",
        ort: "Martigny VS",
        url: "https://www.gianadda.ch/fondation/organisation/",
        kurz: "Der nächstliegende Vergleichsfall: von einer Familie gegründet, \
               nach dem Tod des Gründers von der Familie weitergeführt.",
        gegruendet: "1978 von Léonard Gianadda zum Andenken an seinen 1976 bei \
                     einem Flugzeugabsturz umgekommenen Bruder Pierre.",
        traegerschaft: "Ein Ensemble aus Kunstmuseum, Skulpturenpark, \
                        Automobilmuseum sowie Räumen für Geschichte und \
                        Archäologie. Seit der Eröffnung über 11 Millionen \
                        Besucherinnen und Besucher.",
        fuehrung: "Der Stiftungsrat zählt neun bis fünfzehn Mitglieder und \
                   setzt das jährliche Betriebsbudget fest, vorbehältlich der \
                   Genehmigung durch die Geldgeber. Léonard Gianadda starb \
                   Ende 2023; seit dem 22. Mai 2024 präsidiert François \
                   Gianadda den Stiftungsrat.",
        finanzierung: "Getragen von Eintritten und einem dichten \
                       Ausstellungs- und Konzertbetrieb, ergänzt durch \
                       Leihgaben aus bedeutenden Privatsammlungen und \
                       Museen. Zur genauen Aufteilung zwischen Eigenertrag \
                       und Beiträgen liegen öffentlich keine belastbaren \
                       Zahlen vor.",
        angemeldet: "Sitz und Handelsregister Wallis.",
    },
    Portrait {
        name: "Stiftung Rosengart",
        ort: "Luzern",
        url: "https://www.rosengart.ch/de/Museum/Angela-Rosengart-und-die-Stiftung",
        kurz: "Der Fall einer Stifterin, die die Nachfolge zu Lebzeiten \
               geordnet hat - und deren Tod die Stiftung soeben überstanden \
               hat.",
        gegruendet: "1992 von Angela Rosengart, die ihre gesamte private \
                     Sammlung einbrachte. Der Kunsthändler Siegfried \
                     Rosengart hatte sie mit seiner Tochter aufgebaut: weit \
                     über 300 Werke der klassischen Moderne.",
        traegerschaft: "Die Stiftung ist für Erhaltung und Betrieb der \
                        Sammlung zuständig. Das Museum eröffnete 2002 im \
                        ehemaligen Gebäude der Schweizerischen Nationalbank.",
        fuehrung: "Angela Rosengart präsidierte die Stiftung selbst; \
                   Vizepräsident war Max Galliker, im Rat sass unter anderem \
                   Stefan Sägesser, Leiter Kulturförderung des Kantons \
                   Luzern. Am 30. November 2018 wurden vier neue Mitglieder \
                   gewählt - die Verbreiterung geschah zu ihren Lebzeiten. \
                   Angela Rosengart starb am 30. Juni 2026.",
        finanzierung: "Getragen vom eingebrachten Sammlungsvermögen und vom \
                       Museumsbetrieb. Die Stiftungskonstruktion stellt \
                       sicher, dass die Sammlung auch nach dem Tod der \
                       Stifterin öffentlich zugänglich bleibt.",
        angemeldet: "Sitz und Handelsregister Luzern.",
    },
    Portrait {
        name: "Fondation Gandur pour l'Art",
        ort: "Genf",
        url: "https://www.fg-art.org/",
        kurz: "Sammlungsstiftung ohne eigenes Haus - die Werke arbeiten in \
               fremden Museen.",
        gegruendet: "2010 von Jean Claude Gandur, um seine Sammlungen der \
                     Öffentlichkeit zugänglich zu machen. Als gemeinnützig \
                     anerkannt.",
        traegerschaft: "Vier Sammlungsbereiche: Antiken mit über 1'200 \
                        Objekten, Ethnologie, Beaux-Arts mit rund 1'000 \
                        Werken des 20. Jahrhunderts und Kunstgewerbe mit rund \
                        400 Objekten.",
        fuehrung: "Stiftungsrat mit Sitz in Genf; die Stiftung bewahrt, \
                   dokumentiert, erweitert und zeigt ihre Sammlungen in \
                   anerkannten Institutionen im In- und Ausland.",
        finanzierung: "Kein eigener Museumsbetrieb, deshalb keine \
                       Eintrittserträge. Die Stiftung unterstützt umgekehrt \
                       Schweizer Kulturinstitutionen finanziell, organisiert \
                       internationale Ausstellungen und gibt ihre Werke als \
                       Langzeitleihgaben ab. Ein Mäzenatsprogramm finanziert \
                       Restaurierungs- und Erhaltungsprojekte.",
        angemeldet: "Sitz und Handelsregister Genf.",
    },
    Portrait {
        name: "Fundaziun Muzeum Susch / Art Stations Foundation CH",
        ort: "Susch GR",
        url: "https://muzeumsusch.ch/",
        kurz: "Die geografisch nächste Vergleichsstiftung - gleicher Kanton, \
               gleiche Ausgangslage aus historischer Bausubstanz.",
        gegruendet: "Eröffnet 2019. Gegründet und finanziert von Grażyna \
                     Kulczyk, polnische Unternehmerin und Mäzenin.",
        traegerschaft: "Grundlage ist eine seit langem leerstehende \
                        historische Brauerei aus dem 19. Jahrhundert. Den \
                        Umbau besorgten Chasper Schmidlin, der im Tal \
                        aufwuchs, und Lukas Voellmy.",
        fuehrung: "Trägerin ist die Stiftung der Gründerin; zur operativen \
                   Leitung liegen öffentlich keine gesicherten Angaben vor.",
        finanzierung: "Privat finanziert durch die Gründerin. Der Betrieb \
                       stützt sich nicht auf öffentliche Beiträge - was den \
                       Fall für uns interessant, aber auch fragil macht: er \
                       hängt an einer Person.",
        angemeldet: "Sitz im Kanton Graubünden - derselbe Handelsregister- \
                     und Aufsichtskreis, in dem auch wir uns anmelden.",
    },
    Portrait {
        name: "Ernst Ludwig Kirchner Stiftung Davos",
        ort: "Davos GR",
        url: "https://kirchnermuseum.ch/",
        kurz: "Zeigt, was ein Ausbau kostet und wie schnell öffentliches Geld \
               an der Urne scheitert.",
        gegruendet: "1. Juli 1982. Zweck: Förderung der Erinnerung an Ernst \
                     Ludwig Kirchner und Erhaltung seines Werks, \
                     einschliesslich Errichtung und Betrieb eines Kirchner \
                     Museums in der Landschaft Davos.",
        traegerschaft: "Die Stiftung ist Eigentümerin der Sammlung. Daneben \
                        besteht der Kirchner Verein, der das Museum in \
                        Sammeln, Bewahren, Forschen und Vermitteln \
                        unterstützt.",
        fuehrung: "Stiftungsrat der Ernst Ludwig Kirchner Stiftung Davos, in \
                   vertraglich geregelter Zusammenarbeit mit dem Verein.",
        finanzierung: "Gemischt aus Stiftungsmitteln, privatem Sponsoring und \
                       öffentlichen Beiträgen. Beim Erweiterungsbau von \
                       veranschlagt CHF 11.5 Mio. sagten Stiftung, private \
                       Sponsoren und das World Economic Forum zusammen 7 Mio. \
                       zu. Den Gemeindebeitrag von 4 Mio. lehnten die Davoser \
                       Stimmberechtigten im November 2024 ab.",
        angemeldet: "Sitz und Handelsregister Graubünden.",
    },
    Portrait {
        name: "Stiftung Langmatt",
        ort: "Baden AG",
        url: "https://www.langmatt.ch/",
        kurz: "Der Präzedenzfall, den wir am genauesten lesen sollten: \
               Liegenschaft plus Sammlung, Stiftungskapital aufgezehrt.",
        gegruendet: "Trägerin der Villa Langmatt und der Sammlung des \
                     Ehepaars Sidney und Jenny Brown.",
        traegerschaft: "Museum in der eigenen Villa - dieselbe Verbindung von \
                        denkmalwürdiger Bausubstanz und Kunstsammlung, die \
                        auch unsere Stiftung eingeht.",
        fuehrung: "Stiftungsrat der Stiftung Langmatt; der Sanierungs- und \
                   Verkaufsentscheid wurde öffentlich als \"Tabubruch mit \
                   Ansage\" verhandelt.",
        finanzierung: "Sanierungsbedürftig war nicht nur die Villa, sondern \
                       auch das Kapital der Stiftung. Im November 2023 \
                       versteigerte das Museum drei Gemälde von Paul Cézanne \
                       bei Christie's in New York für zusammen rund \
                       CHF 40.5 Mio., einzig um das Stiftungskapital \
                       wiederherzustellen, aus dessen Erträgen der Betrieb \
                       finanziert wird. Die Gesamtsanierung des Hauses kostet \
                       CHF 18.8 Mio.; die Stadt Baden sprach zusätzlich \
                       CHF 10 Mio. Baubeginn Frühjahr 2024, Wiedereröffnung \
                       2026.",
        angemeldet: "Sitz und Handelsregister Aargau; die städtischen Beiträge \
                     gingen durch eine Volksabstimmung in Baden.",
    },
    Portrait {
        name: "Fundaziun Chastè da Tarasp",
        ort: "Tarasp GR",
        url: "https://www.schloss-tarasp.ch/",
        kurz: "Der einzige Bündner Fall und die umgekehrte Bauart: die \
               Liegenschaft bleibt in privater Hand, die Stiftung betreibt - \
               und beide Rollen hält dieselbe Person.",
        gegruendet: "Am 1. November 2010 von der Gemeinde Tarasp errichtet, \
                     zunächst mit dem Ziel, das Schloss selbst zu erwerben. \
                     Dieser Erwerb scheiterte. Am 30. März 2016 kaufte der \
                     Künstler Not Vital das Schloss für CHF 7.9 Mio.; Prinz \
                     Philipp von Hessen übergab ihm den Schlüssel.",
        traegerschaft: "Eigentümer ist Not Vital persönlich, nicht die \
                        Stiftung. Die Fundaziun führt den Kulturbetrieb in \
                        einem Haus, das ihm gehört, und zeigt darin unter \
                        anderem sein eigenes Werk. Eine spätere Überführung \
                        des Schlosses in eine Stiftung ist angekündigt; ob sie \
                        erfolgt ist, liess sich aus öffentlichen Quellen nicht \
                        feststellen.",
        fuehrung: "Personalunion in drei Rollen: Not Vital ist Eigentümer \
                   des Schlosses, Präsident des Stiftungsrats und der \
                   ausgestellte Künstler zugleich. Dem Rat gehören ferner \
                   Andri Riatsch als Vizepräsident sowie Giorgio Cappellin und \
                   Annatina Miescher an; Revisionsstelle ist die RBT AG. Das \
                   Schloss ist nur mit Führung zugänglich, die Anmeldung läuft \
                   über notvital.com. Genau diese Häufung - Stiftung erhält \
                   und bespielt eine Liegenschaft, deren Eigentümer sie \
                   präsidiert und dessen Werk sie zeigt - ist der Punkt, den \
                   eine Aufsichtsbehörde zuerst prüft: Interessenkonflikt, \
                   Selbstkontrahierung, Ausstand. Für uns folgt daraus \
                   dreierlei: eine Ausstandsregelung für Geschäfte zwischen \
                   Stiftung und Stifter, eine nicht der Familie angehörende \
                   Mehrheit im Stiftungsrat, damit der Ausstand überhaupt \
                   wirkt, und eine ausdrückliche Regel, ob und wieviel die \
                   Stiftung für die Nutzung der Liegenschaft bezahlt.",
        finanzierung: "Eintritte aus den Führungen - Erwachsene CHF 10, Kinder \
                       CHF 5 - tragen den Betrieb nicht allein. Der Erwerb \
                       wurde privat finanziert; die Stiftung war dazu nicht in \
                       der Lage, was ihren gescheiterten Kaufversuch erklärt. \
                       Der Geldfluss geht damit vom Stifter zur Stiftung, \
                       nicht umgekehrt: eine Entschädigung an ihn ist in \
                       keiner öffentlichen Quelle erwähnt. Belegen lässt sich \
                       das nicht, denn die Jahresrechnung ist nicht \
                       öffentlich.",
        angemeldet: "Sitz Tarasp, Handelsregister Graubünden. \
                     Aufsichtsbehörde ist die Stiftungsaufsicht des Kantons \
                     Graubünden, weil der Zweck ortsgebunden ist und im \
                     Sitzkanton erfüllt wird - dieselbe Lage wie bei uns.",
    },
];

/// Zweiter Strang: Stiftungen, die sich mit psychischer Gesundheit befassen,
/// namentlich mit ADHS und Neurodiversität. Gleiche Struktur wie oben, damit
/// die beiden Teile nebeneinander lesbar bleiben.
const PSYCHE: &[Portrait] = &[
    Portrait {
        name: "Gehirn- und Traumastiftung Graubünden/Schweiz",
        ort: "Chur GR",
        url: "https://gtsg.ch/",
        kurz: "Der wichtigste Fund dieser Recherche: eine ADHS-Stiftung im \
               eigenen Kanton, deren Zweck sich mit unserem zweiten Strang \
               fast deckt.",
        gegruendet: "Besteht seit 2006, mit Sitz an der Poststrasse 22 in \
                     Chur.",
        traegerschaft: "Zweck sind die Unterstützung von Menschen mit \
                        Behinderung bei der beruflichen und schulischen \
                        Integration, neurobiologische Forschung und die \
                        Aufklärung der Öffentlichkeit über neurobiologische \
                        Erkenntnisse. Erklärtes Ziel ist \
                        anwendungsorientierte Forschung, die den Menschen \
                        unmittelbar zugutekommt.",
        fuehrung: "Präsident des Stiftungsrats ist Dr. iur. Giusep Nay, alt \
                   Bundesrichter. Geschäftsführer ist Dr. phil. Andreas \
                   Müller, Psychotherapeut. Im Stiftungsrat sitzen zudem \
                   Prof. Dr. Pius Baschera, Prof. Dr. Theodor Leuenberger, \
                   lic. phil. Paul Ruschetti und Dr. med. Eric Thomann.",
        finanzierung: "Projektfinanzierung durch fördernde Stiftungen. Das \
                       Projekt \"Personalisierte Medizin bei ADHS\" lief von \
                       2014 bis 2020 mit CHF 570'000 der Hirschmann Stiftung, \
                       zusammen mit der Psychiatrischen Universitätsklinik \
                       Zürich. Ziel war die Objektivierung von Diagnose und \
                       Therapie über Biomarker, um Fehlbehandlungen zu \
                       verringern.",
        angemeldet: "Handelsregister Graubünden; Mitglied der Academia \
                     Raetica. Damit dieselbe Aufsichtsbehörde, mit der wir es \
                     zu tun haben.",
    },
    Portrait {
        name: "Schweizerische Stiftung Pro Mente Sana",
        ort: "Zürich",
        url: "https://promentesana.ch/ueber-uns",
        kurz: "Die nationale Fachorganisation für psychische Gesundheit - und \
               ein Beleg dafür, dass Grösse und Alter vor Schieflage nicht \
               schützen.",
        gegruendet: "1978 gegründet. Sie vertritt Interessen und Rechte von \
                     Menschen mit psychischen Beeinträchtigungen, engagiert \
                     sich in der Früherkennung und bietet Dienstleistungen im \
                     Bereich der psychischen Gesundheit an.",
        traegerschaft: "Klassische Fachorganisation mit Beratung, \
                        Interessenvertretung in Politik und Gesellschaft und \
                        Öffentlichkeitsarbeit.",
        fuehrung: "Muriel Langenberger übernahm Anfang September 2022 die \
                   Geschäftsleitung. Wenige Monate danach wurde mit externer \
                   Unterstützung eine Sanierung der Stiftung eingeleitet, die \
                   seit Jahren unter finanziellem Druck stand. Seit Januar \
                   2026 präsidiert Matthias Jäger den Stiftungsrat.",
        finanzierung: "Öffentliche Beiträge von Bund, Kantonen und Gemeinden, \
                       Spenden von Privatpersonen und Unternehmen sowie \
                       Erträge aus dem Stiftungskapital und aus den eigenen \
                       Dienstleistungen.",
        angemeldet: "Sitz und Handelsregister Zürich; national tätig.",
    },
    Portrait {
        name: "Gesundheitsförderung Schweiz",
        ort: "Bern und Lausanne",
        url: "https://gesundheitsfoerderung.ch/stiftung",
        kurz: "Die finanziell stabilste Konstruktion im Feld - weil ihr Geld \
               nicht aus Spenden kommt, sondern aus einem gesetzlichen \
               Prämienzuschlag.",
        gegruendet: "Privatrechtliche Stiftung mit gesetzlichem Auftrag, \
                     getragen von den Kantonen und den Versicherern. Sie \
                     initiiert, koordiniert und evaluiert Massnahmen zur \
                     Förderung der Gesundheit und zur Verhütung von \
                     Krankheiten.",
        traegerschaft: "Trägerschaft aus Kantonen und Krankenversicherern - \
                        eine Konstruktion zwischen Privatrecht und \
                        öffentlichem Auftrag, die es im Kulturbereich so \
                        nicht gibt.",
        fuehrung: "Stiftungsrat aus Vertretungen der Trägerorganisationen; \
                   die Mittelverwendung wird dem Parlament über einen Bericht \
                   des Eidgenössischen Departements des Innern ausgewiesen.",
        finanzierung: "Finanziert durch einen Zuschlag auf die Prämien der \
                       nach KVG versicherten Personen, festgesetzt vom EDI. \
                       Der Bundesrat erhöhte ihn in zwei Schritten von \
                       jährlich CHF 2.40 auf CHF 3.60 im Jahr 2017 und auf \
                       CHF 4.80 pro versicherte Person ab 2018. Rund 40 \
                       Prozent des Zusatzbetrags, etwa CHF 7.68 Mio., waren \
                       für die Umsetzung der Massnahmen des Berichts \
                       \"Psychische Gesundheit in der Schweiz\" bestimmt.",
        angemeldet: "Privatrechtliche Stiftung mit gesetzlicher Grundlage im \
                     KVG; Rechenschaft gegenüber dem EDI und dem Parlament.",
    },
    Portrait {
        name: "Hirschmann Stiftung",
        ort: "Zürich",
        url: "https://www.hirschmann-stiftung.ch/",
        kurz: "Die Förderstiftung, die im ADHS-Feld tatsächlich Geld \
               ausschüttet - eine mögliche Partnerin für unseren zweiten \
               Strang.",
        gegruendet: "1985 gegründet; Geschäftsstelle an der Breitingerstrasse \
                     35 in Zürich.",
        traegerschaft: "Förderstiftung ohne eigenen Betrieb. Förderschwerpunkte \
                        sind Forschungsprojekte zu Gehirn, Trauma und \
                        nachhaltigen Finanzmärkten.",
        fuehrung: "Stiftungsrat mit Geschäftsstelle; Gesuche werden \
                   projektweise beurteilt.",
        finanzierung: "Vergibt Fördermittel, statt selbst welche zu suchen. \
                       Das ADHS-Projekt der Gehirn- und Traumastiftung erhielt \
                       über sechs Jahre CHF 570'000 - eine Grössenordnung, die \
                       zeigt, was für ein Aufklärungs- und Forschungsvorhaben \
                       im ADHS-Bereich realistisch erhältlich ist.",
        angemeldet: "Schweizer gemeinnützige Stiftung; Ergebnisse der \
                     geförderten Projekte werden wissenschaftlich publiziert.",
    },
    Portrait {
        name: "Unterstiftung Neurodiversität und Lebenskunst",
        ort: "unter der Stiftung Freie Gemeinschaftsbank, Basel",
        url: "https://stiftungfgb.ch/dachstiftung/unterstiftungen/neurodiversitaetundlebenskunst",
        kurz: "Zeigt zweierlei: das Dachstiftungsmodell als Alternative zur \
               eigenen Stiftung - und eine Liegenschaft, die selbst der Zweck \
               ist.",
        gegruendet: "Unterstiftung im Dachstiftungsmodell der Stiftung Freie \
                     Gemeinschaftsbank mit Sitz in Basel, die mehrere \
                     spezialisierte Unterstiftungen führt.",
        traegerschaft: "Betreibt ein therapeutisches Rückzugszentrum für \
                        neurodivergente Menschen und Menschen mit \
                        Verhaltenssüchten: ein Zufluchtsort für jene, die von \
                        reizintensiven Umgebungen überfordert sind. Grundlage \
                        ist der Berghof Fennematt im Elsass auf 900 Metern - \
                        ein bio-optimierter Passivholzbau mit zehn Studios und \
                        sechs Zimmern, 60 Hektaren Land und bewusst \
                        strahlungsarmer Infrastruktur, ohne WLAN, nur mit \
                        LAN-Anschlüssen.",
        fuehrung: "Geführt über die Dachstiftung; die Unterstiftung braucht \
                   keine eigene Rechtspersönlichkeit, keinen eigenen \
                   Stiftungsrat und keine eigene Revisionsstelle.",
        finanzierung: "Zwei Betriebsformate - freie Studiovermietung und \
                       strukturierte Erholungsaufenthalte für 15 Personen mit \
                       täglicher Meditation, gemeinsamer Arbeit in Küche, \
                       Garten und Unterhalt sowie gemeinsamen Mahlzeiten. \
                       Dazu in der Schweiz steuerabzugsfähige Spenden, die \
                       über eine eigene Kostenstelle zugeordnet werden.",
        angemeldet: "Keine eigene Anmeldung: die Rechtsform ist die der \
                     Dachstiftung in Basel.",
    },
    Portrait {
        name: "Stiftung Rheinleben",
        ort: "Basel",
        url: "https://www.rheinleben.ch/stiftung",
        kurz: "Das Leistungsvertragsmodell - so finanziert sich die \
               Gesundheitsseite, wenn sie Leistungen erbringt statt nur \
               aufklärt.",
        gegruendet: "Gemeinnützige Stiftung; begleitet rund 2'000 Klientinnen \
                     und Klienten mit psychischer Erkrankung, deren Angehörige \
                     sowie Unternehmen im Raum Basel.",
        traegerschaft: "Anerkannte Vertragspartnerin der kantonalen und \
                        nationalen Leistungseinkäufer.",
        fuehrung: "Die Geschäftsleitung führt Martina Pongratz; daneben ein \
                   Stiftungsrat.",
        finanzierung: "Leistungsverträge mit den Kantonen, Invalidenversicherung, \
                       Pro Infirmis, Spenden und Beiträge der Klientinnen und \
                       Klienten. Getragen überwiegend vom Kanton Basel-Stadt \
                       und vom Kanton Basel-Landschaft. Einzelprojekte werden \
                       zusätzlich gesprochen - der Kanton Basel-Stadt \
                       bewilligte etwa CHF 61'000 für das Jahr 2027 für das \
                       Projekt \"Irre Normal\".",
        angemeldet: "Sitz und Handelsregister Basel-Stadt.",
    },
    Portrait {
        name: "Stiftung Pro Juventute",
        ort: "Zürich",
        url: "https://www.projuventute.ch/de/stiftung/wer-wir-sind",
        kurz: "Die spendenfinanzierte Variante: hohe Reichweite, aber die \
               öffentliche Hand trägt nur die Hälfte, auch bei einem \
               Notruftelefon.",
        gegruendet: "1912 gegründet; seit über hundert Jahren für Kinder, \
                     Jugendliche und Familien in der Schweiz und in \
                     Liechtenstein tätig.",
        traegerschaft: "Betreibt unter anderem Beratung + Hilfe 147 und die \
                        Elternberatung.",
        fuehrung: "Der Stiftungsrat bildet das strategische Zentrum, die \
                   erweiterte Geschäftsleitung führt die Organisation.",
        finanzierung: "Rund 84 Prozent aus Spenden, zweckgebundenen Erträgen \
                       aus Legaten sowie Marken- und Artikelverkauf; etwa ein \
                       Drittel der Mittel stammt aus Marken- und \
                       Artikelverkauf, sozialen Dienstleistungen, Spenden, \
                       Legaten und Sponsoring. Beim Notruf 147 deckt die \
                       öffentliche Hand rund die Hälfte der Kosten: ein \
                       Bundesbeitrag von CHF 600'000 und erwartete Kantons- \
                       und Gemeindebeiträge von rund CHF 600'000.",
        angemeldet: "Sitz und Handelsregister Zürich; die Jahresrechnung wird \
                     von einer zugelassenen Revisionsgesellschaft geprüft und \
                     veröffentlicht.",
    },
    Portrait {
        name: "elpos Schweiz - zum Vergleich, ein Verein",
        ort: "schweizweit, regionale Fachstellen",
        url: "https://elpos.ch/ueber-den-verein/",
        kurz: "Die grösste ADHS-Organisation des Landes ist keine Stiftung. \
               Das ist der Befund, auf den es ankommt.",
        gegruendet: "1974 gegründet, weil das Wissen über ADHS damals nur bei \
                     wenigen Fachleuten vorhanden war. Rechtsform: Verein.",
        traegerschaft: "Verein für Kinder, Jugendliche und Erwachsene mit ADHS \
                        sowie deren Bezugs- und Fachpersonen. Regionale \
                        Fachstellen begleiten in allen Lebensphasen - vom \
                        Verdacht über die Abklärung bis zu Fragen von Schule, \
                        Erziehung, Berufsleben und Partnerschaft.",
        fuehrung: "Vereinsvorstand mit regionalen Fachstellen; daneben \
                   bestehen der Verein ADHS/ADS Schweiz und die \
                   Schweizerische Fachgesellschaft ADHS als Fachgesellschaft.",
        finanzierung: "Mitgliederbeiträge, Dienstleistungen, Spenden und \
                       Beiträge des Bundesamts für Sozialversicherungen.",
        angemeldet: "Vereinsregistereintrag; keine Stiftungsaufsicht, weil \
                     keine Stiftung.",
    },
];

/// Folgerung für die FUNDAZIUN DA VAZ. Titel und Fliesstext.
struct Befund {
    titel: &'static str,
    text: &'static str,
}

include!("befunde.rs");

/// Quellen als (Bezeichnung, URL). Die URL wird als eigene Zeile in
/// LINK_FONT_SIZE gesetzt und nachträglich mit einer Link-Annotation belegt.
const QUELLEN_KUNST: &[(&str, &str)] = &[
    ("SwissFoundations, Stiftungsreport 2023",
     "https://www.swissfoundations.ch/aktuell/stiftungsreport-2023-preview/"),
    ("SwissFoundations, Kunst & Kultur",
     "https://www.swissfoundations.ch/themen/kunst-kultur/"),
    ("Fondation Beyeler, Beyeler-Stiftung",
     "https://www.fondationbeyeler.ch/museum/beyeler-stiftung"),
    ("Fondation Beyeler, Museum, Geschichte und Leitbild",
     "https://www.fondationbeyeler.ch/museum"),
    ("Grosser Rat Basel-Stadt, Ratschlag Beyeler Museum AG 2024-2027",
     "https://grosserrat.bs.ch/dokumente/100405/000000405724.pdf"),
    ("bz Basel, Riehen und die Fondation Beyeler - die kritischen Stimmen \
      werden lauter",
     "https://www.bzbasel.ch/basel/basel-stadt/subventionen-riehen-und-die-fondation-beyeler-die-kritischen-stimmen-werden-lauter-ld.2572228"),
    ("Stiftung Sammlung E. G. Bührle", "https://buehrle.ch/"),
    ("Wikipedia, Stiftung Sammlung E. G. Bührle",
     "https://de.wikipedia.org/wiki/Stiftung_Sammlung_E._G._B%C3%BChrle"),
    ("Tages-Anzeiger, Lukas Gloor gibt die Direktion der Bührle-Stiftung ab \
      (November 2021)",
     "https://www.tagesanzeiger.ch/lukas-gloor-gibt-die-direktion-der-buehrle-stiftung-ab-865345469623"),
    ("Fondation Pierre Gianadda, Organisation",
     "https://www.gianadda.ch/fondation/organisation/"),
    ("Wikipedia, Fondation Pierre Gianadda",
     "https://en.wikipedia.org/wiki/Fondation_Pierre_Gianadda"),
    ("Sammlung Rosengart, Angela Rosengart und die Stiftung",
     "https://www.rosengart.ch/de/Museum/Angela-Rosengart-und-die-Stiftung"),
    ("Luzerner Zeitung, Frisches Blut im Stiftungsrat der Sammlung Rosengart",
     "https://www.luzernerzeitung.ch/zentralschweiz/frisches-blut-im-stiftungsrat-der-sammlung-rosengart-ld.1077156"),
    ("ch-cultura.ch, Zum Tod der Museumsstifterin Angela Rosengart",
     "https://ch-cultura.ch/kulturfoerderung-kulturvermittlung-kultur-und-medienpolitik/luzern-zum-tod-der-schweizer-kunsthaendlerin-und-museumsstifterin-angela-rosengart/"),
    ("Fondation Gandur pour l'Art", "https://www.fg-art.org/"),
    ("Muzeum Susch", "https://muzeumsusch.ch/"),
    ("NZZ, Ein Museum in Susch für die Gegenwartskunst",
     "https://www.nzz.ch/feuilleton/ein-museum-in-susch-fuer-die-gegenwartskunst-polnische-sammlerin-erfuellt-sich-ihren-traum-ld.1450073"),
    ("Kirchner Museum Davos", "https://kirchnermuseum.ch/"),
    ("SWI swissinfo.ch, Davos lehnt Investitionsbeitrag für Kirchner Museum ab",
     "https://www.swissinfo.ch/ger/davos-lehnt-investitionsbeitrag-f%C3%BCr-kirchner-museum-ab/88305939"),
    ("SRF Kultur, Tabubruch mit Ansage: Verkauf von drei Cézanne-Gemälden soll \
      Museum Langmatt retten",
     "https://www.srf.ch/kultur/kunst/tabubruch-mit-ansage-verkauf-von-drei-cezanne-gemaelden-soll-museum-langmatt-retten"),
    ("Museum Langmatt, Gesamtsanierung",
     "https://www.langmatt.ch/langmatt/gesamtsanierung"),
    ("Zofinger Tagblatt, Zukunft des Museums nach Verkauf der Cézanne-Bilder \
      gesichert",
     "https://zofingertagblatt.ch/baden-langmatt-versteigerung-zukunft-des-museums-nach-verkauf-der-cezanne-bilder-gesichert/"),
    ("Schloss Tarasp / Chastè da Tarasp", "https://www.schloss-tarasp.ch/"),
    ("StiftungSchweiz, Fundaziun CHASTÈ DA TARASP - Not Vital",
     "https://stiftungen.stiftungschweiz.ch/organisation/fundaziun-chaste-da-tarasp-not-vital"),
    ("SRF, Not Vital: Neuer Herr von Schloss Tarasp",
     "https://www.srf.ch/news/graubuenden-not-vital-neuer-herr-von-schloss-tarasp"),
    ("Wikipedia, Schloss Tarasp", "https://de.wikipedia.org/wiki/Schloss_Tarasp"),
];

const QUELLEN_GESUNDHEIT: &[(&str, &str)] = &[
    ("Gehirn- und Traumastiftung Graubünden/Schweiz", "https://gtsg.ch/"),
    ("Academia Raetica, Gehirn- und Trauma-Stiftung Graubünden/Schweiz",
     "https://academiaraetica.ch/forschung-und-bildung/institutionen/gtsg"),
    ("Hirschmann Stiftung, Personalisierte Medizin bei ADHS",
     "https://www.hirschmann-stiftung.ch/de/projekte/personalisierte-medizin-bei-adhs/"),
    ("Pro Mente Sana, Über uns", "https://promentesana.ch/ueber-uns"),
    ("Historisches Lexikon der Schweiz, Pro Mente Sana",
     "https://hls-dhs-dss.ch/de/articles/025813/"),
    ("Gesundheitsförderung Schweiz, Stiftung",
     "https://gesundheitsfoerderung.ch/stiftung"),
    ("Bundesrat, Der Prämienbeitrag für die allgemeine Gesundheitsförderung",
     "https://www.admin.ch/de/nsb?id=61377"),
    ("EDI, Bericht zur Mittelverwendung Gesundheitsförderung Schweiz 2024",
     "https://www.parlament.ch/centers/documents/de/EDI-Bericht%20Mittelverwendung_GFCH%202024_D.pdf"),
    ("Stiftung Freie Gemeinschaftsbank, Unterstiftung Neurodiversität und \
      Lebenskunst",
     "https://stiftungfgb.ch/dachstiftung/unterstiftungen/neurodiversitaetundlebenskunst"),
    ("Stiftung Rheinleben", "https://www.rheinleben.ch/stiftung"),
    ("Pro Juventute, Wer wir sind",
     "https://www.projuventute.ch/de/stiftung/wer-wir-sind"),
    ("BSV, Bericht Pro Juventute Beratung und Hilfe 147",
     "https://www.bsv.admin.ch/dam/bsv/de/dokumente/fgg/berichte-vorstoesse/br-bericht-projuventute-beratung-hilfe-147.pdf.download.pdf/br-bericht-projuventute-beratung-hilfe-147-de.pdf"),
    ("ADHS-Organisation elpos Schweiz, Über den Verein",
     "https://elpos.ch/ueber-den-verein/"),
    ("Schweizerische Fachgesellschaft ADHS", "https://www.sfg-adhs.ch/"),
];

const QUELLEN_BEIDE: &[(&str, &str)] = &[
    ("Eidgenössische Stiftungsaufsicht ESA, Aufsicht",
     "https://www.esa.admin.ch/de/aufsicht"),
    ("Eidgenössische Stiftungsaufsicht ESA, Stiftungsverzeichnis",
     "https://www.esa.admin.ch/de/stiftungsverzeichnis"),
    ("Fundraiso Schweiz, Stiftungsaufsicht eidgenössisch / kantonal",
     "https://www.fundraiso.ch/de/page/stiftungsaufsicht-eidgenoessisch-kantonal"),
];

// ---------------------------------------------------------------------------
// Satz
// ---------------------------------------------------------------------------

/// Jede `\n`-getrennte Zeile wird ein eigener Absatz, damit gesetzte
/// Zeilenumbrüche erhalten bleiben; innerhalb einer Zeile bricht genpdf um.
///
/// Der Stil muss zweimal gesetzt werden: `push_styled` färbt nur den
/// Textlauf, während die Zeilenhöhe aus dem Stil des umgebenden Elements
/// berechnet wird. Ohne das zusätzliche `styled` bekäme eine 30-pt-Zeile die
/// Zeilenhöhe der 10-pt-Grundschrift, und mehrzeilige Titel würden
/// übereinanderfallen.
fn push_lines(doc: &mut genpdf::Document, text: &str, style: Style, align: Alignment) {
    for line in text.split('\n') {
        let mut p = Paragraph::default();
        p.push_styled(line.to_string(), style);
        doc.push(p.aligned(align).styled(style));
    }
}

fn body(doc: &mut genpdf::Document, text: &str) {
    push_lines(doc, text, Style::new().with_color(INK).with_font_size(10), Alignment::Left);
}

/// Kapiteltitel: Goldene Vorzeile, darunter der Titel gross in Schiefer.
fn h1(doc: &mut genpdf::Document, kicker: &str, titel: &str) {
    push_lines(
        doc,
        kicker,
        Style::new().with_color(GOLD).with_font_size(9).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.3));
    push_lines(
        doc,
        titel,
        Style::new().with_color(SLATE).with_font_size(19).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.8));
}

fn h2(doc: &mut genpdf::Document, titel: &str) {
    doc.push(Break::new(0.7));
    push_lines(
        doc,
        titel,
        Style::new().with_color(SLATE).with_font_size(12).bold(),
        Alignment::Left,
    );
    doc.push(Break::new(0.35));
}

/// Anzeigeform einer URL. Sie muss auf eine Zeile passen: eine URL enthält
/// keine Leerzeichen, kann also nicht umbrochen werden, und genpdf lässt ein
/// Wort, das nicht in die Zeile passt, ersatzlos weg. Deshalb fallen Schema
/// und `www.` weg, und ein zu langer Pfad wird in der Mitte elidiert. Verlinkt
/// wird immer die vollständige Adresse.
fn link_text(url: &str) -> String {
    let s = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let s = s.strip_prefix("www.").unwrap_or(s);
    if s.chars().count() <= MAX_LINK_CHARS {
        return s.to_string();
    }
    let (host, rest) = match s.find('/') {
        Some(i) => s.split_at(i),
        None => (s, ""),
    };
    let platz = MAX_LINK_CHARS.saturating_sub(host.chars().count() + 2);
    let rest: Vec<char> = rest.chars().collect();
    let ab = rest.len().saturating_sub(platz);
    let schwanz: String = rest[ab..].iter().collect();
    format!("{host}/…{schwanz}")
}

/// URL-Zeile. Sie ist die einzige Stelle im Dokument, die in
/// LINK_FONT_SIZE gesetzt wird - daran erkennt `add_links` sie später im
/// Inhaltsstrom wieder.
fn push_link(doc: &mut genpdf::Document, url: &str) {
    let style = Style::new().with_color(LINK).with_font_size(LINK_FONT_SIZE);
    let mut p = Paragraph::default();
    p.push_styled(link_text(url), style);
    doc.push(p.styled(style));
}

/// Zeile eines Steckbriefs: fette goldene Marke, danach der Text im Fluss.
fn kv(doc: &mut genpdf::Document, label: &str, wert: &str) {
    let mut p = Paragraph::default();
    p.push_styled(
        format!("{label}  "),
        Style::new().with_color(GOLD).with_font_size(10).bold(),
    );
    p.push_styled(
        wert.to_string(),
        Style::new().with_color(INK).with_font_size(10),
    );
    doc.push(p);
    doc.push(Break::new(0.3));
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


// ---------------------------------------------------------------------------
// Auswahl der Stränge
// ---------------------------------------------------------------------------

/// Welche Stränge das PDF enthält. Ohne Argument beides - der Bericht bildet
/// dann die beiden Zweckstränge der Stiftung ab.
#[derive(Clone, Copy, PartialEq)]
enum Auswahl {
    Beides,
    Kunst,
    Gesundheit,
}

impl Auswahl {
    fn kunst(self) -> bool {
        self != Auswahl::Gesundheit
    }

    fn gesundheit(self) -> bool {
        self != Auswahl::Kunst
    }

    /// Titel auf dem Deckblatt.
    fn titel(self) -> &'static str {
        match self {
            Auswahl::Beides => "Stiftungen\nin der Schweiz",
            Auswahl::Kunst => "Kunststiftungen\nin der Schweiz",
            Auswahl::Gesundheit => "Stiftungen für\npsychische Gesundheit",
        }
    }

    fn untertitel(self) -> &'static str {
        match self {
            Auswahl::Beides => {
                "Kunst und psychische Gesundheit – die beiden Stränge\nder \
                 Stiftung, je an ihren Vorbildern gemessen"
            }
            Auswahl::Kunst => {
                "Wie sie betrieben werden, wer sie führt,\nwovon sie ihren \
                 Unterhalt bestreiten und wo sie angemeldet sind"
            }
            Auswahl::Gesundheit => {
                "ADHS, Neurodiversität und psychische Gesundheit:\nwer trägt \
                 sie, wer führt sie, wovon leben sie"
            }
        }
    }

    /// Vorgabename der Ausgabedatei.
    fn dateiname(self) -> &'static str {
        match self {
            Auswahl::Beides => DEFAULT_OUT,
            Auswahl::Kunst => "Kunststiftungen_Schweiz_Recherche.pdf",
            Auswahl::Gesundheit => "Gesundheitsstiftungen_Schweiz_Recherche.pdf",
        }
    }

    /// Die Porträts in der Reihenfolge, in der sie gedruckt werden.
    fn portraits(self) -> Vec<(&'static str, &'static str, &'static str, &'static [Portrait])> {
        let mut teile = Vec::new();
        if self.kunst() {
            teile.push((
                "TEIL I",
                "Acht Kunststiftungen",
                "Der Strang von Jürg Davatz. Jeder Steckbrief beantwortet \
                 dieselben vier Fragen: wer trägt die Stiftung, wer führt sie, \
                 wovon lebt der Unterhalt, und wo ist sie angemeldet.",
                PORTRAITS,
            ));
        }
        if self.gesundheit() {
            teile.push((
                if self.kunst() { "TEIL II" } else { "TEIL I" },
                "Acht Stiftungen für psychische Gesundheit",
                "Der Strang von Dr. med. Ursula Davatz. Dieselben vier Fragen, \
                 gestellt an das Feld der psychischen Gesundheit, der \
                 Neurodiversität und namentlich der ADHS. Der letzte Eintrag \
                 ist bewusst keine Stiftung – er trägt den wichtigsten Befund \
                 dieses Teils.",
                PSYCHE,
            ));
        }
        teile
    }

    /// Die Folgerungen, in einer durchgehenden Nummerierung.
    fn befunde(self) -> Vec<&'static Befund> {
        let mut v: Vec<&Befund> = Vec::new();
        if self.kunst() {
            v.extend(BEFUNDE_KUNST.iter());
        }
        v.extend(BEFUNDE_BEIDE.iter());
        if self.gesundheit() {
            v.extend(BEFUNDE_GESUNDHEIT.iter());
        }
        v
    }

    fn quellen(self) -> Vec<&'static (&'static str, &'static str)> {
        let mut v: Vec<&(&str, &str)> = Vec::new();
        if self.kunst() {
            v.extend(QUELLEN_KUNST.iter());
        }
        if self.gesundheit() {
            v.extend(QUELLEN_GESUNDHEIT.iter());
        }
        v.extend(QUELLEN_BEIDE.iter());
        v
    }

    /// Alle URLs in genau der Reihenfolge, in der sie im PDF gesetzt werden:
    /// erst die Website je Porträt, dann die Quellen. `add_links` verlässt
    /// sich darauf, weil es die Textursprünge in Zeichenreihenfolge einsammelt.
    fn urls(self) -> Vec<&'static str> {
        let mut v: Vec<&str> = Vec::new();
        for (_, _, _, portraits) in self.portraits() {
            v.extend(portraits.iter().map(|p| p.url));
        }
        v.extend(self.quellen().iter().map(|q| q.1));
        v
    }
}

// ---------------------------------------------------------------------------
// Kapitel
// ---------------------------------------------------------------------------

fn push_cover(doc: &mut genpdf::Document, auswahl: Auswahl) {
    // Der Titelblock sitzt im oberen Drittel; der Rest der Seite bleibt leer.
    doc.push(Break::new(8.0));
    push_lines(
        doc,
        "RECHERCHE FÜR DIE FUNDAZIUN DA VAZ – VAL MÜSTAIR",
        Style::new().with_color(GOLD).with_font_size(10).bold(),
        Alignment::Center,
    );
    doc.push(Break::new(2.2));
    push_lines(
        doc,
        auswahl.titel(),
        Style::new().with_color(SLATE).with_font_size(30).bold(),
        Alignment::Center,
    );
    doc.push(Break::new(1.6));
    push_lines(
        doc,
        auswahl.untertitel(),
        Style::new().with_color(INK).with_font_size(13).italic(),
        Alignment::Center,
    );
    doc.push(Break::new(3.0));
    let anzahl: usize = auswahl.portraits().iter().map(|t| t.3.len()).sum();
    push_lines(
        doc,
        &format!(
            "{} Porträts und {} Folgerungen\nfür die Stiftungsurkunde",
            anzahl,
            auswahl.befunde().len()
        ),
        Style::new().with_color(MUTED).with_font_size(11),
        Alignment::Center,
    );
    doc.push(Break::new(2.0));
    push_lines(
        doc,
        STAND,
        Style::new().with_color(MUTED).with_font_size(10),
        Alignment::Center,
    );
    doc.push(PageBreak::new());
}

fn push_vorbemerkung(doc: &mut genpdf::Document, auswahl: Auswahl) {
    h1(doc, "AUSGANGSLAGE", "Vorbemerkung zur Auswahl");
    body(
        doc,
        "Eine amtliche Rangliste der Schweizer Stiftungen nach Vermögen gibt es \
         nicht. Stiftungen müssen ihr Vermögen nicht veröffentlichen, und die \
         vermögendsten Schweizer Stiftungen überhaupt – Jacobs Foundation, die \
         Holdingstiftung der Rolex SA, Ernst Göhner, Fondation Botnar, Aga \
         Khan, IKEA – gehören weder ins Kunst- noch ins Gesundheitsfeld. Die \
         folgende Auswahl ordnet deshalb nach Betriebsgrösse und öffentlicher \
         Bedeutung, nicht nach Bilanzsumme.",
    );
    doc.push(Break::new(0.5));
    body(
        doc,
        "Zur Einordnung: Das Stiftungsvermögen in der Schweiz beläuft sich auf \
         rund CHF 139.5 Milliarden, und knapp ein Viertel aller Schweizer \
         Stiftungen unterstützt kulturelle Projekte. Das Feld ist gross, aber \
         sehr ungleich ausgestattet: zwischen der Fondation Beyeler und einer \
         Sammlungsstiftung ohne eigenes Haus liegen zwei Grössenordnungen.",
    );
    doc.push(Break::new(0.5));
    if auswahl.kunst() {
        body(
            doc,
            "Für den Kunststrang wurden acht Fälle ausgewählt, die für unsere \
             Frage etwas hergeben: zwei Kantonsnachbarn in Graubünden, zwei \
             familiengegründete Stiftungen, die den Tod des Stifters bereits \
             überstanden haben, das grösste Haus des Landes, zwei \
             Sammlungsstiftungen ohne eigenen Betrieb und der eine Fall, in \
             dem eine denkmalwürdige Liegenschaft samt Sammlung die Stiftung \
             beinahe ruiniert hat.",
        );
        doc.push(Break::new(0.5));
    }
    if auswahl.gesundheit() {
        body(
            doc,
            "Für den Gesundheitsstrang wurde gezielt nach ADHS und \
             Neurodiversität gesucht. Das Ergebnis ist selbst schon ein \
             Befund: Die ADHS-Arbeit der Schweiz ist fast durchwegs in \
             Vereinen organisiert, nicht in Stiftungen. Aufgenommen sind \
             deshalb die eine Stiftung mit ausgeprägtem ADHS-Schwerpunkt, die \
             grossen Stiftungen der psychischen Gesundheit, zwei \
             Finanzierungsmodelle, die es im Kulturbereich nicht gibt, und zum \
             Vergleich der grösste ADHS-Verein des Landes.",
        );
        doc.push(Break::new(0.5));
    }
    body(
        doc,
        "Die Angaben stammen aus öffentlich zugänglichen Quellen: \
         Stiftungswebseiten, Parlaments- und Regierungsvorlagen, \
         Medienberichte. Wo Zahlen aus älteren Vorlagen stammen, ist das im \
         Text vermerkt.",
    );
    doc.push(PageBreak::new());
}

fn push_teil(
    doc: &mut genpdf::Document,
    kicker: &str,
    titel: &str,
    intro: &str,
    portraits: &[Portrait],
) {
    h1(doc, kicker, titel);
    body(doc, intro);

    for (i, p) in portraits.iter().enumerate() {
        if i > 0 {
            doc.push(PageBreak::new());
        } else {
            doc.push(Break::new(1.0));
        }
        h2(doc, &format!("{}. {}", i + 1, p.name));
        push_lines(
            doc,
            p.ort,
            Style::new().with_color(GOLD).with_font_size(10).bold(),
            Alignment::Left,
        );
        doc.push(Break::new(0.4));
        push_lines(
            doc,
            p.kurz,
            Style::new().with_color(SLATE).with_font_size(11).italic(),
            Alignment::Left,
        );
        doc.push(Break::new(0.8));
        kv(doc, "Gegründet", p.gegruendet);
        kv(doc, "Trägerschaft", p.traegerschaft);
        kv(doc, "Führung", p.fuehrung);
        kv(doc, "Finanzierung des Unterhalts", p.finanzierung);
        kv(doc, "Angemeldet", p.angemeldet);
        doc.push(Break::new(0.4));
        push_link(doc, p.url);
    }
    doc.push(PageBreak::new());
}

fn push_registrierung(doc: &mut genpdf::Document, auswahl: Auswahl) {
    let kicker = if auswahl == Auswahl::Beides { "TEIL III" } else { "TEIL II" };
    h1(doc, kicker, "Wo diese Stiftungen angemeldet sind");
    body(
        doc,
        "Zwei Register sind zu unterscheiden, und sie folgen unterschiedlichen \
         Anknüpfungspunkten.",
    );
    h2(doc, "Handelsregister: der Sitz entscheidet");
    body(
        doc,
        "Die Stiftung entsteht mit dem Eintrag im Handelsregister des \
         Sitzkantons; der Eintrag ist konstitutiv, nicht bloss deklaratorisch.",
    );
    doc.push(Break::new(0.4));
    if auswahl.kunst() {
        body(
            doc,
            "Alle acht Kunststiftungen sind im Handelsregister ihres \
             Sitzkantons eingetragen – Beyeler in Basel-Stadt, Bührle in \
             Zürich, Gianadda im Wallis, Rosengart in Luzern, Gandur in Genf, \
             Muzeum Susch und Kirchner in Graubünden, Langmatt im Aargau.",
        );
        doc.push(Break::new(0.4));
    }
    if auswahl.gesundheit() {
        body(
            doc,
            "Im Gesundheitsfeld ist das Bild gemischter: Die Gehirn- und \
             Traumastiftung ist in Graubünden eingetragen, Pro Mente Sana und \
             Pro Juventute in Zürich, Rheinleben in Basel-Stadt. Zwei Fälle \
             fallen aus dem Raster. Die Unterstiftung \"Neurodiversität und \
             Lebenskunst\" hat gar keinen eigenen Eintrag, weil sie die \
             Rechtsform der Basler Dachstiftung nutzt. Und Gesundheitsförderung \
             Schweiz ist zwar eine privatrechtliche Stiftung, hat ihre \
             Grundlage aber im KVG und legt dem EDI und dem Parlament \
             Rechenschaft ab.",
        );
        doc.push(Break::new(0.4));
    }
    body(
        doc,
        "Für uns heisst das: Handelsregister Graubünden, solange der Sitz in \
         Sta. Maria bleibt.",
    );
    h2(doc, "Stiftungsaufsicht: die Tätigkeit entscheidet");
    body(
        doc,
        "Die Aufsicht richtet sich nicht nach dem Sitz, sondern nach dem \
         Gemeinwesen, dem die Stiftung nach ihrer Bestimmung angehört. Eine \
         kantonale Aufsichtsbehörde beaufsichtigt sämtliche Stiftungen, die \
         ihren Sitz im Kanton haben und ihren Zweck mehrheitlich in diesem \
         Kanton ausüben. Gesamtschweizerisch oder international tätige \
         Stiftungen unterstehen dagegen der Eidgenössischen Stiftungsaufsicht \
         ESA in Bern, angesiedelt beim Eidgenössischen Departement des \
         Innern. Auf kommunaler Ebene bleibt die Aufsicht bei der Gemeinde.",
    );
    doc.push(Break::new(0.5));
    body(
        doc,
        "Der Massstab der Aufsichtstätigkeit ist Art. 84 Abs. 2 ZGB: die \
         Aufsichtsbehörde hat dafür zu sorgen, dass das Stiftungsvermögen \
         seinen Zwecken gemäss verwendet wird. Die ESA prüft dazu jährlich die \
         Berichterstattung, genehmigt Statuten- und Reglementsänderungen, \
         Aufhebungen und Fusionen sowie Befreiungen von der Revisionspflicht, \
         und bietet eine freiwillige Vorprüfung von Stiftungsprojekten an – \
         das Gegenstück zur Vorabklärung, die Frau Justiz für uns gemacht hat. \
         Polizeiliche Befugnisse hat sie keine.",
    );
    doc.push(Break::new(0.5));
    body(
        doc,
        "Von der Aufsicht grundsätzlich ausgenommen sind Familien- und \
         Kirchenstiftungen. Das betrifft uns nicht: unsere Stiftung verfolgt \
         mit dem Museum, den Publikationen und der ADHS-Aufklärung einen nach \
         aussen gerichteten Zweck und ist damit eine klassische Stiftung, auch \
         wenn der Stiftungsrat auf die Familie beschränkt bleibt. Diese \
         Kombination – öffentlicher Zweck, familieninternes Organ – ist \
         zulässig, aber sie ist genau das, was eine Aufsichtsbehörde aufmerksam \
         macht.",
    );
    doc.push(PageBreak::new());
}

fn push_befunde(doc: &mut genpdf::Document, auswahl: Auswahl) {
    let befunde = auswahl.befunde();
    let kicker = if auswahl == Auswahl::Beides { "TEIL IV" } else { "TEIL III" };
    h1(
        doc,
        kicker,
        &format!("{} Folgerungen für unsere Urkunde", befunde.len()),
    );
    body(
        doc,
        "Was aus den Vergleichsfällen für die FUNDAZIUN DA VAZ – VAL MÜSTAIR \
         folgt, in der Reihenfolge ihrer Dringlichkeit.",
    );
    doc.push(Break::new(0.6));

    for (i, b) in befunde.iter().enumerate() {
        push_lines(
            doc,
            &format!("{}. {}", i + 1, b.titel),
            Style::new().with_color(SLATE).with_font_size(12).bold(),
            Alignment::Left,
        );
        doc.push(Break::new(0.35));
        body(doc, b.text);
        doc.push(Break::new(0.9));
    }
    doc.push(PageBreak::new());
}

fn push_quellen(doc: &mut genpdf::Document, auswahl: Auswahl) {
    h1(doc, "ANHANG", "Quellen");
    body(
        doc,
        "Recherche vom 14. August 2026. Betriebszahlen von Stiftungen sind nur \
         so aktuell wie die letzte veröffentlichte Vorlage oder \
         Medienmitteilung; wo eine Zahl aus einer älteren Quelle stammt, ist \
         das im Text vermerkt.",
    );
    doc.push(Break::new(0.7));
    for (bezeichnung, url) in auswahl.quellen() {
        let mut p = Paragraph::default();
        p.push_styled(
            "– ".to_string(),
            Style::new().with_color(GOLD).with_font_size(9),
        );
        p.push_styled(
            (*bezeichnung).to_string(),
            Style::new().with_color(INK).with_font_size(9),
        );
        doc.push(p.styled(Style::new().with_font_size(9)));
        push_link(doc, url);
        doc.push(Break::new(0.3));
    }
    doc.push(Break::new(1.2));
    push_lines(
        doc,
        "Erstellt mit Rust und genpdf, ohne Browser – dieselbe Pipeline wie in \
         listingtracker. Sämtliche URL-Zeilen sind anklickbar.",
        Style::new().with_color(MUTED).with_font_size(7).italic(),
        Alignment::Left,
    );
}

fn render(out: &Path, font_dir: &str, auswahl: Auswahl) -> Result<()> {
    let family = load_font_family(font_dir)?;
    let mut doc = genpdf::Document::new(family);
    let kopf = match auswahl {
        Auswahl::Beides => "Stiftungen in der Schweiz",
        Auswahl::Kunst => "Kunststiftungen in der Schweiz",
        Auswahl::Gesundheit => "Stiftungen für psychische Gesundheit",
    };
    doc.set_title(format!("{kopf} – Recherche FUNDAZIUN DA VAZ"));
    doc.set_minimal_conformance();
    doc.set_font_size(10);
    doc.set_line_spacing(1.35);

    let mut deco = genpdf::SimplePageDecorator::new();
    deco.set_margins(22);
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

    push_cover(&mut doc, auswahl);
    push_vorbemerkung(&mut doc, auswahl);
    for (kicker, titel, intro, portraits) in auswahl.portraits() {
        push_teil(&mut doc, kicker, titel, intro, portraits);
    }
    push_registrierung(&mut doc, auswahl);
    push_befunde(&mut doc, auswahl);
    push_quellen(&mut doc, auswahl);

    doc.render_to_file(out)
        .map_err(|e| anyhow!("PDF schreiben {}: {}", out.display(), e))?;

    let urls = auswahl.urls();
    let gesetzt = add_links(out, &urls)?;
    if gesetzt != urls.len() {
        return Err(anyhow!(
            "Link-Overlay: {} URL-Zeilen im Satz gefunden, aber {} URLs erwartet - \
             die Zuordnung wäre verschoben",
            gesetzt,
            urls.len()
        ));
    }
    println!("  {gesetzt} Links gesetzt");
    Ok(())
}

/// Legt über jede in LINK_FONT_SIZE gesetzte Textzeile eine Link-Annotation.
///
/// Der Inhaltsstrom wird Seite für Seite in Zeichenreihenfolge durchlaufen;
/// printpdf schreibt je Zeile `BT / TL / Td x y / Tf /F n / TJ [...]`, sodass
/// das letzte `Td` vor einem `TJ` die Grundlinie der Zeile angibt. Die so
/// eingesammelten Ursprünge werden der Reihe nach den URLs zugeordnet - die
/// Reihenfolge stimmt, weil `Auswahl::urls()` dieselbe Ordnung liefert wie der
/// Satz. Stimmt die Anzahl nicht, bricht `render` ab, statt falsch zu verlinken.
fn add_links(pdf: &Path, urls: &[&str]) -> Result<usize> {
    use lopdf::{Dictionary, Document, Object, StringFormat};

    let mut doc = Document::load(pdf)?;
    let seiten: Vec<(u32, lopdf::ObjectId)> =
        doc.get_pages().into_iter().collect();

    let num = |o: &Object| -> Option<f64> {
        match o {
            Object::Real(r) => Some(*r as f64),
            Object::Integer(i) => Some(*i as f64),
            _ => None,
        }
    };

    let mut gesetzt = 0usize;
    for (_, page_id) in seiten {
        let content = doc.get_and_decode_page_content(page_id)?;

        let mut pos = (0.0f64, 0.0f64);
        let mut size = 0.0f64;
        let mut origins: Vec<(f64, f64)> = Vec::new();
        for op in &content.operations {
            match op.operator.as_str() {
                "Td" | "TD" if op.operands.len() >= 2 => {
                    if let (Some(x), Some(y)) = (num(&op.operands[0]), num(&op.operands[1])) {
                        pos = (x, y);
                    }
                }
                "Tm" if op.operands.len() >= 6 => {
                    if let (Some(x), Some(y)) = (num(&op.operands[4]), num(&op.operands[5])) {
                        pos = (x, y);
                    }
                }
                "Tf" if op.operands.len() >= 2 => {
                    if let Some(s) = num(&op.operands[1]) {
                        size = s;
                    }
                }
                "Tj" | "TJ" => {
                    if (size - LINK_FONT_SIZE as f64).abs() < 0.01
                        && origins.last() != Some(&pos)
                    {
                        origins.push(pos);
                    }
                }
                _ => {}
            }
        }
        if origins.is_empty() {
            continue;
        }

        let mut annots: Vec<Object> = Vec::new();
        for (x, y) in &origins {
            let Some(url) = urls.get(gesetzt) else { break };
            gesetzt += 1;

            // Klickfläche nach der *angezeigten* Länge bemessen, nicht nach
            // der vollen URL - lange Adressen stehen gekürzt im Satz.
            let breite = (link_text(url).chars().count() as f64)
                * LINK_FONT_SIZE as f64
                * AVG_ADVANCE_EM;
            let rechts = (x + breite + 2.0).min(A4_WIDTH_PT - MARGIN_PT);

            let mut action = Dictionary::new();
            action.set("S", Object::Name(b"URI".to_vec()));
            action.set(
                "URI",
                Object::String(url.as_bytes().to_vec(), StringFormat::Literal),
            );

            let mut annot = Dictionary::new();
            annot.set("Type", Object::Name(b"Annot".to_vec()));
            annot.set("Subtype", Object::Name(b"Link".to_vec()));
            annot.set(
                "Rect",
                Object::Array(vec![
                    Object::Real((*x - 2.0) as f32),
                    Object::Real((*y - 2.0) as f32),
                    Object::Real(rechts as f32),
                    Object::Real((*y + LINK_FONT_SIZE as f64 + 2.0) as f32),
                ]),
            );
            // Kein sichtbarer Rahmen - die blaue Schrift markiert den Link.
            annot.set("Border", Object::Array(vec![0.into(), 0.into(), 0.into()]));
            annot.set("A", Object::Dictionary(action));
            annots.push(Object::Dictionary(annot));
        }

        if let Ok(page) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
            page.set("Annots", Object::Array(annots));
        }
    }

    doc.save(pdf)?;
    Ok(gesetzt)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let will_kunst = args.iter().any(|a| a == "--k" || a == "--kunst");
    let will_gesundheit = args.iter().any(|a| a == "--g" || a == "--gesundheit");
    // Ohne Argument - und ebenso, wenn beide gesetzt sind - der volle Bericht.
    let auswahl = match (will_kunst, will_gesundheit) {
        (true, false) => Auswahl::Kunst,
        (false, true) => Auswahl::Gesundheit,
        _ => Auswahl::Beides,
    };

    let out = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(auswahl.dateiname()));
    let font_dir = env::var("FONT_DIR").unwrap_or_else(|_| DEFAULT_FONT_DIR.to_string());

    render(&out, &font_dir, auswahl)?;
    let bytes = std::fs::metadata(&out)?.len();
    println!("→ {} ({} B)", out.display(), bytes);
    Ok(())
}
