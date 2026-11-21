//! Fonts shared by the editor ui (egui) and the game (bevy text).
//!
//! Han characters take regional glyph forms, so CJK fonts are ordered by [`CjkRegion`].

use crate::abort_broken_install;
use bevy::prelude::*;
use bevy::text::{CosmicFontSystem, FontAtlasSet, SwashCache, TextPipeline};
use cosmic_text::{fontdb, Fallback, FontSystem};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use unicode_script::Script;

/// The regional glyph forms Han characters take
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CjkRegion {
    #[default]
    SimplifiedChinese,
    TraditionalChinese,
    Japanese,
}

impl CjkRegion {
    pub fn from_language(language: &str) -> Self {
        match language {
            "zh_tw" => Self::TraditionalChinese,
            "ja_jp" => Self::Japanese,
            _ => Self::SimplifiedChinese,
        }
    }

    /// A regional subset first, then the full Simplified Chinese font for everything it lacks, including Hangul
    fn cjk_fonts(self) -> &'static [CjkFont] {
        match self {
            Self::SimplifiedChinese => &[SIMPLIFIED_CHINESE],
            Self::TraditionalChinese => &[TRADITIONAL_CHINESE, SIMPLIFIED_CHINESE],
            Self::Japanese => &[JAPANESE, SIMPLIFIED_CHINESE],
        }
    }

    fn locale(self) -> &'static str {
        match self {
            Self::SimplifiedChinese => "zh-CN",
            Self::TraditionalChinese => "zh-TW",
            Self::Japanese => "ja",
        }
    }
}

struct CjkFont {
    family: &'static str,
    regular: &'static str,
    medium: &'static str,
}

const SIMPLIFIED_CHINESE: CjkFont = CjkFont {
    family: "Noto Sans CJK SC",
    regular: "NotoSansCJKsc-Regular.otf",
    medium: "NotoSansCJKsc-Medium.otf",
};

const TRADITIONAL_CHINESE: CjkFont = CjkFont {
    family: "Noto Sans TC",
    regular: "NotoSansTC-Regular.otf",
    medium: "NotoSansTC-Medium.otf",
};

const JAPANESE: CjkFont = CjkFont {
    family: "Noto Sans JP",
    regular: "NotoSansJP-Regular.otf",
    medium: "NotoSansJP-Medium.otf",
};

/// Characters the CJK fonts lack, such as `ł`, `≒` and `✗`, as `(family, file)`
const SYMBOL_FONTS: [(&str, &str); 4] = [
    ("Noto Sans", "NotoSans-Regular.ttf"),
    ("Noto Sans Math", "NotoSansMath-Regular.ttf"),
    ("Noto Sans Symbols", "NotoSansSymbols-Regular.ttf"),
    ("Noto Sans Symbols 2", "NotoSansSymbols2-Regular.ttf"),
];

fn font_dir() -> PathBuf {
    let root =
        env::var("BEVY_ASSET_ROOT").expect("BEVY_ASSET_ROOT should be set by setup_assets()");
    PathBuf::from(root).join("assets/font")
}

fn read_font(file: &str) -> Vec<u8> {
    std::fs::read(font_dir().join(file))
        .unwrap_or_else(|err| abort_broken_install(&format!("font {file}"), err.into()))
}

/// Font files are read once and kept for the whole run, so egui and bevy share one copy
fn font_bytes(file: &'static str) -> &'static [u8] {
    static FONTS: OnceLock<Mutex<HashMap<&'static str, &'static [u8]>>> = OnceLock::new();
    let mut fonts = FONTS.get_or_init(Default::default).lock().unwrap();
    fonts
        .entry(file)
        .or_insert_with(|| Box::leak(read_font(file).into_boxed_slice()))
}

/// Falls back to the CJK fonts in [`CjkRegion`] order instead of the platform's font names
struct RegionFallback {
    cjk: Vec<&'static str>,
    all: Vec<&'static str>,
}

impl RegionFallback {
    fn new(region: CjkRegion) -> Self {
        let cjk: Vec<_> = region.cjk_fonts().iter().map(|font| font.family).collect();
        let all = cjk
            .iter()
            .copied()
            .chain(SYMBOL_FONTS.iter().map(|(family, _)| *family))
            .collect();
        Self { cjk, all }
    }
}

impl Fallback for RegionFallback {
    fn common_fallback(&self) -> &[&'static str] {
        &self.all
    }

    fn forbidden_fallback(&self) -> &[&'static str] {
        &[]
    }

    fn script_fallback(&self, script: Script, _locale: &str) -> &[&'static str] {
        match script {
            Script::Han
            | Script::Hiragana
            | Script::Katakana
            | Script::Hangul
            | Script::Bopomofo => &self.cjk,
            _ => &[],
        }
    }
}

fn font_system(region: CjkRegion) -> FontSystem {
    let mut db = fontdb::Database::new();
    let files = region
        .cjk_fonts()
        .iter()
        .map(|font| font.regular)
        .chain(SYMBOL_FONTS.iter().map(|(_, file)| *file));
    for file in files {
        db.load_font_source(fontdb::Source::Binary(Arc::new(font_bytes(file))));
    }
    FontSystem::new_with_locale_and_db_and_fallback(
        region.locale().to_owned(),
        db,
        RegionFallback::new(region),
    )
}

/// Replaces every text cache and lays out all text again.
/// Fallbacks are cached per script, and laid out text keeps font ids.
fn apply_cjk_region_system(
    region: Res<CjkRegion>,
    mut commands: Commands,
    mut texts: Query<&mut TextFont>,
) {
    commands.insert_resource(CosmicFontSystem(font_system(*region)));
    commands.insert_resource(TextPipeline::default());
    commands.insert_resource(FontAtlasSet::default());
    commands.insert_resource(SwashCache::default());
    for mut font in &mut texts {
        font.set_changed();
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<CjkRegion>().add_systems(
        Update,
        apply_cjk_region_system.run_if(resource_changed::<CjkRegion>),
    );
}

#[cfg(feature = "egui")]
pub mod egui_fonts {
    use super::{font_bytes, read_font, CjkRegion, SYMBOL_FONTS};
    use bevy_egui::egui::{FontData, FontDefinitions, FontFamily};
    use std::sync::Arc;

    /// Headings, in the medium weight
    pub const BOLD: &str = "bold";
    /// Numbers, in a monospace font so digits keep their width while dragging
    pub const NUMBER: &str = "number";

    const NUMBER_FONT: &str = "JetBrainsMono-Regular.ttf";

    /// Line metrics egui uses for Noto Sans CJK, in font units of 1000 per em.
    /// The font's own 1160 / -288 makes text edits taller than a 20 pt control at 13 pt.
    /// The line box is centered on the ideographs, so CJK text sits in the middle of buttons.
    const CJK_ASCENDER: i16 = 1000;
    const CJK_DESCENDER: i16 = -230;

    /// egui reads the line height from the `hhea` table, since these fonts do not set `USE_TYPO_METRICS`
    fn with_cjk_line_metrics(mut bytes: Vec<u8>) -> Vec<u8> {
        let num_tables = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
        let hhea = (0..num_tables)
            .map(|i| 12 + i * 16)
            .find(|&record| &bytes[record..record + 4] == b"hhea")
            .map(|record| u32::from_be_bytes(bytes[record + 8..record + 12].try_into().unwrap()))
            .expect("CJK fonts have an hhea table") as usize;
        bytes[hhea + 4..hhea + 6].copy_from_slice(&CJK_ASCENDER.to_be_bytes());
        bytes[hhea + 6..hhea + 8].copy_from_slice(&CJK_DESCENDER.to_be_bytes());
        bytes[hhea + 8..hhea + 10].copy_from_slice(&0i16.to_be_bytes());
        bytes
    }

    fn add(fonts: &mut FontDefinitions, file: &'static str, cjk: bool) -> String {
        let data = if cjk {
            FontData::from_owned(with_cjk_line_metrics(read_font(file)))
        } else {
            FontData::from_static(font_bytes(file))
        };
        fonts.font_data.insert(file.to_owned(), Arc::new(data));
        file.to_owned()
    }

    /// The proportional family plus [`BOLD`] and [`NUMBER`], which fall back to it.
    ///
    /// Icon fonts are added by the caller and must stay out of [`NUMBER`].
    /// JetBrains Mono has Powerline glyphs where icon fonts place their icons.
    pub fn definitions(region: CjkRegion) -> FontDefinitions {
        let mut fonts = FontDefinitions::default();

        let mut proportional = Vec::new();
        for font in region.cjk_fonts() {
            proportional.push(add(&mut fonts, font.regular, true));
        }
        for (_, file) in SYMBOL_FONTS {
            proportional.push(add(&mut fonts, file, false));
        }
        proportional.extend(fonts.families[&FontFamily::Proportional].iter().cloned());

        let mut bold = Vec::new();
        for font in region.cjk_fonts() {
            bold.push(add(&mut fonts, font.medium, true));
        }
        bold.extend(proportional.iter().cloned());

        let mut number = vec![add(&mut fonts, NUMBER_FONT, false)];
        number.extend(proportional.iter().cloned());

        fonts
            .families
            .insert(FontFamily::Proportional, proportional);
        fonts.families.insert(FontFamily::Name(BOLD.into()), bold);
        fonts
            .families
            .insert(FontFamily::Name(NUMBER.into()), number);
        fonts
    }
}
