use clap::{Parser, Subcommand};
use colorize::AnsiColor;
use home::home_dir;
use include_dir::include_dir;
use inquire::{InquireError, Select, Text};
use regex::Regex;
use std::fs::{self, metadata};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Boot,
    Setup,
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Boot => boot_project(),
        Commands::Setup => setup(),
    }
}

fn flakes_destination_path() -> String {
    prepend_home(".pm/flakes")
}

fn setup() {
    println!("\n");

    create_pmrc();

    create_flakes_dir();

    let success_log = format!("setup completed");

    println!("\n{}", success_log.bold().b_green());
}

fn create_pmrc() {
    let gen_log = format!("generating .pmrc...");
    println!("{}", gen_log.italic().black());

    let prompt_result: Result<String, String> = Text::new("where do you keep your projects?")
        .prompt()
        .map_err(|e| e.to_string())
        .and_then(|v| {
            let exploded_path = v.replace("~", home_dir().unwrap().to_str().unwrap());
            return metadata(exploded_path.clone())
                .map_err(|e| e.to_string())
                .and_then(|p| {
                    if p.is_dir() {
                        return Ok(exploded_path.clone());
                    }
                    return Err("path is not a directory".to_string());
                });
        });

    match prompt_result {
        Ok(r) => {
            match fs::write(prepend_home(".pmrc"), format!("project_dir={}", r)) {
                Ok(_) => {
                    // nothing to do here
                }
                Err(e) => {
                    println!("error writing .pmrc: {}", e)
                }
            };
        }
        Err(e) => {
            println!("error writing .pmrc: {}", e);
            println!("bye! 👋");
        }
    };
}

fn create_flakes_dir() {
    include_dir!("./src/flakes").dirs().for_each(|d| {
        let flake_dir = d.path().file_name().unwrap().to_str().unwrap();

        let gen_log = format!("generating {} flake...", flake_dir);
        println!("{}", gen_log.italic().black());

        let flake_content = d
            .get_file(format!("{}/flake.nix", flake_dir))
            .unwrap()
            .contents_utf8()
            .unwrap();

        let flake_destination_path = format!("{}/{}", flakes_destination_path(), flake_dir);

        fs::create_dir_all(flake_destination_path.clone()).unwrap();

        match fs::write(
            format!("{}/flake.nix", flake_destination_path.clone()),
            flake_content,
        ) {
            Ok(_) => {
                // println!("created {} flake", flake_dir)
            }
            Err(e) => {
                println!("error writing {} flake: {}", flake_dir, e)
            }
        };
    });
}

fn boot_project() {
    let known_projects = vec![
        "notificationservice",
        "billingservice",
        "frontend",
        "rustcli",
        "haskellcli",
        "adinsightsservice",
        "",
        "-- other projects --",
        "",
    ];
    let mut suggestions = known_projects.clone();

    let all_projects_path = get_projects_folder();
    println!("path {}", all_projects_path);
    let _all_projects = list_files(all_projects_path.as_str());

    let all_projects: Vec<&str> = _all_projects
        .iter()
        .filter(|v| !known_projects.contains(&v.as_str()))
        .map(|s| &**s)
        .collect();

    suggestions.extend(all_projects.clone());

    let prompt_result: Result<&str, InquireError> =
        Select::new("which project should we boot?", suggestions).prompt();

    match prompt_result {
        Ok(proj) => {
            if proj == "" || proj == "-- other projects --" {
                return;
            }

            let boot_log = format!("\nbooting {}...", proj);
            println!("{}", boot_log.italic().black());

            let flake_location = flake_location(proj, known_projects);

            set_flake_project(flake_location.clone(), proj);

            let flake_log = format!("using flake at {}\n", flake_location.clone());
            println!("{}", flake_log.italic().black());

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

            cleanup(proj, flake_location.clone());

            let flake_log = format!("booted {}", proj);
            println!("{}", flake_log.bold().b_green());
        }
        Err(_) => println!("bye! 👋"),
    }
}

fn list_files(path: &str) -> Vec<String> {
    let all_projects = fs::read_dir(path)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|dir| dir.file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    all_projects
}

const DEFAULT_FLAKE: &str = "go_1_22";

fn cleanup(proj: &str, flake_location: String) {
    let flake_path = format!("{}/flake.nix", flake_location);
    let flake_content = match fs::read_to_string(flake_path.clone()) {
        Ok(v) => v,
        Err(e) => {
            panic!("could not read flake at {}: {}", flake_path.clone(), e)
        }
    };

    match fs::write(
        flake_path.clone(),
        flake_content
            .replace(proj, "__PROJECTNAME__")
            .replace(&get_projects_folder(), "__PROJECTDIR__"),
    ) {
        Ok(_) => {
            // nothing to do here
        }
        Err(e) => {
            println!("error updating the standard flake: {}", e)
        }
    };
}

fn default_flake_location() -> String {
    format!("{}/{}", flakes_destination_path(), DEFAULT_FLAKE)
}

fn flake_location(proj: &str, known_projects: Vec<&str>) -> String {
    if known_projects.contains(&proj) {
        return format!("{}/{}", flakes_destination_path(), proj);
    }

    println!(
        "\n{}\n",
        "booting with a generic go 1_22 flake"
            .b_yellow()
            .bold()
            .underlined()
    );

    return default_flake_location();
}

fn set_flake_project(flake_location: String, proj: &str) {
    let flake_path = format!("{}/flake.nix", flake_location);

    let default_flake_content = match fs::read_to_string(flake_path.clone()) {
        Ok(v) => v,
        Err(e) => {
            panic!("could not read flake at {}: {}", flake_path.clone(), e)
        }
    };

    match fs::write(
        flake_path.clone(),
        default_flake_content
            .replace("__PROJECTNAME__", proj)
            .replace("__PROJECTDIR__", &get_projects_folder()),
    ) {
        Ok(_) => {
            // nothing to do here
        }
        Err(e) => {
            println!("error updating the standard flake: {}", e)
        }
    };
}

fn get_projects_folder() -> String {
    let re = Regex::new(r"project_dir=(.+)$").unwrap();
    return fs::read(prepend_home(".pmrc"))
        .map(|c| {
            let s = String::from_utf8(c).unwrap();
            return re.captures(&s).unwrap().get(1).unwrap().as_str().to_owned();
        })
        .unwrap();
}

fn prepend_home(s: &str) -> String {
    let r = format!("{}/{}", home_dir().unwrap().to_str().unwrap(), s);
    r
}
