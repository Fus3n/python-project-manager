use clap::{Subcommand, Args};
use colored::Colorize;

use std::{fs, process::{Command}, io::{Write}, path::Path, time::Instant, collections::HashMap};
use crate::utils::*;
use crate::settings::*;


const STARTER_SOURCE_PY: &'static str = "\r
def main():
    print('Hello From PPM!')

if __name__ == '__main__':
    main()
";


#[derive(Subcommand, Debug)]
pub enum Action {
    /// Create New Project With Given Name
    New(ProjectConf),

    /// Initialize Project In Current Directory
    Init(ProjectConf),

    /// Add new packages to project 
    Add(AddPackage),

    /// Remove packages from project
    Rm(RemovePackage),

    /// Run a script defined in project.toml
    Run(RunScript),

    /// Install packages from project.toml or provided requirements.txt
    Install(Installer),

    /// Run main script defined in project.toml
    Start,

    /// Generate requirements.txt file
    Gen,

    /// Show the project.toml file
    Info,

    /// Update all packages 
    Update,
}

pub struct ProjectCreator {
    project: ProjectConf,
    is_init: bool,
}

impl ProjectCreator {
    fn new(project: ProjectConf, is_init: bool) -> ProjectCreator {
        ProjectCreator {
            project: project,
            is_init: is_init,
        }
    }

    fn get_path_with(&self, path: &str) -> String{
        if self.is_init {
            return path.to_string();
        } else{
            return format!("{}/{}", self.project.name, path);
        }
    }

    fn create_git(&self) -> Result<(), ()> {
        if self.project.git {
            let path = if self.is_init { ".".to_string() } else { format!("{}/", self.project.name) };
            let git_repo = Command::new("git")
                .arg("init")
                .arg(path)
                .output();
            if git_repo.is_err() {
                eprint(git_repo.unwrap_err().to_string());
                return Err(());
            }
            // add build to gitignore
            let git_ignore = fs::File::create(self.get_path_with(".gitignore"));
            if git_ignore.is_err() {
                eprint(git_ignore.unwrap_err().to_string());
                return Err(());
            }
            let mut git_ignore = git_ignore.unwrap();
            match git_ignore.write_all(b"/build\n") {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprint(e.to_string());
                    git_ignore.flush().unwrap();
                    return Err(());
                }
            }
        } else {
            Ok(())
        }
        
    }
    
    fn create_boilerplate_files(&self) -> Result<(), ()> {
        let proj_dest = self.get_path_with("src");
        let main_file = fs::File::create(format!("{}/main.py", proj_dest));
        if main_file.is_err() {
            eprint(main_file.unwrap_err().to_string());
            return Err(());
        }
        // write started source
        let mut main_file = main_file.unwrap();
        match main_file.write_all(STARTER_SOURCE_PY.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprint(e.to_string());
                // close
                main_file.flush().unwrap();
                return Err(());
            }
        }
        // close files
        main_file.flush().unwrap();
        Ok(())
    }

    fn save_config(&self) -> Result<(), ()> {
        let mut conf = Config::new(
            Project::new(
                self.project.name.clone(),
                self.project.version.clone(),
                self.project.description.clone(),
                if self.is_init { "./main.py".to_string() } else { "./src/main.py".to_string() }  
            ),
            HashMap::new(),
            HashMap::new(),
        );
        conf.scripts.insert("upgrade-pip".to_string(), "python -m pip install --upgrade pip".to_string());

        match conf.write_to_file(self.get_path_with("project.toml").as_str()){
            Ok(_) => Ok(()),
            Err(e) => {
                eprint(e.to_string());
                return Err(());
            }
        }
    }

    pub fn create_project(&self) {
        let start = Instant::now();
        let proj_dest = self.get_path_with("src");
        if project_exists(&self.project.name,  self.is_init) {
            eprint(format!("Project With Name '{}' Already Exists", &self.project.name));
            return;
        }
        let dir_create = fs::create_dir_all(&proj_dest);
        if dir_create.is_err() {
            eprint(dir_create.unwrap_err().to_string());
            return;
        } 
        
        // create main.py file
        if self.create_boilerplate_files().is_err() {
            return;
        }

        // setup git
        if self.create_git().is_err() {
            return;
        }

        // venv
        if !self.project.no_venv {
            if setup_venv(self.get_path_with("venv")).is_err() {
                eprint("Failed to setup venv".to_owned());
                return;
            }
        } else {
            wprint("Virtual environment is disabled, some commands might not work".to_string());
        }

        // save
        if self.save_config().is_err() {
           return;
        }

        let elapsed = start.elapsed();
        iprint(format!("{} in {}s", "Completed".green(), elapsed.as_secs()));
        println!("\nTo get started:");
        if !self.is_init {
            println!("  cd {}", self.project.name.blue());
        }
        println!("  {} start\n", "ppm".red());
    }

}

#[derive(Args, Debug, Clone)]
pub struct ProjectConf {
    /// Set Project Name
    name: String,

    /// Set Project Version
    #[clap(short = 'v', long = "version", default_value = "0.1.0")]
    version: String,

    /// Set Project Description
    #[clap(short = 'd', long = "description", default_value = "")]
    description: String,

    /// Enable Git
    #[clap(short = 'g', long = "git", takes_value = false)]
    git: bool,

    /// Don't Create Virtual Environment
    #[clap(short = 'e', long = "no-venv", takes_value = false)]
    no_venv: bool,

}


impl ProjectConf {

    pub fn create_project(&self, is_init: bool) {
        let proj_creator = ProjectCreator::new(self.clone(), is_init);
        proj_creator.create_project();
    }

}


#[derive(Args, Debug)]
pub struct AddPackage {

    /// List of packages to add
    pub pkg_names: Vec<String>,

}

impl AddPackage {

    pub fn add_package(&self) {
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

        for pkg_name in self.pkg_names.iter() {
            let (vname, mut ver) = parse_version(pkg_name.clone());

            if install_package(pkg_name.clone()) {
                if ver.len() == 0 {
                    // if no version, set to latest
                    let v = get_pkg_version(&pkg_name);
                    if v.is_err() {
                        continue;
                    }
                    ver = v.unwrap();
                } 
                conf.packages.insert(vname.clone(), ver.clone());
                match conf.write_to_file("project.toml") {
                    Ok(_) => {
                        iprint(format!("Package '{}' added successfully", &vname));
                    },
                    Err(e) => {
                        eprint(e.to_string());
                        continue;
                    }
                }
            } else {
                eprint(format!("Package '{}' failed to install", &vname));
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
        let venv = Command::new("./venv/Scripts/pip.exe")
            .arg("uninstall")
            .arg("-y")
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
        for pkg_name in self.pkg_names.iter() {
            if !conf.packages.contains_key(pkg_name) {
                eprint(format!("Package '{}' does not exist", pkg_name));
                continue;
            }
            if self.uninstall_package(pkg_name.to_string()) {
                conf.packages.remove(pkg_name);
                match conf.write_to_file("project.toml") {
                    Ok(_) => {
                        iprint(format!("Package '{}' removed successfully", pkg_name));      
                    },
                    Err(e) => {
                        eprint(e.to_string());
                        continue;
                    }
                }
            } else {
                eprint(format!("Failed to remove '{}'", pkg_name));
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

        if !conf.scripts.contains_key(self.script_name.as_str()) {
            eprint(format!("Script with name '{}' does not exist", self.script_name));
            return;
        }
        let cmd_str = conf.scripts.get(self.script_name.as_str()).unwrap();

        // temporary, later will add support for other os properly
        // currently missing alot of features
        let mut cmd;
        if cfg!(target_os = "windows") {
            cmd = Command::new("cmd");
            cmd.arg("/C");
        } else if cfg!(target_os = "linux") {
            cmd = Command::new("bash");
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
                return;
            }
        }
    }
}


#[derive(Args, Debug)]
pub struct Installer {
    
    /// Install from requirements
    #[clap(short = 'r', long = "requirements", default_value = "")]
    pub requirements: String,

}

impl Installer {

    fn install_from_req(&self) {
        if !check_venv_dir_exists() {
            wprint("Could not find venv directory".to_owned());
            if ask_if_create_venv() {
                if setup_venv("./venv".to_owned()).is_err() {
                    eprint("Failed to setup venv".to_owned());
                    return;
                }
            } else {
                wprint("Installation Cancelled".to_owned());
                return;
            }
        }

        let req_file = match fs::read_to_string(self.requirements.as_str()) {
            Ok(f) => f,
            Err(_) => {
                eprint(format!("Failed to read {}, make sure to specify correct path", self.requirements));
                return;
            }
        };

        if !Path::new("project.toml").exists() {
            eprint("Could not find project.toml".to_owned());
            return;
        } 

        let mut pkg_names = Vec::new();
        for line in req_file.lines() {
            if line.contains("#") {
                continue;
            }
            pkg_names.push(line.to_string());
        }
 
        let mut conf = match Config::load_from_file("project.toml") {
            Ok(conf) => conf,
            Err(e) => {
                eprint(e.to_string());
                return;
            }
        };


        for pkg_name in pkg_names.iter() {
            let (vname, mut ver) = parse_version(pkg_name.clone());

            if install_package(pkg_name.clone()) {
                if ver.len() == 0 {
                    // if no version, set to latest
                    let v = get_pkg_version(&pkg_name);
                    if v.is_err() {
                        return;
                    }
                    ver = v.unwrap();
                } 
                conf.packages.insert(vname.clone(), ver.clone());
                match conf.write_to_file("project.toml") {
                    Ok(_) => {
                        iprint(format!("Package '{}' installed successfully", &vname));
                    },
                    Err(e) => {
                        eprint(e.to_string());
                        return;
                    }
                }
            } else {
                eprint(format!("Package '{}' failed to install", &vname));
            }
        }
    }

    pub fn install_packages(&self) {

        if self.requirements.len() > 0 {
            self.install_from_req();
            return;
        }

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
                wprint("Installation Cancelled".to_owned());
                return;
            }
        }

        let mut cmd = Command::new("./venv/Scripts/pip.exe");
        cmd.arg("install");
        for (name, version) in conf.packages.iter() {
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
}