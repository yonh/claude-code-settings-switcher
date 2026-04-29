mod config;
mod editor;
mod profile;

use colored::Colorize;

use clap::Parser;

#[derive(Parser)]
#[command(name = "ccss")]
#[command(about = "Claude Code Settings Switcher - manage and switch between configuration profiles")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// List all available profiles
    List,
    /// Show the currently active profile
    Current,
    /// Switch to a profile
    Use {
        /// Profile name to switch to
        name: String,
        /// Apply to global ~/.claude/settings.json instead of project .claude/settings.local.json
        #[arg(short, long)]
        global: bool,
    },
    /// Create a new profile from current settings
    Add {
        /// Name for the new profile
        name: String,
        /// Create an empty profile instead of copying current settings
        #[arg(short, long)]
        empty: bool,
    },
    /// Remove a profile
    Remove {
        /// Profile name to remove
        name: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Display profile settings
    Show {
        /// Profile name to show
        name: String,
    },
    /// Edit profile settings in your editor
    Edit {
        /// Profile name to edit
        name: String,
    },
    /// Compare current settings with a profile
    Diff {
        /// Profile name to compare against
        name: String,
    },
    /// View or set ccss configuration
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },
}

#[derive(clap::Subcommand)]
enum ConfigActions {
    /// Show current configuration
    Show,
    /// Set the preferred editor
    Editor {
        /// Editor command (e.g., "code", "vim", "nano"). Omit to show current setting.
        command: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let paths = config::Paths::new();

    let result = match cli.command {
        Commands::List => profile::list(&paths),
        Commands::Current => profile::current(&paths),
        Commands::Use { name, global } => profile::use_profile(&paths, &name, global),
        Commands::Add { name, empty } => profile::add(&paths, &name, empty),
        Commands::Remove { name, yes } => profile::remove(&paths, &name, yes),
        Commands::Show { name } => profile::show(&paths, &name),
        Commands::Edit { name } => profile::edit(&paths, &name),
        Commands::Diff { name } => profile::diff(&paths, &name),
        Commands::Config { action } => match action {
            ConfigActions::Show => editor::config_show(&paths),
            ConfigActions::Editor { command } => {
                editor::config_editor(&paths, command.as_deref())
            }
        },
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
