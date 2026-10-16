use governed_coding_agent::github_tools::IssueFetcher;

#[test]
fn test_parse_valid_issue_urls() {
    let url = "https://github.com/nebius/token-factory/issues/105";
    let (owner, repo, number) = IssueFetcher::parse_issue_url(url).expect("Should parse valid URL");
    assert_eq!(owner, "nebius");
    assert_eq!(repo, "token-factory");
    assert_eq!(number, 105);

    let url_trailing = "https://github.com/nvidia/nemotron/issues/42/";
    let (owner, repo, number) = IssueFetcher::parse_issue_url(url_trailing).expect("Should parse trailing slash");
    assert_eq!(owner, "nvidia");
    assert_eq!(repo, "nemotron");
    assert_eq!(number, 42);
}

#[test]
fn test_parse_invalid_issue_urls() {
    let bad_urls = [
        "https://example.com/not/github",
        "https://github.com/onlyowner",
        "https://github.com/owner/repo/pull/42",
        "https://github.com/owner/repo/issues/notanumber",
        "not_a_url",
    ];

    for bad in bad_urls {
        assert!(
            IssueFetcher::parse_issue_url(bad).is_err(),
            "Expected error for URL: {bad}"
        );
    }
}
