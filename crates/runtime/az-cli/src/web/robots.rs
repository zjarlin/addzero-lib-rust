use anyhow::{Context, Result};
use regex::Regex;
use reqwest::StatusCode;
use reqwest::Url;
use reqwest::blocking::Client;

#[derive(Debug)]
pub struct RobotsPolicy {
    rules: Vec<Rule>,
}

#[derive(Debug)]
struct Rule {
    allow: bool,
    raw_pattern: String,
    matcher: Regex,
}

#[derive(Debug, Default)]
struct Group {
    user_agents: Vec<String>,
    rules: Vec<RawRule>,
}

#[derive(Debug)]
struct RawRule {
    allow: bool,
    raw_pattern: String,
}

impl RobotsPolicy {
    pub fn load(client: &Client, start_url: &Url, user_agent: &str) -> Result<Self> {
        let robots_url = robots_url(start_url)?;
        let response = client
            .get(robots_url.clone())
            .send()
            .with_context(|| format!("request robots.txt: {}", robots_url.as_str()))?;

        match response.status() {
            StatusCode::NOT_FOUND | StatusCode::GONE => return Ok(Self { rules: Vec::new() }),
            status if !status.is_success() => {
                anyhow::bail!(
                    "robots.txt request failed with status {} for {}",
                    status,
                    robots_url
                );
            }
            _ => {}
        }

        let body = response.text().context("read robots.txt body")?;
        Self::from_text(&body, user_agent)
    }

    pub fn ensure_allowed(&self, url: &Url) -> Result<()> {
        if self.is_allowed(url) {
            return Ok(());
        }

        anyhow::bail!("robots.txt disallows {}", url.as_str())
    }

    pub(crate) fn from_text(text: &str, user_agent: &str) -> Result<Self> {
        let groups = parse_groups(text);
        let user_agent = user_agent.to_ascii_lowercase();

        let matching_groups = groups
            .iter()
            .filter(|group| {
                group
                    .user_agents
                    .iter()
                    .any(|candidate| candidate != "*" && user_agent.contains(candidate.as_str()))
            })
            .collect::<Vec<_>>();

        let selected_groups = if matching_groups.is_empty() {
            groups
                .iter()
                .filter(|group| group.user_agents.iter().any(|candidate| candidate == "*"))
                .collect::<Vec<_>>()
        } else {
            matching_groups
        };

        let rules = selected_groups
            .into_iter()
            .flat_map(|group| group.rules.iter())
            .filter(|rule| !rule.raw_pattern.is_empty())
            .map(Rule::compile)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { rules })
    }

    fn is_allowed(&self, url: &Url) -> bool {
        let mut path = url.path().to_owned();
        if let Some(query) = url.query() {
            path.push('?');
            path.push_str(query);
        }

        let best_match = self
            .rules
            .iter()
            .filter(|rule| rule.matcher.is_match(&path))
            .max_by_key(|rule| rule.raw_pattern.len());

        best_match.map(|rule| rule.allow).unwrap_or(true)
    }
}

impl RawRule {
    fn allow(raw_pattern: String) -> Self {
        Self {
            allow: true,
            raw_pattern,
        }
    }

    fn disallow(raw_pattern: String) -> Self {
        Self {
            allow: false,
            raw_pattern,
        }
    }
}

impl Rule {
    fn compile(rule: &RawRule) -> Result<Self> {
        let matcher = compile_pattern(&rule.raw_pattern)?;
        Ok(Self {
            allow: rule.allow,
            raw_pattern: rule.raw_pattern.clone(),
            matcher,
        })
    }
}

fn robots_url(start_url: &Url) -> Result<Url> {
    let host = start_url
        .host_str()
        .context("start URL does not contain a host")?;
    let mut robots = format!("{}://{}", start_url.scheme(), host);
    if let Some(port) = start_url.port() {
        robots.push(':');
        robots.push_str(&port.to_string());
    }
    robots.push_str("/robots.txt");
    Url::parse(&robots).with_context(|| format!("build robots.txt URL from {}", start_url))
}

fn parse_groups(text: &str) -> Vec<Group> {
    let mut groups = Vec::new();
    let mut current = Group::default();
    let mut has_rules = false;

    for raw_line in text.lines() {
        let line = strip_comments(raw_line).trim().to_owned();
        if line.is_empty() {
            if !current.user_agents.is_empty() || !current.rules.is_empty() {
                groups.push(std::mem::take(&mut current));
                has_rules = false;
            }
            continue;
        }

        let Some((field, value)) = line.split_once(':') else {
            continue;
        };
        let field = field.trim().to_ascii_lowercase();
        let value = value.trim().to_ascii_lowercase();

        match field.as_str() {
            "user-agent" => {
                if has_rules {
                    groups.push(std::mem::take(&mut current));
                    has_rules = false;
                }
                current.user_agents.push(value);
            }
            "allow" => {
                current.rules.push(RawRule::allow(value));
                has_rules = true;
            }
            "disallow" => {
                current.rules.push(RawRule::disallow(value));
                has_rules = true;
            }
            _ => {}
        }
    }

    if !current.user_agents.is_empty() || !current.rules.is_empty() {
        groups.push(current);
    }

    groups
}

fn strip_comments(line: &str) -> &str {
    line.split('#').next().unwrap_or(line)
}

fn compile_pattern(pattern: &str) -> Result<Regex> {
    let anchored = pattern.ends_with('$');
    let source = if anchored {
        &pattern[..pattern.len() - 1]
    } else {
        pattern
    };

    let escaped = regex::escape(source).replace(r"\*", ".*");
    let expression = if anchored {
        format!("^{escaped}$")
    } else {
        format!("^{escaped}")
    };

    Regex::new(&expression)
        .with_context(|| format!("compile robots.txt pattern `{pattern}` into regex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_matching_rule_should_win() {
        let policy = RobotsPolicy::from_text(
            "User-agent: *\nDisallow: /book\nAllow: /book/public\n",
            "addzdero-cli/0.1",
        )
        .expect("policy parses");

        assert!(
            policy.is_allowed(&Url::parse("https://example.com/book/public/ch1").expect("url"))
        );
        assert!(!policy.is_allowed(&Url::parse("https://example.com/book/hidden").expect("url")));
    }

    #[test]
    fn specific_group_should_override_wildcard_group_selection() {
        let text = "\
User-agent: *\n\
Disallow: /\n\
\n\
User-agent: addzdero-cli\n\
Allow: /novel\n\
Disallow: /novel/private\n";

        let policy = RobotsPolicy::from_text(text, "addzdero-cli/0.1").expect("policy parses");

        assert!(policy.is_allowed(&Url::parse("https://example.com/novel/1").expect("url")));
        assert!(
            !policy.is_allowed(&Url::parse("https://example.com/novel/private/1").expect("url"))
        );
    }
}
