use tree_sitter_highlight::HighlightConfiguration;

pub struct LanguageEntry {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    config_fn: fn() -> HighlightConfiguration,
}

impl LanguageEntry {
    pub fn config(&self, highlight_names: &[String]) -> HighlightConfiguration {
        let mut config = (self.config_fn)();
        config.configure(highlight_names);
        config
    }
}

macro_rules! lang {
    ($name:expr, $exts:expr, $lang_fn:expr, $highlights:expr) => {
        LanguageEntry {
            name: $name,
            extensions: $exts,
            config_fn: || {
                HighlightConfiguration::new(
                    $lang_fn.into(),
                    $name,
                    $highlights,
                    "", // injections
                    "", // locals
                )
                .expect(concat!("Failed to create highlight config for ", $name))
            },
        }
    };
}

pub fn language_entries() -> Vec<LanguageEntry> {
    vec![
        lang!(
            "rust",
            &["rs"],
            tree_sitter_rust::LANGUAGE,
            tree_sitter_rust::HIGHLIGHTS_QUERY
        ),
        lang!(
            "javascript",
            &["js", "jsx", "mjs", "cjs"],
            tree_sitter_javascript::LANGUAGE,
            tree_sitter_javascript::HIGHLIGHT_QUERY
        ),
        lang!(
            "typescript",
            &["ts", "tsx"],
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            tree_sitter_typescript::HIGHLIGHTS_QUERY
        ),
        lang!(
            "python",
            &["py", "pyi"],
            tree_sitter_python::LANGUAGE,
            tree_sitter_python::HIGHLIGHTS_QUERY
        ),
        lang!(
            "json",
            &["json", "jsonc"],
            tree_sitter_json::LANGUAGE,
            tree_sitter_json::HIGHLIGHTS_QUERY
        ),
        lang!(
            "toml",
            &["toml"],
            tree_sitter_toml_ng::LANGUAGE,
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY
        ),
        lang!(
            "css",
            &["css"],
            tree_sitter_css::LANGUAGE,
            tree_sitter_css::HIGHLIGHTS_QUERY
        ),
        lang!(
            "html",
            &["html", "htm"],
            tree_sitter_html::LANGUAGE,
            tree_sitter_html::HIGHLIGHTS_QUERY
        ),
        lang!(
            "go",
            &["go"],
            tree_sitter_go::LANGUAGE,
            tree_sitter_go::HIGHLIGHTS_QUERY
        ),
        lang!(
            "ruby",
            &["rb", "rake", "gemspec"],
            tree_sitter_ruby::LANGUAGE,
            tree_sitter_ruby::HIGHLIGHTS_QUERY
        ),
        lang!(
            "bash",
            &["sh", "bash", "zsh"],
            tree_sitter_bash::LANGUAGE,
            tree_sitter_bash::HIGHLIGHT_QUERY
        ),
        lang!(
            "yaml",
            &["yml", "yaml"],
            tree_sitter_yaml::LANGUAGE,
            tree_sitter_yaml::HIGHLIGHTS_QUERY
        ),
    ]
}

pub fn detect_language(path: &std::path::Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?;
    // Check against known extensions
    for entry in language_entries() {
        if entry.extensions.contains(&ext) {
            return Some(entry.name);
        }
    }
    // Filename-based detection
    let filename = path.file_name()?.to_str()?;
    match filename {
        "Makefile" | "makefile" | "GNUmakefile" => Some("bash"),
        "Dockerfile" => Some("bash"),
        "Gemfile" | "Rakefile" => Some("ruby"),
        _ => None,
    }
}
