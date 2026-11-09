# ⚡ Antinot

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows&logoColor=white)](https://microsoft.com/windows)
[![Framework](https://img.shields.io/badge/GUI-egui%20%2F%20eframe%200.37-blue)](https://github.com/emilk/egui)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Active%20Development-success)]()

**Le bloc-notes flottant et scratchpad instantané ultra-rapide pour Windows, écrit en Rust.**  
*The lightweight, blazing-fast floating scratchpad & note-taking tool for Windows, powered by Rust & egui.*

[Fonctionnalités](#-fonctionnalités-clés) • [Raccourcis](#-raccourcis-clavier) • [Installation](#-installation--compilation) • [Architecture](#-structure-du-projet)

</div>

---

## 📖 À propos / About

**Antinot** est une application de prise de notes rapide, minimaliste et élégante, inspirée du style épuré de macOS et conçue pour la productivité maximale sur Windows.

Grâce à un raccourci global (`Alt + A`), Antinot apparaît et disparaît instantanément au-dessus de vos applications sans jamais casser votre flux de travail. Développé en **Rust natif** avec `egui`, il ne consomme qu'une infime fraction des ressources de votre machine tout en offrant une réactivité immédiate.

---

## ✨ Fonctionnalités clés

- ⚡ **Invocation Globale Instantanée** : Pressez `Alt + A` n'importe où sous Windows pour afficher ou masquer immédiatement la fenêtre.
- 📌 **Always-on-Top (Épinglage)** : Épinglez la fenêtre pour la garder toujours visible au-dessus de vos fenêtres de travail (IDE, navigateur, réunions).
- 🗂️ **Multi-Slots Dédiés (0 à 5)** :
  - **Slot 0 (⚡ Scratchpad)** : Idéal pour les notes éphémères, copier-coller temporaires ou idées à la volée.
  - **Slots 1 à 5** : Notes persistantes catégorisées, accessibles instantanément par `Ctrl + 1..5`.
- 🧮 **Calculatrice Inline & Live Math** :
  - Évaluez directement vos calculs en tapant `=` à la fin d'une ligne (ex: `15 * 4 =` ➔ `60`, `sqrt(144) + 10 =` ➔ `22`).
  - Détection automatique et affichage en temps réel dans le bandeau inférieur avec un bouton pour insérer le résultat en 1 clic.
- 🎨 **Personnalisation Visuelle Complète** :
  - **Palette de couleurs d'écriture rapide (`Ctrl + P`)** : Presets haute visibilité (Blanc pur, Jaune bloc-notes, Vert menthe, Bleu glacier, Orange ambre, etc.) ou sélecteur RGB libre.
  - Possibilité de personnaliser la couleur par note ou de l'appliquer à toutes les notes.
  - **Thème Sombre & Thème Clair** soignés avec bordures et contrastes optimisés.
  - **Ajustement de la taille de texte** en direct (`Ctrl + +` / `Ctrl + -` ou boutons A- / A+).
- 📋 **Presse-papier Intelligent** :
  - `Ctrl + Shift + V` : Nettoie et colle le texte brut sans formatage parasite.
  - `Ctrl + L` : Nettoyage rapide du slot actif.
  - Bouton **Copier tout** (`📋`) pour récupérer l'intégralité du contenu en un clic.
- 🪟 **Interface Flottante Moderne (Style macOS)** :
  - Boutons de contrôle colorés (*Traffic Lights*) pour masquer, minimiser ou rétablir la taille d'origine (460×540).
  - Coins arrondis élégants, fenêtre déplaçable avec poignée intégrée (*drag & drop*).
  - Option de masquage automatique lors de la perte de focus (*Auto-hide on blur*).
- 🔔 **Intégration Plateau Système (System Tray)** :
  - Icône discrète dans la barre des tâches avec menu contextuel complet (Afficher/Masquer, Always-on-top, Effacer Scratchpad, Quitter).
- 💾 **Persistance Locale & Zéro Latence** :
  - Vos notes et vos réglages sont automatiquement sauvegardés localement au format JSON (dans le dossier `AppData` de votre profil utilisateur).
  - Aucune dépendance externe ni envoi de données sur le cloud : 100% privé et sécurisé.

---

## ⌨️ Raccourcis clavier

| Raccourci | Action |
| :--- | :--- |
| **`Alt + A`** | **Afficher / Masquer instantanément Antinot** (Raccourci global système) |
| **`Échap`** | Masquer la note vers la barre des tâches |
| **`Ctrl + 0`** | Basculer vers le **Scratchpad** |
| **`Ctrl + 1` .. `5`** | Basculer vers les notes persistantes (Slots 1 à 5) |
| **`Ctrl + P`** | Ouvrir / Fermer la palette de couleurs d'écriture |
| **`Ctrl + +` / `Ctrl + -`** | Agrandir / Diminuer la taille du texte |
| **`Ctrl + Shift + V`** | Coller le texte nettoyé (texte brut sans formatage) |
| **`Ctrl + L`** | Vider le contenu de la note active |
| **`=` en fin de ligne** | Résoudre automatiquement l'expression mathématique |

---

## 🚀 Installation & Compilation

### Prérequis
- [Rust & Cargo](https://rustup.rs/) (Édition 2024 recommandée ou version stable récente de Rust).
- Système d'exploitation : **Windows 10 / 11**.

### 1. Cloner le dépôt
```bash
git clone https://github.com/VOTRE_PSEUDO/antinot.git
cd antinot
```

### 2. Compiler en mode Release
Pour obtenir un exécutable optimisé, ultra-léger et **sans console noire en arrière-plan** :
```bash
cargo build --release
```

L'exécutable final sera disponible sous :
```
target/release/antinot.exe
```

### 3. Exécuter
Double-cliquez simplement sur `antinot.exe` ou lancez-le depuis votre terminal :
```bash
cargo run --release
```

---

## 📁 Structure du projet

```text
antinot/
├── Cargo.toml          # Dépendances et configuration du package Rust
├── Cargo.lock          # Verrouillage exact des versions des dépendances
├── .gitignore          # Fichiers exclus du versionnement Git
├── README.md           # Documentation officielle du projet
└── src/
    ├── main.rs         # Point d'entrée, backend WGPU/DirectX & initialisation eframe
    ├── app.rs          # Interface egui, cycle de vie, gestion des événements & rendu
    ├── calculator.rs   # Moteur d'évaluation mathématique inline (evalexpr)
    ├── clipboard.rs    # Gestionnaire du presse-papier & nettoyage de texte
    ├── hotkey.rs       # Enregistrement du raccourci clavier global (Alt + A)
    ├── state.rs        # Modèle de données (Slots, Paramètres) & persistance JSON
    └── tray.rs         # Icône et menu contextuel dans la barre des tâches Windows
```

---

## 🛠️ Technologies utilisées

- **Langage** : [Rust](https://www.rust-lang.org/) (Sûreté mémoire, performance et zéro garbage collection).
- **Interface graphique** : [`egui`](https://github.com/emilk/egui) / [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe) pour un rendu immédiat fluide accéléré matériellement.
- **Backend Graphique** : Direct3D (DirectX 12/11 via `wgpu`) garantissant une stabilité sans crash sur Windows.
- **Raccourcis globaux** : [`global-hotkey`](https://github.com/tauri-apps/global-hotkey) pour la réactivité du `Alt + A`.
- **System Tray & Menus** : [`tray-icon`](https://github.com/tauri-apps/tray-icon) & [`muda`](https://github.com/tauri-apps/muda).
- **Moteur de calcul** : [`evalexpr`](https://github.com/ISibboI/evalexpr) pour l'évaluation mathématique intégrée.
- **Presse-papier** : [`arboard`](https://github.com/1Password/arboard).
- **Sérialisation** : [`serde`](https://serde.rs/) & [`serde_json`](https://github.com/serde-rs/json).

---

## 🤝 Contribuer

Les contributions, suggestions et signalements de bugs sont les bienvenus !
1. Forkez le projet.
2. Créez votre branche de fonctionnalité (`git checkout -b feature/SuperFeature`).
3. Commitez vos modifications (`git commit -m 'Add SuperFeature'`).
4. Pushez sur votre branche (`git push origin feature/SuperFeature`).
5. Ouvrez une Pull Request.

---

## 📄 Licence

Ce projet est sous licence MIT - voir le fichier [LICENSE](LICENSE) pour plus de détails.
