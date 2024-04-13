use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KushConfig {
    pub inference: Vec<Inference>,
    pub providers: Vec<Provider>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Inference {
    pub provider: String,
    pub pattern: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    pub uri: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub filters: Vec<Filter>,
    pub shell: Shell,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    pub vars: String,
    pub cmd: Vec<String>,
    pub preview: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shell {
    pub cmd: Vec<String>,
}

impl KushConfig {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let cfg: Self = confy::load_path(path)?;
        Ok(cfg)
    }

    // FIXME: Playing with Url's parsing. Map this into a real usable provider
    pub fn play(&self, target: &str) -> Result<()> {
        'provider_loop: for provider in &self.providers {
            let uri = Url::parse(&provider.uri)?;

            let mut map = HashMap::new();

            if !uri.scheme().is_empty() {
                map.insert("scheme".to_owned(), uri.scheme());
            }
            if !uri.username().is_empty() {
                map.insert("username".to_owned(), uri.username());
            }
            if let Some(host) = uri.host_str() {
                map.insert("host".to_owned(), host);
            }
            if let Some(path_segments) = uri.path_segments() {
                for (i, seg) in path_segments.enumerate() {
                    let key = format!("path:{i}");
                    map.insert(key, seg);
                }
            }
            if let Some(fragment) = uri.fragment() {
                map.insert("fragment".to_owned(), fragment);
            }
            // TODO: password, port, query; other types like not-a-base

            let target_uri = Url::parse(target)?;

            let mut target_map = HashMap::new();

            if !target_uri.scheme().is_empty() {
                target_map.insert("scheme".to_owned(), target_uri.scheme());
            }
            if !target_uri.username().is_empty() {
                target_map.insert("username".to_owned(), target_uri.username());
            }
            if let Some(host) = target_uri.host_str() {
                target_map.insert("host".to_owned(), host);
            }
            if let Some(path_segments) = target_uri.path_segments() {
                for (i, seg) in path_segments.enumerate() {
                    let key = format!("path:{i}");
                    target_map.insert(key, seg);
                }
            }
            if let Some(fragment) = target_uri.fragment() {
                target_map.insert("fragment".to_owned(), fragment);
            }

            for (key, val) in &map {
                let target_def = target_map.get(key);

                // WRONG! this entire loop is bad logic
                if key == "scheme" && val != target_def.unwrap() {
                    continue 'provider_loop;
                }

                println!("{:?} -> {:?}", val, target_def);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn get_test_config() -> KushConfig {
        KushConfig::new("testdata/config.yaml").unwrap()
    }

    #[rstest]
    #[case("k8s://")]
    #[case("k8s://my-ns")]
    #[case("k8s://my-ns/my-pod")]
    #[case("k8s://my-ns/my-pod/my-ctr")]
    #[case("k8s:///my-pod")]
    #[case("k8s:///my-pod/my-ctr")]
    #[case("docker:///my-ctr")]
    fn test_play(#[case] target: &str) {
        let cfg = get_test_config();
        let _ = cfg.play(target);
    }
}
