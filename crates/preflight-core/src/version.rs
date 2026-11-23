use crate::model::Tool;
use semver::Version;
/// Parse only known version prefixes; never return raw process text.
pub fn parse(tool: Tool, text: &str) -> Option<Version> {
    for line in text.lines().map(str::trim) {
        let token = match tool {
            Tool::Python => line
                .strip_prefix("Python ")
                .and_then(|s| s.split_whitespace().next()),
            Tool::Git => line
                .strip_prefix("git version ")
                .and_then(|s| s.split_whitespace().next()),
            Tool::Node => line
                .strip_prefix('v')
                .filter(|s| !s.contains(char::is_whitespace)),
            Tool::Java => line
                .strip_prefix("openjdk version ")
                .or_else(|| line.strip_prefix("java version "))
                .or_else(|| line.strip_prefix("openjdk "))
                .or_else(|| line.strip_prefix("java "))
                .or_else(|| line.strip_prefix("javac "))
                .and_then(|s| s.split_whitespace().next())
                .map(|s| s.trim_matches('"')),
        };
        if let Some(token) = token {
            if let Some(v) = normalize(tool, token) {
                return stable(v);
            }
        }
    }
    None
}
fn normalize(tool: Tool, token: &str) -> Option<Version> {
    if token.len() > 80 {
        return None;
    }
    if tool == Tool::Node {
        return stable(Version::parse(token).ok()?);
    }
    let token = if tool == Tool::Git {
        token.split(".windows.").next()?
    } else {
        token
    };
    let token = if tool == Tool::Java {
        token.split('+').next()?
    } else {
        token
    };
    if let Ok(v) = Version::parse(token) {
        return Some(v);
    }
    if tool == Tool::Java {
        if let Some(legacy) = token.strip_prefix("1.") {
            let (base, update) = legacy.split_once('_').unwrap_or((legacy, "0"));
            let mut parts = base.split('.');
            let major = parts.next()?.parse().ok()?;
            if parts.next()? != "0" || parts.next().is_some() {
                return None;
            }
            return Some(Version::new(major, 0, update.parse().ok()?));
        }
        let parts = token
            .split('.')
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        return match parts.as_slice() {
            [a] => Some(Version::new(*a, 0, 0)),
            [a, b] => Some(Version::new(*a, *b, 0)),
            [a, b, c, _] => Some(Version::new(*a, *b, *c)),
            _ => None,
        };
    }
    None
}

fn stable(mut version: Version) -> Option<Version> {
    if !version.pre.is_empty() {
        return None;
    }
    version.build = semver::BuildMetadata::EMPTY;
    Some(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_versions() {
        for (t, s, v) in [
            (Tool::Python, "Python 3.12.7", "3.12.7"),
            (Tool::Git, "git version 2.53.0.windows.2", "2.53.0"),
            (Tool::Git, "git version 2.39.3 (Apple Git-146)", "2.39.3"),
            (Tool::Node, "v22.14.0", "22.14.0"),
            (
                Tool::Java,
                "openjdk version \"21.0.2\" 2024-01-16\nOpenJDK Runtime Environment",
                "21.0.2",
            ),
            (Tool::Java, "java version \"1.8.0_412\"", "8.0.412"),
            (Tool::Java, "openjdk 25 2025-09-16", "25.0.0"),
            (Tool::Java, "javac 17.0.10", "17.0.10"),
            (Tool::Java, "java version \"17.0.10+7\"", "17.0.10"),
        ] {
            assert_eq!(parse(t, s).unwrap().to_string(), v)
        }
    }
    #[test]
    fn rejects_unrelated_output() {
        assert!(parse(Tool::Python, "/home/alice/Python 3.12.7").is_none());
        assert!(parse(Tool::Node, "v22.x").is_none());
        assert!(parse(Tool::Python, "Python 3.13.0rc1").is_none());
    }
}
