use anyhow::Result;
use astu::db::DuckDb;
use astu::normalize::Normalizer;
use astu::resolve::Host;
use astu::resolve::Target;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub fn render_command(template: &str, target: &Target, param: Option<&str>) -> String {
    let host = target.host().map_or_else(
        || "{host}".to_owned(),
        |h| match h {
            Host::Ip(ip) => ip.to_string(),
            Host::Domain(domain) => domain,
        },
    );
    let user = target.user().unwrap_or("{user}").to_owned();
    let ip = target
        .ip()
        .map_or_else(|| "{ip}".to_owned(), |x| x.to_string());
    let param = param.unwrap_or("{param}").to_owned();

    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert("host".to_owned(), host);
    vars.insert("user".to_owned(), user);
    vars.insert("ip".to_owned(), ip);
    vars.insert("param".to_owned(), param);

    strfmt::strfmt(template, &vars).unwrap_or_else(|_| template.to_owned())
}

pub fn task_template_values(target: &Target, param: Option<&str>) -> BTreeMap<String, String> {
    let mut vars = BTreeMap::new();
    if let Some(host) = target.host() {
        let host = match host {
            Host::Ip(ip) => ip.to_string(),
            Host::Domain(domain) => domain,
        };
        vars.insert("{host}".to_owned(), host);
    }
    if let Some(user) = target.user() {
        vars.insert("{user}".to_owned(), user.to_owned());
    }
    if let Some(ip) = target.ip() {
        vars.insert("{ip}".to_owned(), ip.to_string());
    }
    if let Some(param) = param {
        vars.insert("{param}".to_owned(), param.to_owned());
    }
    vars
}

pub async fn append_task_template_vars(
    db: &DuckDb,
    task_id: &str,
    target: &Target,
    param: Option<&str>,
) -> Result<()> {
    for (token, value) in task_template_values(target, param) {
        db.append_task_var(task_id, &token, &value).await?;
    }
    Ok(())
}

pub fn normalize_stream_bytes(normalizer: &Normalizer, bytes: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(bytes);
    let mut out = String::with_capacity(text.len());
    for (idx, line) in text.lines().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push_str(&normalizer.normalize(line));
    }
    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::str::FromStr;

    use astu::normalize::Normalizer;
    use astu::resolve::Target;

    use super::normalize_stream_bytes;
    use super::render_command;

    #[test]
    fn render_command_formats_target_vars() {
        let target = Target::from_str("dummy://alice@host.test").expect("target");
        let out = render_command("ssh {user}@{host} echo {param}", &target, Some("ok"));
        assert_eq!(out, "ssh alice@host.test echo ok");
    }

    #[test]
    fn normalize_stream_preserves_line_boundaries() {
        let mut vars = BTreeMap::new();
        vars.insert("{user}".to_owned(), "alice".to_owned());
        let normalizer = Normalizer::from_token_values(vars);
        let out = normalize_stream_bytes(&normalizer, b"user=alice\nuser=alice");
        assert_eq!(out, b"user={user}\nuser={user}");
    }
}
