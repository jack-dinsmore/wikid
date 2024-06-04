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
    /// Open the blog locally with Chrome
    Open,
    /// Add a section
    Add(AddSettings),
    // Rm(RmSettings),
    // Mv(MvSettings),
    // Root(RootSettings),
    Syntax,
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
        Commands::Open => {
            std::process::Command::new("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
                .arg("--allow-file-access-from-files")
                .arg(&root::Root::get_path_from_local("html/index.html").unwrap())
                .spawn().unwrap();
            Ok(())
        }
        Commands::Build(m) => m.run(),
        Commands::Add(m) => m.run(),
        Commands::Syntax => {
            display_syntax();
            Ok(())
        },
        // Commands::Rm(m) => m.run(),
        // Commands::Mv(m) => m.run(),
    };

    match result {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    };
}

fn display_syntax() {
    const HELP_STR: &'static str = "\x1b[1;32mWikid syntax\x1b[0m
\x1b[1;36mText\x1b[0m
-  _italics_
-  *bold*

\x1b[1;36mLinks\x1b[0m
-  [link text](hyperlink)
-  ![Caption]{path_to_figure}
-  ?[Caption]{path_to_applet}
-  [link text]{reference}, where reference is an equation, figure, table, note, section, or subsection
- [footnote]
- For all links, {} represent a local path and [] represent a global path

\x1b[1;36mLaTeX\x1b[0m
-  ~label, before the equation
-  Reference equations with []{equation label}
-  Use double dollar signs in a new line to start a newline equation
-  Amsmath is loaded, with \\bm, \\parens, \\brackets, \\braces, \\eval, \\fraci, and \\expp
";

    // Create a command to execute `less`
    let mut less = std::process::Command::new("less")
        .arg("-R")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn less command");

    // Write the string to the stdin of the `less` command
    if let Some(stdin) = &mut less.stdin {
        std::io::Write::write_all(stdin, HELP_STR.as_bytes()).expect("Failed to write to less stdin");
    }

    // Wait for the `less` command to finish
    let status = less.wait().expect("Failed to wait on less command");

    if !status.success() {
        eprintln!("less command failed with status: {}", status);
    }
}