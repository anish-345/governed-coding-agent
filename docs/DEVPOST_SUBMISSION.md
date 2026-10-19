# Devpost Submission: Governed Coding Agent with Evidence Trail

## Basic Info
- **Project Title:** Governed Coding Agent with Evidence Trail
- **Tagline:** Judge-Proof AI Code Remediation with Cryptographic Evidence Trails and Self-Healing Sandbox Loops.
- **Track:** Coding & Agentic Engineering
- **Repository URL:** `https://github.com/YOUR_USERNAME/governed-coding-agent`
- **Video Demo URL:** `https://youtu.be/YOUR_VIDEO_ID`
- **Working Demo URL:** `http://YOUR_DEPLOYED_URL_OR_LOCALHOST`

---

## Devpost Project Story

### 💡 Inspiration
AI coding assistants have made huge strides, but when used in production, developers and teams face a critical trust deficit:
1. **Blind Trust in Test Claims:** When an AI agent submits a pull request saying *"All tests pass"*, how do you know the tests were actually run and didn't fail silently?
2. **Untracked Hallucinations:** Current tools have no governance mechanism to capture hallucinated API calls or subtle logic errors before code is pushed.
3. **Flawed Iteration:** If an initial fix causes a regression, existing agents either crash or repeatedly hammer broken patches into CI without root cause analysis.

We asked ourselves: **What if an AI coding agent was legally held to its claims?** What if every claim made by the model had to be cryptographically linked to an immutable, checkable artifact (such as a SHA-256 hashed test execution log or documentation source citation) before any PR could be merged?

That is **Governed Coding Agent**.

---

### 🚀 What It Does
Governed Coding Agent takes any GitHub issue URL and autonomously resolves it with a **provable evidence trail**:
1. **Grounding & Retrieval:** Fetches the issue context and uses **Tavily Search** to ground the analysis in official technical documentation and live web examples.
2. **Root Cause Analysis & Codegen:** Prompts **NVIDIA Nemotron-3 (via Nebius Token Factory)** to formulate a structured remediation plan and generate the patch diff.
3. **Sandbox Verification:** Executes the changes in an isolated sandbox (`cargo check`, `cargo clippy`, `cargo test`).
4. **Self-Healing Sandbox Loop:** If a test fails or a clippy warning is triggered, the claim is marked `Refuted`. The agent feeds the exact sandbox `stderr`, panic backtraces, and refuted claims back into Nemotron to generate a self-healed patch. Judges can view the **Refuted ➜ Verified** transition directly in the UI timeline!
5. **Cryptographic Artifact Registry:** Every patch diff, compilation log, test output, and web citation is assigned a SHA-256 hash.
6. **Graceful PR Delivery:** Opens a live GitHub Pull Request containing the full Markdown Verification Report, or smoothly falls back to **Verified PR Preview Mode** if running against external repositories where write permissions are absent (avoiding HTTP 403 crashes).

---

### 🏗️ How We Built It
- **Language & Systems Runtime:** Built entirely in **Rust** for rock-solid memory safety, concurrency, and deterministic execution.
- **Inference Engine:** **NVIDIA Nemotron-3 Super 120B** hosted on **Nebius Token Factory**, providing high-throughput code generation with low latency (~20-50 tokens/sec).
- **Web Grounding:** **Tavily Search API** for retrieving API reference documentation and real-world code examples to prevent hallucinations.
- **Sandboxed Execution:** Dual-mode architecture featuring `HostSandbox` (native isolated execution on Windows and Linux) and `DockerSandbox` with automatic graceful fallback.
- **API & Web Dashboard:** **Axum** async web framework serving a responsive, dark-mode dashboard with 1-click scenario pills, live sponsor telemetry metrics, self-healing timeline toggles, and PR copy modals.

---

### ⚡ Sponsor Technology Deep-Dive

#### 1. NVIDIA Nemotron via Nebius Token Factory
- **Why we used it:** We needed an enterprise-grade LLM capable of strict JSON instruction adherence, complex multi-file code reasoning, and precise error diagnosis.
- **How we used it:** Nemotron drives both the high-level planning phase and the recursive self-healing codegen loop. In the self-healing step, failed test output is fed directly into Nemotron's context window, allowing it to accurately pinpoint off-by-one errors and race conditions.
- **Performance:** Through Nebius Token Factory's OpenAI-compatible endpoint (`api.studio.nebius.ai/v1`), inference was consistently fast with live latency and token telemetry streamed directly to the frontend.

#### 2. Nebius Token Factory Sandboxes
- **Why we used it:** Running untrusted, model-generated code on production infrastructure is a severe security risk.
- **How we used it:** Sandboxed environments isolate test suites and clippy invocations, outputting raw logs whose cryptographic hashes are recorded in the evidence trail.

#### 3. Tavily Grounding
- **Why we used it:** Modern software stacks evolve faster than model training cutoffs. 
- **How we used it:** Tavily searches live documentation sites (docs.rs, GitHub issues, official guides) to ground Nemotron's solutions with real, clickable citations embedded in the verification report.

---

### 🤔 Challenges We Ran Into
- **Deterministic Trust Scoring:** Creating a formal mathematical mapping between dynamic test outputs, clippy warnings, and discrete claim states (`Verified`, `Refuted`, `Unverified`) required careful state machine modeling.
- **Cross-Platform Sandboxing:** Ensuring that both Windows development hosts and Linux container runtimes handle path separators, environment variables, and shell invocations identically.
- **Graceful Permission Trapping:** Avoiding 403 authorization crashes when judges test public repositories by implementing an unauthenticated REST fallback and PR Preview mode.

---

### 🏆 Accomplishments We're Proud Of
- **Self-Healing Demo:** A fully reproducible scenario (Demo 1) where judges can visually toggle between Attempt #1 (33% Trust Score, Refuted panic caught) and Attempt #2 (100% Trust Score, Self-Healed clean pass).
- **12/12 Passing Tests with 0 Compiler Warnings:** Comprehensive test coverage spanning LLM client mocks, API routing, verification math, self-healing state transitions, and read-only PR previews.
- **Judge-Proof UX:** 3 one-click preset scenario buttons that allow judges to evaluate the entire pipeline in under 30 seconds without needing to find or create a GitHub issue.

---

### 📚 What We Learned & Feedback for Nebius & NVIDIA
- **Nebius Token Factory:** The migration was seamless due to full OpenAI API compatibility. Latency was consistently stable. *Feature Request:* We would love to see native streaming for sandbox container logs and WebSocket status events.
- **NVIDIA Nemotron:** Demonstrated superior capability in understanding compiler error messages and self-correcting logic regressions compared to standard general-purpose models. *Feature Request:* A dedicated diff-generation decoding mode would further reduce prompt token overhead.
- **Tavily:** Search quality for code snippets and technical documentation is noticeably superior to general web search APIs.

---

### 🔮 What's Next for Governed Coding Agent
- **Continuous Multi-Agent Debate:** Pair Nemotron with an adversarial "Prosecutor" agent that attempts to write breaking unit tests specifically designed to disprove the patch.
- **CI/CD GitHub Action:** Packaging the governed engine as a standard GitHub Action that gates pull request merges based on minimum Trust Score thresholds.
- **On-Chain Cryptographic Attestations:** Publishing evidence trail hashes to decentralized ledgers for enterprise compliance and security auditing.
