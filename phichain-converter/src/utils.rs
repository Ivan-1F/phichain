use rust_i18n::t;
use std::borrow::Cow;

pub fn i18n_str(key: &str) -> &str {
    match t!(key) {
        Cow::Borrowed(s) => s,
        Cow::Owned(_) => unreachable!(),
    }
}
