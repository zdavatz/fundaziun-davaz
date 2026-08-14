// Neutrale Fassung der Folgerungen.
//
// build.rs kopiert diese Datei nach src/befunde.rs, wenn dort noch keine
// liegt. Die echte Fassung ist in .gitignore ausgeschlossen, weil sie
// Anfangskapital, Lohn und die Familienregelung der Stiftung beim Namen
// nennt. Hier stehen dieselben Folgerungen ohne die konkreten Angaben -
// der Bericht bleibt damit lesbar, auch wenn ihn jemand ohne die privaten
// Daten baut.

/// Folgerungen aus dem Kunststrang.
const BEFUNDE_KUNST: &[Befund] = &[
    Befund {
        titel: "Keine dieser Stiftungen finanziert ihren Unterhalt aus \
                Mieterträgen",
        text: "Das Muster ist überall dasselbe: Ertrag aus dem \
               Stiftungskapital, Eintritte, Sponsoring und Gönner, dazu \
               öffentliche Beiträge. Eine Stiftung, die den Unterhalt \
               denkmalwürdiger Liegenschaften aus den Mieterträgen anderer \
               Liegenschaften bestreiten will, hat unter den \
               Vergleichsfällen kein Vorbild. Das ist kein Einwand gegen die \
               Konstruktion, aber es heisst: die Ertragsrechnung im Gesuch \
               muss aus sich selbst überzeugen, weil sie sich auf keinen \
               Präzedenzfall berufen kann.",
    },
    Befund {
        titel: "Langmatt ist die Warnung, nicht das Vorbild",
        text: "Villa und Sammlung in einer Stiftung, das Stiftungskapital \
               aufgezehrt, Sanierungsbedarf CHF 18.8 Mio. – und am Ende der \
               Verkauf dreier Cézannes für CHF 40.5 Mio., allein um das \
               Kapital wiederherzustellen. Wer Liegenschaften für \
               unveräusserlich erklärt, Kunstwerke aber veräusserlich lässt, \
               legt genau diese Konstruktion an. Langmatt zeigt, wie daraus \
               im Ernstfall der Verkauf zur Selbstrettung wird. Wer das \
               nicht will, muss entweder das Anfangskapital erhöhen oder die \
               Unterhaltsrückstellungen in der Urkunde beziffern statt nur \
               erwähnen.",
    },
    Befund {
        titel: "Zweistufigkeit trennt das Vermögen vom Betrieb – und den Lohn \
                vom Stiftungsrat",
        text: "Bei Beyeler hält die Stiftung Sammlung und Bau, die Beyeler \
               Museum AG führt den Betrieb, und die Stiftung deckt deren \
               Verluste durch Einlagen. Daran ist das Wichtigste: in einer \
               Betriebsgesellschaft ist eine Entschädigung ein gewöhnlicher \
               Geschäftsführerlohn. In der Stiftung selbst ist sie eine \
               Zuwendung an ein Stiftungsratsmitglied – und damit der wunde \
               Punkt beim Steuerbefreiungsgesuch, zusammen mit einer \
               Liegenschaftsverwaltung zu Marktpreis und Dienstleistungen \
               nahestehender Personen.",
    },
    Befund {
        titel: "Bührle zeigt, wie man die Unterhaltslast abgibt – und was \
                das kostet",
        text: "Die Stiftung besitzt die Werke, gab 2012 rund 190 davon als \
               Langzeitleihgabe ans Kunsthaus Zürich, und Unterhalt, \
               Konservierung und Vermittlung sind seither im Businessplan der \
               Zürcher Kunstgesellschaft budgetiert, nicht bei der Stiftung. \
               Wer keine eigene Betriebsorganisation aufbauen will, verlagert \
               die Last auf eine bestehende Institution. Die Kehrseite ist \
               ebenso deutlich: Die Stiftung verliert damit die \
               Deutungshoheit über ihre eigene Sammlung. Der Rücktritt von \
               Lukas Gloor Ende 2021 mitten in der Herkunftsdebatte ist die \
               Rechnung dafür.",
    },
    Befund {
        titel: "Familiengeführt geht – familienexklusiv im Zweiergremium ist \
                die Ausnahme",
        text: "Gianadda ist der nächste Vergleichsfall: 1978 von Léonard \
               Gianadda gegründet, nach dessen Tod Ende 2023 seit Mai 2024 \
               von François Gianadda präsidiert. Aber der Stiftungsrat zählt \
               neun bis fünfzehn Mitglieder und ist nicht auf die Familie \
               beschränkt. Bei Rosengart präsidierte die Stifterin selbst, \
               daneben sassen ein Vizepräsident und der Leiter Kulturförderung \
               des Kantons Luzern im Rat; 2018 wurden vier neue Mitglieder \
               gewählt – die Verbreiterung geschah zu Lebzeiten, und als \
               Angela Rosengart am 30. Juni 2026 starb, war die Stiftung \
               vorbereitet. Ein anfänglich zweiköpfiger, dauerhaft auf eine \
               Familie beschränkter Stiftungsrat ist demgegenüber die enge \
               Variante. Das ist zulässig, aber es erklärt, warum die \
               Aufsichtsbehörde danach fragen wird.",
    },
    Befund {
        titel: "Öffentliche Beiträge sind nicht selbstverständlich, auch in \
                Graubünden nicht",
        text: "Kirchner Davos: Erweiterung veranschlagt auf CHF 11.5 Mio., \
               Stiftung, private Sponsoren und das WEF sagten 7 Mio. zu – den \
               Gemeindebeitrag von 4 Mio. lehnten die Davoser im November 2024 \
               an der Urne ab. Und selbst bei Beyeler, das von Riehen ab \
               Herbst 2025 jährlich CHF 1.126 Mio. plus Baurechtszinserlass \
               erhält, werden die kritischen Stimmen lauter. Wer mit \
               Subventionen plant, plant mit einer Abstimmung. Der Nachweis \
               der Zweckerreichbarkeit muss ohne öffentliche Beiträge \
               aufgehen.",
    },
];

/// Gilt für beide Stränge und steht deshalb in jeder Fassung des Berichts.
const BEFUNDE_BEIDE: &[Befund] = &[
    Befund {
        titel: "Der Sitz entscheidet über das Handelsregister, die Tätigkeit \
                über die Aufsicht",
        text: "Die Stiftung entsteht mit dem Eintrag im Handelsregister des \
               Sitzkantons. Die Aufsicht richtet sich dagegen nach dem \
               Gemeinwesen, dem die Stiftung nach ihrer Bestimmung angehört: \
               eine kantonale Behörde beaufsichtigt Stiftungen, die ihren Sitz \
               im Kanton haben und ihren Zweck mehrheitlich in diesem Kanton \
               ausüben; gesamtschweizerisch oder international tätige \
               Stiftungen unterstehen der Eidgenössischen Stiftungsaufsicht \
               ESA in Bern. Wer das Tätigkeitsgebiet über den Sitzkanton \
               hinaus öffnet, muss diese Frage vorher klären – davon hängt \
               ab, wo künftig Rechenschaft abzulegen ist. Muzeum Susch zeigt \
               immerhin, dass eine international ausstrahlende Kunststiftung \
               in Graubünden ihren Platz hat.",
    },
];

/// Folgerungen aus dem Gesundheits- und ADHS-Strang.
const BEFUNDE_GESUNDHEIT: &[Befund] = &[
    Befund {
        titel: "Die ADHS-Landschaft der Schweiz ist vereinsförmig, nicht \
                stiftungsförmig",
        text: "Die grösste ADHS-Organisation des Landes, elpos, ist seit 1974 \
               ein Verein. Daneben stehen der Verein ADHS/ADS Schweiz und die \
               Schweizerische Fachgesellschaft ADHS als Fachgesellschaft. Eine \
               ADHS-Stiftung von nationalem Gewicht gibt es nicht; die einzige \
               Stiftung mit ausgeprägtem ADHS-Schwerpunkt ist die Gehirn- und \
               Traumastiftung in Chur. Für ein Steuerbefreiungsgesuch ist das \
               ein starkes Argument: ein ADHS-Zweckstrang besetzt eine \
               tatsächlich offene Stelle, statt bestehende Angebote zu \
               doppeln. Das gehört in die Begründung.",
    },
    Befund {
        titel: "Im Kanton Graubünden sitzt bereits eine ADHS-Stiftung – als \
                Partnerin, nicht als Konkurrentin",
        text: "Die Gehirn- und Traumastiftung Graubünden/Schweiz in Chur \
               besteht seit 2006, präsidiert von alt Bundesrichter Giusep Nay, \
               geführt von Andreas Müller. Ihr Zweck nennt wörtlich die \
               Aufklärung der Öffentlichkeit über neurobiologische \
               Erkenntnisse. Wo eine neue Stiftung Zusammenarbeit mit \
               Fachpersonen, Fachgesellschaften und Schulen vorsieht, wäre ein \
               Kontakt noch vor der Beurkundung klug – fachlich, und weil \
               beide Stiftungen dieselbe Aufsichtsbehörde haben werden.",
    },
    Befund {
        titel: "Der ADHS-Strang finanziert sich völlig anders als der \
                Kunststrang",
        text: "Die Kunstseite lebt von Eintritten, Sammlungserträgen und \
               Gönnern. Die Gesundheitsseite lebt von Leistungsverträgen mit \
               Kantonen, von IV-Beiträgen, von Beiträgen des Bundesamts für \
               Sozialversicherungen, von Förderstiftungen wie Hirschmann – und \
               im Extremfall von einem gesetzlich festgesetzten \
               Prämienzuschlag, wie bei Gesundheitsförderung Schweiz mit \
               CHF 4.80 pro versicherte Person, wovon rund CHF 7.68 Mio. in \
               die psychische Gesundheit fliessen. Ein zweiter Strang öffnet \
               damit Geldquellen, die dem ersten verschlossen sind. Aber nur, \
               wenn er als Leistung ausgestaltet ist und nicht bloss als \
               Werkwidmung: Vorträge, Kurse, Publikationen und \
               Veranstaltungen gehören ausdrücklich in die Urkunde.",
    },
    Befund {
        titel: "Auch grosse, alte Gesundheitsstiftungen geraten in Schieflage",
        text: "Pro Mente Sana, seit 1978 die nationale Fachorganisation für \
               psychische Gesundheit, stand jahrelang unter finanziellem \
               Druck; wenige Monate nach dem Antritt der neuen \
               Geschäftsleiterin Muriel Langenberger im September 2022 wurde \
               eine Sanierung mit externer Unterstützung eingeleitet, und seit \
               Januar 2026 präsidiert Matthias Jäger den Stiftungsrat. \
               Spendenfinanzierte Gesundheitsstiftungen sind nicht stabiler \
               als kunstfinanzierte. Im Vorteil ist, wer auf ein bestehendes \
               Werk und einen eingeführten Namen aufsetzen kann – nicht auf \
               eine Organisation, die erst noch bekannt werden muss.",
    },
    Befund {
        titel: "Die Liegenschaft darf der Zweck sein – das ist das stärkste \
                Argument gegenüber der Steuerverwaltung",
        text: "Die Unterstiftung \"Neurodiversität und Lebenskunst\" unter der \
               Stiftung Freie Gemeinschaftsbank in Basel betreibt ein \
               Rückzugszentrum für neurodivergente Menschen: ein \
               Passivholzbau auf 900 Metern, zehn Studios, sechs Zimmer, 60 \
               Hektaren Land, bewusst strahlungsarm ohne WLAN. Dort ist die \
               Liegenschaft nicht Kapitalanlage, sondern Mittel der \
               Zweckverwirklichung – genau die Begründungsfigur, die eine \
               liegenschaftenhaltende Stiftung braucht. Nebenbei zeigt der \
               Fall, dass ein Neurodiversitätszweck auch ohne eigene Stiftung \
               zu haben wäre, unter einem Dach. Wo Hypotheken, eine \
               Kunstsammlung und Urheberrechte im Spiel sind, kommt das nicht \
               in Frage – aber es ist die Antwort, falls jemand fragt, warum \
               es eine eigene Stiftung sein muss.",
    },
];
