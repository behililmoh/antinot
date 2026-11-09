use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const NUM_SLOTS: usize = 6; // Slot 0: Scratchpad, 1..=5: Slots persistants

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NoteSlot {
    pub id: usize,
    pub name: String,
    pub content: String,
    pub text_color: Option<[u8; 3]>,
}

impl Default for NoteSlot {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Scratchpad".to_string(),
            content: String::new(),
            text_color: None,
        }
    }
}

impl NoteSlot {
    pub fn new(id: usize, name: &str, content: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            content: content.to_string(),
            text_color: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub always_on_top: bool,
    pub auto_hide_on_blur: bool,
    pub inline_calc: bool,
    pub dark_theme: bool,
    pub font_size: f32,
    pub window_width: f32,
    pub window_height: f32,
    pub active_slot: usize,
    pub text_color: [u8; 3],
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            always_on_top: true,
            auto_hide_on_blur: false,
            inline_calc: true,
            dark_theme: true,
            font_size: 14.5,
            window_width: 460.0,
            window_height: 540.0,
            active_slot: 0,
            text_color: [245, 245, 250],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppData {
    pub settings: AppSettings,
    pub slots: Vec<NoteSlot>,
}

impl Default for AppData {
    fn default() -> Self {
        let mut slots = Vec::with_capacity(NUM_SLOTS);
        slots.push(NoteSlot::new(0, "Scratchpad", ""));
        for i in 1..NUM_SLOTS {
            slots.push(NoteSlot::new(i, &format!("Note {}", i), ""));
        }

        Self {
            settings: AppSettings::default(),
            slots,
        }
    }
}

pub struct StorageManager {
    file_path: PathBuf,
}

impl StorageManager {
    pub fn new() -> Self {
        let dir = if let Some(proj_dirs) = directories::ProjectDirs::from("com", "antinot", "antinot") {
            proj_dirs.config_dir().to_path_buf()
        } else {
            PathBuf::from(".antinot")
        };

        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("antinot_data.json");
        Self { file_path }
    }

    /// Charge les données existantes ou initialise avec les valeurs par défaut
    pub fn load(&self) -> AppData {
        if self.file_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.file_path) {
                if let Ok(mut data) = serde_json::from_str::<AppData>(&content) {
                    // Vérification que tous les slots 0..NUM_SLOTS existent
                    while data.slots.len() < NUM_SLOTS {
                        let i = data.slots.len();
                        let name = if i == 0 { "Scratchpad" } else { &format!("Note {}", i) };
                        data.slots.push(NoteSlot::new(i, name, ""));
                    }
                    return data;
                }
            }
        }
        AppData::default()
    }

    /// Sauvegarde les données sur le disque
    pub fn save(&self, data: &AppData) {
        if let Ok(json) = serde_json::to_string_pretty(data) {
            let _ = fs::write(&self.file_path, json);
        }
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backwards_compatible_deserialization() {
        // Simulation d'un fichier JSON ancien sans le champ `text_color`
        let old_json = r#"{
            "settings": {
                "always_on_top": true,
                "auto_hide_on_blur": false,
                "inline_calc": true,
                "dark_theme": true,
                "font_size": 14.5,
                "window_width": 460.0,
                "window_height": 540.0,
                "active_slot": 0
            },
            "slots": [
                { "id": 0, "name": "Scratchpad", "content": "Hello" },
                { "id": 1, "name": "Note 1", "content": "World" }
            ]
        }"#;

        let data: Result<AppData, _> = serde_json::from_str(old_json);
        assert!(data.is_ok(), "L'ancien format JSON doit se désérialiser sans erreur");
        let data = data.unwrap();

        // Vérification de la valeur par défaut pour la couleur d'écriture (haute visibilité)
        assert_eq!(data.settings.text_color, [245, 245, 250]);
        assert_eq!(data.slots[0].text_color, None);
        assert_eq!(data.slots[1].text_color, None);
    }

    #[test]
    fn test_custom_color_serialization() {
        let mut data = AppData::default();
        data.settings.text_color = [255, 235, 90]; // Jaune
        data.slots[0].text_color = Some([115, 245, 160]); // Vert menthe

        let json = serde_json::to_string(&data).expect("Serialization failed");
        let deserialized: AppData = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.settings.text_color, [255, 235, 90]);
        assert_eq!(deserialized.slots[0].text_color, Some([115, 245, 160]));
        assert_eq!(deserialized.slots[1].text_color, None);
    }
}
