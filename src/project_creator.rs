use clap::{Subcommand, Args};
use ini::Ini;

use std::{fs, process::{exit, self}, io::Write, path::Path, time::Instant};

use crate::utils::*;



const STARTER_SOURCE_PY: &'static str = "\r
def main():
    print('Hello PPM!')

if __name__ == '__main__':
    main()
";



#[derive(Subcommand, Debug)]
pub enum Action {
    /// Create New Project With Given Name
    New(Project),

    /// Add new packages to project 
    Add(AddPackage),

    /// Remove packages from project
    Remove(RemovePackage),

    /// Run a script defined in project.ini
    Run(RunScript),

    /// Run main script defined in project.ini
    Start,

    /// Generate requirements.txt file
    Gen,

    /// Show the project.ini file
    Info,

    /// Install packages from project.ini and create venv if not found
    Install,

    /// Update all packages 
    Update,
}



#[derive(Args, Debug)]
pub struct Project {
    /// Set Project Name
    name: String,

    /// Set Project Version
    #[clap(short = 'v', long = "version", default_value = "0.1.0")]
    version: String,

    /// Set Project Description
    #[clap(short = 'd', long = "description", default_value = "\"\"")]
    description: String,

    /// Enable Git
    #[clap(short = 'g', long = "git", takes_value = false)]
    git: bool,

    /// Don't Create Virtual Environment
    #[clap(short = 'n', long = "no-venv", takes_value = false)]
    no_venv: bool,

}




impl Project {

    fn create_git(&self) {
        if self.git {
            let git_repo = process::Command::new("git")
                .arg("init")
                .arg(format!("{}/", self.name))
                .output();
            if git_repo.is_err() {
                eprint(git_repo.unwrap_err().to_string());
                exit(1);
            }
            // add build to gitignore
            let git_ignore = fs::File::create(format!("{}/.gitignore", self.name));
            if git_ignore.is_err() {
                eprint(git_ignore.unwrap_err().to_string());
                exit(1);
            }
            let mut git_ignore = git_ignore.unwrap();
            match git_ignore.write_all(b"/build\n") {
                Ok(_) => (),
                Err(e) => {
                    eprint(e.to_string());
                    git_ignore.flush().unwrap();
                    exit(1);
                }
            }
        }
        
    }
    
    fn create_boilerplate_files(&self) {
        let proj_dest = format!("{}/src", self.name);
        let main_file = fs::File::create(format!("{}/main.py", proj_dest));
        if main_file.is_err() {
            eprint(main_file.unwrap_err().to_string());
            exit(1);
        }
        // write started source
        let mut main_file = main_file.unwrap();
        match main_file.write_all(STARTER_SOURCE_PY.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprint(e.to_string());
                // close
                main_file.flush().unwrap();
                exit(1);
            }
        }
        // close files
        main_file.flush().unwrap();
    }

    fn save_config(&self) {
        let mut conf = Ini::new();
        conf.with_section(Some("Project"))
        .set("name", self.name.as_str())
        .set("version", self.version.as_str())
        .set("description", self.description.as_str())
        .set("main", "./src/main.py");
        conf.with_section(Some("Packages"));
        conf.with_section(Some("Scripts"));
        conf.write_to_file(format!("{}/project.ini", self.name)).unwrap();
    }

    


    pub fn create_project(&self) {
        let start = Instant::now();
        let proj_dest = format!("{}/src", self.name);
        if project_exists(&self.name) {
            eprint(format!("Project With Name '{}' Already Exists", self.name));
            exit(1);
        }
        let dir_create = fs::create_dir_all(&proj_dest);
        if dir_create.is_err() {
            eprint(dir_create.unwrap_err().to_string());
            exit(1);
        } 
        
        // create main.py file
        self.create_boilerplate_files();

        // setup git
        self.create_git();

        // venv
        if !self.no_venv {
            setup_venv(format!("{}/venv", self.name));
        } else {
            wprint("Virtual environment is disabled, some commands might not work".to_string());
        }

        // save
        self.save_config();

        let elapsed = start.elapsed();
        iprint(format!("Completed in {}s", elapsed.as_secs()));
        println!("\nTo get started:");
        println!("\tcd {}", self.name);
        println!("\tppm start");
    }

}


#[derive(Args, Debug)]
pub struct AddPackage {

    /// List of packages to add
    pub pkg_names: Vec<String>,

}

impl AddPackage {

    fn get_pkg_version(&self, pkg: &String) -> String {
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

    fn install_package(&self, pkg: String) -> bool{
        if !check_venv_dir_exists() {
            eprint("Virtual Environment Not Found".to_owned());
            return false;
        }
        iprint(format!("Installing '{}'", pkg));
        let venv = process::Command::new("./venv/Scripts/pip.exe")
            .arg("install")
            .arg(pkg)
            .spawn()
            .unwrap();

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
    
    pub fn add_package(&self) {
        if !Path::new(&"project.ini").exists() {  
            eprint("No project.ini found".to_owned());
            return;
        }
        let mut conf = match load_ini() {
            Ok(conf) => conf,
            Err(e) => {
                eprint(e.to_string());
                return;
            }
        };
        for pkg_name in self.pkg_names.iter() {
            if self.install_package(pkg_name.clone()) {
                conf.set_to(Some("Packages"), pkg_name.to_owned(), self.get_pkg_version(&pkg_name));
                match conf.write_to_file("project.ini") {
                    Ok(_) => {
                        iprint(format!("Package '{}' added successfully", pkg_name));
                    },
                    Err(e) => {
                        eprint(e.to_string());
                        exit(1);
                    }
                }
            } else {
                eprint(format!("Package '{}' failed to install", pkg_name));
            }
        }


        
    }
}


#[derive(Args, Debug)]
pub struct RemovePackage {

    /// List of packages to remove
    pub pkg_names: Vec<String>,

}


impl RemovePackage {
    
    fn uninstall_package(&self, pkg: String) -> bool {
        if !check_venv_dir_exists() {
            eprint("Virtual Environment Not Found".to_owned());
            return false;
        }
        iprint(format!("Uninstalling {}", pkg));
        // print any out put to stdout
        let venv = process::Command::new("./venv/Scripts/pip.exe")
            .arg("uninstall")
            .arg(pkg)
            .spawn()
            .unwrap();
        
        match venv.wait_with_output() {
            Ok(out) => {
                if !out.status.success() {
                    println!("{}", String::from_utf8_lossy(&out.stderr));
                    return false;
                } else {
                    println!("{}", String::from_utf8_lossy(&out.stdout));
                    return true;
                }
            },
            Err(e) => {
                eprint(e.to_string());
                return false;
            }
         }
    }

    pub fn remove_package(&self) {
        if !Path::new(&"project.ini").exists() {  
            eprint("No project.ini found".to_owned());
            return;
        }

        for pkg_name in self.pkg_names.iter() {
            let mut conf = match load_ini() {
                Ok(conf) => conf,
                Err(e) => {
                    eprint(e.to_string());
                    return;
                }
            };
    
            let packages = conf.section_mut(Some("Packages"));

            match packages {
                Some(p) => {
                    if !p.contains_key(pkg_name.as_str()) {
                        eprint(format!("Package '{}' does not exist", pkg_name));
                        return;
                    }
                    if self.uninstall_package(pkg_name.to_string()) { 
                        p.remove(pkg_name.as_str());    
                        match conf.write_to_file("project.ini") {
                            Ok(_) => {
                                iprint(format!("Package '{}' removed successfully", pkg_name));      
                            },
                            Err(e) => {
                                eprint(e.to_string());
                                exit(1);
                            }
                        }
                    } else {
                        eprint(format!("Package '{}' could not be removed", pkg_name));
                    }
                     
                }
                None => {
                    eprint(format!("Package '{}' does not exist", pkg_name));
                    exit(1);
                }
            }
        }
        
        
    }
}


#[derive(Args, Debug)]
pub struct RunScript {

    /// Script Name
    pub script_name: String,

}

impl RunScript {
    pub fn run_script(&self) {
        if !Path::new(&"project.ini").exists() {  
            eprint("No project.ini found".to_owned());
            return;
        }
        let mut conf = match load_ini() {
            Ok(conf) => conf,
            Err(e) => {
                eprint(e.to_string());
                return;
            }
        };

        let scripts = conf.section_mut(Some("Scripts"));
        match scripts {
            Some(s) => {
                if !s.contains_key(self.script_name.as_str()) {
                    eprint(format!("Script '{}' does not exist", self.script_name));
                    return;
                }
                let cmd_str = s.get(self.script_name.as_str()).unwrap();

                // temporary, later will add support for other os properly
                // currently missing alot of features
                let mut cmd;
                if cfg!(target_os = "windows") {
                    cmd = process::Command::new("cmd");
                    cmd.arg("/C");
                } else if cfg!(target_os = "linux") {
                    cmd = process::Command::new("bash");
                    cmd.arg("-c");
                } else {
                    eprint("Unsupported OS".to_owned());
                    return;
                }
                cmd.env("PATH", "./venv/Scripts");
                cmd.arg(cmd_str); 

                match cmd.spawn() {
                    Ok(mut o) => {
                        let _ = o.wait();
                    },
                    Err(e) => {
                        eprint(e.to_string());
                        exit(1);
                    }
                }
            }
            None => {
                eprint(format!("Script '{}' does not exist", self.script_name));
                exit(1);
            }
        }
    }
}