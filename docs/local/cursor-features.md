# Nachträglich bearbeitbare Mausdarstellung

Im Editor unter Cursor stehen Dunkel, Hell, Farbe, Punkt und Laserpointer zur
Verfügung. Größe: 5–160 pt. Zeigerfarbe lässt sich für alle Stile frei wählen und
auf die jeweilige Standardfarbe zurücksetzen. Der Laser hat einen farbigen Kern,
einen hellen Mittelpunkt und einen weichen Leuchteffekt. Optional kann ein
Hervorhebungskreis mit eigener Farbe, Radius und Deckkraft eingeschaltet werden.
Klickeffekte und deren Farbe bleiben separat einstellbar.

Änderungen werden im Projekt gespeichert, unterstützen vorhandenes Undo/Redo
und verwenden für Vorschau und Export denselben Compositor. Bildschirmvideo und
Mausdaten bleiben unverändert. Alte Projekte ohne color-Feld behalten ihre
bisherige Farbpalette. Voraussetzung ist eine Aufnahme mit separaten Mausdaten;
ein bereits in fremde Videos eingebrannter Mauszeiger wird damit nicht entfernt.

Prüfung: sieben Renderingtests auf dem echten Compositor bestanden, darunter
Farbwechsel, Größenwirkung, zentrierter Laser bei verschiedenen Cursorformen
und vorhandene Klickdarstellung. Ein zusätzlicher Test prüft alte Einstellungen
und verlustfreies Speichern/Laden der neuen Farbe, Größe und Laser-Auswahl.

Videoexport geprüft: synthetische 12-Sekunden-Aufnahme mit grünem Laser,
1280×720, 30 fps, 360 Bilder. ffprobe bestätigt Dauer/Bildzahl; vollständiges
Decodieren mit ffmpeg ohne Fehler. Exportbild visuell kontrolliert.
Nachweise: ScreenCraft-setup/cursor-render-tests.log, cursor-config-tests.log,
cursor-export.log und cursor-proof/laser-export.mp4.
