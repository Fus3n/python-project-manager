// #![allow(dead_code)]
use colored::*;
use std::{path::Path, process};

pub fn eprint(msg: String) {
    println!("{}: {}", "Error".bright_red().bold(), msg.bright_red());
}

pub fn wprint(msg: String) {
    println!("{}: {}", "Warning".bright_yellow().bold(), msg.bright_yellow());
}

pub fn iprint(msg: String) {
    println!("{}", msg.bright_green().bold());
}


pub fn project_exists(name: &String) -> bool {
    if Path::new(name).exists() {
        if Path::new(&format!("{}/project.ini", name)).exists() {
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


fn load_ini() -> Result<ini::Ini, String> {
    match ini::Ini::load_from_file("project.ini") {
        Ok(conf) => Ok(conf),
        Err(e) => {
            return Err(format!("Could not load project.ini: {}", e));
        }
    }
}

pub fn show_project_info() {
    if !Path::new("project.ini").exists() {
        eprint("Could not find project.ini".to_owned());
        return;
    } 
    let conf = load_ini();
    if conf.is_err() {
        eprint(conf.err().unwrap());
        return;
    }
    let conf = conf.unwrap();

    let project = match conf.section(Some("Project")) {
        Some(section) => section,
        None => {
            eprint("Could not find project section in project.ini".to_owned());
            return;
        }
    };

    let name = project.get("name").expect("Could not find project name");
    let version = project.get("version").expect("Could not find project version");
    let description = project.get("description").expect("Could not find project description");

    println!("{}: {}", "Project".green().bold(),name.bright_cyan().bold());
    println!("{}: {}", "Version".green().bold(), version.bright_red().bold());
    println!("{}: {}", "Description".green().bold() ,description.bright_white().bold());
    
    let packages = conf.section(Some("Packages"));
    if packages.is_some() {
        let count = packages.unwrap().len();
        println!("-- {} {} --", count.to_string().green().bold(),  if count == 1 { "Package".to_owned() } else { "Packages".to_owned() });
        for (name, version) in packages.unwrap().iter().take(10) {
            println!("{}=={}", name.bright_yellow().bold(), version.bright_red().bold());
        }
        if packages.unwrap().len() > 10 {
            println!("... and {} more", packages.unwrap().len() - 10);
        }
    }
}

pub fn gen_requirements() {
    if !Path::new("project.ini").exists() {
        eprint("Could not find project.ini".to_owned());
        return;
    } 

    let conf = load_ini();
    if conf.is_err() {
        eprint(conf.err().unwrap());
        return;
    }
    let conf = conf.unwrap();

    let packages = conf.section(Some("Packages"));
    if packages.is_none() {
        eprint("Could not find packages section in project.ini".to_owned());
        return;
    }
    let packages = packages.unwrap();
    let mut reqs = String::new();
    for (name, version) in packages.iter() {
        reqs.push_str(&format!("{}=={}\n", name, version));
    }
    match std::fs::write("requirements.txt", reqs) {
        Ok(_) => iprint("Generated requirements.txt ".to_owned()),
        Err(e) => eprint(format!("Could not write requirements.txt: {}", e)),
    }
    
}

pub fn start_project() {
    if !Path::new("project.ini").exists() {
        eprint("Could not find project.ini".to_owned());
        return;
    } 

    let conf = load_ini();
    if conf.is_err() {
        eprint(conf.err().unwrap());
        return;
    }
    let conf = conf.unwrap();

    let project = match conf.section(Some("Project")) {
        Some(section) => section,
        None => {
            eprint("Could not find project section in project.ini".to_owned());
            return;
        }
    };

    match project.get("main") {
        Some(main) => {
            let venv = process::Command::new("./venv/Scripts/python.exe")
                .arg(main)
                .stdin(process::Stdio::piped())
                .spawn();
            
            match venv {
                Ok(mut o) => {
                    let _ = o.wait();
                    let _ = o.kill();
                }
                Err(e) => {
                    eprint("Failed to start main file".to_owned());
                    eprint(e.to_string());
                }
            }


        }
        None => {
            eprint("Could not find key 'main' in project.ini".to_owned());
        }
    }

}