/// Module de gestion du presse-papier système pour Antinot.

pub struct ClipboardHelper;

impl ClipboardHelper {
    /// Récupère le texte du presse-papier nettoyé de tout formatage riche ou retours de ligne exotiques.
    pub fn get_clean_text() -> Option<String> {
        let mut clipboard = arboard::Clipboard::new().ok()?;
        let text = clipboard.get_text().ok()?;
        Some(Self::clean_text(&text))
    }

    /// Copie du texte dans le presse-papier système.
    pub fn set_text(text: &str) -> bool {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            clipboard.set_text(text).is_ok()
        } else {
            false
        }
    }

    /// Nettoie une chaîne : standardise les fins de ligne (\r\n -> \n)
    pub fn clean_text(text: &str) -> String {
        text.replace("\r\n", "\n").replace('\r', "\n")
    }
}
