
use clap::{Subcommand, Args};
use ini::Ini;
// reqwes blocking client

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

    /// Add a new module to project 
    Add(AddPackage),

    /// Remove a module from project
    Remove(RemovePackage),

    /// Run a script
    Run(RunScript),

    /// Run main.py
    Start,

    /// Generate requirements.txt file
    Gen,

    /// Show the project.ini file
    Info,
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
    #[clap(short = 'v', long = "no-venv", takes_value = false)]
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
        // create empty section Packages
        conf.with_section(Some("Packages"));
        // create empty section Scripts
        conf.with_section(Some("Scripts"));
        conf.write_to_file(format!("{}/project.ini", self.name)).unwrap();
    }

    fn setup_venv(&self) {
        iprint("Setting Up Virtual Environment...".to_string());
        let venv = process::Command::new("python")
            .arg("-m")
            .arg("venv")
            .arg(format!("{}/venv", self.name))
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
        iprint("Virtual Environment Created Successfully".to_string());
       
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
            self.setup_venv();
        } else {
            wprint("Virtual environment is disable 'add' and 'remove' command will not work".to_string());
        }

        // save
        self.save_config();

        let elapsed = start.elapsed();
        iprint(format!("Completed in {}s", elapsed.as_secs()));

    }

}


#[derive(Args, Debug)]
pub struct AddPackage {

    /// Package Name takes multiple values
    pub pkg_name: Vec<String>,

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
        for pkg_name in self.pkg_name.iter() {
            if !Path::new(&"project.ini").exists() {  
                eprint("No project.ini found".to_owned());
                return;
            }
            let mut conf = Ini::load_from_file("project.ini").unwrap();
            let packages = conf.section_mut(Some("Packages"));
    
            match packages {
                Some(p) => {
                    if p.contains_key(pkg_name.as_str()) {
                        eprint(format!("Package '{}' already exists", pkg_name));
                        return;
                    }
                    if self.install_package(pkg_name.clone()) {
                        p.append(pkg_name, self.get_pkg_version(pkg_name));
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
                None => {
                    conf.with_section(Some("Packages"))
                        .set(pkg_name.as_str(), self.get_pkg_version(pkg_name));
                    match conf.write_to_file("project.ini") {
                        Ok(_) => {
                            self.install_package(pkg_name.clone());
                            iprint(format!("Package '{}' added successfully", pkg_name));
                        },
                        Err(e) => {
                            eprint(e.to_string());
                            exit(1);
                        }
                    }
                }
            }
        }
        
    }
}


#[derive(Args, Debug)]
pub struct RemovePackage {

    /// Package Name
    pub pkg_name: String,

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
        let mut conf = Ini::load_from_file("project.ini").unwrap();
        let packages = conf.section_mut(Some("Packages"));

        for (key, v) in packages.as_ref().unwrap().iter() {
            println!("{} : {}", key, v);
        }

        match packages {
            Some(p) => {
                if !p.contains_key(self.pkg_name.as_str()) {
                    eprint(format!("Package '{}' does not exist", self.pkg_name));
                    return;
                }
                if self.uninstall_package(self.pkg_name.to_string()) { 
                    p.remove(self.pkg_name.as_str());    
                    match conf.write_to_file("project.ini") {
                        Ok(_) => {
                            iprint(format!("Package '{}' removed successfully", self.pkg_name));      
                        },
                        Err(e) => {
                            eprint(e.to_string());
                            exit(1);
                        }
                    }
                } else {
                    eprint(format!("Package '{}' could not be removed", self.pkg_name));
                }
                 
            }
            None => {
                eprint(format!("Package '{}' does not exist", self.pkg_name));
                exit(1);
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
        let mut conf = match Ini::load_from_file("project.ini")  {
            Ok(c) => c,
            Err(e) => {
                eprint("Failed to load project.ini".to_owned());
                eprint(e.to_string());
                exit(1);
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
                let mut cmd = process::Command::new("cmd");
                cmd.args(&["/C", cmd_str]);
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