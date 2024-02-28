// (Full example with detailed comments in examples/01b_quick_example.rs)
//
// This example demonstrates clap's full 'builder pattern' style of creating arguments which is
// more verbose, but allows easier editing, and at times more advanced options, or the possibility
// to generate arguments dynamically.
use clap::{Parser, Subcommand};

mod root;
mod constants;
mod build;
mod section;
use build::BuildSettings;
use root::InitSettings;
use section::AddSettings;

static mut VERBOSE: bool = false;
fn is_verbose() -> bool {
    unsafe {VERBOSE}
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes a wiki
    Init(InitSettings),
    /// Compile markdown to HTML
    Build(BuildSettings),
    /// Add a section
    Add(AddSettings),
    // Rm(RmSettings),
    // Mv(MvSettings),
    // Root(RootSettings),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.commands {
        Commands::Init(m) => m.run(),
        Commands::Build(m) => m.run(),
        Commands::Add(m) => m.run(),
        // Commands::Rm(m) => m.run(),
        // Commands::Mv(m) => m.run(),
    };

    match result {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    };
}
