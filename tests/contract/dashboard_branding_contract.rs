use crate::dashboard_fixture::{DashboardTestResult, require, require_eq};
use boundline::domain::dashboard::DashboardColorProfile;

#[test]
fn dashboard_branding_uses_simple_boundline_wordmark() -> DashboardTestResult {
    let colored = boundline_dashboard::branding::dashboard_branding(false);
    require_eq(colored.wordmark_lines, vec!["boundline".to_string()], "wordmark lines")?;
    require_eq(colored.color_profile, DashboardColorProfile::Color, "color profile")?;
    require_eq(colored.fallback_label, "boundline".to_string(), "fallback")?;
    require(colored.min_width <= 20, "wordmark must fit narrow terminal headers")
}

#[test]
fn no_color_branding_keeps_semantics_without_image_assets() -> DashboardTestResult {
    let plain = boundline_dashboard::branding::dashboard_branding(true);
    require_eq(plain.color_profile, DashboardColorProfile::Monochrome, "no-color profile")?;
    require_eq(boundline_dashboard::branding::plain_wordmark(), "boundline", "plain wordmark")
}
