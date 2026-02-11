//! CLI interface for browsing

use anyhow::Result;
use browsing::{Browser, Config};
use browsing::browser::profile::BrowserProfile;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(name = "browsing")]
#[command(about = "Autonomous web browsing for AI agents", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Run an autonomous browsing task")]
    Run {
        #[arg(help = "Task description for the agent")]
        task: String,

        #[arg(short, long, help = "Starting URL")]
        url: Option<String>,

        #[arg(long, help = "Maximum number of steps", default_value = "100")]
        max_steps: u32,

        #[arg(long, help = "Run browser in headless mode")]
        headless: bool,

        #[arg(long, help = "Enable vision capabilities")]
        vision: bool,
    },

    #[command(about = "Launch a browser and connect to it")]
    Launch {
        #[arg(long, help = "Run browser in headless mode")]
        headless: bool,

        #[arg(long, help = "User data directory")]
        user_data_dir: Option<PathBuf>,
    },

    #[command(about = "Connect to an existing browser via CDP URL")]
    Connect {
        #[arg(help = "CDP WebSocket URL (e.g., ws://localhost:9222/devtools/browser/...)")]
        cdp_url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    browsing::init();

    if cli.verbose {
        unsafe {
            std::env::set_var("RUST_LOG", "browsing=debug,info");
        }
    }

    let _config = if let Some(config_path) = cli.config {
        Config::load_from_file(config_path)?
    } else {
        Config::from_env()
    };

    match cli.command {
        Commands::Run {
            task,
            url,
            max_steps: _,
            headless,
            vision: _,
        } => {
            info!("Starting autonomous browsing task: {}", task);

            println!("\n=== Autonomous Browsing ===");
            println!("Task: {}", task);
            println!("\nNote: Full agent implementation requires an LLM provider.");
            println!("Please implement the ChatModel trait for your LLM.");
            println!("See docs/LIBRARY_USAGE.md for details.");
            
            // For now, just demonstrate browser capabilities
            let mut profile = BrowserProfile::default();
            profile.headless = Some(headless);
            
            let mut browser = Browser::new(profile);
            browser.start().await?;
            info!("Browser launched successfully");

            if let Some(start_url) = url {
                browser.navigate(&start_url).await?;
                info!("Navigated to: {}", start_url);
                
                let current_url = browser.get_current_url().await?;
                let title = browser.get_current_page_title().await?;
                println!("\n✓ Browser ready");
                println!("  URL: {}", current_url);
                println!("  Title: {}", title);
            }

            println!("\nTo use the full agent, implement ChatModel trait for your LLM provider.");
            let _ = browser.stop().await;
        }

        Commands::Launch {
            headless,
            user_data_dir,
        } => {
            let mut profile = BrowserProfile::default();
            profile.headless = Some(headless);
            if let Some(dir) = user_data_dir {
                profile.user_data_dir = Some(dir);
            }

            let mut browser = Browser::new(profile);
            browser.start().await?;

            println!("Browser launched successfully!");
            println!("\nPress Ctrl+C to close the browser...");

            tokio::signal::ctrl_c().await?;
            println!("\nClosing browser...");
            let _ = browser.stop().await;
        }

        Commands::Connect { cdp_url } => {
            info!("Connecting to browser at: {}", cdp_url);
            let profile = BrowserProfile::default();
            let mut browser = Browser::new(profile).with_cdp_url(cdp_url.clone());
            browser.start().await?;

            println!("Connected to browser successfully!");
            println!("CDP URL: {}", cdp_url);
            println!("\nPress Ctrl+C to disconnect...");

            tokio::signal::ctrl_c().await?;
            println!("\nDisconnecting...");
            let _ = browser.stop().await;
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn create_llm_from_config(_config: &browsing::config::LlmConfig) -> Result<Box<dyn browsing::ChatModel>> {
    Err(anyhow::anyhow!(
        "LLM implementation required. Please implement ChatModel trait for your LLM provider.\n\
         Example: Use watsonx-rs crate with ibm/granite-4-h-small model.\n\
         Set LLM_API_KEY and LLM_MODEL environment variables."
    ))
}
