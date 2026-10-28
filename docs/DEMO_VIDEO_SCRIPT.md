# 🎬 3-Minute Video Demo Script & Recording Guide

**Project:** Governed Coding Agent with Evidence Trail  
**Target Duration:** 2 minutes 45 seconds (comfortably under the 3-minute Devpost limit)  
**Tools Needed:** Screen recorder (Loom, OBS Studio, or Windows Game Bar `Win+G`), microphone.

---

## ⏱️ Video Timeline Breakdown (Spoken Word Script)

| Timestamp | What You Show on Screen | What You Say (Narration) | Why Judges Love It |
|---|---|---|---|
| **0:00 - 0:25** | Slides / Title graphic or split-screen showing a broken AI PR merged into production. | *"Every developer knows this feeling: you ask an AI coding assistant to fix a bug, it gives you a pull request saying 'Fixed! All tests pass', you merge it... and your app crashes in production. Why? Because existing AI agents guess instead of proving. Today, we're introducing **Governed Coding Agent** — an AI developer that actually proves its code works. It runs your tests in an isolated sandbox, automatically heals its own mistakes, and gives you real proof you can trust."* | Hook: An instantly relatable developer pain point. |
| **0:25 - 0:45** | Browser showing the dashboard at `http://localhost:8080`. Point cursor to the top header badges and preset pills. | *"Here is our dashboard, built in high-performance Rust. At the top, you can see our live sponsor integrations: **NVIDIA Nemotron-3 Super 120B** hosted on **Nebius Token Factory**, with **Tavily** documentation search. To make judging effortless, we built three 1-click presets right here so you can test the full pipeline in seconds."* | Immediate visibility of sponsors and effortless testing. |
| **0:45 - 1:30** | Click **"Demo 1: Self-Healing Loop"**. Watch the pipeline progress: Analyzing ➔ Generating ➔ Sandbox ➔ Self-Healing ➔ Verified. | *"Let's click Demo 1: an off-by-one bug. Watch the pipeline in real-time. In Attempt #1, our sandbox execution caught a panic on line 10. The Trust Score immediately dropped to 0%, marking the claim Refuted. But instead of giving up or asking you to debug it, the agent triggered an autonomous **Self-Healing Loop**: it fed the exact panic stderr back into NVIDIA Nemotron. Within seconds, Nemotron generated a corrected patch that passed all unit tests with exit code 0! You can click right here between Attempt #1 and Attempt #2 on the timeline to inspect the failure logs, the healed code diff, and the SHA-256 hashes."* | Demonstrates the core innovation: real Self-Healing and Refuted ➜ Verified progression. |
| **1:30 - 2:00** | Click **"Demo 2: Tavily Grounded"**. Point to the Sponsor Telemetry card and Tavily citation panel. | *"Next, let's run Demo 2: an async concurrency bug. Look at the Sponsor Telemetry card: we see live inference latency, prompt and completion tokens, and generation speed from Nebius Token Factory. Below, our Tavily Grounding panel shows the exact queries executed and clickable documentation citations that grounded Nemotron's fix in official Rust Tokio best practices."* | Highlights sponsor telemetry and live web grounding. |
| **2:00 - 2:25** | Click **"Demo 3: Public GitHub Issue"**. Click **"Preview PR & Markdown Report"** button to open modal, then point to **Evidence Banner**. | *"What if you test a public repository where you don't have write access? In Demo 3, we test a live Tokio issue. The agent detects read-only access and smoothly transitions into **Verified PR Preview Mode** — zero 403 crashes! We see our **Verifiable Evidence Trail** with a canonical Merkle Root proving every test log and diff, and you can download the full proof JSON with a single click."* | Demonstrates zero 403 crashes, clean UX, and verifiable evidence. |
| **2:25 - 2:45** | Switch to terminal showing `cargo test` passing 13/13 tests and Dockerfile. Final slide with GitHub repo link. | *"Under the hood, the entire engine is written in safe, async Rust with 13 out of 13 tests passing cleanly and a Docker container ready for cloud deployment. Governed Coding Agent eliminates AI hallucinations by replacing 'trust me' with real, checkable proof. Thank you to Nebius, NVIDIA, and Tavily for empowering this project!"* | Engineering excellence, rock-solid tests, and strong closing. |

---

## 🎥 Recording Tips & Checklist

1. **Pre-flight Check:**
   - Run `cargo run -- serve` in the terminal so the server is up on `http://localhost:8080`.
   - Browser zoom at 100% or 110% for crisp readability.
   - Clean window / 1920x1080 resolution.
2. **Narration:**
   - Speak naturally and with energy, like showing a cool project to a fellow developer.
3. **Export:**
   - Upload as "Unlisted" or "Public" to YouTube, Vimeo, or Loom and paste the link into the Devpost submission form.
