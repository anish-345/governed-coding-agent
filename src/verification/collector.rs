use crate::verification::evidence::*;
use std::sync::{Arc, Mutex};
use tracing::info;

/// Collects artifacts and claims during an agent run, then links them.
/// Thread-safe via `Arc<Mutex<_>>` so it can be shared across async tasks.
#[derive(Debug, Clone)]
pub struct EvidenceCollector {
    inner: Arc<Mutex<CollectorState>>,
}

#[derive(Debug)]
struct CollectorState {
    run_id: String,
    issue_url: String,
    issue_title: String,
    artifacts: Vec<Artifact>,
    claims: Vec<Claim>,
    evidence_links: Vec<EvidenceLink>,
}

impl EvidenceCollector {
    /// Create a new collector for a given agent run.
    pub fn new(run_id: impl Into<String>, issue_url: impl Into<String>, issue_title: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CollectorState {
                run_id: run_id.into(),
                issue_url: issue_url.into(),
                issue_title: issue_title.into(),
                artifacts: Vec::new(),
                claims: Vec::new(),
                evidence_links: Vec::new(),
            })),
        }
    }

    /// Record an artifact and return its ID for linking.
    pub fn record_artifact(&self, artifact: Artifact) -> String {
        let id = artifact.id.clone();
        info!(
            artifact_id = %id,
            kind = ?artifact.kind,
            sha256 = %artifact.sha256,
            "Recorded artifact"
        );
        self.inner.lock().unwrap().artifacts.push(artifact);
        id
    }

    /// Register a claim and return its ID for linking.
    pub fn register_claim(&self, claim: Claim) -> String {
        let id = claim.id.clone();
        info!(claim_id = %id, statement = %claim.statement, "Registered claim");
        self.inner.lock().unwrap().claims.push(claim);
        id
    }

    /// Link a claim to an artifact with an explanation.
    pub fn link_evidence(&self, link: EvidenceLink) {
        info!(
            claim = %link.claim_id,
            artifact = %link.artifact_id,
            rel = ?link.relationship,
            "Linked evidence"
        );
        self.inner.lock().unwrap().evidence_links.push(link);
    }

    /// Convenience: record an artifact, register a claim, and link them.
    pub fn record_and_link(
        &self,
        artifact: Artifact,
        claim: Claim,
        relationship: EvidenceRelationship,
        explanation: impl Into<String>,
    ) -> (String, String) {
        let artifact_id = self.record_artifact(artifact);
        let claim_id = self.register_claim(claim);
        self.link_evidence(EvidenceLink {
            claim_id: claim_id.clone(),
            artifact_id: artifact_id.clone(),
            relationship,
            explanation: explanation.into(),
        });
        (artifact_id, claim_id)
    }

    /// Resolve a previously refuted claim with new healed evidence from a successful sandbox iteration.
    pub fn resolve_claim_with_healed_evidence(
        &self,
        claim_id: &str,
        new_artifact: Artifact,
        explanation: impl Into<String>,
    ) -> String {
        let mut state = self.inner.lock().unwrap();
        // Remove prior contradictory links for this claim
        state
            .evidence_links
            .retain(|l| !(l.claim_id == claim_id && l.relationship == EvidenceRelationship::Contradicts));
        let artifact_id = new_artifact.id.clone();
        state.artifacts.push(new_artifact);
        state.evidence_links.push(EvidenceLink {
            claim_id: claim_id.to_string(),
            artifact_id: artifact_id.clone(),
            relationship: EvidenceRelationship::Supports,
            explanation: explanation.into(),
        });
        artifact_id
    }

    /// Verify all claims by checking whether they have supporting/contradicting evidence.
    pub fn verify_claims(&self) {
        let mut state = self.inner.lock().unwrap();
        let evidence_links = state.evidence_links.clone();
        for claim in &mut state.claims {
            let links: Vec<&EvidenceLink> = evidence_links
                .iter()
                .filter(|l| l.claim_id == claim.id)
                .collect();

            if links.is_empty() {
                claim.verified = VerificationStatus::Unverified;
                continue;
            }

            let has_contradiction = links
                .iter()
                .any(|l| l.relationship == EvidenceRelationship::Contradicts);
            let has_support = links
                .iter()
                .any(|l| l.relationship == EvidenceRelationship::Supports);

            claim.verified = if has_contradiction {
                VerificationStatus::Refuted
            } else if has_support {
                VerificationStatus::Verified
            } else {
                VerificationStatus::Unverified
            };
        }
    }

    /// Update a specific claim's verification status directly.
    pub fn set_claim_status(&self, claim_id: &str, status: VerificationStatus) {
        let mut state = self.inner.lock().unwrap();
        if let Some(claim) = state.claims.iter_mut().find(|c| c.id == claim_id) {
            claim.verified = status;
        }
    }

    /// Finalize and produce the complete verification report.
    pub fn finalize(&self) -> VerificationReport {
        // Run verification before finalizing.
        self.verify_claims();

        let state = self.inner.lock().unwrap();
        let summary = ReportSummary::from_claims(&state.claims);

        VerificationReport {
            run_id: state.run_id.clone(),
            issue_url: state.issue_url.clone(),
            issue_title: state.issue_title.clone(),
            created_at: chrono::Utc::now(),
            claims: state.claims.clone(),
            artifacts: state.artifacts.clone(),
            evidence_links: state.evidence_links.clone(),
            summary,
        }
    }
}
