use eframe::egui;
use std::collections::BTreeMap;

pub struct ExtensionGrouper;

impl ExtensionGrouper {
    pub fn group_by_first_letter(supported_extensions: &[String]) -> Vec<(String, Vec<String>)> {
        let mut groups: BTreeMap<char, Vec<String>> = BTreeMap::new();

        for ext in supported_extensions {
            let first_char = ext.chars().next().unwrap_or('?');
            let group_key = if first_char.is_ascii_alphabetic() {
                first_char.to_ascii_uppercase()
            } else {
                '#' // Group non-letter extensions under '#'
            };

            groups.entry(group_key).or_default().push(ext.clone());
        }

        // Sort extensions within each group
        for extensions in groups.values_mut() {
            extensions.sort();
        }

        // Convert to Vec with proper group names
        groups
            .into_iter()
            .map(|(key, extensions)| {
                let group_name = key.to_string();
                (group_name, extensions)
            })
            .collect()
    }
}

pub struct FontManager;

impl FontManager {
    pub fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // Load custom fonts from assets
        let droid_font_data = include_bytes!("../assets/DroidSansFallback.ttf");
        let jetbrains_font_data = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

        fonts.font_data.insert(
            "DroidSans".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(droid_font_data)),
        );

        fonts.font_data.insert(
            "JetBrainsMono".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(jetbrains_font_data)),
        );

        // Set default fonts
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "DroidSans".to_owned());

        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "JetBrainsMono".to_owned());

        ctx.set_fonts(fonts);
    }
}
