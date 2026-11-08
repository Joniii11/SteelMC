# PR842: Abgleich mit lokaler Optimierungsstudie

Geprüft2026-11-07: PR-Head `1554078be5fa2c993d3137dc15c0dd3e993470b4`,
Base `b7e2a97f3f2219c462ba58fc60cb755b54874d5a`; lokaler HEAD79b3a521241
plus behaltene47/70. Quelle: https://github.com/Steel-Foundation/SteelMC/pull/842
und Diskussion4498735535. Nur Quellen/Diff/Studien gelesen; keine neuen Builds,
Benchmarks, Profile oder GitHub-Kommentare. Rohsnapshot lokal ignoriert unter
worldgen-machine-audit-20261104/pr842-review.json.

| PR-Änderung | Lokaler Stand / Bewertung |
|---|---|
| Interpolationsbuffer außerhalb Zellschleife | Bereits in HEAD: noise_chunk.rs:336, Commit37b738a3bc6. Nicht nochmals als zusätzlichen Gewinn zählen. |
| Uniforme Surface-Biomegruppen teilen | Bereits in HEAD: Commit27150f8093c, SurfaceQuartBiomes::uniform_group_biome. Lokale Variante nutzt Unread/Mixed/Uniform(u16), PR nutzt u16-Sentinels. Dieselbe vermiedene Klassifikationsarbeit; Sentinel-/Layouttausch separat bewerten. |
| Klima-only Cache | Wesentliche Überschneidung mit behaltenem47. Lokal eigener OverworldClimate-Graph, Cache und Noisebesitz; PR gemeinsame Typen mit ensure_climate/init_grid_climate und getrennten Validitätsflags, auch Nether. Funktionsziel ähnlich, Speicherwirkung nicht gleich: PR reduziert Cacheberechnung, implementiert nicht unseren getrennten Noisebesitz. Nicht Gewinne addieren oder lokale Typen blind zurückstellen. |
| Klimabaum: partielle Distanz und Offsetterm | Selber Hotspot wie25/27, andere Umsetzung. PR bricht pro Achse bei erreichtem Bound ab und berechnet den Offsetterm bei Aufbau;27 versucht kleinere Multiplikationseingänge unter geprüftem Bereich. Aktuell nicht implementiert. Neue Branches können bisherige SIMD-Ketten verändern; separater nativer ASM-/Timingvergleich erforderlich. TargetPoint::to_parameter_array liefert lokal tatsächlich Offset0. |
| Aquifer: Solid-Guard inline, Rest separater Helper | Neue Aufteilung, keine Wiederholung unserer18/19/22/28-Auswahl-/Similarityversuche. Auch nicht unser Ore-Inlining70. |
| Ore-vein Ergebnisbuffer einmal anlegen und fill(None) | Neu im lokalen Stand. Betroffen ist Surface-Materialbuffer, nicht OreFeature-Bitset/Z-Cache. Aktueller PR verwendet bereits Box<[Option<BlockStateId>]>, nicht mehr Vec. |
| Heightmaps spaltenweise aktualisieren | Neu. Eigenständiger Schritt, nicht unser43-Entwurf zum Überspringen homogener Sectionlagen. |
| Heightmap-Priming über flache Sectionwerte | Selber Bereich wie43/41, anderer Ansatz: direkte Building-/Heterogeneous-Slices statt get_at_index pro Block. Nicht lokal übernommen. |
| Bulk-Noise Koordinatenreihenfolge | Neue Korrektheitsänderung; keine Wiederholung unseres f64-Floors14. Lokales Vanilla NoiseStack.addToVolume reicht scale*frequency weiter, Point-get multipliziert Koordinaten mit frequency. PR trennt Aufrufer. Vollständiger Vanilla-Jar-Abgleich von uns nicht durchgeführt; PR nennt ihn nach dem Fix selbst noch offen. Stage-Hashes allein sehen diese behaupteten ULP-Unterschiede laut PR nicht. |

## SmallVec-Kommentar

Lokal generierter Overworld-Code liefert material_ore_vein_value_count()=4;
vanilla.rs teilt durch2, also2Ergebnisse. Der bisherige SmallVec hat genau
2Inlineplätze und benötigt für diesen Buffer im heutigen Overworld keine
Heapablage. Der PR-Boxbuffer legt dagegen Heapstorage einmal pro Chunk an.
Bei0Einträgen ist keine Payloadallokation erforderlich; größere Erweiterungs-
listen können beim SmallVec ohnehin spillen.

Dafür entfernt der PR clear+resize aus dem Blockpfad und benutzt fill(None).
Ein Vergleich einmal dimensionierter SmallVec+fill gegen Box+fill ist nötig,
um Containerwahl und Schleifenänderung auseinanderzuhalten. Instruktionszahlen
sind brauchbare Arbeitshinweise, aber keine Zyklen/CPS. Ebenso beweist vorhandene
Stackablage keine höhere Geschwindigkeit. Unsere verworfene67-Variante zeigt
konkret mögliche SmallVec-Kopier-/Stackkosten, betrifft aber einen anderen
Buffer und beweist hier keine Regression.

## Messung und Integrationsentscheidung

PR nennt −12,6% Callgrind-Instruktionen und ungefähr151→142ms im Ein-Chunk-
Pipelinebenchmark einschließlich niedrigerer Nachbarstatus. Das ist kein
gemessener Full-Pregen-CPS-Gewinn auf unserem5800X3D; die Beschreibung nennt
zudem etwa±10% Streuung und Timings vor dem Klima-only-Zusatz.

Beim Integrieren zuerst die zwei bereits vorhandenen HEAD-Optimierungen
abgleichen, Klima47 bewusst zusammenführen, die übrigen Änderungen getrennt
prüfen. f64-Floor14 und Ore-Inlining70 nicht als PR-Duplikate entfernen.
Die Korrektheitsänderung braucht rohe Noise-Bitvergleiche gegen Vanilla,
zusätzlich zu Stage-Hashes. Die damalige Reihenfolge27/26/69 ist historisch. Die einzige aktuelle
Versuchsliste steht in [der Roadmap](WORLDGEN-OPTIMIZATION-ROADMAP.md);
dieser Quellenreview startet keine Messserie und behauptet keine heutige
PR-Head-Aktualität.
