mod project_creator;
mod utils;

use project_creator::Action;
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
        Action::New(project) => project.create_project(),
        Action::Add(add_proj) => add_proj.add_package(),
        Action::Remove(rp) => rp.remove_package(),
        Action::Run(run) => run.run_script(),
        Action::Info => utils::show_project_info(),
        Action::Gen => utils::gen_requirements(),
        Action::Start => utils::start_project(),
        Action::Install => utils::install_packages(),
        Action::Update => utils::update_packages(),
    }
}