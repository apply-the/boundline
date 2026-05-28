use boundline::{DistributionChannel, SUPPORTED_CANON_VERSION, supported_distribution_channels};

#[test]
fn supported_distribution_channels_always_include_source_fallback() {
    let channels = supported_distribution_channels();

    assert!(channels.contains(&DistributionChannel::Source));
    assert_eq!(SUPPORTED_CANON_VERSION, "0.61.0");
}
