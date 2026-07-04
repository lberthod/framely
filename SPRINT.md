# Plan de sprints — Framely

Cycles de 1 semaine. Objectif : arriver à un MVP vendable (v0.1) le plus vite possible, puis v1.0.
Voir [ARCHITECTURE.md](ARCHITECTURE.md) pour le détail technique des modules cités ici.

## Sprint 0 — Cadrage & squelette projet (semaine 1)

Objectif : projet qui compile, fenêtre vide qui s'ouvre, décisions de nom/design figées.

- [x] Trancher le nom de travail (Framely retenu par défaut) — réservation domaine / bundle id encore à faire par toi
- [x] Initialiser repo git + workspace Cargo (`framely-app`, `framely-core`, `framely-render`, `framely-capture`, `framely-io`, `framely-presets`)
- [x] Squelette `eframe`/`egui` : fenêtre principale avec layout preview (gauche) + panneau réglages (droite) + barre d'état, sans logique
- [ ] Icône app + identité visuelle de base (palette, typo SF Pro)
- [x] Choix définitif des libs : `tiny-skia`, binding ScreenCaptureKit (à venir Sprint 2), `arboard`/NSPasteboard (à venir Sprint 2)
- [x] Setup CI basique (build + clippy + tests)

**Sortie du sprint** : `cargo run` ouvre une fenêtre avec le layout statique de l'éditeur.

## Sprint 1 — Pipeline d'enjolivement (semaine 2)

Objectif : à partir d'une image chargée en dur, obtenir le rendu enjolivé complet.

- [x] `framely-core` : types `Document`, `Style`, `Background`, `Ratio`, `ShadowParams`
- [x] `framely-render` : composition fond (dégradé/couleur/transparent) + marge + coins arrondis + ombre via `tiny-skia`
- [x] Upload de la texture rendue dans `egui` (`TextureHandle`) et affichage dans la zone preview
- [x] Panneau réglages fonctionnel : sliders marge/coins/ombre, sélecteur de fond, mis à jour en direct (pas de bouton Appliquer)
- [x] Fonction `auto_balance()` : valeurs par défaut esthétiques appliquées à l'ouverture d'une image
- [x] 14 presets de dégradés en place (`framely-presets`) — validation esthétique définitive encore à faire par toi

**Sortie du sprint** : on peut charger une image de test et jouer avec tous les réglages visuels en direct.

## Sprint 2 — Entrées & sorties réelles (semaine 3)

Objectif : le flux critique complet fonctionne avec de vraies captures.

- [x] `framely-capture` : capture d'écran réelle via ScreenCaptureKit (écran entier, fenêtre précise, zone rectangulaire par coordonnées) — testé en conditions réelles sur cette machine (capture 3440×1440 + liste de 39 fenêtres)
- [x] Capture de fenêtre — sélection via une liste cliquable des fenêtres visibles (`list_windows`), pas encore par survol avec surbrillance (voir gap ci-dessous)
- [x] Raccourci global ⇧⌘2 (zone = écran entier pour l'instant) et ⇧⌘4 (fenêtre) enregistrés via `global-hotkey`
- [x] `framely-io` : coller (⌘V) depuis presse-papiers (arboard, testé en réel), import fichier PNG/JPEG (HEIC non supporté par la crate `image`, backlog)
- [x] `framely-io` : copier le résultat (⌘C) vers presse-papiers (testé en réel), export fichier PNG via dialogue natif (`rfd`)
- [x] Menu bar (`tray-icon`) : icône + menu Capturer zone / Capturer fenêtre / Coller / Quitter (pas encore d'entrée "Réglages" séparée, les réglages sont déjà dans le panneau principal)
- [ ] Gestion permission « Capture d'écran » : le mapping d'erreur existe (`CaptureError::PermissionDenied`) mais le fallback UX (message clair + bascule mode import) n'est pas encore affiné

- [x] Overlay plein écran interactif pour la sélection de zone par glisser-déposer (assombrissement, rectangle de sélection, dimensions en direct, Échap pour annuler) — implémenté via le support multi-viewport natif d'egui (`show_viewport_immediate`), branché sur `framely_capture::capture_region`

**Gaps connus, non couverts par ce sprint (à traiter avant v0.1) :**
- L'overlay se place sur l'écran principal par défaut (`with_fullscreen` sans écran ciblé explicitement) — pas nécessairement celui sous le curseur en configuration multi-écrans.
- Pas de détection de fenêtre au survol avec surbrillance — la capture de fenêtre passe par une liste cliquable.
- Le drag-out (glisser la preview vers une autre app) n'est pas branché (prévu Sprint 3, nécessite un pont `objc2`/`NSDraggingSession`).
- Import HEIC non supporté (la crate `image` ne le décode pas).
- Identité visuelle de l'icône menu bar toujours provisoire (carré arrondi indigo procédural).
- **Le geste de glisser-déposer de l'overlay n'a pas pu être testé interactivement de bout en bout** : l'app n'étant pas encore empaquetée en `.app`, je n'ai pas pu lui accorder l'accès aux outils de contrôle d'écran pour simuler un vrai drag souris, et l'automatisation de la frappe clavier système (`osascript`) a été bloquée par macOS (permission Accessibilité non accordée). Le code compile, type-check contre l'API réelle d'egui/screencapturekit, et l'app démarre sans crash avec l'overlay chargé — mais le geste lui-même reste à valider manuellement.

**Sortie du sprint** : flux bout-en-bout fonctionnel — ⇧⌘2 → sélection interactive d'une zone → capture réelle → éditeur auto-enjolivé → ⌘C → collable ailleurs. À valider manuellement : le geste de glisser-déposer de l'overlay (voir gap ci-dessus).

## Sprint 3 — Finitions MVP & robustesse (semaine 4)

Objectif : transformer le prototype fonctionnel en produit livrable v0.1.

- [x] Ratios export (Auto, 16:9, 1:1, 4:3, 3:2 + presets réseaux X/Instagram/LinkedIn) — X et Instagram réutilisent les mêmes proportions que 16:9/1:1 (pas de ratio dédié différent), LinkedIn a son propre 1200:627
- [x] Échelle @2x à l'export (fait au Sprint 1)
- [x] Undo/redo (⌘Z/⇧⌘Z) sur la pile `Style` + boutons dédiés, réinitialisation (⌘R) vers l'auto-balance. Les curseurs (marge/coins/ombre) ne poussent qu'une entrée d'annulation par geste de glisser (pas une par frame) via `Document::commit_history` — testé (`framely-core` : 4 tests unitaires)
- [ ] Drag-out de la preview vers une autre app — toujours différé, nécessite un pont `objc2`/`NSDraggingSession` non trivial
- [x] Persistance des derniers réglages (ratio/échelle/fond) et dernier dossier d'export entre sessions, dans `~/Library/Application Support/Framely/settings.json` — testé en écriture/lecture disque réelles
- [x] Cas limites : damier de transparence dans la preview (fond réellement transparent à l'export), downscale de la preview pour les images > 2200px (l'export reste pleine résolution) — testés unitairement (`framely-app::preview`, 4 tests). Presse-papiers vide déjà géré (Sprint 2, `IoError::ClipboardEmpty`).
- [ ] Multi-écrans/échelles mixtes : pas encore de gestion dédiée (l'overlay et les captures ciblent l'écran principal par défaut)
- [~] Accessibilité de base : navigation clavier complète sur les réglages (curseurs, combos, boutons) déjà supportée nativement par `egui`/`accesskit` (VoiceOver) sans code dédié ; le respect de « Réduire les animations » n'a pas été vérifié explicitly
- [ ] Onboarding minimal (écran de bienvenue avec les 2 raccourcis clés) — pas fait, l'app démarre directement sur l'éditeur avec l'image de démo
- [x] Raccourcis clavier du README implémentés dans l'app : ⌘Z/⇧⌘Z/⌘R/⌘C/⌘V/⌘S/←→ (⇧⌘2/⇧⌘4 déjà globaux depuis le Sprint 2)
- [ ] Tests manuels du parcours complet sur plusieurs configs (mono/multi-écran, Retina/non-Retina) — non faits, nécessite du matériel supplémentaire ou une validation par toi

**Sortie du sprint** : v0.1 complète et testée, prête pour packaging.

## Sprint 4 — Packaging & lancement (semaine 5)

Objectif : `.dmg` distribuable, prêt à vendre.

- [ ] Bundling `.app` (Info.plist, entitlements, icône finale)
- [ ] Signature développeur Apple + notarisation + stapling
- [ ] Génération `.dmg` avec fond personnalisé soigné
- [ ] Vérification lancement < 200 ms et empreinte mémoire ~10–20 Mo (mesure réelle, pas estimation)
- [ ] Page produit / landing minimale (argument vitesse + beau par défaut + comparatif concurrents)
- [ ] Beta privée (quelques testeurs) avant mise en vente

**Sortie du sprint** : v0.1 vendable, en ligne.

## Backlog v1.0 (post-MVP, à planifier une fois v0.1 validée sur le marché)

- Cadre de fenêtre macOS stylé (barre de titre + feux tricolores) autour de la capture
- Annotations légères : flèche, rectangle, texte, surlignage
- Floutage / pixellisation de zones sensibles
- Presets de marque (couleurs, dégradés, logo, watermark)
- Extension Finder (clic droit → « Enjoliver avec Framely »)
- Historique des dernières captures

## Backlog v2+ (roadmap long terme)

Templates réseaux sociaux, export par lot, mockup 3D léger, mini-enregistrement écran→GIF, OCR, sync des presets iCloud.

## Règle de priorisation

À chaque arbitrage, trancher dans cet ordre :
1. Est-ce que ça sert la **rapidité** du flux capture → collé ?
2. Est-ce que ça sert le **beau par défaut** (zéro réglage requis) ?
3. Est-ce que ça garde l'app **légère** (menu bar, lancement instantané) ?

Toute fonctionnalité qui ne sert aucun de ces trois points passe en backlog v2+.
