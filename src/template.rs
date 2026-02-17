/// Context variables available for template substitution.
pub struct TemplateContext {
    pub filename: String,
    pub line_start: u32,
    pub line_end: u32,
    pub diff_content: String,
    pub context: String,
    pub comments: String,
    pub hunk_header: String,
}

/// Render a template string by substituting `{variable}` placeholders.
pub fn render_template(template: &str, ctx: &TemplateContext) -> String {
    template
        .replace("{filename}", &ctx.filename)
        .replace("{line_start}", &ctx.line_start.to_string())
        .replace("{line_end}", &ctx.line_end.to_string())
        .replace("{diff_content}", &ctx.diff_content)
        .replace("{context}", &ctx.context)
        .replace("{comments}", &ctx.comments)
        .replace("{hunk_header}", &ctx.hunk_header)
}
