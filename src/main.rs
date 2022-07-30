mod project_managers;
mod utils;
mod settings;
mod ppm_functions;

use project_managers::Action;
use clap::Parser;


const VERSION : &str = env!("CARGO_PKG_VERSION");
const ABOUT: &'static str = env!("CARGO_PKG_DESCRIPTION");
const AUTHOR : &'static str = env!("CARGO_PKG_AUTHORS");

/// Python Project Manager
#[derive(Parser, Debug)]
#[clap(author=AUTHOR, version=VERSION, about=ABOUT, long_about = None)]
struct Cli {

    #[clap(subcommand)]
    command: Action,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Action::New(project) => project.create_project(false),
        Action::Init(project) => project.create_project(true),
        Action::Add(add_proj) => add_proj.add_package(),
        Action::Rm(rp) => rp.remove_package(),
        Action::Run(run) => run.run_script(),
        Action::Install(installer) => installer.install_packages(),
        Action::Info => ppm_functions::show_project_info(),
        Action::Gen => ppm_functions::gen_requirements(),
        Action::Start => ppm_functions::start_project(),
        Action::Update => ppm_functions::update_packages(),
    }
}