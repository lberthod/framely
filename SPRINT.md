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

- [ ] `framely-capture` : capture de zone via ScreenCaptureKit + overlay plein écran (assombrissement, réticule, dimensions live)
- [ ] Capture de fenêtre (détection au survol + surbrillance)
- [ ] Raccourci global ⇧⌘2 (et ⇧⌘4 pour fenêtre) enregistré au niveau système
- [ ] `framely-io` : coller (⌘V) depuis presse-papiers, glisser-déposer sur fenêtre/Dock, import fichier PNG/JPEG/HEIC
- [ ] `framely-io` : copier le résultat (⌘C) vers presse-papiers, export fichier (⌘S) avec mémorisation du dernier dossier
- [ ] Menu bar (`NSStatusItem`) : icône + menu Capturer zone / Capturer fenêtre / Coller / Réglages / Quitter
- [ ] Gestion permission « Capture d'écran » (demande, refus → fallback mode import)

**Sortie du sprint** : flux n°1 complet — ⇧⌘2 → sélection → éditeur auto-enjolivé → ⌘C → collable ailleurs, en moins de 3 secondes.

## Sprint 3 — Finitions MVP & robustesse (semaine 4)

Objectif : transformer le prototype fonctionnel en produit livrable v0.1.

- [ ] Ratios export (Auto, 16:9, 1:1, 4:3, 3:2 + presets réseaux X/Instagram/LinkedIn)
- [ ] Échelle @2x à l'export
- [ ] Undo/redo (⌘Z/⇧⌘Z) sur la pile `Style`, réinitialisation (⌘R) vers l'auto-balance
- [ ] Drag-out de la preview vers une autre app
- [ ] Persistance des derniers réglages/dossier/ratio/format entre sessions (`framely-presets`)
- [ ] Cas limites : très grandes captures (downscale preview / export plein), PNG transparent (damier), multi-écrans/échelles mixtes, presse-papiers vide
- [ ] Accessibilité de base : navigation clavier complète, labels VoiceOver, respect « Réduire les animations »
- [ ] Onboarding minimal : demande permission expliquée + écran unique montrant les 2 raccourcis clés, image de démo pré-enjolivée
- [ ] Passage complet de tous les raccourcis clavier du README
- [ ] Tests manuels du parcours complet sur plusieurs configs (mono/multi-écran, Retina/non-Retina)

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
