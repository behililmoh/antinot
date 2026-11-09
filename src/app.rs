use crate::calculator::Calculator;
use crate::clipboard::ClipboardHelper;
use crate::hotkey::{poll_hotkey_events, HotkeyManager};
use crate::state::{AppData, StorageManager, NUM_SLOTS};
use crate::tray::{poll_tray_events, TrayAction, TrayManager};
use eframe::egui::{self, Color32, CornerRadius, Sense, Stroke, Vec2, ViewportCommand};
use std::time::Instant;

/// Préréglages de couleurs haute visibilité pour thème sombre
pub const DARK_COLOR_PRESETS: &[(&str, [u8; 3])] = &[
    ("Blanc pur (Haute visibilité)", [255, 255, 255]),
    ("Jaune bloc-notes", [255, 235, 90]),
    ("Vert menthe", [115, 245, 160]),
    ("Bleu glacier", [120, 215, 255]),
    ("Orange ambre", [255, 175, 80]),
    ("Rose pastel", [250, 160, 225]),
    ("Matrix rétro", [65, 255, 85]),
    ("Lavande", [200, 175, 255]),
];

/// Préréglages de couleurs haute visibilité pour thème clair
pub const LIGHT_COLOR_PRESETS: &[(&str, [u8; 3])] = &[
    ("Noir profond (Haute visibilité)", [20, 22, 28]),
    ("Bleu nuit", [25, 65, 150]),
    ("Vert forêt", [20, 110, 50]),
    ("Bordeaux", [150, 30, 40]),
    ("Ambre foncé", [160, 80, 15]),
    ("Violet royal", [100, 40, 140]),
    ("Gris ardoise", [70, 75, 85]),
    ("Marron chaud", [90, 50, 30]),
];

pub struct AntinotApp {
    pub data: AppData,
    pub storage: StorageManager,
    pub tray: Option<TrayManager>,
    pub hotkey: Option<HotkeyManager>,

    pub is_visible: bool,
    pub show_settings: bool,
    pub show_color_palette: bool,
    pub last_save: Instant,
    pub is_dirty: bool,
    pub request_focus_editor: bool,
    pub live_math_result: Option<String>,
    pub status_message: Option<(String, Instant)>,
}

impl AntinotApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let storage = StorageManager::new();
        let data = storage.load();

        let tray = match TrayManager::new(data.settings.always_on_top) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("Erreur lors de l'initialisation de la barre des tâches: {e}");
                None
            }
        };

        let hotkey = match HotkeyManager::new() {
            Ok(h) => Some(h),
            Err(e) => {
                eprintln!("Erreur lors de l'enregistrement du raccourci global: {e}");
                None
            }
        };

        // Applique l'état AlwaysOnTop initial
        cc.egui_ctx.send_viewport_cmd(ViewportCommand::WindowLevel(
            if data.settings.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            },
        ));

        let app = Self {
            data,
            storage,
            tray,
            hotkey,
            is_visible: true,
            show_settings: false,
            show_color_palette: false,
            last_save: Instant::now(),
            is_dirty: false,
            request_focus_editor: true,
            live_math_result: None,
            status_message: Some(("Prêt (Alt+A pour afficher/masquer)".to_string(), Instant::now())),
        };

        app.apply_visuals(&cc.egui_ctx);
        app
    }

    /// Applique les visuels (thème sombre ou clair)
    pub fn apply_visuals(&self, ctx: &egui::Context) {
        let mut visuals = if self.data.settings.dark_theme {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        if self.data.settings.dark_theme {
            visuals.panel_fill = Color32::from_rgb(24, 25, 30);
            visuals.window_fill = Color32::from_rgb(24, 25, 30);
            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(24, 25, 30);
            visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(150, 155, 170);
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(32, 34, 42);
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(42, 45, 56);
            visuals.widgets.active.bg_fill = Color32::from_rgb(52, 56, 70);
        } else {
            visuals.panel_fill = Color32::from_rgb(246, 247, 250);
            visuals.window_fill = Color32::from_rgb(246, 247, 250);
            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(246, 247, 250);
            visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(100, 105, 120);
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(236, 238, 244);
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(222, 226, 235);
            visuals.widgets.active.bg_fill = Color32::from_rgb(208, 213, 225);
        }
        ctx.set_visuals(visuals);
    }

    /// Récupère la couleur RGB du texte pour un slot donné
    pub fn get_slot_rgb(&self, slot_idx: usize) -> [u8; 3] {
        if let Some(slot) = self.data.slots.get(slot_idx) {
            if let Some(rgb) = slot.text_color {
                return rgb;
            }
        }
        self.data.settings.text_color
    }

    /// Récupère la couleur Color32 du texte pour un slot donné
    pub fn get_slot_text_color(&self, slot_idx: usize) -> Color32 {
        let [r, g, b] = self.get_slot_rgb(slot_idx);
        Color32::from_rgb(r, g, b)
    }

    /// Définit la couleur du texte pour le slot spécifié
    pub fn set_slot_text_color(&mut self, slot_idx: usize, rgb: [u8; 3]) {
        if let Some(slot) = self.data.slots.get_mut(slot_idx) {
            slot.text_color = Some(rgb);
            self.is_dirty = true;
        }
    }

    /// Réinitialise la couleur personnalisée du slot vers la couleur par défaut
    pub fn reset_slot_text_color(&mut self, slot_idx: usize) {
        if let Some(slot) = self.data.slots.get_mut(slot_idx) {
            slot.text_color = None;
            self.is_dirty = true;
        }
    }

    /// Applique une couleur de texte à toutes les notes et la définit par défaut
    pub fn apply_text_color_to_all(&mut self, rgb: [u8; 3]) {
        self.data.settings.text_color = rgb;
        for slot in &mut self.data.slots {
            slot.text_color = Some(rgb);
        }
        self.is_dirty = true;
    }

    /// Bascule la visibilité de la fenêtre
    pub fn toggle_visibility(&mut self, ctx: &egui::Context) {
        self.is_visible = !self.is_visible;
        ctx.send_viewport_cmd(ViewportCommand::Visible(self.is_visible));
        if self.is_visible {
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            self.request_focus_editor = true;
        }
    }

    /// Définit un statut temporaire
    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), Instant::now()));
    }

    /// Sauvegarde automatique immédiate
    pub fn save_if_dirty(&mut self) {
        if self.is_dirty {
            self.storage.save(&self.data);
            self.is_dirty = false;
            self.last_save = Instant::now();
        }
    }

    /// Gère les raccourcis clavier
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Touche Échap pour masquer la fenêtre
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !self.show_settings {
            self.toggle_visibility(ctx);
            return;
        }

        let ctrl = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        if ctrl {
            // Ctrl + 0 .. Ctrl + 5 pour basculer rapidement entre slots
            if ctx.input(|i| i.key_pressed(egui::Key::Num0)) {
                self.switch_slot(0);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
                self.switch_slot(1);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
                self.switch_slot(2);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num3)) {
                self.switch_slot(3);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num4)) {
                self.switch_slot(4);
            } else if ctx.input(|i| i.key_pressed(egui::Key::Num5)) {
                self.switch_slot(5);
            }

            // Ctrl + Shift + V : Collage propre (clean paste)
            if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::V)) {
                self.paste_clean_text();
            }

            // Ctrl + L : Effacer le contenu du slot actuel
            if ctx.input(|i| i.key_pressed(egui::Key::L)) {
                self.clear_current_slot();
            }

            // Ctrl + P : Basculer l'affichage de la palette de couleurs
            if ctx.input(|i| i.key_pressed(egui::Key::P)) {
                self.show_color_palette = !self.show_color_palette;
            }

            // Ctrl + + / Ctrl + - pour ajuster rapidement la taille de police
            if ctx.input(|i| i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals)) {
                if self.data.settings.font_size < 32.0 {
                    self.data.settings.font_size += 1.0;
                    self.is_dirty = true;
                    self.set_status(&format!("Taille de police : {:.0}px", self.data.settings.font_size));
                }
            } else if ctx.input(|i| i.key_pressed(egui::Key::Minus)) {
                if self.data.settings.font_size > 11.0 {
                    self.data.settings.font_size -= 1.0;
                    self.is_dirty = true;
                    self.set_status(&format!("Taille de police : {:.0}px", self.data.settings.font_size));
                }
            }
        }
    }

    pub fn switch_slot(&mut self, slot_idx: usize) {
        if slot_idx < NUM_SLOTS {
            self.data.settings.active_slot = slot_idx;
            self.is_dirty = true;
            self.request_focus_editor = true;
        }
    }

    pub fn clear_current_slot(&mut self) {
        let idx = self.data.settings.active_slot;
        if let Some(slot) = self.data.slots.get_mut(idx) {
            slot.content.clear();
            self.is_dirty = true;
            self.set_status("Contenu effacé");
        }
    }

    pub fn paste_clean_text(&mut self) {
        if let Some(clean) = ClipboardHelper::get_clean_text() {
            let idx = self.data.settings.active_slot;
            if let Some(slot) = self.data.slots.get_mut(idx) {
                slot.content.push_str(&clean);
                self.is_dirty = true;
                self.set_status("Texte brut inséré");
            }
        }
    }

    pub fn copy_all(&mut self) {
        let idx = self.data.settings.active_slot;
        if let Some(slot) = self.data.slots.get(idx) {
            if ClipboardHelper::set_text(&slot.content) {
                self.set_status("Tout le contenu a été copié !");
            }
        }
    }

    /// Rendu de la barre de titre personnalisée (style macOS flottant)
    fn render_titlebar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();

        ui.horizontal(|ui| {
            ui.add_space(4.0);

            // Boutons Traffic Lights (Style macOS)
            // Rouge: masquer vers le tray
            let close_btn = ui.add(
                egui::Button::new(
                    egui::RichText::new("●").color(Color32::from_rgb(255, 95, 87)).size(14.0),
                )
                .frame(false),
            );
            if close_btn.clicked() {
                self.toggle_visibility(&ctx);
            }
            close_btn.on_hover_text("Masquer dans la barre des tâches (Échap ou Alt+A)");

            // Jaune: minimiser
            let min_btn = ui.add(
                egui::Button::new(
                    egui::RichText::new("●").color(Color32::from_rgb(254, 188, 46)).size(14.0),
                )
                .frame(false),
            );
            if min_btn.clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            min_btn.on_hover_text("Minimiser");

            // Vert: réinitialiser taille
            let reset_btn = ui.add(
                egui::Button::new(
                    egui::RichText::new("●").color(Color32::from_rgb(40, 200, 64)).size(14.0),
                )
                .frame(false),
            );
            if reset_btn.clicked() {
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(460.0, 540.0)));
            }
            reset_btn.on_hover_text("Taille par défaut (460x540)");

            ui.add_space(6.0);

            // Sélecteur d'onglets / Slots avec indicateur de couleur
            ui.horizontal(|ui| {
                let active_slot = self.data.settings.active_slot;
                for i in 0..NUM_SLOTS {
                    let label = if i == 0 {
                        "⚡ Scratchpad".to_string()
                    } else {
                        format!("{}", i)
                    };

                    let is_selected = active_slot == i;
                    let slot_color = self.get_slot_text_color(i);

                    let btn_text = egui::RichText::new(&label)
                        .size(11.5)
                        .color(if is_selected {
                            slot_color
                        } else {
                            Color32::from_rgb(150, 155, 170)
                        });

                    let btn = ui.selectable_label(is_selected, btn_text);
                    if btn.clicked() {
                        self.switch_slot(i);
                    }
                    if i == 0 {
                        btn.on_hover_text("Scratchpad temporaire (Ctrl+0)");
                    } else {
                        btn.on_hover_text(format!("Note persistante slot {} (Ctrl+{})", i, i));
                    }
                }
            });

            // Poignée de drag pour déplacer la fenêtre
            let available_width = ui.available_width() - 145.0;
            if available_width > 20.0 {
                let (_drag_rect, drag_resp) = ui.allocate_exact_size(Vec2::new(available_width, 24.0), Sense::drag());
                if drag_resp.dragged() {
                    ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(4.0);

                // Bouton paramètres ⚙️
                let settings_btn = ui.selectable_label(
                    self.show_settings,
                    egui::RichText::new("⚙").size(14.0),
                );
                if settings_btn.clicked() {
                    self.show_settings = !self.show_settings;
                }
                settings_btn.on_hover_text("Préférences & Couleurs d'écriture");

                // Bouton Pin (Always on top) 📌
                let pin_icon = if self.data.settings.always_on_top { "📌" } else { "📍" };
                let pin_btn = ui.selectable_label(
                    self.data.settings.always_on_top,
                    egui::RichText::new(pin_icon).size(13.0),
                );
                if pin_btn.clicked() {
                    self.data.settings.always_on_top = !self.data.settings.always_on_top;
                    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(
                        if self.data.settings.always_on_top {
                            egui::WindowLevel::AlwaysOnTop
                        } else {
                            egui::WindowLevel::Normal
                        },
                    ));
                    if let Some(tray) = &self.tray {
                        tray.always_on_top_item.set_checked(self.data.settings.always_on_top);
                    }
                    self.is_dirty = true;
                    self.set_status(if self.data.settings.always_on_top {
                        "Premier plan activé"
                    } else {
                        "Premier plan désactivé"
                    });
                }
                pin_btn.on_hover_text("Garder la note toujours au premier plan");

                // Bouton Palette de couleurs d'écriture 🎨
                let palette_btn = ui.selectable_label(
                    self.show_color_palette,
                    egui::RichText::new("🎨").size(13.0),
                );
                if palette_btn.clicked() {
                    self.show_color_palette = !self.show_color_palette;
                }
                palette_btn.on_hover_text("Changer la couleur et la visibilité d'écriture (Ctrl+P)");

                // Bouton collage propre 🧹
                let clean_paste_btn = ui.button(egui::RichText::new("🧹").size(13.0));
                if clean_paste_btn.clicked() {
                    self.paste_clean_text();
                }
                clean_paste_btn.on_hover_text("Coller sans formatage (Ctrl+Shift+V)");

                // Bouton copier tout 📋
                let copy_btn = ui.button(egui::RichText::new("📋").size(13.0));
                if copy_btn.clicked() {
                    self.copy_all();
                }
                copy_btn.on_hover_text("Copier tout le contenu du slot");
            });
        });
    }

    /// Rendu de la barre rapide de sélection de couleur et visibilité
    fn render_color_palette_bar(&mut self, ui: &mut egui::Ui) {
        let active_slot = self.data.settings.active_slot;
        let is_dark = self.data.settings.dark_theme;
        let presets = if is_dark {
            DARK_COLOR_PRESETS
        } else {
            LIGHT_COLOR_PRESETS
        };

        let current_rgb = self.get_slot_rgb(active_slot);

        egui::Frame::new()
            .fill(if is_dark {
                Color32::from_rgb(32, 35, 45)
            } else {
                Color32::from_rgb(235, 238, 245)
            })
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(8, 5))
            .stroke(Stroke::new(
                1.0,
                if is_dark {
                    Color32::from_rgb(52, 58, 75)
                } else {
                    Color32::from_rgb(215, 220, 230)
                },
            ))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🎨 Couleur :")
                            .size(11.5)
                            .strong()
                            .color(if is_dark {
                                Color32::from_rgb(210, 215, 230)
                            } else {
                                Color32::from_rgb(50, 55, 70)
                            }),
                    );

                    // Pastilles de couleurs pré-définies
                    for &(name, rgb) in presets {
                        let color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                        let is_active = current_rgb == rgb;

                        let (rect, resp) = ui.allocate_exact_size(Vec2::new(18.0, 18.0), Sense::click());
                        let painter = ui.painter();

                        if resp.hovered() {
                            painter.circle_stroke(
                                rect.center(),
                                9.5,
                                Stroke::new(1.5, Color32::from_rgb(100, 180, 255)),
                            );
                        }
                        if is_active {
                            painter.circle_stroke(
                                rect.center(),
                                10.0,
                                Stroke::new(2.0, if is_dark { Color32::WHITE } else { Color32::BLACK }),
                            );
                        }

                        painter.circle_filled(rect.center(), 7.0, color);

                        if resp.clicked() {
                            self.set_slot_text_color(active_slot, rgb);
                            self.set_status(&format!("Couleur : {}", name));
                        }
                        resp.on_hover_text(name);
                    }

                    ui.add_space(4.0);

                    // Sélecteur libre de couleur
                    let mut custom_rgb = current_rgb;
                    let resp = ui.color_edit_button_srgb(&mut custom_rgb);
                    if resp.changed() {
                        self.set_slot_text_color(active_slot, custom_rgb);
                        self.set_status("Couleur personnalisée appliquée");
                    }
                    resp.on_hover_text("Sélecteur de couleur personnalisé (Palette libre)");

                    ui.separator();

                    // Ajustement rapide de la taille de texte A- / A+
                    let btn_minus = ui.small_button("A-");
                    if btn_minus.clicked() {
                        if self.data.settings.font_size > 11.0 {
                            self.data.settings.font_size -= 1.0;
                            self.is_dirty = true;
                            self.set_status(&format!("Taille : {:.0}px", self.data.settings.font_size));
                        }
                    }
                    btn_minus.on_hover_text("Diminuer la taille du texte (Ctrl+-)");

                    let btn_plus = ui.small_button("A+");
                    if btn_plus.clicked() {
                        if self.data.settings.font_size < 32.0 {
                            self.data.settings.font_size += 1.0;
                            self.is_dirty = true;
                            self.set_status(&format!("Taille : {:.0}px", self.data.settings.font_size));
                        }
                    }
                    btn_plus.on_hover_text("Agrandir la taille du texte (Ctrl++)");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Bouton fermer la barre de couleur
                        let close_btn = ui.small_button("✕");
                        if close_btn.clicked() {
                            self.show_color_palette = false;
                        }
                        close_btn.on_hover_text("Fermer la palette");

                        // Bouton appliquer à toutes les notes
                        let all_btn = ui.small_button("⇶ Tous");
                        if all_btn.clicked() {
                            self.apply_text_color_to_all(current_rgb);
                            self.set_status("Couleur appliquée à toutes les notes !");
                        }
                        all_btn.on_hover_text("Appliquer cette couleur à toutes les notes");

                        // Bouton réinitialiser
                        let reset_btn = ui.small_button("⟳");
                        if reset_btn.clicked() {
                            self.reset_slot_text_color(active_slot);
                            self.set_status("Couleur par défaut rétablie");
                        }
                        reset_btn.on_hover_text("Rétablir la couleur par défaut pour cette note");
                    });
                });
            });
    }

    /// Rendu du pied de page / barre d'état
    fn render_footer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // Statistiques de texte
            let active_idx = self.data.settings.active_slot;
            let (chars_count, words_count, lines_count) = if let Some(slot) = self.data.slots.get(active_idx) {
                let chars = slot.content.chars().count();
                let words = slot.content.split_whitespace().count();
                let lines = if slot.content.is_empty() { 0 } else { slot.content.lines().count() };
                (chars, words, lines)
            } else {
                (0, 0, 0)
            };

            ui.label(
                egui::RichText::new(format!("{} car • {} mots • {} lig", chars_count, words_count, lines_count))
                    .size(11.0)
                    .color(Color32::from_rgb(120, 125, 140)),
            );

            // Live Math Evaluation Banner au centre si calcul disponible
            if let Some(result) = &self.live_math_result {
                let calc_pill = ui.button(
                    egui::RichText::new(format!("⚡ = {}", result))
                        .size(11.5)
                        .color(Color32::from_rgb(80, 220, 120))
                        .strong(),
                );
                if calc_pill.clicked() {
                    // Insérer le résultat dans le texte
                    let active_idx = self.data.settings.active_slot;
                    if let Some(slot) = self.data.slots.get_mut(active_idx) {
                        slot.content.push_str(&format!(" {}", result));
                        self.is_dirty = true;
                        self.set_status("Résultat inséré !");
                    }
                }
                calc_pill.on_hover_text("Cliquer pour insérer le résultat du calcul");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);

                // Statut de message temporaire ou indicateur de sauvegarde
                if let Some((msg, created_at)) = &self.status_message {
                    if created_at.elapsed().as_secs() < 3 {
                        ui.label(
                            egui::RichText::new(msg)
                                .size(11.0)
                                .color(Color32::from_rgb(100, 180, 255)),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("✓ Sauvegardé")
                                .size(11.0)
                                .color(Color32::from_rgb(90, 160, 110)),
                        );
                    }
                } else {
                    ui.label(
                        egui::RichText::new("✓ Sauvegardé")
                            .size(11.0)
                            .color(Color32::from_rgb(90, 160, 110)),
                    );
                }
            });
        });
    }

    /// Rendu du panneau des préférences
    fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("⚙ Préférences Antinot");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕ Fermer").clicked() {
                            self.show_settings = false;
                        }
                    });
                });

                ui.separator();

                ui.checkbox(&mut self.data.settings.always_on_top, "Garder au premier plan (Always on Top)");
                ui.checkbox(&mut self.data.settings.inline_calc, "Calculatrice mathématique inline automatique");
                ui.checkbox(&mut self.data.settings.auto_hide_on_blur, "Masquer automatiquement lors de la perte de focus");

                // Thème
                if ui.checkbox(&mut self.data.settings.dark_theme, "Mode sombre (Dark theme)").changed() {
                    self.is_dirty = true;
                    self.apply_visuals(ui.ctx());
                }

                ui.separator();
                ui.label(egui::RichText::new("Visibilité & Écriture :").strong());

                // Taille de police
                ui.horizontal(|ui| {
                    ui.label("Taille de police:");
                    if ui.add(egui::Slider::new(&mut self.data.settings.font_size, 11.0..=28.0).suffix(" px")).changed() {
                        self.is_dirty = true;
                    }
                });

                // Couleur d'écriture par défaut
                ui.horizontal(|ui| {
                    ui.label("Couleur d'écriture principale:");
                    let resp = ui.color_edit_button_srgb(&mut self.data.settings.text_color);
                    if resp.changed() {
                        self.is_dirty = true;
                    }
                });

                // Préréglages haute lisibilité
                ui.horizontal(|ui| {
                    ui.label("Préréglages haute visibilité:");
                    if ui.button("⚪ Blanc pur").clicked() {
                        self.apply_text_color_to_all([255, 255, 255]);
                        self.set_status("Blanc pur haute visibilité appliqué à toutes les notes");
                    }
                    if ui.button("🟡 Jaune mémo").clicked() {
                        self.apply_text_color_to_all([255, 235, 90]);
                        self.set_status("Jaune mémo appliqué à toutes les notes");
                    }
                    if ui.button("🟢 Vert menthe").clicked() {
                        self.apply_text_color_to_all([115, 245, 160]);
                        self.set_status("Vert menthe appliqué à toutes les notes");
                    }
                    if ui.button("🔵 Bleu ciel").clicked() {
                        self.apply_text_color_to_all([120, 215, 255]);
                        self.set_status("Bleu ciel appliqué à toutes les notes");
                    }
                });

                // Encadré d'aperçu en direct de l'écriture
                let active_slot = self.data.settings.active_slot;
                let active_color = self.get_slot_text_color(active_slot);
                let current_font_size = self.data.settings.font_size;
                ui.add_space(2.0);
                egui::Frame::new()
                    .fill(if self.data.settings.dark_theme {
                        Color32::from_rgb(20, 21, 26)
                    } else {
                        Color32::from_rgb(240, 241, 245)
                    })
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Aperçu : Voici votre texte avec la couleur et la taille choisies !")
                                .font(egui::FontId::proportional(current_font_size))
                                .color(active_color)
                                .strong(),
                        );
                    });

                ui.separator();
                ui.label(egui::RichText::new("Raccourcis clavier :").strong());
                ui.label("• Alt + A : Afficher / Masquer instantanément la note");
                ui.label("• Échap : Masquer la note");
                ui.label("• Ctrl + P : Ouvrir / Fermer la palette de couleurs d'écriture");
                ui.label("• Ctrl + + / Ctrl + - : Agrandir / Réduire la taille de police");
                ui.label("• Ctrl + 0 : Ouvrir le Scratchpad temporaire");
                ui.label("• Ctrl + 1 à 5 : Basculer vers les notes slots 1 à 5");
                ui.label("• Ctrl + Shift + V : Coller en texte brut nettoyé");
                ui.label("• Ctrl + L : Vider la note active");

                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Stockage : {}", self.storage.get_path().display()))
                        .size(10.5)
                        .color(Color32::GRAY),
                );

                if ui.button("🗑 Effacer toutes les notes et réinitialiser").clicked() {
                    self.data = AppData::default();
                    self.storage.save(&self.data);
                    self.set_status("Notes réinitialisées");
                }
            });
        });
    }
}

impl eframe::App for AntinotApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Polling du raccourci global (Alt+A)
        if let Some(hotkey) = &self.hotkey {
            if poll_hotkey_events(hotkey) {
                self.toggle_visibility(ctx);
            }
        }

        // 2. Polling des événements de la barre des tâches (Tray)
        if let Some(tray) = &self.tray {
            if let Some(action) = poll_tray_events(tray) {
                match action {
                    TrayAction::ToggleWindow => {
                        self.toggle_visibility(ctx);
                    }
                    TrayAction::SetAlwaysOnTop(on_top) => {
                        self.data.settings.always_on_top = on_top;
                        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(if on_top {
                            egui::WindowLevel::AlwaysOnTop
                        } else {
                            egui::WindowLevel::Normal
                        }));
                        self.is_dirty = true;
                    }
                    TrayAction::ClearScratchpad => {
                        if let Some(slot) = self.data.slots.get_mut(0) {
                            slot.content.clear();
                            self.is_dirty = true;
                            self.set_status("Scratchpad effacé");
                        }
                    }
                    TrayAction::Quit => {
                        self.save_if_dirty();
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                }
            }
        }

        // 3. Sauvegarde automatique après 500ms d'inactivité
        if self.is_dirty && self.last_save.elapsed().as_millis() > 500 {
            self.save_if_dirty();
        }

        // Repaint périodique pour garantir la réactivité des hotkeys et du tray
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Si la fenêtre est masquée, on ne dessine rien
        if !self.is_visible {
            return;
        }

        // Gestion de l'auto-hide sur perte de focus si activé
        if self.data.settings.auto_hide_on_blur {
            let has_focus = ctx.input(|i| i.viewport().focused.unwrap_or(true));
            if !has_focus && self.is_visible {
                self.toggle_visibility(&ctx);
                return;
            }
        }

        // Gestion des raccourcis clavier
        self.handle_keyboard_shortcuts(&ctx);

        // Cadre principal avec coins arrondis et bordure élégante style macOS
        let frame = egui::Frame::new()
            .fill(if self.data.settings.dark_theme {
                Color32::from_rgb(26, 27, 34)
            } else {
                Color32::from_rgb(250, 250, 252)
            })
            .corner_radius(CornerRadius::same(12))
            .stroke(Stroke::new(
                1.0,
                if self.data.settings.dark_theme {
                    Color32::from_rgb(48, 52, 65)
                } else {
                    Color32::from_rgb(215, 218, 228)
                },
            ))
            .inner_margin(egui::Margin::same(6));

        egui::CentralPanel::default().frame(frame).show(ui, |ui| {
            ui.vertical(|ui| {
                // Barre de titre personnalisée
                self.render_titlebar(ui);

                ui.add_space(4.0);

                // Palette rapide de couleurs si ouverte
                if self.show_color_palette {
                    self.render_color_palette_bar(ui);
                    ui.add_space(4.0);
                }

                // Panneau de préférences si ouvert
                if self.show_settings {
                    self.render_settings_panel(ui);
                    ui.add_space(4.0);
                }

                // Zone d'édition du texte principale
                let active_slot = self.data.settings.active_slot;
                let font_size = self.data.settings.font_size;
                let inline_calc_enabled = self.data.settings.inline_calc;
                let text_color = self.get_slot_text_color(active_slot);

                if let Some(slot) = self.data.slots.get_mut(active_slot) {
                    let text_edit = egui::TextEdit::multiline(&mut slot.content)
                        .font(egui::FontId::proportional(font_size))
                        .text_color(text_color)
                        .hint_text("Tapez une pensée, un calcul rapide comme '24 * 5 =', ou collez du texte...")
                        .desired_width(f32::INFINITY)
                        .lock_focus(true)
                        .frame(egui::Frame::new().fill(Color32::TRANSPARENT));

                    let output = egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let text_response = ui.add_sized(ui.available_size() - Vec2::new(0.0, 30.0), text_edit);
                            if self.request_focus_editor {
                                text_response.request_focus();
                                self.request_focus_editor = false;
                            }
                            text_response
                        });

                    if output.inner.changed() {
                        self.is_dirty = true;

                        // Vérification automatique de calcul inline
                        if inline_calc_enabled {
                            // Analyse de la dernière ligne saisie
                            if let Some(last_line) = slot.content.lines().last() {
                                if let Some((_expr, res)) = Calculator::try_eval_equation(last_line) {
                                    // Complétion automatique de l'équation
                                    slot.content.push_str(&format!(" {}", res));
                                    self.live_math_result = None;
                                } else {
                                    self.live_math_result = Calculator::analyze_line(last_line);
                                }
                            } else {
                                self.live_math_result = None;
                            }
                        }
                    }
                }

                ui.add_space(4.0);
                ui.separator();

                // Barre d'état en bas
                self.render_footer(ui);
            });
        });
    }
}
