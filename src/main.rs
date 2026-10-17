#![allow(dead_code, unused_imports)]

use clap::{Parser, Subcommand};
use governed_coding_agent::agent;
use governed_coding_agent::api;
use governed_coding_agent::config::{AppConfig, SandboxBackend};
use governed_coding_agent::sandbox::{DockerSandbox, HostSandbox, NebiusSandbox, SandboxExecutor};
use governed_coding_agent::verification;
use std::sync::Arc;
use tracing::info;

/// Governed Coding Agent — AI code fixes with verifiable evidence trails.
#[derive(Parser)]
#[command(name = "gca", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP API server.
    Serve,
    /// Resolve a GitHub issue directly from the CLI.
    Resolve {
        /// GitHub issue URL (e.g., https://github.com/owner/repo/issues/42)
        #[arg(long)]
        issue_url: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Load .env file if present.
    dotenvy::dotenv().ok();

    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,governed_coding_agent=debug".into()),
        )
        .with_target(false)
        .init();

    info!("Governed Coding Agent v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration.
    let config = AppConfig::from_env()?;

    // Initialize the sandbox backend.
    let sandbox: Arc<dyn SandboxExecutor> = match config.sandbox_backend {
        SandboxBackend::Nebius => {
            info!("Using Nebius Token Factory Sandbox");
            Arc::new(NebiusSandbox::new(
                &config.nebius_sandbox_url,
                &config.nebius_api_key,
                &config.nebius_project_id,
            ))
        }
        SandboxBackend::Docker => {
            info!("Using Docker sandbox (with Host fallback)");
            Arc::new(DockerSandbox::default())
        }
        SandboxBackend::Host => {
            info!("Using Host Process Sandbox");
            Arc::new(HostSandbox::new())
        }
    };

    info!(backend = sandbox.name(), "Sandbox backend initialized");

    // Build the orchestrator.
    let orchestrator = Arc::new(
        agent::AgentOrchestrator::new(config.clone(), sandbox)?,
    );

    match cli.command {
        Some(Commands::Resolve { issue_url }) => {
            // CLI mode: resolve a single issue.
            info!(issue_url = %issue_url, "CLI mode: resolving issue");

            let run_id = orchestrator.start_run(issue_url);
            info!(run_id = %run_id, "Agent run started. Waiting for completion...");

            // Poll until done.
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                if let Some(run) = orchestrator.get_run(&run_id) {
                    match &run.status {
                        agent::AgentRunStatus::Completed => {
                            println!("\n✅ Issue resolved successfully!");
                            if let Some(pr_url) = &run.pr_url {
                                println!("📎 Pull Request: {}", pr_url);
                            }
                            if let Some(report) = &run.report {
                                let md = verification::ReportGenerator::to_markdown(report);
                                println!("\n{}", md);
                            }
                            break;
                        }
                        agent::AgentRunStatus::Failed { reason } => {
                            eprintln!("\n❌ Agent run failed: {}", reason);
                            std::process::exit(1);
                        }
                        status => {
                            print!("\r⏳ Status: {:?}      ", status);
                        }
                    }
                }
            }
        }
        Some(Commands::Serve) | None => {
            // Server mode: start the HTTP API.
            let addr = format!("{}:{}", config.host, config.port);
            info!(addr = %addr, "Starting HTTP server");

            let app = api::create_router(orchestrator);

            let listener = tokio::net::TcpListener::bind(&addr).await?;
            info!("🚀 Server listening on http://{}", addr);

            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
