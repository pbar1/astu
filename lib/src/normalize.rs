use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Normalizer {
    deduplicators: Vec<(String, String)>,
}

impl Normalizer {
    #[must_use]
    pub fn from_replacements(replacements: BTreeMap<String, String>) -> Self {
        let mut pairs: Vec<(String, String)> = replacements.into_iter().collect();
        pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        Self {
            deduplicators: pairs,
        }
    }

    #[must_use]
    pub fn from_token_values(token_values: BTreeMap<String, String>) -> Self {
        let replacements = token_values
            .into_iter()
            .map(|(token, value)| (value, token))
            .collect::<BTreeMap<_, _>>();
        Self::from_replacements(replacements)
    }

    #[must_use]
    pub fn normalize(&self, input: &str) -> String {
        let mut out = input.to_string();
        for (from, to) in &self.deduplicators {
            out = replace_token(&out, from, to);
        }
        out
    }
}

fn replace_token(input: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return input.to_string();
    }

    let mut out = String::with_capacity(input.len());
    let mut cursor = 0usize;
    for (start, matched) in input.match_indices(needle) {
        let end = start + matched.len();
        if !is_token_boundary(input, start, end) {
            continue;
        }
        out.push_str(&input[cursor..start]);
        out.push_str(replacement);
        cursor = end;
    }
    out.push_str(&input[cursor..]);
    out
}

fn is_token_boundary(s: &str, start: usize, end: usize) -> bool {
    let left_ok = s[..start]
        .chars()
        .next_back()
        .is_none_or(|c| !is_word_char(c));
    let right_ok = s[end..].chars().next().is_none_or(|c| !is_word_char(c));
    left_ok && right_ok
}

const fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Normalizer;

    #[test]
    fn normalizes_known_variators() {
        let mut map = BTreeMap::new();
        map.insert("{user}".to_string(), "alice".to_string());
        map.insert("{host}".to_string(), "my-host".to_string());
        map.insert("{param}".to_string(), "hello world".to_string());

        let n = Normalizer::from_token_values(map);
        let line = "user=alice host=my-host input=hello world";
        assert_eq!(n.normalize(line), "user={user} host={host} input={param}");
    }

    #[test]
    fn does_not_replace_substrings_inside_other_tokens() {
        let mut map = BTreeMap::new();
        map.insert("{param}".to_string(), "1".to_string());
        let n = Normalizer::from_token_values(map);
        assert_eq!(n.normalize("bucket=10 n=1"), "bucket=10 n={param}");
    }
}
