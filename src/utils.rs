use colored::*;
use std::{path::Path, process::{self, exit}};


pub fn eprint(msg: String) {
    println!("{} {}", "Error:".bright_red().bold(), msg.bright_red());
}

pub fn wprint(msg: String) {
    println!("{} {}", "Warning:".bright_yellow().bold(), msg.bright_yellow());
}

pub fn iprint(msg: String) {
    println!("{} {}", "•".bright_green().bold(), msg.bright_green().bold());
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

pub fn get_pkg_version(pkg: &String) -> String {
    let url = format!("https://pypi.org/pypi/{}/json", pkg);
    let resp = reqwest::blocking::get(&url);
    match resp {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().unwrap();
            let version = json["info"]["version"].as_str().unwrap();
            version.to_string()
        },
        Err(e) => {
            eprint("Failed to retrieve package version".to_string());
            eprint(e.to_string());
            exit(1);
        }
    }
    
}

pub fn setup_venv(venv_path: String) {
    iprint("Setting Up Virtual Environment...".to_string());
    let venv = process::Command::new("python")
        .arg("-m")
        .arg("venv")
        .arg(venv_path)
        .output();
    if venv.is_err() {
        eprint(venv.unwrap_err().to_string());
        exit(1);
    }
    let venv = venv.unwrap();
    if !venv.status.success() {
        eprint(format!("{}", String::from_utf8_lossy(&venv.stderr)));
        exit(1);
    }
}


pub fn load_ini() -> Result<ini::Ini, String> {
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
    let conf = match load_ini() {
        Ok(conf) => conf,
        Err(e) => {
            eprint(e.to_string());
            return;
        }
    };

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
    
    let scripts = conf.section(Some("Scripts"));
    if scripts.is_some() {
        println!("");
        let count = scripts.unwrap().len();
        println!("-- {} {} --", count.to_string().green().bold(),  if count == 1 { "Script".to_owned() } else { "Scripts".to_owned() });
        for (name, cmd) in scripts.unwrap().iter() {
            println!("{}: {}", name.bright_yellow().bold(), cmd.green().bold());
        }

    }

    let packages = conf.section(Some("Packages"));
    if packages.is_some() {
        println!("");
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

    let conf = match load_ini() {
        Ok(conf) => conf,
        Err(e) => {
            eprint(e.to_string());
            return;
        }
    };

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

    let conf = match load_ini() {
        Ok(conf) => conf,
        Err(e) => {
            eprint(e.to_string());
            return;
        }
    };

    let project = match conf.section(Some("Project")) {
        Some(section) => section,
        None => {
            eprint("Could not find Project section in project.ini".to_owned());
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

pub fn install_packages() {
    if !Path::new("project.ini").exists() {
        eprint("Could not find project.ini".to_owned());
        return;
    } 

    let conf = match load_ini() {
        Ok(conf) => conf,
        Err(e) => {
            eprint(e.to_string());
            return;
        }
    };

    let packages = match conf.section(Some("Packages")) {
        Some(section) => section,
        None => {
            eprint("Could not find Packages section in project.ini".to_owned());
            return;
        }
    };

    if packages.is_empty() {
        eprint("No packages to install".to_owned());
        return;
    }

    if !check_venv_dir_exists() {
        wprint("Could not find venv directory".to_owned());
        setup_venv("./venv".to_owned());
    }

    let mut cmd = process::Command::new("./venv/Scripts/pip.exe");
    cmd.arg("install");
    for (name, version) in packages.iter() {
        cmd.arg(format!("{}=={}", name, version));
    }
    
    
    let venv = cmd.spawn();

    match venv {
        Ok(mut o) => {
            let _ = o.wait();
        }
        Err(e) => {
            eprint("Failed to install packages".to_owned());
            eprint(e.to_string());
        }
    }
}


pub fn update_packages() {
    if !Path::new("project.ini").exists() {
        eprint("Could not find project.ini".to_owned());
        return;
    } 
    

    let mut conf = match ini::Ini::load_from_file("project.ini") {
        Ok(conf) => conf,
        Err(e) => {
            eprint(e.to_string());
            return;
        }
    };


    let packages = match conf.section(Some("Packages")) {
        Some(section) => section,
        None => {
            eprint("Could not find Packages section in project.ini".to_owned());
            return;
        }
    };


    if packages.is_empty() {
        eprint("No packages to install".to_owned());
        return;
    }

    if !check_venv_dir_exists() {
        wprint("Could not find venv directory".to_owned());
        setup_venv("./venv".to_owned());
    }


    let mut cmd_args: Vec<(String, String)> = vec![];
    for (name, _) in packages.iter() {
        let latest_ver = get_pkg_version(&name.to_owned());
        cmd_args.push((name.to_owned(), latest_ver));
    }


    /*
    looping to check if each package was successfully installed
    and add it to the ini file if it was
    */
    let mut conf_pkgs: Vec<(String, String)> = vec![];

    for (name, ver) in cmd_args {
        let mut cmd = process::Command::new("./venv/Scripts/pip.exe");
        cmd.arg("install");
        cmd.arg(format!("{}=={}", name, ver));
    
        let venv = cmd.spawn();
    
        match venv {
            Ok(mut o) => {
                let _ = o.wait();
                conf_pkgs.push((name.to_owned(), ver.to_owned()));
                iprint(format!("Updated {}", name));
            }
            Err(e) => {
                eprint(format!("Failed to update '{}'", name));
                eprint(e.to_string());
            }
        }
    }

    for (name, ver) in conf_pkgs {
        conf.set_to(Some("Packages"), name.to_string(), ver);
    }

    match conf.write_to_file("project.ini") {
        Ok(_) => {}
        Err(e) => {
            eprint(e.to_string());
        }
    }
    
}
