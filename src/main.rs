mod ast;
mod codegen;
mod lexer;
mod parser;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "j2bit",
    about = "Java subset compiler for BBC micro:bit",
    version,
    long_about = "Transpiles a subset of Java to C++ (CODAL) for BBC micro:bit v1/v2.\nOutput is a .cpp file ready to compile with arm-none-eabi-gcc."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile a .java file to .cpp
    Compile {
        /// Input Java source file
        input: PathBuf,

        /// Output C++ file (default: same name as input with .cpp extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show generated C++ on stdout instead of writing to file
        #[arg(long)]
        print: bool,

        /// Verbose output (show tokens, AST, etc.)
        #[arg(short, long)]
        verbose: bool,
    },
    /// Check syntax without generating output
    Check {
        /// Input Java source file
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Compile {
            input,
            output,
            print,
            verbose,
        } => {
            let source = std::fs::read_to_string(&input)?;
            let filename = input.display().to_string();

            eprintln!("{} {}", "  Compiling".green().bold(), filename);

            // Lex
            let tokens = lexer::tokenize(&source)?;
            if verbose {
                eprintln!("{} {} tokens", "    Lexed".cyan(), tokens.len());
                for tok in &tokens {
                    eprintln!("      {:?}", tok);
                }
            }

            // Parse
            let program = parser::parse(tokens)?;
            if verbose {
                eprintln!("{} {} declarations", "    Parsed".cyan(), program.classes.len());
            }

            // Codegen
            let cpp = codegen::generate(&program)?;

            if print {
                println!("{}", cpp);
            } else {
                let out_path = output.unwrap_or_else(|| input.with_extension("cpp"));
                std::fs::write(&out_path, &cpp)?;
                eprintln!("{} {}", "  Generated".green().bold(), out_path.display());
            }

            eprintln!("{}", "  Done!".green().bold());
            Ok(())
        }

        Command::Check { input } => {
            let source = std::fs::read_to_string(&input)?;
            let tokens = lexer::tokenize(&source)?;
            let _program = parser::parse(tokens)?;
            eprintln!("{} {}", "  OK".green().bold(), input.display());
            Ok(())
        }
    }
}
