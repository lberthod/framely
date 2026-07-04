# Framely

Enjoliveur de screenshots pour macOS. Transforme n'importe quelle capture d'écran en visuel soigné en moins de 3 secondes, sans réglage.

> Nom de travail : Framely (anciennement pistes : Snapfini, Cadre, Halo, Poser).

## Pitch

**Beau par défaut.** L'utilisateur ne doit rien régler pour obtenir un bon résultat — les réglages existent pour affiner, jamais pour construire. Le chemin `capture → joli → collé` est optimisé à l'extrême.

App desktop native macOS, écrite en Rust, distribuée en `.dmg`.

## Positionnement

| | Framely | Xnapper | CleanShot X | Shottr |
|---|---|---|---|---|
| Cœur | Enjolivement instantané | Enjolivement | Suite complète | Capture légère |
| Priorité n°1 | Vitesse ressentie | Bonne | Moyenne (lourde) | Bonne |
| Prix | 12–19 $ one-time | ~15 $ | ~29 $ / abo | Gratuit / 8 $ |
| Empreinte | Ultra-légère (Rust) | Moyenne | Lourde | Légère |
| Angle | Rapidité + simplicité + prix | Beauté auto | Tout-en-un | Minimalisme |

On ne cherche pas à battre CleanShot sur les fonctionnalités, mais à être le plus rapide et le plus simple pour la tâche « rendre une capture présentable ».

## Fonctionnalités

### MVP (v0.1) — le socle vendable

**Entrée**
- Capture de zone via raccourci global (⇧⌘2 par défaut)
- Capture de fenêtre (détection auto des fenêtres au survol)
- Glisser-déposer d'une image sur l'app ou son icône Dock
- Coller depuis le presse-papiers (⌘V)
- Import fichier (PNG/JPEG/HEIC)

**Enjolivement**
- Fonds : galerie de dégradés préréglés (12–16 presets), couleur unie, transparent, image perso
- Marge (padding) : curseur 0–200 px, presets S/M/L
- Coins arrondis de la capture (rayon réglable)
- Ombre portée douce (intensité réglable)
- Auto-balance : marge + fond harmonieux choisis automatiquement à l'import
- Ratios export : Auto, 16:9, 1:1, 4:3, 3:2 (+ présets réseaux : X, Instagram, LinkedIn)
- Échelle @2x (Retina)

**Sortie**
- Copie presse-papiers (⌘C) — geste le plus fréquent
- Export PNG/JPEG (⌘S), dernier dossier mémorisé
- Drag-out de la preview vers une autre app

### v1.0 — pour justifier l'achat

- Cadre de fenêtre macOS stylé (barre de titre, feux tricolores)
- Annotations légères : flèche, rectangle, texte, surlignage
- Floutage / pixellisation de zones sensibles (argument confidentialité)
- Presets de marque (couleurs, dégradés, logo, watermark)
- Extension Finder : clic droit → « Enjoliver avec Framely »
- Historique des dernières captures

### Roadmap (v2+)

Templates réseaux sociaux, export par lot, mockup 3D léger, mini-enregistrement écran→GIF, OCR, sync des presets iCloud.

## Raccourcis clés

| Action | Raccourci |
|---|---|
| Capturer une zone | ⇧⌘2 |
| Capturer une fenêtre | ⇧⌘4 |
| Coller une image | ⌘V |
| Copier le résultat | ⌘C |
| Exporter | ⌘S |
| Annuler / Rétablir | ⌘Z / ⇧⌘Z |
| Cycler les fonds | ← / → |
| Réinitialiser (auto-balance) | ⌘R |
| Fermer sans enregistrer | ⌘W / Échap |

## Les 3 obsessions de conception

1. **Rapidité** — capture → collé en < 3 s, 2 raccourcis.
2. **Zéro friction** — résultat superbe sans aucun réglage.
3. **Légèreté** — reste en menu bar sans peser, lancement instantané (< 200 ms).

Voir [ARCHITECTURE.md](ARCHITECTURE.md) pour le détail technique et [SPRINT.md](SPRINT.md) pour le plan d'exécution.

## Statut

Projet en phase de cadrage. Pas encore de code — voir SPRINT.md, Sprint 0.
