/// All prompt templates used by the agent.
/// Centralised here for easy tuning and auditability.
pub struct Prompts;

impl Prompts {
    // ─────────────────────────────────────────
    //  Phase 1: Issue Analysis & Planning
    // ─────────────────────────────────────────

    pub const PLANNER_SYSTEM: &'static str = r#"You are a senior software engineer analyzing a GitHub issue. Your job is to produce a structured plan to resolve the issue.

Rules:
1. Output ONLY valid JSON — no markdown fences, no explanation outside the JSON.
2. Use this exact schema:
{
  "analysis": "Brief description of the root cause",
  "files_to_modify": ["path/to/file1.rs", "path/to/file2.rs"],
  "steps": [
    {"step": 1, "action": "description of what to do", "file": "path/to/file.rs"},
    ...
  ],
  "test_strategy": "How to verify the fix",
  "claims": [
    "Claim the agent will make, e.g. 'Fix resolves the null pointer in parse()'",
    "All existing tests continue to pass",
    ...
  ]
}
3. The "claims" field is critical: list every assertion you will make about the fix. Each claim will be verified against artifacts later.
4. Be specific and actionable. Reference exact function names and line numbers when possible."#;

    pub fn planner_user(issue_title: &str, issue_body: &str, tavily_context: &str) -> String {
        format!(
            r#"GitHub Issue:
Title: {issue_title}
Body:
{issue_body}

Relevant Documentation (from Tavily search):
{tavily_context}

Produce your analysis and plan as JSON."#
        )
    }

    // ─────────────────────────────────────────
    //  Phase 2: Code Generation
    // ─────────────────────────────────────────

    pub const CODEGEN_SYSTEM: &'static str = r#"You are a senior software engineer generating code to fix a GitHub issue. You receive a plan and must produce the exact code changes.

Rules:
1. Output ONLY valid JSON — no markdown fences.
2. Use this exact schema:
{
  "files": [
    {
      "path": "src/example.rs",
      "action": "modify",
      "content": "full file content after changes",
      "diff_summary": "Human-readable summary of what changed"
    }
  ],
  "new_tests": [
    {
      "path": "tests/test_fix.rs",
      "content": "full test file content"
    }
  ],
  "explanation": "Brief explanation of the fix"
}
3. "action" can be "modify", "create", or "delete".
4. For "modify", provide the COMPLETE file content, not just the diff.
5. Ensure the code compiles. Do not leave TODO placeholders."#;

    pub fn codegen_user(plan_json: &str, existing_code: &str) -> String {
        format!(
            r#"Plan:
{plan_json}

Existing code context:
{existing_code}

Generate the code changes as JSON."#
        )
    }

    // ─────────────────────────────────────────
    //  Phase 3: Fix Iteration (if tests fail)
    // ─────────────────────────────────────────

    pub const FIX_SYSTEM: &'static str = r#"You are a senior software engineer. The previous code change failed compilation or tests. Analyze the error output and produce a corrected version.

Rules:
1. Output ONLY valid JSON with the same schema as before (files array).
2. Fix the root cause — do not just suppress the error.
3. If multiple errors exist, fix all of them.
4. Preserve all existing tests."#;

    pub fn fix_user(previous_code: &str, error_output: &str) -> String {
        format!(
            r#"Previous code that failed:
{previous_code}

Error output:
{error_output}

Produce corrected code as JSON."#
        )
    }
}
