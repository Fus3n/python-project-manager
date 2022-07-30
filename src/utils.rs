use colored::*;
use std::{path::Path, process::{Command}, io::{self, Write}};


pub fn eprint(msg: String) {
    println!("{} {}", "error:".bright_red().bold(), msg.bright_red());
}

pub fn wprint(msg: String) {
    println!("{} {}", "warning:".bright_yellow().bold(), msg.bright_yellow());
}

pub fn iprint(msg: String) {
    println!("{} {}", "•".bright_green().bold(), msg.bright_green().bold());
}

pub fn project_exists(name: &String, is_init: bool) -> bool {
    if is_init {
        if Path::new("project.toml").exists() {
            return true;
        }
    }
    else if Path::new(name).exists() {
        if Path::new(&format!("{}/project.toml", name)).exists() {
            return true;
        }
    } 
    return false;
}
pub fn check_venv_dir_exists() -> bool  {
    if Path::new(&"./venv/Scripts/").exists() {
        return true;
    }
    return false;
}

pub fn get_pkg_version(pkg: &String) -> Result<String, ()> {
    let url = format!("https://pypi.org/pypi/{}/json", pkg);
    let resp = reqwest::blocking::get(&url);
    match resp {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().unwrap();
            let version = json["info"]["version"].as_str().unwrap();
            Ok(version.to_string())
        },
        Err(e) => {
            eprint("Failed to retrieve package version".to_string());
            eprint(e.to_string());
            Err(())
        }
    }
    
}

pub fn setup_venv(venv_path: String) -> Result<(), ()> {
    iprint("Setting Up Virtual Environment...".to_string());
    let venv = Command::new("python")
        .arg("-m")
        .arg("venv")
        .arg(venv_path)
        .output();
    if venv.is_err() {
        eprint(venv.unwrap_err().to_string());
        return Err(());
    }
    let venv = venv.unwrap();
    if !venv.status.success() {
        eprint(format!("{}", String::from_utf8_lossy(&venv.stderr)));
        return Err(());
    }
    Ok(())
}

pub fn ask_if_create_venv() -> bool {
    let mut answer = String::new();
    print!("{}", "[?] Do you want to create a virtual environment? (y/n): ".green().bold());
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut answer).unwrap();
    if answer.trim().to_lowercase() == "y" {
        return true;
    } else if answer.trim().to_lowercase() == "n" {
        return false;
    } else {
        println!("Invalid option");
        return ask_if_create_venv();
    }
}

/// parse name and version of package if it was name==version
pub fn parse_version(pkg: String) -> (String, String) {
    if pkg.contains("==") {
        let mut pkg_split = pkg.split("==");
        let pkg_name = pkg_split.next().unwrap();
        let pkg_version = pkg_split.next().unwrap();
        return (pkg_name.to_string(), pkg_version.to_string());
    } else {
        return (pkg.to_string(), "".to_string());
    }
}

/// install the specifed package
pub fn install_package(pkg: String) -> bool {
    if !check_venv_dir_exists() {
        eprint("Virtual Environment Not Found".to_owned());
        return false;
    }
    iprint(format!("Installing '{}'", pkg));
    let venv = Command::new("./venv/Scripts/pip.exe")
        .arg("install")
        .arg(pkg)
        .spawn();
    if venv.is_err() {
        eprint(venv.unwrap_err().to_string());
        return false;
    }
    let venv = venv.unwrap();           

    match venv.wait_with_output() {
        Ok(output) => {
            if !output.status.success() {
                println!("{}", String::from_utf8_lossy(&output.stderr));
                return false;
            } else {
                println!("{}", String::from_utf8_lossy(&output.stdout));
                return true;
            }
        },
        Err(e) => {
            eprint(e.to_string());
            return false;
        }
    }

}


