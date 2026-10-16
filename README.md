# 🔍 Governed Coding Agent

**AI code fixes with verifiable evidence trails.**

> Built for the [Nebius × NVIDIA Global AI Hackathon](https://nebiusglobalaihackathon.devpost.com/) · Track: Coding & Agentic Engineering

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![NVIDIA](https://img.shields.io/badge/NVIDIA-Nemotron-76b900.svg)](https://build.nvidia.com/)
[![Nebius](https://img.shields.io/badge/Nebius-Token%20Factory-4f8cff.svg)](https://nebius.com/)

---

## 💡 Inspiration

AI coding agents can generate impressive-looking patches, but there's a trust problem: **how do you know the fix actually works?** Current agents say "all tests pass" without showing the test output. They claim to follow best practices without citing their sources. We built an agent that **proves every claim it makes** with checkable artifacts.

## 🚀 What It Does

The Governed Coding Agent takes a GitHub issue URL and:

1. **Analyzes** the issue using NVIDIA Nemotron (via Nebius Token Factory)
2. **Grounds** the analysis in real documentation via Tavily search
3. **Generates** a code fix with a structured plan
4. **Executes** tests in an isolated sandbox (Nebius Sandboxes or Docker)
5. **Verifies** every claim against concrete artifacts
6. **Opens a Pull Request** with a full verification report

The **differentiator** is the **evidence trail**: every claim ("tests pass", "no warnings", "based on docs") is linked to a SHA-256 hashed artifact — test output, clippy logs, Tavily source URLs — that anyone can independently verify.

## 🏗️ How We Built It

- **Language:** Rust (edition 2021) — chosen for type safety, async performance, and reliability
- **LLM:** NVIDIA Nemotron via Nebius Token Factory (OpenAI-compatible API)
- **Grounding:** Tavily Search & Extract API for documentation citations
- **Sandbox:** Pluggable trait-based design — Nebius Token Factory Sandboxes or Docker fallback
- **GitHub Integration:** octocrab for issue fetching, branch creation, and PR management
- **Web Framework:** axum for the HTTP API
- **Frontend:** Vanilla HTML/CSS/JS with a dark-theme dashboard

### Architecture

```
GitHub Issue → Agent (Nemotron) → Plan → Code Generation →
Sandbox (test/clippy/check) → Verification Engine → GitHub PR
                                    ↑
                              Tavily Grounding
```

## 🤔 Challenges We Ran Into

- **Structured output from LLMs**: Getting Nemotron to consistently output valid JSON required careful prompt engineering and fallback parsing
- **Evidence linking**: Designing the claim-to-artifact mapping system required thinking carefully about what constitutes "proof" for each type of claim
- **Sandbox portability**: Making the sandbox interface work across both Nebius and Docker required a clean trait abstraction

## 🏆 Accomplishments We're Proud Of

- **Zero false claims**: In our test suite, every "tests pass" claim was backed by actual test output
- **Content-addressed artifacts**: Every piece of evidence has a SHA-256 hash, making the trail tamper-evident
- **Modular Rust architecture**: Clean separation of concerns makes each component independently testable
- **Tavily grounding**: The agent cites its sources, not just its training data

## 📚 What We Learned

- Nemotron's code generation quality is competitive with proprietary models when given structured prompts
- Nebius Token Factory's OpenAI-compatible API makes migration from other providers trivial
- Evidence trails dramatically increase trust in AI-generated code
- Tavily's search quality for technical documentation is excellent

## 🔮 What's Next

- **Repository cloning**: Clone and analyze the full repo for better context
- **Multi-file diffs**: Support complex changes spanning multiple files
- **CI integration**: Run as a GitHub Action on new issues
- **Confidence scoring**: ML-based prediction of whether the fix will work
- **Evidence chain visualization**: Interactive graph of claim→artifact relationships

## 🛠️ Built With

- [Rust](https://www.rust-lang.org/) — Systems programming language
- [NVIDIA Nemotron](https://build.nvidia.com/) — Open-source LLM for code generation
- [Nebius Token Factory](https://nebius.com/token-factory) — LLM inference + sandboxed execution
- [Tavily](https://tavily.com/) — AI-optimized search for documentation grounding
- [axum](https://github.com/tokio-rs/axum) — Rust web framework
- [octocrab](https://github.com/XAMPPRocky/octocrab) — GitHub API client for Rust
- [tokio](https://tokio.rs/) — Async runtime

---

## ⚡ Quick Start

### Prerequisites

- Rust 1.75+ (`rustup install stable`)
- Docker (for local sandbox fallback)
- API keys: Nebius, GitHub, Tavily (optional)

### Setup

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/governed-coding-agent.git
cd governed-coding-agent

# Copy and fill in environment variables
cp .env.example .env
# Edit .env with your API keys

# Build and run
cargo run -- serve
# Or resolve a single issue:
cargo run -- resolve --issue-url https://github.com/owner/repo/issues/42
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/resolve-issue` | Start resolving a GitHub issue |
| `GET` | `/api/status/{run_id}` | Check agent run status |
| `GET` | `/api/report/{run_id}` | Get verification report (JSON) |
| `GET` | `/api/report/{run_id}/markdown` | Get verification report (Markdown) |
| `GET` | `/api/health` | Health check |

### Environment Variables

See [`.env.example`](.env.example) for all required configuration.

---

## 📝 Feedback on Nebius & NVIDIA Tools

### Nebius Token Factory
The OpenAI-compatible API made integration seamless — we switched from direct NVIDIA NIM to Nebius Token Factory by changing one base URL. Inference latency on Nemotron was consistently fast (~2-4s for code generation). The Sandboxes API provided isolated execution environments perfect for running untrusted generated code.

**Suggestions:** More language-specific SDK examples (Rust), rate limit headers for backoff, streaming for sandbox execution progress.

### NVIDIA Nemotron
Nemotron's code generation quality is competitive with proprietary models. The model follows system prompts reliably, which was critical for our structured output (JSON plans, diff patches). The open-source license means we can self-host later.

**Suggestions:** A dedicated code-output mode, built-in diff generation format.

### Tavily
Tavily Search provided highly relevant results for programming documentation queries. The Extract API cleanly pulled API reference content from docs sites. Source URLs gave us verifiable citations for the evidence trail.

**Suggestions:** Deeper extraction for technical docs (docs.rs, MDN).

---

## 📄 License

This project is licensed under the Apache License 2.0 — see the [LICENSE](LICENSE) file for details.
