#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Script {
    Ascii,
    Cjk,
}

pub fn split_by_script(s: &str) -> Vec<(String, Script)> {
    if s.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut current_script = None;
    let mut current_text = String::new();

    for c in s.chars() {
        let script = if c.is_ascii() {
            Script::Ascii
        } else {
            Script::Cjk
        };

        if Some(script) != current_script {
            if !current_text.is_empty() {
                result.push((current_text.clone(), current_script.unwrap()));
                current_text.clear();
            }
            current_script = Some(script);
        }

        current_text.push(c);
    }

    if !current_text.is_empty() {
        result.push((current_text, current_script.unwrap()));
    }

    result
}
