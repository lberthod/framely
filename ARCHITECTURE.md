# Architecture — Framely

## 1. Contraintes qui pilotent l'architecture

- **Lancement < 200 ms**, empreinte mémoire native (une fraction d'un équivalent Electron) → pas d'Electron/Chromium, pas de webview lourde. *Mesuré au Sprint 4 : lancement du process < 100 ms, ~97–146 Mo de RSS au repos (contexte OpenGL/WindowServer inclus) — largement sous les 200+ Mo d'Electron, mais le chiffre "10–20 Mo" de la version initiale de ce document était une estimation non vérifiée, pas une mesure ; voir SPRINT.md Sprint 4.*
- **Preview 60 fps** sur chaque changement de réglage (marge, fond, coins, ombre) → rendu 2D accéléré, pas de re-render CPU naïf à chaque frame.
- **Vit en menu bar** en tâche de fond entre deux captures → doit être négligeable au repos (pas de polling, pas de timer actif).
- **macOS only** (v1) → on peut s'appuyer directement sur ScreenCaptureKit, AppKit, Core Animation plutôt que sur une couche d'abstraction cross-platform inutile.

## 2. Stack technique

| Domaine | Choix | Raison |
|---|---|---|
| Langage | Rust (edition 2021+) | Perf, empreinte mémoire, binaire natif unique |
| UI | `egui` / `eframe` | Immediate-mode, léger, redraw ciblé, bon fit pour preview live + panneau de réglages |
| Rendu 2D | `tiny-skia` (+ option GPU via `wgpu` si besoin de marge de perf) | Rendu vectoriel rapide pour fond/ombre/coins arrondis/marge |
| Capture d'écran | `ScreenCaptureKit` (bindings via `objc2` / `screencapturekit-rs`) | API macOS moderne, capture de fenêtre et de zone performante |
| Presse-papiers | `arboard` ou binding NSPasteboard direct | Lecture/écriture image presse-papiers |
| Intégration système | `objc2` / `objc2-app-kit` pour menu bar (`NSStatusItem`), raccourcis globaux, permissions | Nécessaire pour rester natif |
| Persistance des réglages | fichier JSON/TOML dans `~/Library/Application Support/Framely/` | Simplicité, pas de dépendance DB |
| Packaging | `cargo-bundle` ou script maison → `.app` → `.dmg`, notarisation via `xcrun notarytool` | Distribution standard macOS hors App Store (au moins v1) |

Pas de runtime async lourd nécessaire pour le cœur (rendu synchrone). `tokio` optionnel uniquement si l'IPC menu-bar/fenêtre le justifie — sinon canaux std (`std::sync::mpsc`) suffisent.

## 3. Vue d'ensemble des modules

```
framely/
├── crates/
│   ├── framely-app/        # binaire principal : menu bar + fenêtre éditeur (eframe)
│   ├── framely-capture/    # wrapper ScreenCaptureKit : capture zone / fenêtre / overlay
│   ├── framely-render/     # pipeline d'enjolivement (fond, marge, coins, ombre, ratio)
│   ├── framely-io/         # presse-papiers, export fichier, drag-out, import
│   ├── framely-presets/    # dégradés, presets de marque, persistance réglages
│   └── framely-core/       # types partagés (Document, Style, Rect, Export options)
└── packaging/
    ├── Info.plist
    ├── entitlements.plist
    └── build-dmg.sh
```

Workspace Cargo multi-crate : chaque crate a une responsabilité unique, testable isolément (notamment `framely-render`, qui est pur calcul → facile à unit-tester sans UI).

## 4. Flux de données (architecture logique)

```
┌──────────────┐   raccourci global    ┌────────────────────┐
│  Menu bar     │ ───────────────────▶ │ framely-capture     │
│  (NSStatusItem)│                     │ (overlay + capture) │
└──────────────┘                       └─────────┬──────────┘
                                                  │ RawImage (pixels + métadonnées)
                                                  ▼
                                        ┌────────────────────┐
                                        │  Document (state)   │  ← framely-core
                                        │  image + Style      │
                                        └─────────┬──────────┘
                                                  │
                                 ┌────────────────┼────────────────┐
                                 ▼                                 ▼
                        ┌────────────────┐               ┌──────────────────┐
                        │ framely-render  │──preview──▶  │  eframe / egui UI │
                        │ (fond/marge/    │  (texture)   │  (preview + panel)│
                        │  ombre/coins)   │               └─────────┬────────┘
                        └────────────────┘                          │ réglage modifié
                                 ▲                                  │
                                 └──────────────────────────────────┘
                                                  │
                                                  ▼ ⌘C / ⌘S
                                        ┌────────────────────┐
                                        │   framely-io        │
                                        │ (clipboard/export)  │
                                        └────────────────────┘
```

Principe clé : **`Style` est un état immuable simple** (marge, fond, rayon, ombre, ratio). Chaque frame, `framely-render` reprend l'image source + `Style` et régénère la texture de preview. Pas de mutation incrémentale complexe : un réglage change → nouveau rendu complet mais rapide (image déjà en mémoire, opérations vectorielles bon marché sur `tiny-skia`).

## 5. `Document` & `Style` (types centraux)

```rust
struct Document {
    source: RawImage,       // pixels bruts de la capture/import, jamais modifiés
    style: Style,           // état réglages courant
    history: Vec<Style>,    // pile undo/redo (⌘Z/⇧⌘Z)
}

struct Style {
    background: Background, // Gradient(preset_id) | Solid(Color) | Transparent | Image(path)
    padding: u16,            // 0–200px
    corner_radius: f32,
    shadow: ShadowParams,    // intensité, flou, offset
    ratio: Ratio,            // Auto | Fixed(w, h) | SocialPreset(...)
    scale: Scale,            // @1x | @2x
}
```

`history` permet undo/redo en gardant simplement des snapshots de `Style` (léger, pas besoin de diff complexe).

## 6. Auto-balance (le « beau par défaut »)

Fonction pure `fn auto_balance(source: &RawImage) -> Style`, appliquée à chaque nouvelle capture/import avant que l'utilisateur touche à quoi que ce soit :

- Choix de marge proportionnel à la taille de l'image (ratio empirique, pas fixe).
- Sélection d'un preset de dégradé dans une rotation courte (évite la monotonie sans configurabilité).
- Rayon de coin et ombre à des valeurs par défaut testées visuellement, pas calculées dynamiquement.

Cette fonction est le cœur du produit — elle mérite ses propres tests visuels/golden-image en CI dès que possible.

## 7. Capture & overlay plein écran

- `framely-capture` encapsule `ScreenCaptureKit` (ou fallback `CGWindowListCreateImage` si nécessaire pour compat).
- L'overlay de sélection (assombrissement, réticule, dimensions live, surbrillance fenêtre survolée) est une fenêtre `NSWindow` transparente distincte de la fenêtre éditeur — cycle de vie court, détruite après capture ou Échap.
- Gestion des multi-écrans et Retina : capture sur l'écran sous le curseur, on lit le `backingScaleFactor` de cet écran pour le facteur d'échelle, pas une valeur globale.

## 8. Rendu / preview live

- `tiny-skia` pour tout le pipeline 2D vectoriel : fond (dégradé/couleur/image), masque de coins arrondis, ombre portée (flou gaussien approché), composition de l'image source par-dessus.
- Résultat rendu dans une texture uploadée à `egui` (`egui::TextureHandle`), redraw déclenché uniquement au changement de `Style` (pas de boucle continue) — cohérent avec la contrainte de légèreté au repos.
- Pour les très grandes captures (5K/6K) : la preview travaille sur une version downscalée en mémoire ; l'export final re-rend à pleine résolution à la demande (⌘C/⌘S), jamais l'inverse.

## 9. Sortie (presse-papiers, export, drag-out)

- `framely-io::clipboard::write_image` — écrit directement le buffer PNG/bitmap sur `NSPasteboard`, pas de fichier temporaire pour le chemin ⌘C (le plus fréquent, doit être instantané).
- Export fichier : re-rend à résolution native (@1x ou @2x), encode PNG/JPEG, mémorise le dernier dossier utilisé (persistance légère via `framely-presets`).
- Drag-out : `egui` ne gère pas nativement le drag natif macOS → pont via `objc2` vers `NSDraggingSession` déclenché sur la zone preview.

## 10. Permissions & robustesse

- Permission macOS « Capture d'écran » : vérifiée au lancement via l'API système ; si refusée, l'app bascule sur le mode import (coller/glisser) sans bloquer l'usage.
- Presse-papiers vide/non-image au ⌘V : message doux dans la barre d'état, pas d'erreur modale.
- Toute opération destructive (réglage) est réversible via la pile `history` — pas de mutation en place du `Document.source`.

## 11. Packaging & distribution

1. `cargo build --release` → binaire.
2. Assemblage `.app` (Info.plist, icône, entitlements) via `cargo-bundle` ou script `packaging/build-dmg.sh`.
3. Signature développeur + notarisation Apple (`xcrun notarytool submit` puis `stapler`).
4. Génération `.dmg` avec fond personnalisé (l'app se doit d'être belle jusque dans son installeur).

## 12. Ce que l'architecture N'EST PAS (pour rester scope MVP)

- Pas de moteur de plugins.
- Pas d'abstraction cross-platform (Windows/Linux) tant que le MVP macOS n'est pas validé.
- Pas de backend réseau / sync cloud avant v2 (iCloud sync des presets est roadmap, pas MVP).
- Pas de DB embarquée — les presets et réglages sont de simples fichiers de config.
