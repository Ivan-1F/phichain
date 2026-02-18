use rust_i18n::t;
use std::borrow::Cow;

/// Normalize locale from system to rust-i18n format
fn normalize_locale(locale: &str) -> String {
    // Remove encoding suffix and replace underscore
    let base = locale.split('.').next().unwrap_or(locale).replace('_', "-");

    // Map to available translation files
    match base.as_str() {
        "C" | "POSIX" => "en-US".to_string(),
        // macOS verbose formats
        "zh-Hans-CN" | "zh-Hans" | "zh-Hans-SG" => "zh-CN".to_string(),
        "zh-Hant-CN" | "zh-Hant-TW" | "zh-Hant" | "zh-Hant-HK" | "zh-Hant-MO" => {
            "zh-TW".to_string()
        }
        // already normalized
        _ => base,
    }
}

/// Get system locale with fallback
pub fn locale() -> String {
    std::env::var("PHICHAIN_LANG")
        .ok()
        .or(sys_locale::get_locale().map(|loc| normalize_locale(&loc)))
        .unwrap_or_else(|| "en-US".to_string())
}

pub fn i18n_str(key: &str) -> &str {
    match t!(key) {
        Cow::Borrowed(s) => s,
        Cow::Owned(_) => unreachable!(),
    }
}
