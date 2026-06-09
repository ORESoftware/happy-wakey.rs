use std::collections::HashMap;
use std::path::Path;

/// Load `.env`, then parse CLI flags against `.cli-flags.toml`.
/// CLI > .env > system env > defaults from .cli-flags.toml.
pub fn init() {
    // 1. Load .env (does not overwrite existing env vars)
    let _ = dotenvy::from_path_iter(Path::new(".env"));

    // 2. Define flags (same schema as .cli-flags.toml)
    let entries = builtin_flags();

    // 3. Apply defaults first (lowest priority)
    for entry in &entries {
        let key = &entry.env;
        if std::env::var(key).is_err() {
            if let Some(ref d) = entry.default_val {
                std::env::set_var(key, d);
            }
        }
    }

    // 4. Parse CLI flags (highest priority)
    let args: Vec<String> = std::env::args().collect();
    let parsed = parse_flags(&args);

    for entry in &entries {
        if let Some(val) = resolve_flag(&parsed, entry) {
            std::env::set_var(&entry.env, &val);
        }
    }
}

// ---------------------------------------------------------------------------
// Flag definition
// ---------------------------------------------------------------------------

struct FlagEntry {
    env: String,
    aliases: Vec<String>,
    short: Option<String>,
    default_val: Option<String>,
}

fn builtin_flags() -> Vec<FlagEntry> {
    vec![
        FlagEntry {
            env: "SUPABASE_URL".into(),
            aliases: vec!["supabase-url".into()],
            short: Some("s".into()),
            default_val: Some("https://gtbeuxcolbpuipvqiibn.supabase.co".into()),
        },
        FlagEntry {
            env: "SUPABASE_ANON_KEY".into(),
            aliases: vec!["supabase-anon-key".into()],
            short: None,
            default_val: None,
        },
        FlagEntry {
            env: "OPENWEATHER_API_KEY".into(),
            aliases: vec!["openweather-api-key".into(), "owm-key".into()],
            short: Some("w".into()),
            default_val: None,
        },
        FlagEntry {
            env: "FINNHUB_API_KEY".into(),
            aliases: vec!["finnhub-api-key".into()],
            short: Some("f".into()),
            default_val: None,
        },
        FlagEntry {
            env: "NEWSAPI_KEY".into(),
            aliases: vec!["newsapi-key".into(), "news-api-key".into()],
            short: Some("n".into()),
            default_val: None,
        },
        FlagEntry {
            env: "GIT_REPO_PATH".into(),
            aliases: vec!["git-repo".into(), "git-repo-path".into()],
            short: None,
            default_val: None,
        },
        FlagEntry {
            env: "CONFIG_DIR".into(),
            aliases: vec!["config-dir".into()],
            short: None,
            default_val: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Simple flag parser
// ---------------------------------------------------------------------------

/// Map of alias → value (kebab-case, no leading dashes)
type ParsedFlags = HashMap<String, String>;

fn parse_flags(args: &[String]) -> ParsedFlags {
    let mut map = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        // --flag=value
        if arg.find("--").is_some() {
            // Only process if it starts with --
            if arg.starts_with("--") {
                let rest = &arg[2..];
                if let Some(eq) = rest.find('=') {
                    let name = &rest[..eq];
                    let value = &rest[eq + 1..];
                    map.insert(name.to_string(), value.to_string());
                    i += 1;
                    continue;
                }
            }
        }

        if arg.starts_with("--") {
            let name = &arg[2..];
            // --flag value (next arg)
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                map.insert(name.to_string(), args[i + 1].clone());
                i += 2;
                continue;
            }
            // --bool-flag (no value)
            map.insert(name.to_string(), "true".to_string());
            i += 1;
            continue;
        }

        if arg.starts_with('-') && !arg.starts_with("--") && arg.len() == 2 {
            let short = &arg[1..2];
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                map.insert(short.to_string(), args[i + 1].clone());
                i += 2;
                continue;
            }
            map.insert(short.to_string(), "true".to_string());
            i += 1;
            continue;
        }

        i += 1;
    }
    map
}

fn resolve_flag(parsed: &ParsedFlags, entry: &FlagEntry) -> Option<String> {
    // 1. Check long aliases
    for alias in &entry.aliases {
        if let Some(val) = parsed.get(alias) {
            return Some(val.clone());
        }
    }
    // 2. Check short
    if let Some(ref short) = entry.short {
        if let Some(val) = parsed.get(short) {
            return Some(val.clone());
        }
    }
    None
}
