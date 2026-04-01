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
        // TODO: map these to zh-TW once a Traditional Chinese is supported.
        "zh-TW" | "zh-HK" | "zh-MO" | "zh-Hant-CN" | "zh-Hant-TW" | "zh-Hant" | "zh-Hant-HK"
        | "zh-Hant-MO" => "zh-CN".to_string(),
        // Japanese (already matches filename ja-JP.yml)
        // already normalized
        _ => base,
    }
}

/// Get system locale with fallback
pub fn locale() -> String {
    std::env::var("PHICHAIN_LANG")
        .ok()
        .map(|loc| normalize_locale(&loc))
        .or(sys_locale::get_locale().map(|loc| normalize_locale(&loc)))
        .unwrap_or_else(|| "en-US".to_string())
}

// Leaks owned translations to produce `&'static str`.
// Acceptable here because the converter is a short-lived CLI process.
pub fn i18n_str(key: &'static str) -> &'static str {
    match t!(key) {
        Cow::Borrowed(s) => s,
        Cow::Owned(s) => Box::leak(s.into_boxed_str()),
    }
}
