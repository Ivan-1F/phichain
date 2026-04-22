//! CLI i18n helpers shared by non-GUI phichain binaries.
//!
//! Supported locales (BCP 47): `en-US` (fallback), `zh-CN`, `ja-JP`.
//!
//! Each binary ships `locales/*.yml` and calls `rust_i18n::i18n!("locales", fallback = "en-US")` at its crate root.

/// Map a raw locale string into one of the supported keys (`en-US`, `zh-CN`, `ja-JP`), or pass through unchanged.
///
/// Accepts POSIX (`zh_CN.UTF-8`), macOS script-tagged BCP 47 (`zh-Hans-CN`), and bare language subtags (`ja`, `zh`).
pub fn normalize_locale(locale: &str) -> String {
    let base = locale.split('.').next().unwrap_or(locale).replace('_', "-");

    match base.as_str() {
        "C" | "POSIX" => "en-US".to_string(),
        "zh-Hans-CN" | "zh-Hans" | "zh-Hans-SG" => "zh-CN".to_string(),
        // TODO: map to zh-TW once a Traditional Chinese translation exists.
        "zh" | "zh-TW" | "zh-HK" | "zh-MO" | "zh-Hant-CN" | "zh-Hant-TW" | "zh-Hant"
        | "zh-Hant-HK" | "zh-Hant-MO" => "zh-CN".to_string(),
        "ja" => "ja-JP".to_string(),
        _ => base,
    }
}

/// Resolve the active locale: `PHICHAIN_LANG` > system locale > en-US fallback.
pub fn locale() -> String {
    std::env::var("PHICHAIN_LANG")
        .ok()
        .map(|loc| normalize_locale(&loc))
        .or(sys_locale::get_locale().map(|loc| normalize_locale(&loc)))
        .unwrap_or_else(|| "en-US".to_string())
}

/// Resolve a translation key to `&'static str` by leaking, for clap attributes that require it.
///
/// A macro, not a function, because `t!` must expand in the calling binary where `i18n!()` registered the translations.
#[macro_export]
macro_rules! i18n_str {
    ($key:expr) => {
        match ::rust_i18n::t!($key) {
            ::std::borrow::Cow::Borrowed(s) => s,
            ::std::borrow::Cow::Owned(s) => ::std::boxed::Box::leak(s.into_boxed_str()),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::normalize_locale;

    #[test]
    fn strips_posix_encoding_suffix_and_rewrites_separator() {
        assert_eq!(normalize_locale("zh_CN.UTF-8"), "zh-CN");
        assert_eq!(normalize_locale("en_US.UTF-8"), "en-US");
        assert_eq!(normalize_locale("ja_JP.UTF-8"), "ja-JP");
    }

    #[test]
    fn rewrites_posix_separator_without_encoding() {
        assert_eq!(normalize_locale("zh_CN"), "zh-CN");
        assert_eq!(normalize_locale("en_US"), "en-US");
        assert_eq!(normalize_locale("ja_JP"), "ja-JP");
    }

    #[test]
    fn c_and_posix_map_to_english() {
        assert_eq!(normalize_locale("C"), "en-US");
        assert_eq!(normalize_locale("POSIX"), "en-US");
    }

    #[test]
    fn macos_simplified_script_tags_collapse_to_zh_cn() {
        assert_eq!(normalize_locale("zh-Hans"), "zh-CN");
        assert_eq!(normalize_locale("zh-Hans-CN"), "zh-CN");
        assert_eq!(normalize_locale("zh-Hans-SG"), "zh-CN");
    }

    #[test]
    fn traditional_chinese_variants_fall_back_to_zh_cn_for_now() {
        // TODO: update this once zh-TW is added.
        assert_eq!(normalize_locale("zh-TW"), "zh-CN");
        assert_eq!(normalize_locale("zh-HK"), "zh-CN");
        assert_eq!(normalize_locale("zh-MO"), "zh-CN");
        assert_eq!(normalize_locale("zh-Hant"), "zh-CN");
        assert_eq!(normalize_locale("zh-Hant-TW"), "zh-CN");
        assert_eq!(normalize_locale("zh-Hant-HK"), "zh-CN");
    }

    #[test]
    fn bare_japanese_gets_regional_default() {
        assert_eq!(normalize_locale("ja"), "ja-JP");
    }

    #[test]
    fn supported_bcp47_tags_pass_through_unchanged() {
        assert_eq!(normalize_locale("en-US"), "en-US");
        assert_eq!(normalize_locale("zh-CN"), "zh-CN");
        assert_eq!(normalize_locale("ja-JP"), "ja-JP");
    }

    #[test]
    fn unsupported_locales_pass_through_so_they_fall_back_at_lookup() {
        assert_eq!(normalize_locale("fr-FR"), "fr-FR");
        assert_eq!(normalize_locale("de"), "de");
    }

    #[test]
    fn bare_chinese_gets_simplified_default() {
        assert_eq!(normalize_locale("zh"), "zh-CN");
    }
}
