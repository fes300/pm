use clap::{Parser, Subcommand};
use inquire::{InquireError, Select};
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Boot,
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Boot => boot_project(),
    }
}

fn boot_project() {
    let options: Vec<&str> = vec!["notificationservice", "billingservice", "rustcli"];

    let p: Result<&str, InquireError> =
        Select::new("which project should we boot?", options).prompt();

    match p {
        Ok(proj) => {
            println!("booting {}", proj);

            let flake_location = format!("/Users/federicosordillo/flakes/{}", proj);
            println!("using flake at {}", flake_location);

            let mut cmd = Command::new("nix")
                .args(&["develop", &flake_location])
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

                {
                    let stdout = cmd.stdout.as_mut().unwrap();
                    let stdout_reader = BufReader::new(stdout);
                    let stdout_lines = stdout_reader.lines();
            
                    for line in stdout_lines {
                        println!("{}", line.unwrap());
                    }
                }
            
                cmd.wait().unwrap();
        
        }
        Err(_) => println!("bye! 👋"),
    }
}
