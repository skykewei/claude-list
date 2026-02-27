use clap::{Parser, Subcommand};
use claude_list::output::{DetailFormatter, Formatter, JsonFormatter, TableFormatter};
use claude_list::service::ListService;

#[derive(Parser)]
#[clap(name = "claude-list")]
#[clap(about = "List Claude Code skills and MCP servers")]
#[clap(version)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    /// Output in JSON format
    #[clap(short, long, global = true)]
    json: bool,

    /// Show verbose output
    #[clap(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List all skills and MCP servers (default)
    List,
    /// List only skills
    Skills,
    /// List only MCP servers
    Mcps,
    /// Show details of a skill or MCP server
    Show {
        /// Name of the skill or MCP server to show
        name: String,
        /// Show raw file content (for skills)
        #[clap(long)]
        raw: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let service = ListService::new();

    // Handle Show command separately
    if let Some(Commands::Show { name, raw }) = cli.command {
        let detail = service.show(&name).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

        let output: Box<dyn DetailFormatter> = if cli.json {
            Box::new(JsonFormatter::new())
        } else {
            Box::new(TableFormatter::new())
        };

        let formatted = output.format_detail(&detail, raw).unwrap_or_else(|e| {
            eprintln!("Error formatting output: {}", e);
            std::process::exit(1);
        });
        println!("{}", formatted);
        return;
    }

    // Determine what to list based on subcommand
    let data = match cli.command {
        Some(Commands::Skills) => {
            let skills = service.list_skills().unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            claude_list::model::ClaudeList {
                skills,
                mcps: Vec::new(),
            }
        }
        Some(Commands::Mcps) => {
            let mcps = service.list_mcps().unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            claude_list::model::ClaudeList {
                skills: Vec::new(),
                mcps,
            }
        }
        _ => {
            // Default: list all
            service.list_all().unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            })
        }
    };

    // Output formatting
    let output: Box<dyn Formatter> = if cli.json {
        Box::new(JsonFormatter::new())
    } else {
        Box::new(TableFormatter::new().with_verbose(cli.verbose))
    };

    let formatted = output.format(&data).unwrap_or_else(|e| {
        eprintln!("Error formatting output: {}", e);
        std::process::exit(1);
    });
    println!("{}", formatted);
}
