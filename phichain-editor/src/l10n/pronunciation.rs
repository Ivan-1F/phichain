use pinyin::ToPinyin;
use rust_i18n::locale;

/// Checks if a target string contains the query string based on pronunciation
///
/// This function converts the target string to a pronunciation form according to the current locale
/// settings, and then checks if the converted target contains the query
///
/// # Supported Locales
///
/// - `zh_cn`: Converts Chinese characters to Pinyin (without tones)
/// - `en_us`: Uses the original string
/// - `ja_jp`: Converts Japanese characters to Romaji
///
/// Locales without specific support use the original string
pub fn match_pronunciation(query: &str, target: &str) -> bool {
    // keep this updated with lang/meta.json
    let target = match &*locale() {
        "zh_cn" => target
            .to_pinyin()
            .filter_map(|maybe_pinyin| maybe_pinyin.map(|pinyin| pinyin.plain()))
            .collect::<String>(),
        "ja_jp" => kakasi::convert(target).romaji,
        // english and locales without specific support use the original string
        _ => target.to_string(),
    }
    .replace(" ", "")
    .to_ascii_lowercase();

    target.contains(query)
}
