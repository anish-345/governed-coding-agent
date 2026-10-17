use governed_coding_agent::agent::{FileChange, PrPreview};
use governed_coding_agent::api::StatusResponse;
use governed_coding_agent::github_tools::{IssueDetails, IssueFetcher};
use governed_coding_agent::verification::{
    Artifact, ArtifactKind, Claim, ClaimCategory, EvidenceCollector, EvidenceLink,
    EvidenceRelationship, ReportGenerator,
};

#[test]
fn test_read_only_github_pr_preview_fallback_mode() {
    // 1. Simulate an issue from a public repository where write permissions are absent
    let issue_url = "https://github.com/tokio-rs/tokio/issues/6880";
    let issue = IssueDetails {
        number: 6880,
        title: "mpsc unbounded channel memory exhaustion".into(),
        body: "Unbounded channel can consume arbitrary memory when receiver lags".into(),
        labels: vec!["bug".into(), "memory".into()],
        url: issue_url.into(),
        state: "open".into(),
    };

    assert_eq!(issue.number, 6880);
    assert_eq!(issue.state, "open");

    // 2. Generate Evidence and Markdown Verification Report
    let collector = EvidenceCollector::new("run-preview-1", issue_url, &issue.title);
    let claim_id = collector.register_claim(Claim::new(
        "Bounded buffer capacity safeguard introduced",
        ClaimCategory::CodeChangeCorrect,
    ));

    let diff_content = "--- a/tokio/src/sync/mpsc/unbounded.rs\n+++ b/tokio/src/sync/mpsc/unbounded.rs\n@@ -50,6 +50,8 @@\n+ const MAX_WARN_CAPACITY: usize = 100_000;\n+ if current_len > MAX_WARN_CAPACITY { warn!(\"channel backlog exceeded\"); }";
    let diff_art = Artifact::new(ArtifactKind::CodeDiff, diff_content, Some("tokio/src/sync/mpsc/unbounded.rs".into()));
    let diff_id = collector.record_artifact(diff_art);

    collector.link_evidence(EvidenceLink {
        claim_id,
        artifact_id: diff_id,
        relationship: EvidenceRelationship::Supports,
        explanation: "Static verification confirmed backlog warning safeguard added".into(),
    });

    collector.verify_claims();
    let report = collector.finalize();
    let md_report = ReportGenerator::to_markdown(&report);

    // 3. Construct PrPreview representing graceful fallback
    let pr_preview = PrPreview {
        title: format!("Fix: {}", issue.title),
        branch: format!("fix/issue-{}", issue.number),
        body: "Automated governed pull request with verifiable evidence trail".into(),
        diff: diff_content.into(),
        files: vec![FileChange {
            path: "tokio/src/sync/mpsc/unbounded.rs".into(),
            action: "modify".into(),
            content: diff_content.into(),
            diff_summary: "Added capacity warning check".into(),
        }],
        report_markdown: md_report.clone(),
    };

    assert_eq!(pr_preview.branch, "fix/issue-6880");
    assert!(pr_preview.diff.contains("MAX_WARN_CAPACITY"));
    assert!(pr_preview.report_markdown.contains("Verification Report"));
    assert!(pr_preview.report_markdown.contains("Trust Score"));

    // 4. Construct StatusResponse and ensure is_pr_preview is handled with no crashes
    let status_resp = StatusResponse {
        run_id: "run-preview-1".into(),
        status: "Completed (PR Preview Ready)".into(),
        pr_url: None,
        has_report: true,
        iterations: Vec::new(),
        is_pr_preview: true,
        pr_preview: Some(pr_preview),
        telemetry: None,
    };

    assert!(status_resp.is_pr_preview);
    assert!(status_resp.pr_url.is_none());
    assert!(status_resp.pr_preview.is_some());
    assert!(status_resp.has_report);
}

#[tokio::test]
async fn test_issue_fetcher_preset_parsing() {
    let fetcher = IssueFetcher::new("mock_token").expect("create fetcher");

    // Verify preset #101 parsing and retrieval
    let (owner, repo, number) = IssueFetcher::parse_issue_url("https://github.com/nebius-community/sample-repo/issues/101")
        .expect("parsed url 101");
    assert_eq!(owner, "nebius-community");
    assert_eq!(repo, "sample-repo");
    assert_eq!(number, 101);

    let issue_101: IssueDetails = fetcher
        .fetch(&owner, &repo, number)
        .await
        .expect("Preset #101 should be parsed and retrieved");

    assert_eq!(issue_101.number, 101);
    assert!(issue_101.title.contains("off-by-one") || issue_101.title.contains("Off-by-one"));

    // Verify preset #204 parsing and retrieval
    let (owner2, repo2, number2) = IssueFetcher::parse_issue_url("https://github.com/nebius-community/sample-repo/issues/204")
        .expect("parsed url 204");
    assert_eq!(owner2, "nebius-community");
    assert_eq!(repo2, "sample-repo");
    assert_eq!(number2, 204);

    let issue_204: IssueDetails = fetcher
        .fetch(&owner2, &repo2, number2)
        .await
        .expect("Preset #204 should be parsed and retrieved");

    assert_eq!(issue_204.number, 204);
    assert!(issue_204.title.contains("race condition") || issue_204.title.contains("Rate-Limiter"));
}
