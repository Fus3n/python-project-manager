pub(crate) use std::path::Path;
use std::process::{Command, Stdio};
use crate::settings::Config;
use crate::utils::*;
use colored::*;


pub fn show_project_info() {
    if !Path::new("project.toml").exists() {
        eprint("Could not find project.toml".to_owned());
        return;
    } 
    let conf = match Config::load_from_file("project.toml") {
        Ok(conf) => conf,
        Err(e) => {
            eprint(e.to_string());
            return;
        }
    };
    println!("");

    // get pyhon versiopn from ./venv/Scripts/python
    match Command::new("./venv/Scripts/python.exe")
        .arg("--version")
        .stdout(Stdio::piped())
        .output()
    {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            let vers = version.split(" ").collect::<Vec<&str>>();
            println!("{}: {}", vers[0].bold().bright_purple(), vers[1].bold().red());
        },
        Err(_) => {
            eprint("failed to get python version".to_string());
        }
    };

    println!("{}: {}", "Project".green().bold(), conf.project.name.bright_cyan().bold());
    println!("{}: {}", "Version".green().bold(), conf.project.version.bright_red().bold());
    println!("{}: {}", "Description".green().bold(), conf.project.description.bright_white().bold());
    
    println!("");
    let count = conf.scripts.len();
    println!("-- {} {} --", count.to_string().green().bold(),  if count == 1 { "Script".to_owned() } else { "Scripts".to_owned() });
    for (name, cmd) in conf.scripts.iter() {
        println!("{}: {}", name.bright_yellow().bold(), cmd.green().bold());
    }

    println!("");
    let count = conf.packages.len();
    println!("-- {} {} --", count.to_string().green().bold(),  if count == 1 { "Package".to_owned() } else { "Packages".to_owned() });
    for (name, version) in conf.packages.iter().take(10) {
        println!("{}=={}", name.bright_yellow().bold(), version.bright_red().bold());
    }
    if conf.packages.len() > 10 {
        println!("... and {} more", conf.packages.len() - 10);
    }
    println!("");
    
}

pub fn gen_requirements() {
    if !Path::new("project.toml").exists() {
        eprint("Could not find project.toml".to_owned());
        return;
    }  

    let conf = match Config::load_from_file("project.toml") {
        Ok(conf) => conf,
        Err(e) => {
            eprint(e.to_string());
            return;
        }
    };

    let mut reqs = String::new();
    for (name, version) in conf.packages.iter() {
        reqs.push_str(&format!("{}=={}\n", name, version));
    }
    match std::fs::write("requirements.txt", reqs) {
        Ok(_) => iprint("Generated requirements.txt ".to_owned()),
        Err(e) => eprint(format!("Could not write requirements.txt: {}", e)),
    }
    
}

pub fn start_project() {
    if !Path::new("project.toml").exists() {
        eprint("Could not find project.toml".to_owned());
        return;
    } 

    let conf = match Config::load_from_file("project.toml") {
        Ok(conf) => conf,
        Err(e) => {
            eprint(e.to_string());
            return;
        }
    };

    let venv = Command::new("./venv/Scripts/python.exe")
                .arg(conf.project.main_script)
                .stdin(Stdio::piped())
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


pub fn update_packages() {
    if !Path::new("project.toml").exists() {
        eprint("Could not find project.toml".to_owned());
        return;
    } 
    
    let mut conf = match Config::load_from_file("project.toml") {
        Ok(conf) => conf,
        Err(e) => {
            eprint(e.to_string());
            return;
        }
    };


    if conf.packages.is_empty() {
        eprint("No packages to install".to_owned());
        return;
    }

    if !check_venv_dir_exists() {
        wprint("Could not find venv directory".to_owned());
        if ask_if_create_venv() {
            if setup_venv("./venv".to_owned()).is_err() {
                eprint("Failed to setup venv".to_owned());
                return;
            }
        } else {
            wprint("Update Cancelled".to_owned());
            return;
        }
    }

    let mut cmd_args: Vec<(String, String)> = vec![];
    for (name, _) in conf.packages.iter() {
        let latest_ver = get_pkg_version(&name.to_owned());
        if latest_ver.is_err() {
            eprint(format!("Could not find latest version of {}", name));
            continue;
        }
        let latest_ver = latest_ver.unwrap();
        cmd_args.push((name.to_owned(), latest_ver));
    }


    /*
        looping to check if each package was successfully installed
        and add it to the ini file if it was
    */
    let mut conf_pkgs: Vec<(String, String)> = vec![];

    for (name, ver) in cmd_args {
        let mut cmd = Command::new("./venv/Scripts/pip.exe");
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
        conf.packages.insert(name.to_string(), ver);
    }

    match conf.write_to_file("project.toml") {
        Ok(_) => {}
        Err(e) => {
            eprint(e.to_string());
        }
    }
    
}
