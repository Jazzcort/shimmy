mod cli;
mod error;
mod middleman;
mod utils;

use clap::Parser;
use reqwest::Client;
use rmcp::{
    ClientHandler, ServiceExt,
    model::CallToolRequestParams,
    transport::{ConfigureCommandExt, TokioChildProcess},
};

use crate::middleman::spawn_middleman_with_stdio;
use crate::{
    cli::{Cli, Commands},
    middleman::spawn_middleman_with_http,
};

struct McpClient {
    name: String,
    version: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Stdio {
            envs,
            server_command,
            server_args,
        } => {
            if cli.verbose {
                eprintln!(
                    "Spawning stdio server: {} {}\nenvs: {:?}",
                    server_command,
                    server_args.join(" "),
                    envs
                );
            }

            spawn_middleman_with_stdio(server_command, server_args).await?;
        }
        Commands::Http { url } => {
            if cli.verbose {
                eprintln!("Starting HTTP server at {}", url);
            }

            spawn_middleman_with_http(url).await?;
        }
    }

    Ok(())
}
