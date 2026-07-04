//! Menu bar (icône + menu) et raccourcis clavier globaux.
//!
//! Sur macOS, la fenêtre menu bar (`NSStatusItem`) et les raccourcis globaux
//! doivent être créés sur le thread principal — c'est garanti ici car tout
//! est initialisé depuis `FramelyApp::new`, appelée par `eframe::run_native`
//! sur le thread principal.

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub enum SystemAction {
    CaptureZone,
    CaptureWindow,
    Paste,
    Quit,
}

pub struct SystemIntegration {
    // Doit rester en vie tant que l'icône doit être affichée dans la barre de menus.
    _tray_icon: TrayIcon,
    // Doit rester en vie tant que les raccourcis globaux doivent rester actifs.
    _hotkey_manager: GlobalHotKeyManager,
    capture_zone_item: MenuItem,
    capture_window_item: MenuItem,
    paste_item: MenuItem,
    quit_item: MenuItem,
    capture_zone_hotkey: HotKey,
    capture_window_hotkey: HotKey,
}

impl SystemIntegration {
    pub fn new() -> Self {
        let capture_zone_item = MenuItem::new("Capturer une zone (⇧⌘2)", true, None);
        let capture_window_item = MenuItem::new("Capturer une fenêtre (⇧⌘4)", true, None);
        let paste_item = MenuItem::new("Depuis le presse-papiers (⌘V)", true, None);
        let quit_item = MenuItem::new("Quitter", true, None);

        let menu = Menu::new();
        menu.append_items(&[
            &capture_zone_item,
            &capture_window_item,
            &paste_item,
            &quit_item,
        ])
        .expect("la construction du menu de la barre de menus ne devrait pas échouer");

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Framely")
            .with_icon(build_tray_icon())
            .build()
            .expect("l'icône de la barre de menus ne devrait pas échouer à se créer");

        let hotkey_manager = GlobalHotKeyManager::new()
            .expect("le gestionnaire de raccourcis globaux devrait s'initialiser");
        let capture_zone_hotkey =
            HotKey::new(Some(Modifiers::SHIFT | Modifiers::SUPER), Code::Digit2);
        let capture_window_hotkey =
            HotKey::new(Some(Modifiers::SHIFT | Modifiers::SUPER), Code::Digit4);

        // Si l'enregistrement échoue (raccourci déjà pris par une autre app),
        // le raccourci global est simplement indisponible ; le menu et les
        // boutons de l'UI restent utilisables.
        let _ = hotkey_manager.register(capture_zone_hotkey);
        let _ = hotkey_manager.register(capture_window_hotkey);

        Self {
            _tray_icon: tray_icon,
            _hotkey_manager: hotkey_manager,
            capture_zone_item,
            capture_window_item,
            paste_item,
            quit_item,
            capture_zone_hotkey,
            capture_window_hotkey,
        }
    }

    /// À appeler à chaque frame : consomme les événements en attente
    /// (clic menu, raccourci clavier) et renvoie l'action correspondante.
    pub fn poll(&self) -> Option<SystemAction> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == *self.capture_zone_item.id() {
                return Some(SystemAction::CaptureZone);
            }
            if event.id == *self.capture_window_item.id() {
                return Some(SystemAction::CaptureWindow);
            }
            if event.id == *self.paste_item.id() {
                return Some(SystemAction::Paste);
            }
            if event.id == *self.quit_item.id() {
                return Some(SystemAction::Quit);
            }
        }

        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == HotKeyState::Pressed {
                if event.id == self.capture_zone_hotkey.id() {
                    return Some(SystemAction::CaptureZone);
                }
                if event.id == self.capture_window_hotkey.id() {
                    return Some(SystemAction::CaptureWindow);
                }
            }
        }

        None
    }
}

/// Icône 32x32 procédurale (carré arrondi indigo) — identité visuelle
/// définitive à faire par un designer, cf. README.md.
fn build_tray_icon() -> Icon {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let radius = 8i32;
    for y in 0..size as i32 {
        for x in 0..size as i32 {
            let inside_corner =
                |cx: i32, cy: i32| (x - cx).pow(2) + (y - cy).pow(2) <= radius.pow(2);
            let in_bounds = if x < radius && y < radius {
                inside_corner(radius, radius)
            } else if x >= size as i32 - radius && y < radius {
                inside_corner(size as i32 - radius - 1, radius)
            } else if x < radius && y >= size as i32 - radius {
                inside_corner(radius, size as i32 - radius - 1)
            } else if x >= size as i32 - radius && y >= size as i32 - radius {
                inside_corner(size as i32 - radius - 1, size as i32 - radius - 1)
            } else {
                true
            };

            let i = ((y as u32 * size + x as u32) * 4) as usize;
            if in_bounds {
                rgba[i..i + 4].copy_from_slice(&[99, 102, 241, 255]);
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("les données de l'icône devraient être valides")
}
