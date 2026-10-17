use governed_coding_agent::agent::IterationRecord;
use governed_coding_agent::verification::{
    Artifact, ArtifactKind, Claim, ClaimCategory, EvidenceCollector, EvidenceLink,
    EvidenceRelationship, VerificationStatus,
};

#[test]
fn test_self_healing_progression_refuted_to_verified() {
    // ─── Attempt #1: Buggy initial patch triggers test failure ───
    let collector1 = EvidenceCollector::new(
        "run-heal-1",
        "https://github.com/nebius-community/sample-repo/issues/101",
        "Off-by-one index error",
    );

    let claim1_id = collector1.register_claim(Claim::new(
        "Off-by-one bug fixed and all sandbox tests pass",
        ClaimCategory::TestsPassing,
    ));

    // Sandbox execution fails with index out of bounds panic
    let fail_log = "thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 3'\ntest test_off_by_one ... FAILED";
    let fail_art = Artifact::new(ArtifactKind::TestOutput, fail_log, None);
    let fail_sha = fail_art.sha256.clone();
    let fail_art_id = collector1.record_artifact(fail_art);

    collector1.link_evidence(EvidenceLink {
        claim_id: claim1_id.clone(),
        artifact_id: fail_art_id,
        relationship: EvidenceRelationship::Contradicts,
        explanation: "Sandbox test panicked with index out of bounds".into(),
    });

    collector1.verify_claims();
    let report1 = collector1.finalize();

    // Check attempt #1 was refuted and trust score is 0
    let c1 = report1.claims.iter().find(|c| c.id == claim1_id).unwrap();
    assert_eq!(c1.verified, VerificationStatus::Refuted);
    assert_eq!(report1.summary.trust_score, 0.0);

    let iter1 = IterationRecord {
        iteration: 1,
        patch_summary: "Initial patch with boundary condition".into(),
        files_modified: vec!["src/lib.rs".into()],
        trust_score: report1.summary.trust_score,
        status: "Refuted (Caught Flaw)".into(),
        log_sha256: fail_sha,
        log_output: fail_log.into(),
        diff: "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -10,3 +10,3 @@\n- for i in 0..=len\n+ for i in 0..len + 1".into(),
        claims_verified: 0,
        total_claims: 1,
    };

    assert!(iter1.status.contains("Refuted"));
    assert_eq!(iter1.trust_score, 0.0);
    assert_eq!(iter1.claims_verified, 0);

    // ─── Attempt #2: Self-healing iteration with Nemotron produces clean fix ───
    let collector2 = EvidenceCollector::new(
        "run-heal-1",
        "https://github.com/nebius-community/sample-repo/issues/101",
        "Off-by-one index error",
    );

    let claim2_id = collector2.register_claim(Claim::new(
        "Off-by-one bug fixed and all sandbox tests pass",
        ClaimCategory::TestsPassing,
    ));

    // Sandbox execution succeeds
    let pass_log = "running 1 test\ntest test_off_by_one ... ok\ntest result: ok. 1 passed; 0 failed; finished in 0.02s";
    let pass_art = Artifact::new(ArtifactKind::TestOutput, pass_log, None);
    let pass_sha = pass_art.sha256.clone();
    let pass_art_id = collector2.record_artifact(pass_art);

    collector2.link_evidence(EvidenceLink {
        claim_id: claim2_id.clone(),
        artifact_id: pass_art_id,
        relationship: EvidenceRelationship::Supports,
        explanation: "All unit tests in sandbox passed with exit code 0".into(),
    });

    collector2.verify_claims();
    let report2 = collector2.finalize();

    // Check attempt #2 is verified and trust score is 100%
    let c2 = report2.claims.iter().find(|c| c.id == claim2_id).unwrap();
    assert_eq!(c2.verified, VerificationStatus::Verified);
    assert_eq!(report2.summary.trust_score, 100.0);

    let iter2 = IterationRecord {
        iteration: 2,
        patch_summary: "Self-healed patch with exclusive range 0..len".into(),
        files_modified: vec!["src/lib.rs".into()],
        trust_score: report2.summary.trust_score,
        status: "Verified (Self-Healed)".into(),
        log_sha256: pass_sha,
        log_output: pass_log.into(),
        diff: "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -10,3 +10,3 @@\n- for i in 0..=len\n+ for i in 0..len".into(),
        claims_verified: 1,
        total_claims: 1,
    };

    assert!(iter2.status.contains("Verified"));
    assert_eq!(iter2.trust_score, 100.0);
    assert_eq!(iter2.claims_verified, 1);

    // Verify progression in iteration history
    let iterations = vec![iter1, iter2];
    assert_eq!(iterations.len(), 2);
    assert_eq!(iterations[0].iteration, 1);
    assert!(iterations[0].status.contains("Refuted"));
    assert_eq!(iterations[1].iteration, 2);
    assert!(iterations[1].status.contains("Verified"));
    assert_ne!(iterations[0].log_sha256, iterations[1].log_sha256);
}

#[test]
fn test_self_healing_resolves_multi_claim_suite() {
    let collector = EvidenceCollector::new(
        "run-heal-multi",
        "https://github.com/nebius-community/sample-repo/issues/204",
        "Concurrency rate limiter race condition",
    );

    let claim_compile = collector.register_claim(Claim::new("Code compiles with 0 errors", ClaimCategory::NoCompileErrors));
    let claim_clippy = collector.register_claim(Claim::new("Zero clippy warnings emitted", ClaimCategory::NoClippyWarnings));
    let claim_test = collector.register_claim(Claim::new("All concurrency stress tests pass", ClaimCategory::TestsPassing));

    // Initial run: cargo check passes, clippy passes, but stress test fails
    let check_art = Artifact::new(ArtifactKind::CheckOutput, "Finished dev profile [unoptimized + debuginfo]", None);
    let check_id = collector.record_artifact(check_art);
    collector.link_evidence(EvidenceLink::supports(&claim_compile, &check_id, "cargo check passed"));

    let clippy_art = Artifact::new(ArtifactKind::ClippyOutput, "Checking sample-repo v0.1.0 (0 warnings)", None);
    let clippy_id = collector.record_artifact(clippy_art);
    collector.link_evidence(EvidenceLink::supports(&claim_clippy, &clippy_id, "cargo clippy passed with 0 warnings"));

    let fail_test_art = Artifact::new(ArtifactKind::TestOutput, "thread 'stress_test' panicked at 'race condition observed'", None);
    let fail_test_id = collector.record_artifact(fail_test_art);
    collector.link_evidence(EvidenceLink::contradicts(&claim_test, &fail_test_id, "Concurrency race condition detected"));

    collector.verify_claims();
    let rep1 = collector.finalize();

    // 2 verified, 1 refuted -> trust score should be roughly 66.67%
    assert_eq!(rep1.summary.verified, 2);
    assert_eq!(rep1.summary.refuted, 1);
    assert!((rep1.summary.trust_score - 66.67).abs() < 1.0);

    // Iteration 2: Self-healing resolves the failed test with passing concurrency test output
    let healed_art = Artifact::new(ArtifactKind::TestOutput, "test tests::test_concurrency_race_condition ... ok\ntest result: ok. 1 passed; 0 failed", None);
    collector.resolve_claim_with_healed_evidence(&claim_test, healed_art, "Mutex acquired across replenishment: test passed");

    collector.verify_claims();
    let rep2 = collector.finalize();

    // Now all 3 claims are verified -> trust score is 100%
    assert_eq!(rep2.summary.verified, 3);
    assert_eq!(rep2.summary.refuted, 0);
    assert_eq!(rep2.summary.trust_score, 100.0);
}
