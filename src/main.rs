#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod calculator;
mod clipboard;
mod hotkey;
mod state;
mod tray;

use app::AntinotApp;
use eframe::egui::{self, Vec2};

fn main() -> eframe::Result<()> {
    // Sur Windows, privilégie DirectX (dx12, dx11) pour éviter le crash du pilote Intel Vulkan (igvk64.dll 0xc0000005)
    #[cfg(target_os = "windows")]
    if std::env::var_os("WGPU_BACKEND").is_none() {
        // SAFETY: Appel effectué au tout début du thread principal avant le spawn d'autres threads
        unsafe {
            std::env::set_var("WGPU_BACKEND", "dx12,dx11");
        }
    }

    // Configuration des options natives de la fenêtre flottante
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(460.0, 540.0))
            .with_min_inner_size(Vec2::new(320.0, 300.0))
            .with_decorations(false) // Sans barre de titre standard de l'OS (custom macOS style)
            .with_transparent(true)  // Coins arrondis et transparence fluide
            .with_title("Antinot")
            .with_visible(true),
        ..Default::default()
    };

    eframe::run_native(
        "Antinot",
        options,
        Box::new(|cc| Ok(Box::new(AntinotApp::new(cc)))),
    )
}
