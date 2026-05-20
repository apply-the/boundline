use boundline_core::domain::dashboard::{DashboardBrandMark, DashboardColorProfile};

pub const WORDMARK: &str = "boundline";

pub fn plain_wordmark() -> &'static str {
    WORDMARK
}

pub fn dashboard_branding(no_color: bool) -> DashboardBrandMark {
    DashboardBrandMark {
        wordmark_lines: vec![WORDMARK.to_string()],
        color_profile: if no_color {
            DashboardColorProfile::Monochrome
        } else {
            DashboardColorProfile::Color
        },
        min_width: 20,
        fallback_label: WORDMARK.to_string(),
    }
}
