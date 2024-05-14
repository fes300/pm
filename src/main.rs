use clap::{Parser, Subcommand};
use inquire::{InquireError, Select};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::fs;
use colorized::*;


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
    let known_projects: Vec<&str> = vec!["notificationservice", "billingservice", "rustcli"];

    let all_projects = fs::read_dir("/Users/federicosordillo/git")
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|dir| dir.file_name().into_string().unwrap() )
        .collect::<Vec<_>>();
        

    let p: Result<&str, InquireError> =
        Select::new("which project should we boot?", all_projects.iter().map(|s| s as &str).collect()).prompt();

    match p {
        Ok(proj) => {
            println!("booting {}", proj);

            let flake_location = flake_location(proj, known_projects.clone());
            
            println!("using flake at {}\n", flake_location);

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

            cleanup(proj, flake_location)
        }
        Err(_) => println!("bye! 👋"),
    }
}

const DEFAULT_FLAKE_LOCATION: &str = "/Users/federicosordillo/flakes/go_1_22";

fn cleanup(proj: &str, flake_location: String) {
    if flake_location == DEFAULT_FLAKE_LOCATION {
        let default_flake_content = match fs::read_to_string(full_flake_path()) {
            Ok(v) => {v},
            Err(e) => {
                panic!("could not read flake at {}: {}", full_flake_path(), e)
            },
        };
    
        match fs::write(full_flake_path(), default_flake_content.replace(proj, "__PROJECTNAME__")) {
            Ok(_) => {
                // nothing to do here
            },
            Err(e) => {
                println!("error updating the standard flake: {}", e)
            },
        };
    }
}

fn flake_location(proj: &str, known_projects: Vec<&str>) -> String {
    if known_projects.contains(&proj) {
        return format!("/Users/federicosordillo/flakes/{}", proj);
    }

    let default_flake_content = match fs::read_to_string(full_flake_path()) {
        Ok(v) => {v},
        Err(e) => {
            panic!("could not read flake at {}: {}", full_flake_path(), e)
        },
    };

    match fs::write(full_flake_path(), default_flake_content.replace("__PROJECTNAME__", proj)) {
        Ok(_) => {
            // nothing to do here
        },
        Err(e) => {
            println!("error updating the standard flake: {}", e)
        },
    };

    println!("\n{}\n", "booting with a generic go 1_22 flake".color(Colors::YellowFg));


    return  DEFAULT_FLAKE_LOCATION.to_owned();
}

fn full_flake_path() -> String {
    let default_flake_path = format!("{}/flake.nix", DEFAULT_FLAKE_LOCATION);
    default_flake_path
}
