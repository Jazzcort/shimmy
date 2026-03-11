use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "shimmy")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Custom parser to validate and split "KEY=VALUE" into a tuple
fn parse_env_var(s: &str) -> Result<(String, String), String> {
    let Some((key, value)) = s.split_once('=') else {
        return Err(format!(
            "Environment variables must be in 'KEY=VALUE' format. Got: '{}'",
            s
        ));
    };
    Ok((key.to_string(), value.to_string()))
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Spawn the MCP server via standard input/output
    Stdio {
        /// Environment variables to pass to the server (format: KEY=VALUE)
        #[arg(
            short = 'e',
            long = "env",
            value_name = "KEY=VALUE",
            value_parser = parse_env_var
        )]
        envs: Vec<(String, String)>,
        /// The executable command to run the MCP server (e.g., 'node', 'python')
        #[arg(required = true)]
        server_command: String,

        /// Arguments to pass to the MCP server command
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            help = "Arguments and flags to pass to the server command"
        )]
        server_args: Vec<String>,
    },

    /// Spawn the MCP server over HTTP/SSE
    Http {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}
