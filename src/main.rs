use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "dont")]
#[command(version)]
#[command(about = "Epistemic forcing-function CLI for grounded claims")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize dont state in the current project.
    Init,

    /// Introduce an unverified claim.
    Conclude {
        /// Claim statement text.
        statement: String,
    },

    /// Register explicit doubt about a claim.
    Trust {
        /// Claim identifier.
        id: String,

        /// Reason for doubt.
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Verify or add evidence to a claim.
    Dismiss {
        /// Claim identifier.
        id: String,

        /// Evidence URI or reference.
        #[arg(long, short)]
        evidence: Vec<String>,
    },

    /// Show a claim.
    Show {
        /// Claim identifier.
        id: String,
    },

    /// List claims.
    List,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => print_stub("init"),
        Command::Conclude { .. } => print_stub("conclude"),
        Command::Trust { .. } => print_stub("trust"),
        Command::Dismiss { .. } => print_stub("dismiss"),
        Command::Show { .. } => print_stub("show"),
        Command::List => print_stub("list"),
    }
}

fn print_stub(command: &str) {
    println!("{command}: not implemented");
}
