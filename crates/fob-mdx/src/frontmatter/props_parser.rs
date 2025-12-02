//! Winnow parser for MDX prop expressions
//!
//! Parses expressions like: `github.repo("owner/name").field @refresh=60s @client`

use super::props::{PropArg, PropDefinition, PropOptions, RefreshStrategy};
use winnow::{
    Parser, Result as WResult,
    ascii::{alpha1, digit1, multispace0},
    combinator::{alt, delimited, opt, preceded, repeat, separated},
    token::{take_until, take_while},
};

/// Error type for prop parsing
#[derive(Debug, Clone, PartialEq)]
pub struct PropParseError {
    pub message: String,
    pub input: String,
}

impl std::fmt::Display for PropParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse prop '{}': {}", self.input, self.message)
    }
}

impl std::error::Error for PropParseError {}

/// Parse a duration string to seconds
///
/// Supports: "60s" (seconds), "5m" (minutes), "1h" (hours), "1d" (days)
fn parse_duration_seconds(duration: &str) -> Option<u64> {
    let len = duration.len();
    if len < 2 {
        return None;
    }

    let (num_str, unit) = duration.split_at(len - 1);
    let num: u64 = num_str.parse().ok()?;

    match unit {
        "s" => Some(num),
        "m" => Some(num * 60),
        "h" => Some(num * 3600),
        "d" => Some(num * 86400),
        _ => None,
    }
}

/// Parse a prop expression from frontmatter
///
/// # Example
/// ```ignore
/// let prop = parse_prop_expression("stars", "github.repo(\"owner/name\").stargazers_count @refresh=60s")?;
/// assert_eq!(prop.provider, "github");
/// assert_eq!(prop.method, "repo");
/// ```
pub fn parse_prop_expression(name: &str, input: &str) -> Result<PropDefinition, PropParseError> {
    let trimmed = input.trim();

    prop_expression
        .parse(trimmed)
        .map(|(provider, method, args, fields, options)| PropDefinition {
            name: name.to_string(),
            provider,
            method,
            args,
            fields: fields.unwrap_or_default(),
            options: options.unwrap_or_default(),
            raw: input.to_string(),
        })
        .map_err(|e| PropParseError {
            message: e.to_string(),
            input: input.to_string(),
        })
}

// Parser: provider.method(args).field1.field2 @refresh=60s @client
#[allow(clippy::type_complexity)]
fn prop_expression(
    input: &mut &str,
) -> WResult<(
    String,
    String,
    Vec<PropArg>,
    Option<Vec<String>>,
    Option<PropOptions>,
)> {
    let provider = identifier.parse_next(input)?;
    let _ = '.'.parse_next(input)?;
    let method = identifier.parse_next(input)?;
    let args = method_args.parse_next(input)?;
    let fields = opt(field_chain).parse_next(input)?;
    let options = opt(options_parser).parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    Ok((provider, method, args, fields, options))
}

// Parse identifier: alphanumeric + underscore, starting with alpha or _
fn identifier(input: &mut &str) -> WResult<String> {
    (
        alt((alpha1, "_")),
        take_while(0.., |c: char| c.is_alphanumeric() || c == '_'),
    )
        .take()
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

// Parse string literal: "content"
fn string_literal(input: &mut &str) -> WResult<String> {
    delimited('"', take_until(0.., '"'), '"')
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

// Parse a single method argument (string or number)
fn method_arg(input: &mut &str) -> WResult<PropArg> {
    alt((
        string_literal.map(PropArg::String),
        digit1.parse_to::<f64>().map(PropArg::Number),
    ))
    .parse_next(input)
}

// Parse method arguments: ("arg1", "arg2")
fn method_args(input: &mut &str) -> WResult<Vec<PropArg>> {
    delimited(
        '(',
        separated(
            0..,
            (multispace0, method_arg, multispace0).map(|(_, arg, _)| arg),
            ',',
        ),
        ')',
    )
    .parse_next(input)
}

// Parse field chain: .field1.field2
fn field_chain(input: &mut &str) -> WResult<Vec<String>> {
    repeat(1.., preceded('.', identifier)).parse_next(input)
}

// Parse a single option: @refresh=60s or @client
fn single_option(input: &mut &str) -> WResult<(String, Option<String>)> {
    let key = identifier.parse_next(input)?;
    let value = opt(preceded('=', option_value)).parse_next(input)?;
    Ok((key, value))
}

// Parse option value: duration (60s, 5m, 1h) or string
fn option_value(input: &mut &str) -> WResult<String> {
    alt((
        // Duration: number + unit
        (digit1, alt(('s', 'm', 'h', 'd')))
            .take()
            .map(|s: &str| s.to_string()),
        // Plain identifier
        identifier,
    ))
    .parse_next(input)
}

// Parse options: @refresh=60s @client
fn options_parser(input: &mut &str) -> WResult<PropOptions> {
    // First collect raw option values
    let options: Vec<(String, Option<String>)> =
        repeat(0.., preceded((multispace0, '@'), single_option)).parse_next(input)?;

    // Extract values
    let mut refresh_raw: Option<String> = None;
    let mut is_client = false;

    for (key, value) in options {
        match key.as_str() {
            "refresh" => refresh_raw = value,
            "client" => is_client = true,
            "server" => {} // Could be used for server-only in future
            _ => {}        // Unknown options ignored
        }
    }

    // Build RefreshStrategy from collected options
    let strategy = match (&refresh_raw, is_client) {
        (None, _) => RefreshStrategy::BuildTime,
        (Some(dur), false) => {
            let ttl_seconds = parse_duration_seconds(dur).unwrap_or(60);
            RefreshStrategy::RequestTime { ttl_seconds }
        }
        (Some(dur), true) => {
            let interval_seconds = parse_duration_seconds(dur).unwrap_or(60);
            RefreshStrategy::ClientTime { interval_seconds }
        }
    };

    Ok(PropOptions {
        strategy,
        refresh_raw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let prop = parse_prop_expression("repo", "github.repo(\"foxworth-uni/fob\")").unwrap();
        assert_eq!(prop.name, "repo");
        assert_eq!(prop.provider, "github");
        assert_eq!(prop.method, "repo");
        assert_eq!(
            prop.args,
            vec![PropArg::String("foxworth-uni/fob".to_string())]
        );
        assert!(prop.fields.is_empty());
        assert!(prop.options.strategy.is_build_time());
    }

    #[test]
    fn test_with_field_chain() {
        let prop =
            parse_prop_expression("stars", "github.repo(\"owner/name\").stargazers_count").unwrap();
        assert_eq!(prop.provider, "github");
        assert_eq!(prop.fields, vec!["stargazers_count".to_string()]);
    }

    #[test]
    fn test_with_refresh() {
        let prop = parse_prop_expression(
            "stars",
            "github.repo(\"owner/name\").stargazers_count @refresh=60s",
        )
        .unwrap();
        assert_eq!(prop.options.refresh_raw, Some("60s".to_string()));
        assert!(matches!(
            prop.options.strategy,
            RefreshStrategy::RequestTime { ttl_seconds: 60 }
        ));
    }

    #[test]
    fn test_with_client() {
        let prop =
            parse_prop_expression("prefs", "local.storage(\"prefs\") @refresh=30s @client").unwrap();
        assert!(matches!(
            prop.options.strategy,
            RefreshStrategy::ClientTime { interval_seconds: 30 }
        ));
    }

    #[test]
    fn test_full_expression() {
        let prop = parse_prop_expression(
            "stars",
            "github.repo(\"foxworth-uni/fob\").stargazers_count @refresh=60s @client",
        )
        .unwrap();
        assert_eq!(prop.provider, "github");
        assert_eq!(prop.method, "repo");
        assert_eq!(
            prop.args,
            vec![PropArg::String("foxworth-uni/fob".to_string())]
        );
        assert_eq!(prop.fields, vec!["stargazers_count".to_string()]);
        assert_eq!(prop.options.refresh_raw, Some("60s".to_string()));
        assert!(matches!(
            prop.options.strategy,
            RefreshStrategy::ClientTime { interval_seconds: 60 }
        ));
    }

    #[test]
    fn test_multiple_args() {
        let prop = parse_prop_expression("data", "api.query(\"users\", \"active\")").unwrap();
        assert_eq!(
            prop.args,
            vec![
                PropArg::String("users".to_string()),
                PropArg::String("active".to_string())
            ]
        );
    }

    #[test]
    fn test_nested_fields() {
        let prop = parse_prop_expression("avatar", "api.user(\"123\").profile.avatar.url").unwrap();
        assert_eq!(
            prop.fields,
            vec![
                "profile".to_string(),
                "avatar".to_string(),
                "url".to_string()
            ]
        );
    }

    #[test]
    fn test_duration_units() {
        // 5 minutes = 300 seconds
        let prop = parse_prop_expression("data", "api.get(\"x\") @refresh=5m").unwrap();
        assert_eq!(prop.options.refresh_raw, Some("5m".to_string()));
        assert!(matches!(
            prop.options.strategy,
            RefreshStrategy::RequestTime { ttl_seconds: 300 }
        ));

        // 1 hour = 3600 seconds
        let prop = parse_prop_expression("data", "api.get(\"x\") @refresh=1h").unwrap();
        assert_eq!(prop.options.refresh_raw, Some("1h".to_string()));
        assert!(matches!(
            prop.options.strategy,
            RefreshStrategy::RequestTime { ttl_seconds: 3600 }
        ));

        // 1 day = 86400 seconds
        let prop = parse_prop_expression("data", "api.get(\"x\") @refresh=1d").unwrap();
        assert_eq!(prop.options.refresh_raw, Some("1d".to_string()));
        assert!(matches!(
            prop.options.strategy,
            RefreshStrategy::RequestTime { ttl_seconds: 86400 }
        ));
    }

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(parse_duration_seconds("60s"), Some(60));
        assert_eq!(parse_duration_seconds("5m"), Some(300));
        assert_eq!(parse_duration_seconds("2h"), Some(7200));
        assert_eq!(parse_duration_seconds("1d"), Some(86400));
        assert_eq!(parse_duration_seconds("invalid"), None);
        assert_eq!(parse_duration_seconds(""), None);
    }
}
