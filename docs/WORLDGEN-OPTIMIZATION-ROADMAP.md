# Worldgen: Optimierungen, Belege und offene Versuche

Stand: 2026-11-08, Minecraft26.3, Worldgen-Stand`144bce43b7` mit69,26R2 und95; Logger100 zusätzlich übernommen.
Verlinkte Maschinen-/Profil-/PoC-Belege liegen im lokalen ignorierten
Studienarchiv außerhalb von `target/`; sie sind keine mitcommitteten Dateien.
Die zuletzt gebauten99-Vergleichsbinaries gehören zum Runtime-Stand`e78803f6dcb`.

## Aktives vollständiges Optimierungsziel

Der Nutzer hat die vollständige Untersuchung als fortlaufendes Goal gestartet.
[Aktuelle Abdeckung](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/coverage-current.json)
führt339 aktuelle handgeschriebene Dateien:310 aus dem bisherigen Inventar,
24 zusätzliche Bootstrap-/Pregen-/Logger-/Harness-Dateien und fünf mit95
übernommene Dateien. Davon338Rust und ein Python-Testtreiber. Alle339 Dateien
sind durch vollständige Quellreviews abgedeckt; aktuelle SHAs und sämtliche Zeilenbereiche
sind im Index abgeglichen. Semantik-/Implementierungs-/Messgates der Kandidaten
bleiben gesondert offen. Vollständiger Quellreview bedeutet keine validierte
Optimierung und keine universelle Optimalität.

Der neue Master-Merge18e5 verändert drei dieser331 Dateien. Beide Template-
Dateien und die ganze Trig-Datei sind erneut vollständig geprüft; die übrigen
328 Quellfingerprints bleiben gleich. Neue NBT-Konvertierungsgrundlagen sind
separat gelesen. Das ist ein Quellenabgleich, kein Test-/ASM-/CPS-Nachweis für
den neuen vollständigen Server. Alte Binaries bleiben an ihren Ausgangsstand
gebunden; insbesondere Cargo.lock und weitere Core-Grundlagen haben sich geändert.
95ist in144bce übernommen. Fünf neue Dateien sind vollständig gelesen; fünf
geänderte Altdateien sind aus früherem Vollreview, vollständigem Delta und
bytegleichen unveränderten Bereichen abgeglichen.336ist eine explizite
Dateiunion, kein ungeprüfter neuer Testverzeichnis-Glob.
[Merge-Abgleich](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/MERGE18E-CURRENT-SOURCE-REVIEW.json).

Zwischenzeitliche332-Dateien-Stände enthielten zusätzlich den69- oder26-Test.
Commit25737f2 übernimmt nur den Runtime-Patch; der Test und sämtliche Belege
bleiben im isolierten Studiencheckout. Aktueller Region-SHA und Test-only-
Unterschied sind erneut abgeglichen, der332erSchnappschuss bleibt historisch
erhalten. Der Nutzer hat jetzt den26-Test samt Registrierung entfernt und
zwei Leerzeilen eingefügt; diese Änderungen bleiben erhalten. Anschließend hat
der Nutzer in`e78803f6dcb` selbst26R2 ohne die drei Nulladditionen committed.
Der aktuelle359-Zeilen-SHA entspricht exakt dem vorbereiteten Runtimekandidaten;
der331erIndex ist erneut SHA-/zeilengeprüft. Die bisherigen26-CPS-Werte gehören
zuR1; fürR2 ist kein CPS-Gewinn gemessen. Der Commit-Titel ist kein Messbeleg.
[Aktueller26R2-Quellreceipt](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/retained26R2-committed-source-review.json),
[historischer26-Quellreceipt](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/retained26-staged-source-review.json),
[69-Quellreceipt](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/retained69-committed-source-review.json).

[Zusammengeführtes Kandidatenregister](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/candidates-current-index.json):
53 historische Einträge und48 neue Einträge:46 mit vorbereitetem
isoliertem Runtimepatch, eine separate vorbereitete Allocatordiagnose und eine
ausdrücklich unvorbereitete Hypothese. Revisionen
wie26R2/87R2 erhöhen diese Eintragszahl nicht. Darunter sind
Alternativen, Reserven und Vertragsgates; keine101 validierten Optimierungen.
Die Bereichskataloge enthalten Positionen, Beweise, Gegenkosten und kleinste
Validierung. Offene Messungen bleiben offen; das Goal bleibt aktiv.
Der Index führt jetzt den aktuellen Variantenstatus vor den historischen
Vorbereitungsfeldern. Der unabhängige historische89-Einträge-Abgleich übernimmt bestehende
Messentscheidungen, aktuelle Vertragsgates und negative Quellbefunde44/50;
Simplex steht mit32abgeschlossenen Läufen und offenem OW16-Gate im Register.
R1-Belege und die vom Nutzer übernommene26R2 werden getrennt geführt. Der neue
90.Diagnoseeintrag ist zusätzlich quellseitig geprüft, nicht ausgeführt.
91–94 ergänzen Material-Gap, Ore-Abbruch, Zstd-Kontextreuse und das End-Grid.
91–93 bestehen den Apply-Check; Rust-Ausführung und native Messbelege fehlen.
Sie bleiben kleine Reserven. Das End-Grid94 wird inzwischen in der frühen
Kombinationsprüfung mit95 ausgeführt; aktuelle Profilgrenzen stehen bei95.
95 hat zwei isolierte Generatorintegrationen: Batch auf altem Grid und
Grid94 plus Batch95. Beide bestehen unabhängigen Quellreview sowie frühe
e788-Prüfungen ohne LTO: je sieben neue Tests plus fünf bestehende Codegentests.
Die echte FP-Beitragsausführung ist ausdrücklich abgesichert. Ein erster,
wegen alter kopierter Zeitstempel staler Kombinationsbuild ist verworfen.
Nur nachgewiesen frisch erzeugter Code und ein anderes Binary wurden akzeptiert.

**95allein auf altem Grid ist in144bce43b7 übernommen.** Auf identischem18e-
Ausgangsstand bestehen22.500 frühe und22.500 endgültige native Stage-Hashes,
zwölf gezielte Tests in beiden Profilen, Format und native Clippyprüfung.
16unprofilierte End-Läufe über zweiSeeds,257/385 und8/16Worker zeigen
begrenzt auf diese Workloads rund1–4%mehr vollständige End-CPS und rund4–5%
weniger aufsummierte Gen-CPU-Zeit. Acht Overworld-Kontrollen sind praktisch
flat, vier Nether-Kontrollen zeigen keinen erkennbaren Durchsatznachteil.
Keine allgemeine Worldgen-/Architektur- oder RSS-Gewinnzusage.
Native Gegenkosten: init_grid929→2229B, Frame136→648B; echte verbleibende
Bounds-/Sättigungsprüfungen und Call-Live-Zwischenspeicher sind dokumentiert.
Kein unsafe und kein neuer SIMD-Umbau; die beobachtete Tail-Vektorisierung
stammt vom Compiler.95+94 und der interne Integerentry bleiben getrennte
Reserven. [Übernahmeentscheidung](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/95-RETENTION-DECISION-20261108.json),
[End-Rohvergleich](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/95-NATIVE-END-SERIES-RESULT-20261108.json),
[Maschinenvergleich](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/night95-candidate-machine-review/REPORT.md).

Auf warmem positivem altem Grid kann Batch allein weniger Prädikatgraphen
benötigen als die Kombination; kein automatischer Kombinationsgewinn.
[95-Integrationsreview](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/end-batch-integration-independent-peer.md).
96 ist ein reiner Full-Block-Geometriecache mit fünf vorbereiteten Tests.
Build/Test-/Maschinen-/Timingbelege fehlen. Mehr Einträge bedeuten keine
weiteren validierten Verbesserungen.
97 ergänzt den Noise-Fill-Guardpool. V1 ist wegen zweier konkreter Lockzyklen
quellseitig verworfen; V2 versucht alle nötigen Guards nur ohne Warten und
kehrt bei Konkurrenz nach vollständiger Freigabe dauerhaft zum bisherigen
Pfad zurück. Fünf Tests und ein tatsächlich umgerouteter Hashadapter sind
vorbereitet, nicht ausgeführt; zwei unabhängige Quellreviews und Apply-Check
ersetzen keine Korrektheits-/Maschinen-/CPS-Gates.
98 ist ein unabhängiger sechszeiliger End-Guard: Liegt die gesamte25×25-Suche
innerhalb des ohnehin ausgeschlossenen Radius, entfallen625 Integerprüfungen.
Dort gab es zuvor bereits keine Simplex-Aufrufe. Drei Differentialtests sind
vorbereitet; unabhängiger Quellpeer und Apply-Check bestehen, Ausführung fehlt.
Die vorhandenen7500End-Hashes liegen weit außen und decken diesen Zentralzweig
nicht ab. Im bestehenden äußeren Biomepfad ist der neue Guard stets falsch und
verursacht Zusatzarbeit. [Beweis, Regionalbelege und Gegenkosten](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/end-whole-window-independent-peer.md).
99 beschränkt nur die privaten Licht-Stage-Writer auf tatsächlich benötigte
Slots; vollständige äußere Section-Maps benötigen keine kanonischen äußeren
Lichtdaten, kurze Maps behalten den bisherigen Fallback. Gate bleibt4 und
öffentliche/Queued-Pfade bleiben unverändert. Zehn Stage-Tests und22.500Hashes
bestehen auf dem vorherigen Runtime-Stand e788 früh ohne LTO und inzwischen
auch im normalen nativen Profil.
Clippy ist sauber; der native Server ist außerhalb von target/ gesichert.
Der gezielte native Vergleich belegt nun den wirklichen Sprung vor beiden
Writer-CAS bei vollständigem Rand. Funktion+23B, Frame und geprüfte Spillgruppe
gleich; zusätzliche Count-Prüfungen fallen auch bei ohnehin unbeleuchtetem
Rand an. Das ist eine konkrete Gegenkostenstelle, kein gemessener Verlust.
Die begonnene CPS-Serie ist nach gemeldeter Discord-/Spotify-Aktivität
abgebrochen und vollständig von Performanceentscheidungen ausgeschlossen.
Derselbe Kandidat benötigte81,38 und61,23Sekunden: keine belastbare
Gewinn- oder Verlustzuordnung. Kein CPS-Gewinn nachgewiesen oder übernommen.
[Ausführungs-/Abbruchbelege](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/99-benchmark-plan-20261108/ACTIVITY-STOP.json),
[nativer Vergleich](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/light99-candidate-machine-review/REPORT.md),
[PoC und Beweis](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/light-stage-outer-admission-poc99/README.md).
[Statusreview](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/chunk-index-status-review/REVIEW.md).

Das aktuelle Profil ist jetzt [ausgewertet](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/PROFILE-REVIEW.md):
459.842Gen-CPU-Samples; Klimasuche2,57%, Beardifier3,55%, Aquifer4,12% Self.
Diese Werte sind gruppenbezogen, keine Aufrufzahlen und keine CPS-Prognose.
Alte Tabellen unten behalten ihren ausdrücklich historischen Nenner.

PoC27 ist ausschließlich im isolierten Checkout gebaut:10Klimatests und22.500
Stage-Hashes bestehen, native Utils-Clippy und Formatprüfung sind sauber.
der schnelle Suchkörper benötigt2statt4SIMD-Teilprodukte bei unverändert88B
lokaler Stackreserve (Bencher-Aufrufer jedoch+112B, Suchcode zusammen+454B).
BAAB über4reale Zielmengen ergibt grob28…34% mehr lokalen
Lookup-Durchsatz. Die anschließenden16 Full-CPS-Läufe über zwei Seeds und
8/16Worker ergeben nur etwa+0,1…0,4% je Gruppenmittel. Wegen des kleinen,
noch nicht hinreichend reproduzierten Gesamtnutzens und zusätzlichen Suchcodes
**vorerst nicht übernommen**; weder ein Null-Effekt noch eine Regression ist
damit bewiesen. [Ergebnis und Gegenkosten](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/27-FULL-CPS-RESULT.json),
[Micro-Rohdaten](../worldgen-machine-audit-20261104/experiments/micro-results/goal-27-climate-20261107/summary.json).
Zusätzlich sind ein sicherer SearchContext-Versuch gegen rekursive Target-Stores
und ein Unsafe-Versuch nur gegen die privaten Childslice-Checks vorbereitet.
Öffentliche Cache-/Index-Prüfungen bleiben erhalten; unabhängiger Soundnessreview,
Kompilierung und Nutzenprüfung stehen aus. Beide sind Reserven außerhalb der
aktuellen Dreierliste. Die neuen Quellentwürfe bleiben unabhängige Reserven. Die einzige aktuelle
Dreierliste weiter unten ersetzt die abgeschlossenen27/69-Versuche.

69 ist **als Allocation-Optimierung in25737f2 übernommen**:
56Speichertests einschließlich neuem
Dateibyte-/Sektorgrenzen-/Fehlervergleich und22.500 Stage-Hashes bestehen.
Die echte Referenz-Pollfunktion ruft für nichtleeres Padding
`mi_zalloc_aligned(p,1)` und später `mi_free` auf; der Kandidat entfernt diese
konkreten Pfade, lokale Frames bleiben gleich.16Full-CPS-Läufe ergeben vier
Gruppenmittel grob+0,2…0,7%; kein zugesicherter oder isolierter Allocator-CPS-Anteil.
Kein konsistenter RSS-Vorteil. Mindestens77.824 bisherige Allocationrequests
und193.277.905B angeforderter temporärer Payload sind für eine frische
Referenzwelt aus Quelle+Maschinenkode+Headern ableitbar, kein dynamischer
Counter und kein CPS-Fenster. Gegenkosten4096B readonlyNullseite; Diskpadding
bleibt. [Übernahme/Ergebnis](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/69-FULL-CPS-RESULT.json).
[Aktuelle69-Belege](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/69-current-review/README.md).

Arbeitsfenster der abgeschlossenen27/69-Serien: Netflix-Freigabe ohne weitere
Rückfragen. Inzwischen ist mit25737f2 neue Checkoutaktivität erkennbar; vor
der nächsten Serie wurde die Freigabe wieder ausdrücklich geprüft. Der Nutzer
hat weiterhinNetflix und die Freigabe bestätigt; die26-Serie ist abgeschlossen.
Die folgenden26/87/SimplexSerien liefen noch im bestätigtenNetflixfenster.
Nach der erneuten Diskussion ist jetzt wie im /goal eine konkrete Anfrage für
24neueNative-Läufe gestellt (16Nulladdition+8SimplexOW16). Nutzerantwort:
**Jetzt keine Messungen; weiter analysieren.** Diese damalige Serie blieb
aufHold und ist inzwischen Reserve. Jetzt hat der Nutzer Gaming beendet und
ausdrücklich bestätigt: **„Ja, bis ich mich zurückmelde ohne Rückfragen messen“.**
Diese Freigabe galt für das beendete Schaufenster. Der Nutzer hat inzwischen
Discord/Spotify und ausdrücklich Research bis zur nächsten Entwarnung gemeldet.
In dieser Pause waren Builds, Tests und Messungen angehalten. Die laufende99-Serie wurde gezielt
gestoppt; sämtliche Teilläufe sind für Performanceentscheidungen ausgeschlossen.
Vorhandenes ASM und Quellen bleiben nutzbar. Historische Freigaben werden
nicht als aktuelle Zustimmung ausgegeben. **Aktuell hat der Nutzer für die
ganze Nacht freigegeben: Er geht schlafen, alles ist geschlossen. Gute,
validierte Änderungen dürfen übernommen und committed werden.** Die Nachtarbeit
beginnt mit einem neuen18e-Baseline-Paar; dessen normales natives Serverbinary
ist nach6m53 fertig und gesichert. Während Zeitmessungen
werden alle Builds und Agentanalysen angehalten. Die Produktionsquellen werden
für Experimente nicht geändert; eigener Worktree und Artefakte liegen außerhalbtarget.

Neuer Paritätsbefund: Bei geheimen Mansion-1x1-Räumen fehlt gegenüber der lokalen
Vanilla-Quelle ein vorausgehender bounded5-RNG-Draw. Konkrete Seed-Weltabweichung
noch ungeprüft; [Quellenbefund/Testgate](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/algorithms_storage-MANSION-RNG-GATE.md).
PoC65 darf Baselinegleichheit nicht als vollständigen Vanilla-RNG-Nachweis verwenden.
Eine neue [analytische Rejection-Fixture](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/mansion-rng-rejection-fixture-preparation/README.md)
erzwingt quellseitig genau zwei interne Schritte im bounded5-Aufruf; sie verlässt
sich nicht auf zufällig passende Testseeds. Drei echte Room-/RNG-Testquellen
und ein unabhängiger Deltareview sind vorbereitet, nicht ausgeführt. Der
Secret-Test erwartet weiterhin streng Vanilla-Gleichheit und soll die bestehende
Abweichung sichtbar machen; kein angepasster Sollwert oder gefundener Weltseed.

Weiteres bestehendes Paritätsgate bei Strukturreferenzen: Die aktuelle
Fastutil-Rekonstruktion überspielt bei Wachstum die Rehash-Historie; ein
hergeleiteter25-Key-Kollisionsfall hat laut exakter8.5.18-Quelle eine andere
Iterationsfolge. Zusätzlich kann bereits ein Zweier-Kollisionsfall durch den
bestehenden Steel-Caller mit zwei Set-Durchläufen die Reihenfolge ändern.
Dies sind Quellenzeugen, keine ausgeführten Ergebnisse oder gefundenen Weltseeds.
Ein echter-Jar-Java-Oracle und fünf getrennte Rustgates sind
[isoliert vorbereitet](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/structure-reference-order-oracle-preparation/README.md),
noch unkompiliert/ungelaufen. PoC29 ist davon getrennt ein Batchversuch gegen
die bisherige Steel-Baseline und behebt diese bestehenden Abweichungen nicht.

Neues Harness-Gate: Im geprüften Criterion-Worldfixture kann die Besitzkette
Map→Runtime→Task→Map den Refilltask über das Fixtureende hinaus erhalten.
Produktions-Shutdown beendet/entleert diese Arbeit ausdrücklich. Das betrifft
nicht automatisch jeden älteren Benchstand oderPR842. Der isolierte RAII-Entwurf
hat jetzt eine isolierte v2 mit sofortigem Guard an allen vier Aufbaupfaden und
drei echten World-/Ownership-/Unwind-Testquellen. Noch unkompiliert/ungelaufen;
die vier konkreten teuren Builder-Smokes bleiben zusätzlich offen. Bis zur
Validierung keine neuen Worldfixture-Timings als belastbaren Serververgleich verwenden.
[Grenzbereichsreview](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/boundary-REPORT.md).
[Harness-v2](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/boundary-pocs/criterion-world-owner-stop-v2/manifest.json).

Die verbliebenen Gates sind jetzt pro Bereich erneut gegen vollständige
Vorprüfungen, aktuelle Quellen und die tatsächlich vorhandenen Testentwürfe
abgeglichen: [Algorithmen](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/algorithms-validation-backlog-audit.md),
[Features/Surface](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/features-validation-backlog-audit.md),
[Chunks/Licht/Speicher](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/chunk-validation-backlog-audit.md).
Die Bereichszahlen sind kein addierbarer neuer Gesamtkatalog. Quellnegative
Varianten, Vertragsfragen und offene Ausführungen werden getrennt geführt;
vorbereitete Testquellen gelten weiterhin nicht als bestandene Tests.

## Historie und bisheriger Maßstab

Diese Datei ist der Einstieg für neue Threads. Vor jedem Versuch hier nachsehen,
anschließend Ergebnis und Beleg ergänzen – auch negative Ergebnisse. Sie deckt
Subsysteme und bekannte Versuche ab; **kein Bereich gilt als für jede Architektur
optimal bewiesen**. Nicht untersucht ist ein eigener Status, keine Erfolgsmeldung.

Nachtrag2026-11-07: [PR842-Abgleich](WORLDGEN-PR842-REVIEW.md).
Buffer-Reuse und gemeinsame Surface-Biomegruppen sind bereits in lokalem HEAD;
Klima-only-Cache überlappt47, Klimabaumänderung betrifft denselben Hotspot wie27.
Andere Ansätze und die neue Bulk-Noise-Korrektheit getrennt bewerten. Kein neuer
Build-/Messlauf im Rahmen dieses PR-Abgleichs; aktuelle Prioritäten siehe einzige Liste unten.

**Aktuell: Spielpause am 2026-11-06 beendet; Spotify ebenfalls geschlossen.**
47 bleibt als Speicheroptimierung und70 als Durchsatzoptimierung erhalten:
70 zeigt vier Gruppenmittel grob+0,9…1,4% gegen47.28/23/40/67 sind nicht
übernommen, ihre Quellen exakt zurückgestellt.67 ist in zwei Fassungen geprüft:
je39 Speichertests und22.500 Hashes, insgesamt24 Full-CPS-Läufe. Die bessere
ASM-Fassung spart eine272B-Kopie, liegt aber in drei Gruppen etwa0,4…0,8%
hinten, eine Gruppe ist praktisch gleichauf. Lokale Allokationsersparnis zählt,
kein stabiler RSS-/persistenter Heapvorteil. [Abschluss67](../worldgen-machine-audit-20261104/experiments/67-palette-live-20261106/RESULT.md).

Die40-Serie während des Spotify-Wechsels (vermerkt16:24UTC) ist vollständig
ausgeschlossen und ohne Spotify wiederholt. Dieser Absatz beschreibt den abgeschlossenen Stand vor dem Goal. Am2026-11-07 hat der Nutzer perf_event_paranoid auf1
gesetzt; eine neue Samply-Aufnahme des behaltenen47+70-Stands ist erfolgreich:
`full-cps/70-20261107-profile-w8`,66.049 persistierteFull-Chunks. Rohprofil,
Symbole undSHAs außerhalbtarget gesichert. Neue Hotspotgewichte sind im Goal-Abschnitt oben verlinkt; die unten stehenden
älteren Profilanteile bleiben historisch.
Während Zeitmessungen keine Builds/Profiler/Agent-Analyseläufe.

## Belege und Maßstab

Studie: [Experimente](../worldgen-machine-audit-20261104/experiments/), insbesondere
[Live-Ergebnisse](../worldgen-machine-audit-20261104/experiments/LIVE-RESULTS.txt),
[f64-Abschluss](../worldgen-machine-audit-20261104/experiments/F64-ERGEBNIS.txt),
[Laufzustand](../worldgen-machine-audit-20261104/experiments/RUN-STATE.json).
Ältere `candidates.json`-Patches basieren auf `37b738a3bc6`: nicht blind auf HEAD anwenden.
Diese Roadmap und die jüngeren Abschlussberichte berichtigen veraltete Zwischenstände.
Die Studie und das Übergabearchiv sind lokale, in Git ignorierte Belege außerhalb
`target/`. Diese Roadmap ist gezielt von der allgemeinen `docs`-Ignore-Regel
ausgenommen. Bei Übergabe in einen anderen Checkout die Belege separat mitgeben;
ein Dateilink allein archiviert sie nicht.

Gemessen: Ryzen 7 5800X3D, Zen 3, 8C/16T, SSSE3/AVX2; native Rust nightly
2026-10-14, LLVM 23.1.3, Release mit LTO, ein Codegen-Unit, Debugsymbole.
Full-CPS: 257×257 = 66.049 vollständige gespeicherte Overworld-Chunks,
8/16 Gen-Worker, Fenster32. Nicht gleichbedeutend mit Ticking-Bereitschaft.
Die Zeit endet nach der Generation, bevor der Save-/Unloadbacklog vollständig
geleert sein muss; Speicherung überlappt die Generation. Persistierter Full-
Status wird erst nach dem anschließenden Shutdown verifiziert. Drain/Shutdown
liegen außerhalb des CPS-Timers. [Messumfang](../worldgen-machine-audit-20261104/experiments/full-cps/METRIC-SCOPE.md).

Weniger Allokationen oder Speicher gilt laut Nutzer ebenfalls als Nutzen.
Angeforderte Bytes, lebendes Heap-Payload, Allokationszahlen, RSS und CPS getrennt
berichten; kleinere Objekte beweisen keinen geringeren Gesamt-RSS. Korrektheit
bleibt Pflicht, mögliche Durchsatzverluste gehören ausdrücklich in die Entscheidung.

Drei getrennte Aussagen dokumentieren:

1. **Maschinenbeleg:** Binary-SHA, Flags, Symbol/Adresse, tatsächlich erzeugte
   Instruktionen, Kontrollfluss, SIMD-Breite, Loads/Stores und echte Spills.
2. **Abschätzung:** unabhängige Ketten, kritische Abhängigkeiten, Häufigkeiten,
   mögliche Überlappung und zusätzliche Kosten. Zyklen nicht einfach aufsummieren.
3. **Messung:** lokale Kernelzeit oder Full-CPS ausdrücklich unterscheiden.
   ABBA/BAAB, mehrere Seeds, CPU-Zeit, Hintergrundlast und Streuung berichten.
   CPU-Summen aus250-ms-Monitorpunkten sind keine exakte Integration der
   vollständigen Pregen-Grenzen; kleine Unterschiede entsprechend relativieren.

Samply-Basis: `full-cps/accepted-profile-w8`, 451.801 Gen-Samples. Vor f64-Versuch14;
aktuelle Anteile können abweichen. Selbst-/inklusive Samples sind keine Aufrufzahlen;
inklusive Anteile überlappen. Keine I-Cache-/Branch-Miss-Messung daraus ableiten.

## Register bekannter Versuche

| ID / Bereich | Stand und Entscheidung | Beleg / erneuter Versuch nur wenn … |
|---|---|---|
| Ore-Z-Cache | Bereits in HEAD; nicht erneut als neue Idee testen | Historische `evidence/worldgen-ore-study`; aktuelle native Baseline erhalten |
| Beardifier-Y-Rejection | Bereits in HEAD | Historische Studie und aktuelle Quellen; Beiträge und Summenreihenfolge bewahren |
| Gradient-LUT mit SSSE3 | Bestehenden Intrinsic beibehalten | `swizzle-probe`: sichere `swizzle_dyn`-Variante scalarisiert trotz maskierter Indizes; hier kein Beleg für verbleibende Bounds-Panikchecks |
| 01 NoiseChunk-Bounds | Verworfen | Checks verschwinden, Hashes passen, aber kein reproduzierbarer Full-CPS-Gewinn; nur bei geändertem Compiler/Caller neu testen |
| 02 Main-XZ-Spalte | Historischer Entwurf; jetzt24 verwenden | Keine unabhängige Verbesserung neben07/24; Schreibfolge bewahren |
| 03 seltener Nonfinite-Pfad | Historischer Entwurf; auf HEAD als21 vorbereitet | `cold_path`; erst Layout und reale Pfadverteilung prüfen |
| 04 Aquifer-Select | Historischer Entwurf; auf HEAD als18/19 vorbereitet | Auswahlgleichstände und Rechenreihenfolge erhalten |
| 05 Spalten-Batching | Vorbereitet, ungemessen | Patch05; Restspalten und Ausgabereihenfolge prüfen |
| 06 sichere Swizzle-Kontrolle | Isoliertes Codegen schlechter | Gleiche Idee wie LUT-Kontrolle oben; kein Full-CPS-Gewinn behauptet |
| 07 Main-XZ-Cache | Durch aktuellen Quell-PoC24 ersetzt | Alten Patch nicht anwenden: würde den Y-Fastpfad14 teilweise umgehen |
| 08 Floor-Alternative | Durch spätere Floor-Arbeit überholt | Nicht unabhängig als neuen Gewinn zählen |
| 09 Floor nur Blended | Allein nicht übernommen | Lokaler Gewinn etwa15–17%; Full-CPS gemischt, teilweise schlechter |
| 10 Zero-Fudge | Vorbereitet, ungemessen | Patch10; ±0, NaN, Division und Vanilla-FP getrennt beweisen |
| 12 Normal-Y-Floor | Bestandteil14 | Lokale N4/N8-Gewinne sind kein zusätzlicher Full-CPS-Gewinn neben14 |
| 13 Stack-Y-Bereich einmal prüfen | Bestandteil14 | Exakter Wrap-Bereich, positive Frequenzen≤1, private Offsets[0,256); Fallback erhalten |
| 14 kombinierter f64-Floor | Übernommen, in HEAD | 22.500 Hashes;16 Full-CPS-Läufe,4 Gruppenmittel etwa+0,6…1,5%; keine garantierte Bandbreite |
| f32-Floor alte API | Isoliert geprüft, kein Produktionsversuch | `f32-followup-20261105`: API im geprüften Worldgen nicht aktiv; echte Koordinaten sind gemäß Vanilla f64 |
| 15 f32-Gradient-Bitvarianten | Beide verworfen | `15-gradient-probe`:5 Tests, ASM,32 kurze Läufe; Kernel-Durchsatz ca.4…17% schlechter, kein Full-CPS-Test |
| 16 Ore-Y-Cache (=Idee11) | Nicht übernommen; Quelländerung zurückgenommen | 22.500 Hashes, Clippy, ASM;8 Full-CPS-Läufe: +0,58% w8, +0,07% w16; w16 mehr Gen-CPU. Zweiter Seed fehlt |
| 18 Aquifer-Select | Quell-PoC; ungebaut/ungetestet | Aktuelle Variante04; Test aller5^6 Distanzfolgen vorbereitet; mehr CMOV-/Flag-Abhängigkeiten möglich |
| 19 Aquifer-Reject+Select | Quell-PoC; Alternative zu18/22 | Striktes `>` vor Selects; Gleichstände müssen eingefügt werden |
| 20 Ore-Z-Slice | Quell-PoC; kleine Reserve | SmallVec-Auswahl einmal binden; kann identisches ASM ergeben |
| 21 Nonfinite-Cold | Quell-PoC; niedrige Priorität | Aktualisierte03; passender skalarer ASM-Vergleich fehlt |
| 22 Aquifer-Reject+Branches | Quell-PoC; Reserve | Originaleinfügung erhalten, vorher nur sicher ausgeschlossene Kandidaten verwerfen |
| 23 Ore-Bitset-Inlining | **Am Maschinengate verworfen** | Nativer Aufrufer17944 B und Helfer804 B byteidentisch zur Referenz47; Calls und FP-Sicherungen unverändert. Kein neuer Hash-/CPS-Lauf. Quelle exakt zurückgestellt; gezieltes Erzwingen als separater70-Versuch. |
| 24 Main-XZ mit Y-Fastpfad14 | Quell-PoC; Reserve, ersetzt07/02 | Wiederverwendung über Y-Batches, geprüfter Fallback erhalten; Bereichs-/Tailtests vorbereitet; nur Formatierung und Patch-Anwendbarkeit geprüft |
| 25 Klima-Differenzen in i32 | Quell-PoC; Alternative zu27 | Bereich einmal prüfen, Quadrate/Summe bleiben i64; große Werte nutzen alten Pfad; zusätzliche Packs/Shuffles möglich |
| 26 Beardifier-XZ-Rejection | **Übernommen im Worktree** |4fokussierte Tests/22.500Stage-Hashes/Clippy/Fmt bestehen. FrüheXZ-Zweige sparen Convert/sqrt/div, +0 bleibt;36BmehrKode,56BFramegleich.16Paar- plus24Dreiarm-Läufe: acht positive26-Gruppenmittel, grob+0,2…0,4%;kein RSS-/Universalsieg. Gegen87 als kleinste gemeinsame Variante bevorzugt. |
| 27 Klima-Quadrat mit schmalem Eingang | **Vorerst nicht übernommen** |10Klimatests/22.500 Stage-Hashes bestehen.2statt4SIMD-Teilprodukte,Frame88Bgleich;lokal+28…34%Lookupdurchsatz,16 Full-CPS-Läufe nur+0,1…0,4% je Gruppe. Suchkörper zusammen+454B, Bencher-Aufrufer+112B Stack;kein Speichergewinn. Alternative25 bleibt separat ungeprüft. |
| 28 Aquifer-Similarity ganzzahlig entscheiden | **Nach Messung verworfen** | 22.500 Hashes und4 fokussierte Tests bestanden;8 Full-CPS-Läufe etwa−0,1%/+0,1%,kein belegter Speichergewinn,Stack+16 B. Zweiter Seed nicht durchgeführt;[Abschluss](../worldgen-machine-audit-20261104/experiments/28-aquifer-live-20261106/RESULT.md). |
| 29 Strukturreferenz-Batch | Quell-PoC; Reserve | Ein Tabellenaufbau pro Batch; Einfügereihenfolge, Duplikate und Iterator-Panikpräfix erhalten |
| 30 Template-State-Cache | Quell-PoC; Vertragsgate | Aktuelle Properties rein; allgemeine Reinheit zustandsbehafteter Plugins nicht zugesichert |
| 31 Jigsaw-Template-Pointerkey | Quell-PoC; Reserve | Geborgte immutable Identität statt Stringhash/Kopie; keine Pointerdereferenz, Cache nur pro Assembly |
| 32 Feature-Boxzähler | Quell-PoC; Reserve | X/Y/Z-Zähler statt Quell-Division/Modulo; Besuchs-/RNG-Folge und Overflowfallback; tatsächlichen Caller-ASM prüfen |
| 33 Geode-Lazy-Crack | Quell-PoC; Reserve | Crack-Summe erst im relevanten Shellzweig; Geometrieguard erhält alte Overflowfälle |
| 34 Geode-Noise-Arc | Quell-PoC; Reserve | Readonly Noise teilen statt tiefer Clone; Atomics/Cachekosten gegen kleinen Clone messen |
| 35 Tree-Frontier-Array | Quell-PoC; Reserve | Äußere Allokation entfernen, innere Sets/Reihenfolge gleich; möglicher Stacknachteil |
| 36 End-Geometrieguard | Quell-PoC; End-Reserve | 204–217 von625 Kandidatenslots sicher wirkungslos; keine Aussage für Overworld-CPS |
| 37 End-Sättigung | Quell-PoC; End-Reserve | Bei erreichtem Maximum80 keine weitere reine Noiseprobe nötig; Trefferquote fehlt |
| 38 Balancierte Spline-Auswahl | Quell-PoC; Reserve | Generator wählt bei ≥8 geeigneten Knoten balanciert; FP-Leafs/NaN/Ties erhalten; Compilerabbildung offen |
| 39 Normal-Y-Aufrufgrenze | Quell-PoC; Diagnose | inline(never) als Registerdruckversuch; keine entfernte Mathematik, Calls können verlieren |
| 40 Homogene Palette ohne Zwischenallokation | **Nicht übernommen trotz lokaler Allokationsersparnis** | ASM und 12 echte Probeaufrufe belegen den Vorteil; 26 Tests und 22.500 Hashes bestanden. 20 auswertbare Full-CPS-Läufe: w8 etwa −0,25…−0,45% und mehr Gen-CPU, w16 praktisch gleich. Vier Läufe wegen Spotify-Wechsel ausgeschlossen. Quelle zurückgestellt; [Abschluss](../worldgen-machine-audit-20261104/experiments/40-palette-live-20261106/RESULT.md). |
| 41 Flacher Palettenindex | Quell-PoC; Reserve | Sichere Arraysicht; Baseline bereits nur ein Check; mögliche Masken-/Shift-/LEA-Ersparnis |
| 42 Speicher-Packzähler | Quell-PoC; Reserve | Exakte Wort-/Bit-/Tailfolge; echte Division im Speichercaller noch nicht belegt |
| 43 Heightmap homogene Lage | Quell-PoC; Reserve | Nach oberster Lage homogene Section erledigt; Readguard/Prädikate erhalten; mixed-Regression prüfen |
| 44 Dripstone-Höhenmemo | Negativer Quellbefund; nicht für aktuelle Vanilla priorisieren | Extrahierte Bluntness≤1 erlaubt keine Wiederverwendung; nur geänderte Daten könnten Versuch rechtfertigen |
| 45 Light-Level≤1 | Quell-PoC; nachrangig | Source-Write/Recheck bleiben vor Stop; Endpunktfallback; tatsächliche Queuehäufigkeit eventuell null |
| 46 Scattered-State-Memo | Quell-PoC; Vertragsgate | Letzter Transform ohne HashMap; wie30 ist Property-Reinheit für Erweiterungen offen |
| 47 Eigener Klimacache | **Als Speicheroptimierung behalten** | 624 B kleinere Chunk-Sampler-Box; Heap-Payloadmodell −46,7 KiB pro Klimasampler. 22.500 Hashes und 6 gezielte Tests bestanden;16 Full-CPS-Läufe etwa −0,4…+0,1%, kein CPS-Gewinn und kein stabiler RSS-Vorteil. [Ergebnis](../worldgen-machine-audit-20261104/experiments/47-climate-live-20261106/RESULT.md) |
| 48 Monument-Fill-Clipping | Quell-PoC; Reserve | Zwei RNG-freie Helfer schneiden wirkungslose Außenbesuche ab; Orientierung/Visitfolge/Overflowfallback erhalten |
| 49 Terracotta-Bandoffset pro Spalte | Quell-PoC; Reserve | Lazy Cache im konkreten Surface-Systemadapter; gleiche FP-/Y-Indexformel und fremde-X/Z-Fallbacks; dynamische Wiederholungsquote fehlt |
| 50 Temperaturcache pro Spalte | Negativer Quellbefund für aktuelle Vanilla-Regeln | Kein Temperaturbedingungs-Knoten im aufgelösten Materialregelabschluss; keine aktuelle Ersparnis, Adapter kann Mehrkosten erzeugen |
| 51 Blattdistanz im vorhandenen Map-Eintrag | Quell-PoC; Reserve | Zweite HashMap entfällt; gesamte Weltlesespur/BFS/sortierte Writes erhalten, größeres Value als Risiko |
| 52 Mangrovenkandidaten im Array | Quell-PoC; Reserve | Höchstens zwei Positionen ohne kurzlebigen Vec; Positions-/RNG-/Iteratorfolge gleich, Caller-ASM fehlt |
| 53 Sculk-Nachbararray | Quell-PoC; Reserve | Feste18Offsets ohne Vec-Aufbau; unveränderte17Shuffle-Draws; Stack-/Kopierkosten prüfen |
| 54 Beehive-Guard vor Sortierung | Quell-PoC; Reserve | Leere Trunks/Chance/RNG/NaN gleich; Logs/Leaves nur bei bestandenem Guard sortieren; Featureabdeckung nachweisen |
| 55 Stronghold-Fehlschlagsmemo | Quell-PoC; Reserve | Nur reine Geometriefehler innerhalb unveränderter Auswahl merken; Roulette/Fallthrough/RNG bleiben; Wiederholungsquote fehlt |
| 56 Fortress-Fehlschlagsmemo | Quell-PoC; Nether-Reserve | Gleiche Idee getrennt, Indexfortschritt und Filler-RNG erhalten; Nullbit-Fallback bei größeren Enumwerten |
| 57 Monument-Suchstack wiederverwenden | Quell-PoC; Reserve | Scratch pro Raumgraph, vor jeder Suche clear+push; DFS/Markierungen/Short-Circuit unverändert |
| 58 Gepackte Lichtzeile extrudieren | Quell-PoC; Reserve | 128Paare packen+16Zeilen kopieren statt4096Nibble-Updates; Sichtbarkeit und Darstellung einschließlich Packed-Null erhalten |
| 59 Licht-Paketbytes direkt erzeugen | Quell-PoC; separates Streamingziel | Temporäre2048Byte-Box/Kopie/Freigabe vermeiden; Masken unverändert. Kein belegter Full-CPS-Pfad; beleuchteter Benchfixture fehlt |
| 60 Oberste Lichtsection aus Read-Snapshot merken | Quell-PoC; Reserve | Nur vollständige immutable Maps cachen, dynamischer Fallback sonst gleich; zusätzliche25Option-Slots/Initialisierung messen |
| 61 Iceberg-Winkel wiederverwenden | Quell-PoC; FP-Lowering-Gate | Baseline hat sincos-Call pro Ellipsenblock mit echtem Koordinatenspill; nachfolgende Rechnung bereits XMM-SIMD. Native Bitparität/Caller-ASM unbestätigt |
| 62 End-Spikes als Defaultarray | Quell-PoC; End-Reserve | Nur privat generierte10Spikes ohne Vec; beliebige Configliste unverändert, Seed/Shuffle/FP gleich; Stack-/Returnkosten offen |
| 63 DualNoise-Ursprungsprobe teilen | Quell-PoC; Reserve | Genau eine vollständige Noiseprobe pro Providerauswertung weniger; f32-Skalierung vor f64-Erweiterung, State-Auflösungen und Guards erhalten |
| 64 Carver-Registrycache | Quell-PoC; Reserve | Aktuelle identische Overworldlisten erlauben867→3Hashlookups; Keyvergleich und Fallback bleiben, keine entsprechende Zeitquote |
| 65 Mansion-Flachgrid | Quell-PoC; Reserve | Fünf private11×11Grids,55innere Vec-Anlagen weniger; logische In-place-/RNG-Folge erhalten; keine OOM-Gleichheitsgarantie |
| 66 Keine Stackarbeit ohne Exclusion | Quell-PoC; Reserve | Guard erst nach Cycle/Lookup/Rings/Placement, Some-Rekursion unverändert; positive Trefferquote fehlt |
| 67 Palette erst bei Neueintrag besitzen | **Beide Fassungen nicht übernommen** |Je39 Speichertests/22.500 Hashes,24 Full-CPS-Läufe. R2 entfernt272B-Kopie, behält952B-Frame/Misskopien; drei Gruppen−0,4…−0,8%,eine praktischgleichauf. Lokale128…224B-Allocation pro nichtleeremTreffer gespart,kein stabilerRSS-Vorteil. Quelle exakt zurückgestellt; [Abschluss](../worldgen-machine-audit-20261104/experiments/67-palette-live-20261106/RESULT.md). |
| 68 Licht beim Laden vor Boxbildung prüfen | Quell-PoC; separates Ladeziel | Homogene Bytes benötigen keine2048Byte-Box; Warnung/Darstellung gleich; mögliche große Stackkopie bei heterogenem Licht |
| 69 Statisches Sektor-Padding | **Als Allocation-Optimierung übernommen** |56Speichertests/22.500 Stage-Hashes/Clippy bestehen; spezifische Allocation/Free in zwei nativen Pollkörpern entfernt, je−112Codebytes, Frame gleich.16 Full-CPS-Läufe grob+0,2…0,7% je Gruppe, kein konsistenterRSS-Gewinn;4096BreadonlyPayload statt temporäremPuffer. |
| 70 Ore-Bitset gezielt immer inline | **Als Durchsatzoptimierung behalten** | Duplikatpfad umgeht Call und fünf FP-Saves/Reloads; Caller +4351 B, Stack +128 B. Vier Tests und 22.500 Hashes bestanden; 16 Full-CPS-Läufe, vier Gruppenmittel grob +0,9…1,4% gegen 47. Kein Speichergewinn. [Abschluss](../worldgen-machine-audit-20261104/experiments/70-ore-live-20261106/RESULT.md) |
| End-City-Kollision | Semantisches Prüfgate, kein Optimierungspatch | Vanilla prüft ersten Treffer, Steel beliebigen Nichtparent-Treffer; algebraischer Unterschied belegt, Erreichbarkeit durch echte Seeds offen |
| CountOnEveryLayer | Erweiterungsfall offen, kein Optimierungspatch | Java sampelt Count pro for-Bedingung, Steel pro Schicht; aktuelle8Nutzungen konstant. Eager Positionsliste durch echten Java-Caller bestätigt |
| Höhen-RNG erster Draw | Hypothese durch Baseline-ASM verworfen | LLVM entfernt temporären RNG-State und ungenutzte Zustandsupdates bereits; keinen duplizierten RNG-Helfer bauen |
| PGO-Diagnose | Vorbereitet, noch nicht ausgeführt | Passende LLVM-Tools installiert, `pgo-train.py` fertig; kein Trainingsprofil, kein PGO-use-Binary, kein PGO-Gewinn |

Versuch16: [Prüfung](../worldgen-machine-audit-20261104/experiments/16-ore-y-cache/PRUEFUNG.txt),
[Messungen](../worldgen-machine-audit-20261104/experiments/full-cps/16-seed-main-summary.json).
Y-Division verschwindet aus der X/Y-Schleife; dafür +1.732 Byte Funktionskode,
+144 Byte Stack und weitere Iterator-/SmallVec-Verwaltung. Stackzuwachs ist nicht
automatisch Register-Spillzuwachs. Maschinenverbesserung allein genügt nicht.

PoC28 wurde gegen47 vollständig auf Korrektheit geprüft und anschließend
**verworfen**:22.500 Hashes,vier fokussierte Tests,Clippy/Format bestanden;
acht ABBA-Full-CPS-Läufe etwa−0,1% mit8 und+0,1% mit16Workern. Keine belegte
Speicherersparnis,Stack+16B. Zweiter Seed nicht ausgeführt;Aquifer-Quellen exakt
zurückgestellt. [Abschluss28](../worldgen-machine-audit-20261104/experiments/28-aquifer-live-20261106/RESULT.md).

## Abdeckung des gesamten Ablaufs

Die folgenden Zeilen sind eine Arbeitskarte. „Offen“ bedeutet, dass noch kein
belastbarer Maschinen-/Gesamtlaufvergleich für diesen Bereich vorliegt.

| Bereich / Quellpfad | Vorliegender Hinweis | Nächste Untersuchung / zentrale Invariante |
|---|---|---|
| Orchestrierung: `steel-core/src/worldgen/{generator,stages}`, Chunk-Jobs | Historisches OW-Profil gut ausgelastet; aktuelles End-Profil mit deutlichem Scheduling-/Yield-Anteil, keine genaue Lock-Wartezuordnung | Privater kanonischer Light-Fensterentwurf unvollständig; Herkunft, Felder und Publikation vollständig beweisen. Kein allgemeiner Schedulergewinn belegt |
| Terrain/Density: `steel-worldgen/src/density`, generierte Density-Funktionen, Core-Stages | Terrain7,66% selbst/52,46% inkl.; NoiseChunk6,57/26,20%; Kombination3,10% selbst | Mehrfachberechnungen und Datenlebensdauer pro Zelle/Spalte erfassen; generierten Code über Generator ändern, FP-Summenfolge bewahren |
| Perlin/Normal/Blended: `steel-worldgen/src/noise` | scalar sample_and_lerp5,83%, Y4-Kernel2,05%, Normal8 2,73% selbst; Blended-Spalte1,50/3,80% |14/15 berücksichtigen; aktueller X/Z-PoC24 erhält Y-Fastpfad; Callsite-Spills und Restbreiten prüfen; Koordinatenpräzision nicht reduzieren |
| Simplex/End-Islands, PerlinSimplex |95 in144bce übernommen: rund1–4% vollständige End-CPS; Simplex2D9,37% Self im neuen Gen-Profil inklusive Startup |94+95,98 und Integerentry bleiben getrennte Reserven; native Gegenkosten und aktuelle Hashes erhalten, keine zusätzliche Gewinnzusage |
| Aquifer: `noise/aquifer.rs` | 4,04% selbst/6,47% inkl. |28 verworfen;22/18/19 bleiben ungemessene Alternativen; Auswahl, Cachezugriffe, Branch-Verteilung; Gleichstände, Flüssigkeits-/Druckgrenzen erhalten |
| Beardifier/Strukturen | Historischer Beard-Anteil3,41% Self;26R2 vom Nutzer übernommen, native aktuelle Stage-Hashes bestehen | XZ/Y-Guards und nachgewiesene FP-Invarianten erhalten. Eigenständiger26R2-CPS-Gewinn weiterhin ungemessen;87R1 Reserve,87R2 am Maschinenbeleg verworfen |
| Biome/Klima: `steel-worldgen/src/biomes`, `steel-utils/src/climate` | Historisch search_nearest2,49% Self;27 lokal deutlich schneller,16Full-CPS-Läufe nur etwa+0,1…0,4%, nicht übernommen |47 behalten;25/SearchContext/Unsafe-Childslice getrennte Reserven. Guard/Fallback, warmes Tie-Breaking und128Byte-Nodelayout erhalten |
| Oberflächen: `steel-core/src/worldgen/surface`, preliminary_surface_store | surface_rule2,11% selbst/7,73% inkl.; vertical_gradient1,61% |49 Bandcache vorbereitet;50 ohne aktive Temperaturbedingung negativ. Fuzzed-Minimumwahl bereits CMOV/VMIN; RNG-State bereits eliminiert. Keine Gewichtsübertragung auf einzelne Helper |
| Carver: `steel-core/src/worldgen/carver` |64 Lookupcache vorbereitet, weiterer Seedkoeffizientenansatz dokumentiert | FnMut-SkipChecker-/Masken-/Top-Material-Präfixe erhalten; Divisionen nicht ohne Caller-ASM und FP-Nachweis umformen |
| Features/Erze: `steel-core/src/worldgen/feature` | place_ore7,00% selbst/11,81% inkl. im alten Profil |23 wirkungslos,70 nach positiver Messserie behalten,20 Slice als Reserve;16 nicht ohne neue Hypothese wiederholen; RNG und Writes strikt erhalten |
| Ore-Veinifier: `noise/ore_veinifier.rs` | Offen; anderer Pfad als OreFeature | Eigenes Profil, nicht mit Feature-Erzanteil gleichsetzen |
| Andere Features/Vegetation |32–35/51–54/61–63 vorbereitet;44 mit Vanilla-Daten ohne Nutzen | Iceberg-sincos konkret belegt, DualNoise-Doppelprobe vermeidbar; End-Spikes getrennte Dimension. Provider-/Property-/Read-Verträge erhalten |
| Strukturplanung: `steel-worldgen/src/structure`, Core-structure/template |29/31/48/55–57/65–66 vorbereitet;30/46 mit Property-Vertragsgate | Flachgrid und unnötiger Exclusion-Stack; End-City-Kollision vor Optimierung reproduzieren; keine RNG- oder erste-Treffer-Umsortierung |
| Licht/Heightmaps/Postprocessing |43/45/58/60 vorbereitet; neue End-Lightstufe38,79% Gen inklusiv mit Startup, alter OW-Anteil10,7%; keine Übertragung auf Einzelhelper |99 historisch nativ korrekt, aktuelle CPS offen; neuer privater Fensterentwurf unvollständig. Darstellung, Light-Hash, Veröffentlichung und Randübergänge erhalten |
| Region/Blockzugriff: `steel-core/src/worldgen/region.rs`, Chunk-Speicher |40 geprüft und nicht behalten;41 vorbereitet | 40 spart nachweislich eine temporäre Allokation, Gesamtbilanz zu schwach; flache Indexierung kann Adressarbeit sparen, kein pauschaler Check-/Lockabbau |
| Speicherung/Kompression/Unload |67 geprüft/nichtübernommen;69 als Allocation-Optimierung behalten;68 separates Laden,59 zusätzliche Save-Callsite noch ungepatcht | Properties vollständig auswerten, I/O-/Fehlerfolge erhalten. wincode reserviert bereits; kein belegter zstd-Read-Zwischenbuffer. Snapshot/CPU-Kompression/I/O/Drain getrennt messen |
| Daten-/Codegenerierung: `steel-worldgen/build`, Registry-Build |47 als Speicheroptimierung behalten,38 vorbereitet; Generatorvariantenrisiken dokumentiert | Balancierte Splines und eigener Klimagraph; nur Generator ändern, keine generierten Dateien; Graphabschluss/FP/Seed beweisen |
| RNG/Mathematik: `steel-utils`, `steel-math` | Determinismus querschnittlich relevant | Erst gewichtete Aufrufer prüfen; keine Änderung von RNG-Verbrauch, FP-Reihenfolge oder exakten Wrap-/Floor-Grenzen |

## Architektur und Compiler

| Ziel | Aussage heute | Noch erforderlich |
|---|---|---|
| Zen3 /5800X3D native AVX2 | Einzige aktuelle Full-CPS-Messplattform | Neue Compilerstände und relevante Änderungen erneut kontrollieren |
| x86-64 ohne AVX2/SSSE3 | Sichere Fallbacks im Code vorhanden | Eigenständige Korrektheits-/Performanceprüfung; native Ergebnisse nicht übertragen |
| Andere AMD-/Intel-CPUs | Ungeprüft | Reale Hardware, passende Port-/Latenzdaten, gleiche Workloads |
| AVX-512 | Eigener Codepfad, hier nicht messbar | Separate Breiten-/Takt-/Registerdruck- und Hashprüfung |
| AArch64/NEON | Toolchain-Ziel vorhanden, kein Leistungsbeleg | Native Hardware und Assembly, Fallback-/SIMD-Gleichheit prüfen |

Zen3-Referenzen: [VDIVSD](https://uops.info/html-instr/VDIVSD_XMM_XMM_XMM.html#ZEN3)
≤13 Takte gemessene Latenz,4,5 Takte reziproker Durchsatz; dortige AMD-Angabe13/6,5.
[VCVTDQ2PS YMM](https://uops.info/html-instr/VCVTDQ2PS_YMM_YMM.html#ZEN3)
3/0,5. Abgerufen2026-11-05. Fremdmessungen für Zen3, keine CPS-Vorhersage für X3D.
Speicherlatenz, Abhängigkeiten, Ports, Frontend und Branches können dominieren.

PGO kann Inlining, Layout und Registervergabe anhand von Häufigkeiten beeinflussen:
[Rust](https://doc.rust-lang.org/rustc/profile-guided-optimization.html),
[LLVM-profdata](https://llvm.org/docs/CommandGuide/llvm-profdata.html).
Dauerhaft übertragbar sind eventuell getrennte seltene Pfade, gezieltes Inlining,
Spezialisierung und sichtbar gemachte Invarianten. Kein globales `inline(always)`;
kein Entfernen von Fallbacks wegen null Treffern im Training. Instrumentierte
Counter sind keine Instruktionslatenzen; instrumentierten ASM nicht als normalen
Register-/Spillbeleg verwenden. Normale Builds sollen keine Trainingsdaten benötigen.

Für die frühe Korrektheitsauswahl ist jetzt ein
[separater inkrementeller Testmodus](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/FAST-VALIDATION-WORKFLOW.md)
im Studienhelfer vorbereitet:16Codegen-Units,LTOoff,geringere Debugdaten,
eigener Profilcache. Nur Testziele, kein Server/Benchmark/PGO. Normale native
Vergleichsflags bleiben gleich. Der Modus wurde für beide95-Arme und99 benutzt;
die frühen Gates bestehen. Ein kontrollierter Buildzeitvergleich fehlt.
Die Nachtfreigabe erlaubt die Ausführung. Erst passende frühe Korrektheitsgates,
danach die erforderlichen nativen Gates für aussichtsreiche Kandidaten.

## Logger100: Ressourcenoptimierung ohne CPS-Zusage

100 verwendet auf Unix direkt die bestehende Live-Fensterabfrage. Zwei passende
native End257/w8-Aufnahmen belegen2.824eindeutige tput-Prozesse im95-Stand und
keine im Kandidaten. Alle318Warnpayloads inklusive Häufigkeit stimmen überein;
beide Läufe speichern alle66.049Chunks als Full und stoppen regulär. Periodische
Info-/Zeitmeldungen sind nicht bytegleich behauptet. Profilzeiten zählen nicht
als CPS. Der Vergleich über16unprofilierte End-/Overworld-Läufe mit8/16Workern
zeigt Gruppenmittel von etwa−0,5…+0,3%: kein belastbarer zusätzlicher CPS-Gewinn
und kein statistischer Äquivalenzbeweis. Die Übernahme erfolgt für die vermiedene
Prozessarbeit und entfernte tput-Abhängigkeit im Fehlerpfad, nicht für eine
höhere CPS-Zahl oder ungemessene Heap-/RSS-Ersparnis.

Normale native Logger-Libtests,15echte Kernel-Pipe-/PTY-/Controlling-TTY-Schritte,
Zeroachsen/Live-Resize, ein negativer Leerprotokollcheck, Format und Clippy
bestehen. Der erhaltene Repo-Testtreiber ist mit aktuellem nativem Testbinary
am endgültigen Pfad ausgeführt. Ein Docstring zur Benutzung ersetzt lediglich
Vorbereitungshistorie; die ausführbaren AST-Knoten bleiben gleich.

Bewusste UI-Randänderung: Nach Unix-ioctl-Fehlern werden die bisherigen80×30-
Defaults verwendet; ein möglicherweise erfolgreicher Terminfo-Fallback entfällt.
Erfolgreiche Live-TTY-Abfragen und Nullachsenbehandlung bleiben gleich; andere
Plattformen benutzen weiter `size()`. Windows ist hier nicht laufzeitgetestet.
Worldgen-/RNG-/FP-Code ist unverändert; für100 wird kein eigener neuer22.500-
Worldgen-Hashlauf behauptet. Die passende95-Quellbasis ist separat nativ gehasht.
[Prozessnachweis](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/100-NATIVE-MATCHED-PROCESS-PROOF.json),
[Warnparität](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/100-NATIVE-WARNING-PARITY.json),
[CPS und Grenzen](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/100-NATIVE-CPS-RESOURCE-RESULT.json).

## Nächste höchstens drei Versuche – aktuelle Reihenfolge

95 und100 sind abgeschlossen und übernommen. Die folgende Liste ersetzt
alle früheren Reihenfolgen. Neue Timer bleiben an die aktuelle Aktivitätsfreigabe
gebunden; eine spätere Nutzerpause überstimmt die derzeitige Nachtfreigabe.

1. **Kompakte Corner-Puffer V2 als Speicherarm prüfen.** Quellenmodell pro
   temporärem NoiseChunk:OW81.920→39.200B, Nether81.920→6.800B,
   End49.152→4.752B. Keine neue Noise-/Callback-Auswertung; Allokationsanzahl
   bleibt gleich. Keine gemessene Liveheap-/RSS-Ersparnis. Neue NoiseChunks
   entstehen je Terrainfill, ein Produktionspool ist bisher nicht vorhanden.
   Sieben Tests sind vorbereitet, nicht ausgeführt. Zero-Material-Kanäle sind
   von null aktiven Interpolatoren zu unterscheiden: Letztere können bereits
   im alten Shared-Hit-Pfad an `chunks_exact(0)` scheitern. Gültige Proofindizes
   müssen zur tatsächlichen Interpolatorzahl passen; bisheriges Padding ist
   kein freier zusätzlicher Kanal. Quellgrenzen, echte Tests und Hashes vor
   nativem Index-/Bounds-/Speicher-/CPS-Vergleich schließen.
   [Vorbereiteter Arm](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/night-corners-formatted-preparation/manifest.json).
2. **PGO für das gesamte native Programm konkret vorbereiten.** Das bereits
   source-geprüfte strikte N/G/U-Rollengate und der vorhandene Trainer sind
   die Basis; keine parallele neue Diagnosepipeline. Reale Build-/Umwelt-/
   Tool-/Quellidentitäten erfassen, Gate und Vergleichsintegration prüfen,
   erst danach Instrumentierungs-/Use-Builds und Training. End129 ist im
   Trainer vorbereitet, noch ohne beobachtete Abdeckung. Hashparität,
   erfolgreich geladene Profile, uninstrumentierter Maschinenkode und
   unabhängige Holdouts bleiben Pflicht. Keine PGO-Prozentzusage.
   [Offene Gates](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/high-impact-pgo-review/REPORT.md).
3. **Lichtfenster101: Vertragsentscheidung abwarten.** Der private Entwurf ist
   `contract_hold / inactive_source_draft`. Welt-/Positions-/Höhen-/Outer-Map-
   Prüfungen belegen keine begrenzten Weltzugriffe beliebiger öffentlicher
   Shape-/Light-Callbacks; vorhandene Registryinitialisierung bewahrt keinen
   abgefragten Vanilla-Herkunftsnachweis. Die konkrete Nutzerfrage zu einem
   verbindlichen Providervertrag ist offen. Keine Aktivierung, Ausführung,
   neue Foundation oder heuristische Fn-/Block-ID-Prüfung bis zur Klärung.
   Drei vorbereitete Gate-Tests sind kein Stage-/Providerbeweis.99 bleibt
   ein separater historisch nativ geprüfter Gate4-Arm; aktuelle CPS fehlen.
   [Vertragsbefund](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/light-canonical-window-source-trial/REVIEW.md).

Die vorhandene PGO-Vorbereitung ist weiterhin ungelaufen; ihre bisherigen
Quellenbelege bleiben erhalten.
Trainer mit vorbereitetem End129-Fall, gepinnte Backend-Quellen und striktes
N/G/U-Rollengate bleiben erhalten; Trainings-/Use-Builds, erfolgreiche
Profilzuordnung, Hashes und unabhängige Holdouts sind weiterhin offen.
[PGO-Gates](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/high-impact-pgo-review/REPORT.md),
[End-Training](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/pgo-end-mixed-training-preparation/manifest.json).

Die früheren26R2-/Simplex-/Corners-Prioritäten sind jetzt
[historische Reserven](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/MICRO-TRIAL-QUEUE-before-high-impact-focus.md).
26R2 ist vom Nutzer übernommen und hat weiterhin keinen eigenen gemessenen
CPS-Gewinn. Simplex behält seine32Läufe und das offene OW16-Gegenkostengate;
Cornersv2 hat sieben vorbereitete, null ausgeführte Tests. Die abgelehnte
24-Läufe-Anfrage bleibt eine Reserve und wird nicht automatisch nachgeholt.
Die neue Nachtfreigabe gilt für die aktuell begründeten Versuche; während
Zeitmessungen pausieren andere Builds und Analysen.
Der erneute [Corners-Abgleich](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/corners-current-evidence-decision/REVIEW.md)
bestätigt kleinere temporäre Puffer, keine entfernten Noise-Berechnungen.
16 gleichzeitige OW-Fills modellieren rund667,5KiB weniger angeforderten
Plane-Payload, keinen gemessenen Liveheap-/RSS-Gewinn. Zusätzliche sichere
Checks und Stride-Arithmetik bleiben native Gegenkostengates; Speicherreserve.

Weitere Bereiche bleiben abgedeckt: Terrain/Ore/Aquifer/Features und die
Chunk-/Speicherpipeline werden parallel auf große vollständige Arbeitsmengen
geprüft. Ein save-seitiger CCtx-Reuse-Entwurf ist eine Allocation-Reserve;
aktuell kein großer CPS-Anteil oder Schedulerstau belegt. Keine weitere
aktuelle Versuchsliste in Bereichsberichten. Material-Gap, Blended-Input-Slides,
Ore-Containment und Aquifer-Abschlüsse besitzen im vorhandenen Overworldprofil
keinen neu belegten großen entfernbaren Anteil. Noodle-Aux-Skips bleiben wegen
FP-Rekurrenz und öffentlichen Rawkanälen unimplementiert; einfache negative
Cornergrenzen reichen nicht. [Noise-Datenflussreview](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/high-impact-noise-density/REPORT.md).

Diese Liste ersetzt die abgeschlossenen26/27/40/67/69/87-Trialprioritäten.
26 und87 besitzen jetzt einen frischen24-Läufe-Dreiarmvergleich:
26 grob+0,2…0,4% gegen69 in allen vier Gruppen;87R1 bei8Workern etwa0%,
bei16Workern grob+0,4…0,5%. Gegen26 hat87R1 ein gemischtes Bild
(8Workernegativ,16Workerpositiv), jeweils nur2Werte/Arm/Gruppe.
26 ist nach unabhängigen Reviews im Worktree übernommen;
87R1 bleibt als gemessene Reserve außerhalb der Produktion. Kein Null-Effekt-/Signifikanzbeweis.
[Native Ergebnisse](../worldgen-machine-audit-20261104/experiments/goal-audit-20261107/87-FULL-CPS-RESULT.json).

87R2 ist am tatsächlichen Maschinengate verworfen: LLVM materialisiert pro
innerem Lookup576B auf dem Stack (18YMM-Loads+18Stores), Frame616B statt56B,
Kode2010B stattR1s1689B. Diese Arraykopie ist kein Registerspill. R2 wurde
folglich nicht gehasht oder zeitgemessen; Binary/ASM bleiben erhalten.
R1 besteht6fokussierte Tests,22.500Stage-Hashes undClippy; im warmen Lookup
entfallen sqrt/div ohne zusätzlichen Array-Bounds-Panic, aber mit Once-Check
und Tabellenload.576BPayload+Once/Layout und RO-Initializerkonstanten kommen
hinzu; kein Allocation-/Liveheap-/RSS-Gewinn.

47/69/70 sind behalten;23/27/28/40/67 nicht übernommen. Die
[frühere Begründung](../worldgen-machine-audit-20261104/experiments/NEXT-TRIAL-REVIEW-20261106.md)
beschreibt27/26/69 vor den jetzigen Messungen und bleibt historische Evidenz.
[Zen3-Quellen und Abschätzungsgrenzen](../worldgen-machine-audit-20261104/experiments/ZEN3-27-26-REFERENCES-20261106.md)
stehen separat. Der sichere SearchContext-Versuch und der Unsafe-PoC nur gegen private
Climate-Childslice-Checks bleiben unabhängige Reserven. Für Letzteren besteht
der statische Soundnessreview einschließlich u32-Wrap; Runtime-/Value-/öffentliche
Cachechecks bleiben, Kompilierung/ASM/Nutzen sind offen. PGO-Diagnose ist vorbereitet und ungelaufen;
Der private Light-Fensterentwurf braucht weiterhin vollständige Herkunfts-/
Feld-/Publikationsbeweise; ein allgemeiner Schedulerumbau ist nicht belegt.
Weitere End-/Monument-Kandidaten brauchen eigene regionale Häufigkeiten. Bei Zeitgleichstand
nur mit eigenständig belegtem Nutzen behalten; Regressionen explizit abwägen.
Kombinationen separat prüfen, keine universelle Optimalität behaupten.

## Pflege und Abschluss eines Versuchs

Jeder Eintrag erhält ID, Quellposition, Ausgangscommit, Hypothese, kleinsten Patch,
entfallende und zusätzliche Arbeit, Vanilla/RNG/FP-/Speicherinvarianten, Architektur,
Test-/ASM-/Messbelege, Entscheidung und Bedingungen für einen erneuten Versuch.
Tests vor Messung: relevante Grenzen/Bitgleichheit, bei Worldgen alle22.500
Stage-Hashes. Scheduleränderungen benötigen zusätzlich Parallelitätsnachweise;
sequentielle Feature-Hashes beweisen keine parallele Deterministik.

Rawprofile, Symbole, Disassembly und Vergleichsbinaries vor jedem Clean außerhalb
`target/` sichern. Keine alte Auswertung als vorhandenes Rohartefakt ausgeben:
Das ursprüngliche `worldgen-handoff.tar` enthielt keine alten nativen Binaries,
ASM-Dateien oder Samply-Rohprofile. Spätere Rekonstruktionen sind separat markiert.

Speicherstand vor heutigem Cleanup: Full-CPS-Verzeichnis ca.24GB, gesicherte
Artefakte5,8GB, `target/profiling`4,3GB. Acht neue16-Welten wurden nach Prüfung und
Archivierung ihrer STLR-Header entfernt (~4,8GB dezimal). Logs, Configs, Proc-Daten,
Binaries und Profile bleiben. Übrige ältere Welten zuerst inventarisieren; weder
Nutzerwelten noch Rohbelege pauschal löschen. Header belegen Full-Status, ersetzen
keinen Inhaltshash der gelöschten Weltkörper.

## Aktueller Stand der Quell-PoCs

[Paket18–69](../worldgen-machine-audit-20261104/experiments/gaming-pocs-20261105/README.md)
enthielt52 während der Spielpause vorbereitete Einzelpatches;70 ergänzt nun
einen getrennten Folgeversuch (53 Varianten insgesamt). Die Dateihashes halten
den jeweils passenden Ausgangsstand fest; zuletzt
neun weitere aus einer dritten Vertiefung mit drei Agents. Keiner ist im Produktionscode.
Alle Basis-/Kandidaten-/Patchhashes und die unabhängige Anwendbarkeit geprüft;
neue Quellen isoliert formatiert. Keine Typprüfung, Builds, Testläufe oder Messungen
während der Spielphase.47 ist inzwischen nativ geprüft und als Speicheroptimierung
behalten,28/23 sind verworfen. Vorbereitete Varianten sind keine nachgewiesenen Gewinne.

[Erste Vertiefung28–48](../worldgen-machine-audit-20261104/experiments/gaming-pocs-20261105/DEEP-REVIEW.md)
und [zweite Vertiefung49–60](../worldgen-machine-audit-20261104/experiments/gaming-pocs-20261105/DEEPER-REVIEW.md)
sowie [dritte Vertiefung61–69](../worldgen-machine-audit-20261104/experiments/gaming-pocs-20261105/DEEPEST-REVIEW.md)
verknüpft Beweise, konkrete Codepositionen, Risiken, Tests und Benchvorlagen.
[Dateiabdeckung](../worldgen-machine-audit-20261104/experiments/gaming-pocs-20261105/coverage-inventory.json):
310 handgeschriebene Rustdateien textseitig durchsucht;213 mit konkreter
Funktions-/Abschnittsanalyse,97 bislang nur Texttriage. Umfang jetzt einschließlich
der fünf Blockstate-Provider-Dateien, zuvor305Dateien. Keine vollständige
Zeilenprüfung und kein Optimalitätsnachweis. Nicht belegbare Ideen erhalten ein
Prüfgate oder einen negativen Eintrag, keinen erzwungenen spekulativen Patch.

Neue Benchquellen: End6Fälle, Palette/Heightmap8Fälle, Klima-Sampling12Full-/Subset-
Fälle; zusätzlich bestehende Blended-/Normal-/Klima-Lookup-Vorlagen. Alle ungebaut.
Die Fixture- und Timing-Gegenreviews sind dokumentiert. Nicht jeder PoC hat eine
eigene Microbench; relevante deterministische Referenz-/Hashprüfungen bleiben Pflicht.

[Maschinenbelege und Zen3-Szenarien](../worldgen-machine-audit-20261104/experiments/gaming-pocs-20261105/RYZEN-ESTIMATE.md),
[Klima/Beardifier/RNG-Nachprüfung](../worldgen-machine-audit-20261104/experiments/gaming-pocs-20261105/KLIMA-BEARD-RNG.md)
und die neuen Gruppenberichte trennen Baseline-ASM vom fehlenden Kandidaten-ASM.
Zuletzt nur2027Bytes des Iceberg-Blockhelpers dekodiert; einschließlich
der früheren sieben Funktionen9134Bytes, jeweils Build-ID geprüft. Keine umfangreiche
Disassembly. Beim Aquifer wurden vorhandene ASM-Dateien weiter ausgewertet.
Fuzzed-Minimumwahl bereits branchlos; Temperatur-Saves gehören den Cachemisspfaden.
Für neue VMINSD-/VDIVSS-Formen keine ungeprüften Zykluszahlen eingesetzt.
Iceberg verwendet bereits sincos und XMM mit zwei f64-Lanes; echter Spill über
den Call belegt.61 muss gleiche native FP-Abbildung nachweisen. Keine ungeprüften
VDIVPD-Zyklen oder einheitliche Cyclezahl für den sincos-Bibliotheksaufruf behauptet.

47/48 haben noch kein separat belegtes Gesamtzeitgewicht; daraus keine erfundene
CPS-Prognose. Für28 erklärt eine ausdrücklich angenommene5…20%geringere Aquifer-
Selbstzeit lediglich ein ideales Szenario von ungefähr0,2…0,8%Full-CPS. Null oder
Regression bleiben möglich. Es gibt keinen neuen gemessenen CPS-Gewinn.

Die neue Runde erzeugt nur kleine Quellkopien und Metadaten im ignorierten
Studienordner. Keine Binaries, ASM, Rohprofile oder Nutzerwelten gelöscht.
