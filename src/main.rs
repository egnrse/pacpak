//! shallow flatpak integration into pacman (as a wrapper)
// (by egnrse)

// main.rs

use clap::Parser;		// cli input parser
use std::process::exit; // exit with an error
use colored::{Colorize, control};	// format output strings (for the terminal)
//dev; needed
use std::env;	// fetch the environment args
use std::process::{Command, Stdio};
// file path stuff
use std::path::PathBuf;
use std::fs;

// handling of cli args in cli.rs
mod cli;
use cli::Cli;
// flatpak integration in flatpak.rs
mod flatpak;
use flatpak::{FlatpakMeta, FlatpakApp};


const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");

/// return value when an unkown failure within this or within a called program occurs
const EXIT_ERROR: i32 = 255;

/// store (user) settings
struct Config {
	wrap_pacman: bool,
	speed:		bool,
	color:		bool,
}
/// standart values for the settings
impl Default for Config {
	fn default() -> Self {
		Self {
			wrap_pacman: true,
			speed:		false,
			color:		true,
		}
	}
}


/// message strings for eg. -Qi fields, --help (and more)
mod messages {
	use indoc::indoc;	// multiline strings

	pub const NOT_IMPLEMENTED: &str = "[not implemented]";
	pub const NONE: &str = "None";
	pub const INSTALL_REASON_EXP: &str = "Explicitly installed";
	pub const INSTALL_REASON_DEP: &str = "Installed as a dependency for another package";
	pub const VERSION_IDENTATION: &str = "                       ";
	/// help/usage message
	pub const HELP_USAGE: &str = indoc! {r#"
		usage: pacpak <operation> [...]
		operations:
			{-Q, --query}
			{-S, --sync}
			{-R, --remove}
			{-h --help}
			{-V --version}
			TODO	//dev
	"#};
}



///	outputs text similar to `pacman -Qi`
fn print_app_info(app: &FlatpakApp) {
	let mut name = String::new();
	if !app.name.is_empty() {
		name = format!("({})",app.name)
	}
	println!("{} {} {}", "Name		:".bold(),app.id, name);
	println!("{} {} ({})", "Version		:".bold(),app.version, app.branch);
	println!("{} {}", "Description	:".bold(),app.description);
	println!("{} {}", "Architecture	:".bold(),app.arch);
	println!("{} {}", "URL		:".bold(),app.url);
	println!("{} {}", "Licenses	:".bold(),app.license);
	println!("{} {}", "Groups		:".bold(),app.collection);
	println!("{} {}", "Provides	:".bold(),app.provides);
	println!("{} {}", "Depends On	:".bold(),app.depends);
	println!("{} {}", "Optional Deps	:".bold(), messages::NONE);
	println!("{} {}", "Required By	:".bold(), messages::NOT_IMPLEMENTED);
	println!("{} {}", "Optional For	:".bold(), messages::NONE);
	println!("{} {}", "Conflicts With	:".bold(), messages::NOT_IMPLEMENTED);
	println!("{} {}", "Replaces	:".bold(), messages::NOT_IMPLEMENTED);
	println!("{} {}", "Installed Size	:".bold(),app.install_size);
	println!("{} {}", "Packager	:".bold(), app.packager);
	println!("{} {}", "Build Date	:".bold(),app.build_date);
	println!("{} {}", "Install Date	:".bold(),app.install_date);
	
	if app.runtime.is_empty() {
		println!("{} {}", "Install Reason	:".bold(), messages::INSTALL_REASON_DEP);
	} else {
		println!("{} {}", "Install Reason	:".bold(), messages::INSTALL_REASON_EXP);
	}
	
	println!("{} {}", "Install Script	:".bold(), messages::NOT_IMPLEMENTED);
	println!("{} {}", "Validated By 	:".bold(), messages::NOT_IMPLEMENTED);
	println!();
}

/// finds the flatpak package that owns the target file
/// returns its index (from flatpak.apps) or -1 if none was found
fn is_owned_by(flatpak: &mut FlatpakMeta, target: &str) -> std::io::Result<isize> {
	let target_path = PathBuf::from(&target);
	for index in 0..flatpak.apps.len() {
		if flatpak.apps[index].location.is_empty() {
			//dev: only get location (make a FlatpakMeta fn?)
			let _ = flatpak.get_app_info_full(index);
		}
		let app_path = PathBuf::from(&flatpak.apps[index].location);
		let target_path = fs::canonicalize(&target_path)?;
		let app_path = fs::canonicalize(&app_path)?;
		if target_path.starts_with(&app_path) {
			return Ok(index.try_into().unwrap());
		}
	}//for
	Ok(-1)
}


/// entry point
fn main() {
	let args = Cli::parse();
	let args_raw: Vec<String> = env::args().skip(1).collect();
	let config = Config {	//dev: overwrite some fields for now
		//color: false,
		wrap_pacman: false,
		..Default::default()
	};

	let mut args_pacman: Vec<String> = args_raw.clone();
	// add pacman args for color arguments
	args_pacman.insert(0,"--color".to_string());
	if config.color {
		args_pacman.insert(1,"always".to_string());
	} else {
		args_pacman.insert(1,"never".to_string());
		// deactivate the colored crate globally
		control::set_override(false);
	}
	if config.wrap_pacman & !args.help {
		let status = Command::new("pacman")
			.args(&args_pacman)
			.stdin(Stdio::inherit())
			.stdout(Stdio::inherit())
			.stderr(Stdio::inherit())
			.status()
			.expect("ERROR: failed to execute pacman");
		if !status.success() {
			// test if this is an ok error? //dev
			//exit(status.code().unwrap_or(EXIT_ERROR));
		}
	}
	
	// basic operations
	if args.help {
		println!("{}", messages::HELP_USAGE);
		return;
	} else if args.version {
		let indent = if config.wrap_pacman {
			messages::VERSION_IDENTATION
		} else {
			""
		};
		if config.wrap_pacman {
			println!("{}---", indent);
			println!("{}", indent);
		}
		println!("{}Pacpak v{} - {} License", indent, VERSION, LICENSE);
		println!("{}Copyright (C) 2025 {}", indent, AUTHORS);
		return;
	}

	// init flatpak metadata
	let mut flatpak = FlatpakMeta::default();
	// update flatpak app list
	match flatpak.get_apps() {
		Ok(apps) => apps,
		Err(e) => {
			println!("Error: {}", e);
			exit(EXIT_ERROR);
		}
	};

	// the targets for operations (often packages or files)
	let targets : Vec<&str> = args.targets
		.iter()
		.map(|s| s.as_str())
		.collect();
	
	// other operations
	if args.query {
		if args.info {
			// show info for a package
            for index in flatpak.search_apps(&targets) {
                match flatpak.get_app_info_full(index) {
                    Ok(app) => app,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        exit(EXIT_ERROR);
                    }
                };
                print_app_info(&flatpak.apps[index]);
            }
		} else if args.owns {
			// which package owns this file
			//dev: check with parent folder? (appid)
			if targets.len() == 0 {
				eprintln!("Error: {}", "no targets specified");
				exit(1);
			}
			for target in &targets {
				let idx = is_owned_by(&mut flatpak, target);
				let idx = match idx {
					Ok(idx) => idx,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        exit(EXIT_ERROR);
                    }
				};
				if idx >= 0 {
					println!("{}", flatpak.apps[idx as usize].extid);
				} else {
					eprintln!("Error : {} {}", "no package owns", target);
				}
			}//for target
		} else if args.list {
			// list files of a package
			//dev: similar to above
			println!("Operation not implemented.");
		} else {
			//
			for index in flatpak.search_apps(&targets) {
                match flatpak.get_app_info(index) {
                    Ok(app) => app,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        exit(EXIT_ERROR);
                    }
                };
                let app : &FlatpakApp = &flatpak.apps[index];
				println!("{} ({}) {} ({})", app.id.bold(), app.name, app.version.bold().green(), app.branch);
			}
		}
	} else if args.sync {
		//dev
		println!("{}{}", "Hello, world!".bold(), " Line2".green());
		let args_raw: Vec<String> = env::args().skip(1).collect();
		println!("{:?}",args_raw);
		
		if config.wrap_pacman {
			let status = Command::new("pacman")
				.args(&args_raw)
				//.args(["-Q", "sed"])	//dev
				.stdin(Stdio::inherit())
				.stdout(Stdio::inherit())
				.stderr(Stdio::inherit())
				.status()
				.expect("failed to execute pacman");	//dev: error msg

			exit(status.code().unwrap_or(EXIT_ERROR));
		}
	} else if args.remove {
	} else if args.database {
	} else if args.deptest {
	} else if args.upgrade {
	} else if args.files {
	}

	//println!("{}", "Hello, world!".blue());
	//println!("pattern: {:?}, path: {:?}", args.pattern, args.path)
}
