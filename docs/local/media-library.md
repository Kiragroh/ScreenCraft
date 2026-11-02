# Medienbibliothek und getrennte Spuren (2026-11-02)

Die Medienbibliothek ist von Recorder und Editor über „Medien“ erreichbar.
Sie zeigt alle ScreenCraft-Pakete im aktuellen Aufnahmeordner, früher verwendeten
Aufnahmeordnern und favorisierten Ordnern, einschließlich Unterordnern und
unvollständigen Aufnahmen. Keine Begrenzung auf acht Projekte. Überlappende
Ordner werden dedupliziert; Verzeichnislinks/Junctions werden nicht verfolgt.
Nicht erreichbare Ordner werden angezeigt, nicht still als leer interpretiert.
Die Suche und Favoriten filtern die gemeinsame Liste. Metadaten werden im
Hintergrund geladen und bei sichtbarer Bibliothek alle fünf Sekunden erneuert.

Löschen verschiebt nach Bestätigung das vollständige Paket in
`.ScreenCraft-Trash` neben der Aufnahme. Wiederherstellen überschreibt nichts.
Es gibt keine endgültige Löschfunktion. Geöffnete Projekte und laufende
Aufnahmen sind vor Löschen geschützt. Der anwendungsinterne Papierkorb
funktioniert auch ohne Windows-Papierkorb auf Netzlaufwerken.

Vorhandene Aufnahmeformate werden beibehalten: screen.mp4, separate camera.mp4,
input.json und project.json. Mauspositionen/Klicks und Kamerazeitversatz sind
separat vom Bildschirmvideo gespeichert. Die vorhandenen Editor-Panels erlauben
Kameraposition/Größe/Form/Sichtbarkeit und Mausgröße/Stil/Glättung/Klickeffekte.
Die Webcam muss vor der Aufnahme eingeschaltet werden; sie wird nicht automatisch
aktiviert. Ältere Aufnahmen erhalten keine fehlenden Spuren nachträglich.

Kamerastartfehler brechen den Start mit einer sichtbaren Fehlermeldung ab.
Fehler beim Abschließen der Kamera werden gemeldet und zusätzlich im Paket als
recording-warnings.txt vermerkt; das Bildschirmprojekt wird trotzdem gespeichert.
Beim Stoppen werden gepufferte Kamerabilder vollständig abgearbeitet.

Prüfung: Bibliotheks-Tests für mehr als acht Aufnahmen, überlappende Ordner,
Prefs-Migration, vollständiges Löschen/Wiederherstellen und Namenskollisionen.
Synthetischer Kameratest prüft zwölf separat codierte Bilder, Zeitversatz und
unveränderte Bildschirm-/Mausdateien. UI-Vorschau verwendet ausschließlich
synthetische Testpakete. Keine automatische echte Desktop-/Webcamaufnahme.

Verifikation lokal:
- 11 App-Tests bestanden (einschließlich drei Bibliothekstests).
- Separater synthetischer Kameratest bestanden: zwölf Bilder und -50 ms Offset.
- Native Bibliotheksvorschau: zwölf Aufnahmen sichtbar, Favorit zeigt sechs.
- Der native Lösch-Klicktest wurde nicht abgeschlossen; Dateiebene mit
  Trash/Restore und Kollisionsschutz ist getestet.
- Keine echte Webcam-/Desktopaufnahme für die Prüfung gestartet.
