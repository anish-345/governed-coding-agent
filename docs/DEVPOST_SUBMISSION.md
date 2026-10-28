# Devpost Submission: Governed Coding Agent with Evidence Trail

## Basic Info
- **Project Title:** Governed Coding Agent with Evidence Trail
- **Tagline:** The AI coding agent that actually proves its code works — runs tests in a sandbox, heals its own bugs, and gives you proof you can trust.
- **Track:** Coding & Agentic Engineering
- **Repository URL:** `https://github.com/YOUR_USERNAME/governed-coding-agent`
- **Video Demo URL:** `https://youtu.be/YOUR_VIDEO_ID`
- **Working Demo URL:** `http://localhost:8080` *(Runs locally via `cargo run -- serve` or Docker)*

---

## Devpost Project Story

### 💡 Inspiration: The AI Coding "Trust Problem"
Every developer who has used AI coding assistants knows the sinking feeling:
1. You ask an AI to fix a bug in your code.
2. It gives you a pull request with a confident description: *"Fixed! All tests pass."*
3. You merge it... and your application breaks in CI or crashes in production. 💥

Why? **Because existing AI assistants guess instead of proving.** They make assertions with zero proof, hallucinate outdated API methods, and when an initial fix fails, they either give up or blindly push broken commits.

We asked: **What if an AI coding agent had to prove its work before asking you to merge it?** What if every single claim was backed by real sandbox execution logs, hashed with SHA-256, and if a test failed, the agent automatically fixed its own mistakes until everything passed?

That is **Governed Coding Agent**.

---

### 🚀 What It Does (How It Helps You)
Governed Coding Agent takes any GitHub issue URL and autonomously resolves it with **verifiable proof**:

1. **Looks Up Real Documentation (Tavily Grounding):** Instead of guessing APIs from memory, it uses **Tavily Search** to find official technical documentation and real code examples.
2. **Understands & Plans (NVIDIA Nemotron on Nebius):** Uses **NVIDIA Nemotron-3 Super 120B** to diagnose the root cause and generate a structured plan.
3. **Pulls Real Files from GitHub:** Fetches the actual existing code files from the repository so the model edits real code rather than imagining it.
4. **Runs Real Tests in an Isolated Sandbox:** Compiles the patch and runs unit tests in a clean sandbox (`cargo check`, `cargo clippy`, `cargo test`).
5. **Self-Healing Loop (Refuted ➜ Verified):** If a test fails or panics, the agent doesn't crash or ask you to debug it. It feeds the failure message back to Nemotron and automatically heals the bug. You can visually see Attempt #1 (failed) and Attempt #2 (healed) right on the dashboard!
6. **Delivers Real Pull Requests with Proof:** Opens a live GitHub Pull Request containing a clean Markdown Verification Report and a SHA-256 evidence trail. If run on a public repository where you don't have write access, it safely switches to **Verified PR Preview Mode** so it never crashes with a 403 error.

---

### 🏗️ How We Built It
- **Systems Core:** Built in safe, async **Rust** (Axum, Tokio, Octocrab, Reqwest) for speed, deterministic execution, and memory safety.
- **Inference Engine:** **NVIDIA Nemotron-3 Super 120B** hosted on **Nebius Token Factory**, delivering fast, high-throughput code generation with live token and latency metrics.
- **Web Grounding:** **Tavily Search API** for retrieving live, up-to-date documentation citations.
- **Sandboxes:** Dual-mode execution featuring `HostSandbox` (local isolated directories) and `DockerSandbox`.
- **Dashboard:** A clean, dark-mode web UI with 1-click demo presets, live sponsor telemetry badges, interactive self-healing timeline, and diff viewer.

---

### ⚡ Sponsor Technology Deep-Dive

#### 1. NVIDIA Nemotron via Nebius Token Factory
- **Why we used it:** We needed an LLM with superior code reasoning that could understand compiler error messages and self-correct logic bugs.
- **How we used it:** Nemotron powers both the high-level planning and the recursive self-healing loop. Failed compiler panics are fed directly back into Nemotron's context window to produce the corrected fix.
- **Performance:** Through Nebius Token Factory's endpoint (`api.studio.nebius.ai/v1`), inference was consistently fast, and we stream real-time latency and token speed directly to the frontend.

#### 2. Nebius Token Factory Sandboxes
- **Why we used it:** Running unverified code on host infrastructure is dangerous.
- **How we used it:** Sandboxes isolate test runs and clippy checks, producing raw logs whose cryptographic hashes form the tamper-evident proof trail.

#### 3. Tavily Grounding
- **Why we used it:** Software libraries change faster than training cutoffs.
- **How we used it:** Tavily searches live docs (docs.rs, official guides) to ground Nemotron's code in real, verified patterns.

---

### 🏆 Accomplishments We're Proud Of
- **Genuine Self-Healing in Action:** A fully reproducible scenario (Demo 1) where you can watch Attempt #1 catch a panic on line 10, drop the Trust Score, and watch Nemotron fix it on Attempt #2 to reach 100% verified.
- **14/14 Passing Tests with 0 Compiler Warnings:** Full test coverage across LLM client mocks, API routes, verification math, Merkle root computation, and read-only PR previews.
- **1-Click Judge Experience:** 3 preset buttons on the dashboard let judges test the entire end-to-end pipeline in **15 seconds** without needing their own API keys or repository setup.
- **Zero 403 Crashes:** Seamlessly handles read-only external repositories by providing a full PR Preview.
- **Zero-Risk Private Repo Support:** Secure in-memory token authentication enables developers and teams to fix issues in private repositories without storing tokens on disk or in server logs.

---

### 📚 What We Learned & Feedback for Nebius & NVIDIA
- **Nebius Token Factory:** Extremely fast and simple to integrate due to full OpenAI API compatibility. *Feature Request:* Native WebSocket streaming for container sandbox execution logs would be awesome.
- **NVIDIA Nemotron:** Outstanding at understanding Rust compiler error diagnostics and fixing subtle boundary conditions. *Feature Request:* A dedicated patch/diff generation mode to reduce prompt overhead.
- **Tavily:** Search results for developer documentation and code snippets are significantly cleaner and more relevant than general web search engines.

---

### 🔮 What's Next for Governed Coding Agent
- **Multi-Language Sandboxes:** Adding out-of-the-box support for Python (`pytest`), Go (`go test`), and TypeScript (`npm test`).
- **Interactive Multi-Agent Debate:** Pairing Nemotron with an adversarial agent that generates aggressive edge-case tests to stress-test candidate fixes before deployment.
