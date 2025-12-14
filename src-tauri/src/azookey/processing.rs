use std::collections::HashMap;
use std::sync::LazyLock;

use azookey_binding::Candidate;
use itertools::Itertools;


pub static SIGNMAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("-", "ー"),
        ("=", "＝"),
        ("[", "「"),
        ("]", "」"),
        (";", "；"),
        ("@", "＠"),
        (",", "、"),
        (".", "。"),
        ("/", "・"),
        ("!", "！"),
        ("#", "＃"),
        ("$", "＄"),
        ("%", "％"),
        ("^", "＾"),
        ("&", "＆"),
        ("*", "＊"),
        ("(", "（"),
        (")", "）"),
        ("_", "＿"),
        ("+", "＋"),
        ("{", "｛"),
        ("}", "｝"),
        ("|", "｜"),
        (":", "："),
        ("\"", "”"),
        ("<", "＜"),
        (">", "＞"),
        ("?", "？"),
        ("\\", "￥"),
    ])
});

pub fn pre_process_text(text: &str) -> String {
    let mut result = String::new();

    // replace all characters in the text with their corresponding replacements
    for c in text.chars() {
        if let Some(&replacement) = SIGNMAP.get(c.to_string().as_str()) {
            result.push_str(replacement);
        } else {
            result.push(c);
        }
    }

    // push 'n' if the last and second last characters are 'n'
    if result.ends_with('n') {
        let mut chars = result.chars().collect::<Vec<_>>();
        if chars.len() > 1 && chars[chars.len() - 2] != 'n' {
            chars.push('n');
        }
        result = chars.into_iter().collect::<String>();
    }

    // push '§' at the end of the string to avoid unnecessary prediction
    result.push_str("§");

    result
}

pub fn post_process_text(text: &str) -> String {
    let mut result = text.to_string();

    if result.ends_with('§') {
        result.pop();
    }

    result
}

pub fn post_process_candidates(candidates: Vec<Candidate>) -> Vec<Candidate> {
    candidates
        .iter()
        .take(8)
        .map(|c| {
            let mut candidate = c.clone();
            candidate.text = post_process_text(&candidate.text);
            candidate
        })
        .unique_by(|c| c.text.clone())
        .collect()
}
