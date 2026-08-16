use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "rigglow",
    version,
    about = "A colorful live Linux hardware fetcher"
)]
pub struct Cli {
    /// Print a polished system snapshot and exit
    #[arg(long)]
    pub r#static: bool,
    /// Print a small snapshot for SSH and exit
    #[arg(long)]
    pub compact: bool,
    /// Print the collected snapshot as JSON and exit
    #[arg(long)]
    pub json: bool,
    /// Select a built-in theme
    #[arg(long)]
    pub theme: Option<String>,
    /// Select built-in art or a path to custom ASCII art
    #[arg(long)]
    pub ascii: Option<String>,
    /// Hide module icons
    #[arg(long)]
    pub no_icons: bool,
    /// Disable animated color effects
    #[arg(long)]
    pub no_animation: bool,
    /// Dashboard refresh interval in milliseconds
    #[arg(long, value_name = "MS")]
    pub refresh_rate: Option<u64>,
}
