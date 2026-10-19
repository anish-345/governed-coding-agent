# 🎬 3-Minute Video Demo Script & Recording Guide

**Project:** Governed Coding Agent with Evidence Trail  
**Target Duration:** 2 minutes 50 seconds (under the 3-minute Devpost limit)  
**Tools Needed:** Screen recorder (OBS Studio, Loom, or Windows Game Bar `Win+G`), microphone.

---

## ⏱️ Video Timeline Breakdown

| Timestamp | Visual on Screen | Spoken Script / Narration | Key Takeaway for Judges |
|---|---|---|---|
| **0:00 - 0:25** | Slides / Title graphic + split-screen showing a broken AI PR merged into production. | *"Every developer has experienced this: an AI coding agent creates a pull request claiming 'all tests pass', but when merged, it crashes production. Current agents suffer from a dangerous trust problem: they make claims with zero verifiable proof. Today, we're introducing the **Governed Coding Agent** — an autonomous remediation engine that backs every claim with a cryptographic evidence trail, and uses an automated Self-Healing Loop powered by NVIDIA Nemotron on Nebius Token Factory."* | Problem hook & unique value proposition. |
| **0:25 - 0:45** | Browser showing the Governed Coding Agent Dashboard at `http://localhost:8080`. Point cursor to the top header badges and preset pills. | *"Here is our dashboard, built in high-performance Rust. Notice the live sponsor integrations right at the top: NVIDIA Nemotron-3 Super 120B hosted on Nebius Token Factory, with Tavily documentation grounding. To make judging effortless, we built three 1-click demo presets right below the issue input."* | Immediate visibility of Nebius, NVIDIA, and Tavily sponsors. |
| **0:45 - 1:30** | Click **"Demo 1: Self-Healing Loop"**. Watch the pipeline progress: Analyzing ➔ Generating ➔ Sandbox ➔ Self-Healing ➔ Verified. | *"Let's click Demo 1: an off-by-one bug. Watch the pipeline in real-time. In Attempt #1, our sandbox execution caught a panic on line 10. The Trust Score immediately dropped to 0%, marking the claim Refuted. But instead of failing, the agent triggered a **Self-Healing Loop**: it fed the exact sandbox stderr back into NVIDIA Nemotron. Within seconds, Nemotron generated a corrected patch that passed all unit tests with exit code 0! Judges can click between Attempt #1 and Attempt #2 right here on the timeline to inspect the exact failure logs, code diffs, and SHA-256 hashes."* | Proves Self-Healing loop and Refuted ➜ Verified progression. |
| **1:30 - 2:00** | Click **"Demo 2: Tavily Grounded"**. Point to the Sponsor Telemetry card and Tavily citation panel. | *"Next, let's run Demo 2: an asynchronous concurrency bug. Look at the Sponsor Telemetry card: we see live inference latency, prompt and completion tokens, and generation speed from Nebius Token Factory. Below, our Tavily Grounding panel shows the exact queries executed and clickable source documentation citations that grounded Nemotron's fix in official Rust Tokio best practices."* | Highlights sponsor telemetry and live web grounding. |
| **2:00 - 2:25** | Click **"Demo 3: Public GitHub Issue"**. Click **"Preview PR & Markdown Report"** button to open modal. | *"What if a judge tests a public repository where write permissions are absent? In Demo 3, we test a live Tokio issue. The agent detects read-only access and gracefully transitions into **Verified PR Preview Mode** — zero 403 crashes! We can open the modal to inspect the unified diff or copy the complete Markdown Verification Report and Cryptographic Artifact Registry with a single click."* | Demonstrates zero 403 crashes and complete evidence report. |
| **2:25 - 2:50** | Switch to terminal showing `cargo test` passing 12/12 tests and Dockerfile. Final slide with GitHub repo link. | *"Under the hood, the entire engine is written in safe, async Rust with 12 out of 12 tests passing cleanly and a multi-stage Docker container ready for cloud deployment. Governed Coding Agent eliminates AI hallucinations by replacing 'trust me' with cryptographically checkable proof. Thank you to Nebius, NVIDIA, and Tavily for empowering this project!"* | Engineering excellence, rock-solid tests, and strong closing. |

---

## 🎥 Recording Tips & Checklist

1. **Pre-flight Check:**
   - Run `cargo run -- serve` in the terminal to have the web server running on `http://localhost:8080`.
   - Ensure your browser zoom is at 100% or 110% for crisp readability.
   - Set browser to full screen or clean window (1920x1080).
2. **Audio:**
   - Speak clearly and with energy; avoid filler words.
3. **Mouse movements:**
   - Keep mouse movements smooth and deliberate when highlighting the telemetry badges, timeline toggle, and copy modal.
4. **Export:**
   - Export video in MP4 1080p.
   - Upload as "Unlisted" or "Public" to YouTube, Vimeo, or Loom and paste the link into the Devpost submission form.
