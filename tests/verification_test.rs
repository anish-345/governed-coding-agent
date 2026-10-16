use governed_coding_agent::verification::{
    Artifact, ArtifactKind, Claim, ClaimCategory, EvidenceCollector, EvidenceLink,
    EvidenceRelationship, ReportGenerator, VerificationStatus,
};

#[test]
fn test_artifact_sha256_hash_computation() {
    let content = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let artifact = Artifact::new(
        ArtifactKind::CodeDiff,
        content,
        Some("src/math.rs".into()),
    );

    assert_eq!(artifact.kind, ArtifactKind::CodeDiff);
    assert!(!artifact.sha256.is_empty());
    assert_eq!(artifact.sha256.len(), 64); // SHA-256 in hex is 64 chars

    // Hash should be deterministic
    let artifact2 = Artifact::new(
        ArtifactKind::CodeDiff,
        content,
        Some("src/math.rs".into()),
    );
    assert_eq!(artifact.sha256, artifact2.sha256);
}

#[test]
fn test_evidence_collector_claims_and_linking() {
    let collector = EvidenceCollector::new("run-1", "https://github.com/owner/repo/issues/1", "Issue title");

    // Register claims
    let claim1_id = collector.register_claim(Claim::new(
        "All unit tests pass",
        ClaimCategory::TestsPassing,
    ));
    let claim2_id = collector.register_claim(Claim::new(
        "No clippy warnings emitted",
        ClaimCategory::NoClippyWarnings,
    ));

    // Record supporting artifact for claim 1
    let art1 = Artifact::new(
        ArtifactKind::TestOutput,
        "test test_empty_vector ... ok\ntest result: ok. 1 passed; 0 failed;",
        None,
    );
    let art1_id = collector.record_artifact(art1);

    collector.link_evidence(EvidenceLink {
        claim_id: claim1_id.clone(),
        artifact_id: art1_id,
        relationship: EvidenceRelationship::Supports,
        explanation: "Unit test passed with 0 failures".into(),
    });

    // Verify claims
    collector.verify_claims();

    let report = collector.finalize();

    // Claim 1 should be verified
    let c1 = report.claims.iter().find(|c| c.id == claim1_id).unwrap();
    assert_eq!(c1.verified, VerificationStatus::Verified);

    // Claim 2 should be unverified (no evidence linked)
    let c2 = report.claims.iter().find(|c| c.id == claim2_id).unwrap();
    assert_eq!(c2.verified, VerificationStatus::Unverified);

    // Trust score: 1 verified out of 2 claims = 50%
    assert_eq!(report.summary.trust_score, 50.0);
}

#[test]
fn test_refuted_claim_when_contradicted() {
    let collector = EvidenceCollector::new("run-2", "https://github.com/owner/repo/issues/2", "Bug fix");

    let claim_id = collector.register_claim(Claim::new(
        "Code passes cargo clippy without any warnings",
        ClaimCategory::NoClippyWarnings,
    ));

    // Record contradicting artifact (clippy failed)
    let art = Artifact::new(
        ArtifactKind::ClippyOutput,
        "warning: unused variable `x`\n --> src/lib.rs:10:9\nwarning: 1 warning emitted",
        None,
    );
    let art_id = collector.record_artifact(art);

    collector.link_evidence(EvidenceLink {
        claim_id: claim_id.clone(),
        artifact_id: art_id,
        relationship: EvidenceRelationship::Contradicts,
        explanation: "Clippy found 1 warning".into(),
    });

    collector.verify_claims();
    let report = collector.finalize();

    let c = report.claims.iter().find(|c| c.id == claim_id).unwrap();
    assert_eq!(c.verified, VerificationStatus::Refuted);
    assert_eq!(report.summary.trust_score, 0.0);
}

#[test]
fn test_markdown_report_generation() {
    let collector = EvidenceCollector::new("run-3", "https://github.com/owner/repo/issues/3", "Doc update");

    let claim_id = collector.register_claim(Claim::new(
        "Updated API documentation according to docs.rs",
        ClaimCategory::BasedOnDocumentation,
    ));

    let art = Artifact::new(
        ArtifactKind::TavilySource,
        "Example API docs explaining the new syntax",
        Some("https://docs.rs/example/latest".into()),
    );
    let art_id = collector.record_artifact(art);

    collector.link_evidence(EvidenceLink {
        claim_id: claim_id.clone(),
        artifact_id: art_id,
        relationship: EvidenceRelationship::Supports,
        explanation: "Doc matches official docs.rs guide".into(),
    });

    let report = collector.finalize();
    let md = ReportGenerator::to_markdown(&report);

    assert!(md.contains("Verification Report"));
    assert!(md.contains("Trust Score"));
    assert!(md.contains("100.0%"));
    assert!(md.contains("Updated API documentation"));
    assert!(md.contains("Artifact Registry"));
}
