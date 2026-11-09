use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub struct TrayManager {
    _tray_icon: TrayIcon,
    pub item_open_id: muda::MenuId,
    pub item_always_on_top_id: muda::MenuId,
    pub item_clear_scratchpad_id: muda::MenuId,
    pub item_quit_id: muda::MenuId,
    pub always_on_top_item: CheckMenuItem,
}

impl TrayManager {
    pub fn new(always_on_top: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();

        let item_open = MenuItem::new("Afficher / Masquer Antinot (Alt+A)", true, None);
        let item_always_on_top = CheckMenuItem::new("Toujours au premier plan", true, always_on_top, None);
        let item_clear_scratchpad = MenuItem::new("Effacer le Scratchpad", true, None);
        let separator = PredefinedMenuItem::separator();
        let item_quit = MenuItem::new("Quitter", true, None);

        let item_open_id = item_open.id().clone();
        let item_always_on_top_id = item_always_on_top.id().clone();
        let item_clear_scratchpad_id = item_clear_scratchpad.id().clone();
        let item_quit_id = item_quit.id().clone();

        menu.append(&item_open)?;
        menu.append(&item_always_on_top)?;
        menu.append(&item_clear_scratchpad)?;
        menu.append(&separator)?;
        menu.append(&item_quit)?;

        let icon = generate_tray_icon()?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Antinot - Scratchpad instantané")
            .with_icon(icon)
            .build()?;

        Ok(Self {
            _tray_icon: tray_icon,
            item_open_id,
            item_always_on_top_id,
            item_clear_scratchpad_id,
            item_quit_id,
            always_on_top_item: item_always_on_top,
        })
    }
}

/// Génère une icône 32x32 RGBA épurée et moderne pour la barre système (symbole "A" blanc sur fond sombre arrondi)
fn generate_tray_icon() -> Result<Icon, Box<dyn std::error::Error>> {
    const SIZE: u32 = 32;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;

            // Distance au centre pour un carré arrondi
            let dx = (x as f32 - 15.5).abs();
            let dy = (y as f32 - 15.5).abs();

            // Fond arrondi sombre (style macOS)
            let in_rounded_rect = dx <= 13.5 && dy <= 13.5 && (dx <= 9.0 || dy <= 9.0 || (dx - 9.0).hypot(dy - 9.0) <= 4.5);

            if in_rounded_rect {
                // Fond sombre doux
                rgba[idx] = 30;
                rgba[idx + 1] = 34;
                rgba[idx + 2] = 42;
                rgba[idx + 3] = 255;

                // Dessin d'un "A" ou bloc bloc-notes minimaliste blanc
                let is_letter_a =
                    // Barre gauche du A
                    ((x >= 9 && x <= 12) && (y >= 10 && y <= 22)) ||
                    // Barre droite du A
                    ((x >= 19 && x <= 22) && (y >= 10 && y <= 22)) ||
                    // Barre supérieure du A
                    ((x >= 9 && x <= 22) && (y >= 9 && y <= 12)) ||
                    // Barre centrale du A
                    ((x >= 9 && x <= 22) && (y >= 15 && y <= 17));

                if is_letter_a {
                    rgba[idx] = 255;
                    rgba[idx + 1] = 255;
                    rgba[idx + 2] = 255;
                    rgba[idx + 3] = 255;
                }
            } else {
                // Transparent
                rgba[idx + 3] = 0;
            }
        }
    }

    Ok(Icon::from_rgba(rgba, SIZE, SIZE)?)
}

#[derive(Debug, PartialEq, Eq)]
pub enum TrayAction {
    ToggleWindow,
    SetAlwaysOnTop(bool),
    ClearScratchpad,
    Quit,
}

pub fn poll_tray_events(tray: &TrayManager) -> Option<TrayAction> {
    // 1. Événements de clic sur l'icône de la barre des tâches
    if let Ok(event) = TrayIconEvent::receiver().try_recv() {
        match event {
            TrayIconEvent::Click { button, .. } => {
                if button == tray_icon::MouseButton::Left {
                    return Some(TrayAction::ToggleWindow);
                }
            }
            TrayIconEvent::DoubleClick { .. } => {
                return Some(TrayAction::ToggleWindow);
            }
            _ => {}
        }
    }

    // 2. Événements du menu contextuel
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        if event.id == tray.item_open_id {
            return Some(TrayAction::ToggleWindow);
        } else if event.id == tray.item_always_on_top_id {
            let is_checked = tray.always_on_top_item.is_checked();
            return Some(TrayAction::SetAlwaysOnTop(is_checked));
        } else if event.id == tray.item_clear_scratchpad_id {
            return Some(TrayAction::ClearScratchpad);
        } else if event.id == tray.item_quit_id {
            return Some(TrayAction::Quit);
        }
    }

    None
}
