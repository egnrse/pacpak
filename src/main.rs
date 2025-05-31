//! shallow flatpak integration into pacman (as a wrapper)
// (by egnrse)

// main.rs

use clap::Parser;		// cli input parser
use std::process::{exit, ExitStatus}; // exit with an error
use colored::{Colorize, control};	// format output strings (for the terminal)
//dev; needed
use std::env;			// fetch the environment args
use std::process::{Command, Stdio};
use std::io::Read;		// pipe the output of a command
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

/// exit statuses for pacpak (often the return status of pacman is used)
mod exit_status {
	/// a command returns no results
	pub const NOT_FOUND: i32 = 1;
	/// an unkown failure within this or within a called program occurs
	pub const ERROR: i32 = 255;

}

/// store (user) settings
struct Config {
	wrap_pacman: bool,
	color:		bool,
}
/// standart values for the settings
impl Default for Config {
	fn default() -> Self {
		Self {
			wrap_pacman: true,
			color:		true,
		}
	}
}


/// message strings for eg. -Qi fields, --help (and more)
mod text {
	use indoc::indoc;	// multiline strings

	pub const NOT_IMPLEMENTED: &str = "[not implemented]";
	pub const NONE: &str = "None";
	pub const INSTALL_REASON_EXP: &str = "Explicitly installed";
	pub const INSTALL_REASON_DEP: &str = "Installed as a dependency for another package";
	pub const INSTALLED_MARKER: &str = "[installed]";
	/// Prefix for error messages
	pub const ERROR_PREFIX:&str = "error:";
	pub const NO_TARGETS:&str = "no targets specified (use -h for help)";
	/// Identation for the version display
	pub const VERSION_IDENTATION: &str = "                       ";
	/// Identation for the description in eg. -Ss
	pub const DESCRIPTION_IDENTATION: &str = "    ";
	/// help/usage message
	pub const HELP_USAGE: &str = indoc! {r#"
		usage: pacpak <operation> [...]
		operations:
			{-Q, --query}
			{-S, --sync}
			{-R, --remove}
			{-h --help}
			{-V --version}
			
			(or other pacman operations)
	"#};
}



/// output the given app in the format:
///		`id (name) version (branch)`
/// (similar to `pacman -Q`)
fn print_app_short(app: &FlatpakApp) {
	println!("{} ({}) {} ({})", app.id.bold(), app.name, app.version.bold().green(), app.branch);
}

/// output the given app in the format:
///		`remote/id (name) version (branch)`
/// (similar to `pacman -Ss`)
fn print_app_long(app: &FlatpakApp, installed: bool) {
	let installed_str: &str = if installed {
		&text::INSTALLED_MARKER.cyan().bold().to_string()
	} else { "" };
	
	println!("{}{}{} ({}) {} ({}) {}", app.origin.magenta().bold(), "/".bold(), app.id.bold(), app.name, app.version.bold().green(), app.branch, installed_str);
	println!("{}{}", text::DESCRIPTION_IDENTATION, app.description);
}

///	output the given app similar to `pacman -Qi`
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
	println!("{} {}", "Optional Deps	:".bold(), text::NONE);
	println!("{} {}", "Required By	:".bold(), text::NOT_IMPLEMENTED);
	println!("{} {}", "Optional For	:".bold(), text::NONE);
	println!("{} {}", "Conflicts With	:".bold(), text::NONE);
	println!("{} {}", "Replaces	:".bold(), text::NOT_IMPLEMENTED);
	println!("{} {}", "Installed Size	:".bold(),app.install_size);
	println!("{} {}", "Packager	:".bold(), app.packager);
	println!("{} {}", "Build Date	:".bold(),app.build_date);
	println!("{} {}", "Install Date	:".bold(),app.install_date);
	
	if app.runtime.is_empty() {
		println!("{} {}", "Install Reason	:".bold(), text::INSTALL_REASON_DEP);
	} else {
		println!("{} {}", "Install Reason	:".bold(), text::INSTALL_REASON_EXP);
	}
	
	println!("{} {}", "Install Script	:".bold(), text::NOT_IMPLEMENTED);
	println!("{} {}", "Validated By 	:".bold(), text::NOT_IMPLEMENTED);
	println!();
}

/// find the installed flatpak package that owns the target file  
/// return its index (in flatpak.apps) or -1 if none was found
fn is_owned_by(flatpak: &mut FlatpakMeta, target: &str) -> std::io::Result<isize> {
	let target_path = PathBuf::from(&target);
	for index in 0..flatpak.apps.len() {
		if flatpak.apps[index].location.is_empty() {
			let _ = flatpak.get_location(index);
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


/// call pacman with the given args (inherit buffers)  
/// return the exit status
fn pacman_exec(args: &Vec<String>) -> ExitStatus {
	Command::new("pacman")
		.args(args)
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.status()
		.expect(&format!("{prefix} failed to execute pacman", prefix = text::ERROR_PREFIX.red().bold()))
}

/// call pacman with the given args (pipe buffers)  
/// return a tuple of (stdout, stderr, exit status)
fn pacman_run(args: &Vec<String>) -> (String, String, ExitStatus) {
	let mut child = Command::new("pacman")
		.args(args)
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect(&format!("{prefix} failed to execute pacman", prefix = text::ERROR_PREFIX.red().bold()));

	let mut stdout = String::new();
	let mut stderr = String::new();

	if let Some(mut out) = child.stdout.take() {
		out.read_to_string(&mut stdout)
			.expect(&format!("{prefix} failed to read stdout", prefix = text::ERROR_PREFIX.red().bold()));
	}
	if let Some(mut err) = child.stderr.take() {
		err.read_to_string(&mut stderr)
			.expect(&format!("{prefix} failed to read stderr", prefix = text::ERROR_PREFIX.red().bold()));
	}

	let status = child.wait()
		.expect(&format!("{prefix} failed to wait for pacman child process", prefix = text::ERROR_PREFIX.red().bold()));
	(stdout, stderr, status)
}

/// call flatpak with the given args (pipe buffers)  
/// return a tuple of (stdout, stderr, exit status)
fn flatpak_run(args: &Vec<String>) -> (String, String, ExitStatus) {
	let mut child = Command::new("flatpak")
		.args(args)
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect(&format!("{prefix} failed to execute flatpak", prefix = text::ERROR_PREFIX.red().bold()));

	let mut stdout = String::new();
	let mut stderr = String::new();

	if let Some(mut out) = child.stdout.take() {
		out.read_to_string(&mut stdout)
			.expect(&format!("{prefix} failed to read stdout", prefix = text::ERROR_PREFIX.red().bold()));
	}
	if let Some(mut err) = child.stderr.take() {
		err.read_to_string(&mut stderr)
			.expect(&format!("{prefix} failed to read stderr", prefix = text::ERROR_PREFIX.red().bold()));
	}

	let status = child.wait()
		.expect(&format!("{prefix} failed to wait for flatpak child process", prefix = text::ERROR_PREFIX.red().bold()));
	(stdout, stderr, status)
}


/// entry point
fn main() {
	let args = Cli::parse();
	let args_raw: Vec<String> = env::args().skip(1).collect();
	let config = Config {	//dev: overwrite some fields for now
		//color: false,
		wrap_pacman: true,
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
	
	// pacman integration
	let mut stderr_pacman = String::new();
	let status_true: ExitStatus = Command::new("true")
		.status()
		.expect(&format!("{prefix} failed to get exit status", prefix = text::ERROR_PREFIX.red().bold()));
	let mut status: ExitStatus = status_true.clone();

	// basic operations
	if args.help {
		println!("{}", text::HELP_USAGE);
		return;
	} else if args.version {
		let indent = if config.wrap_pacman {
			text::VERSION_IDENTATION
		} else {
			""
		};
		println!("{}", indent);
		println!("{}Pacpak v{} - {} License", indent, VERSION, LICENSE);
		println!("{}Copyright (C) 2025 {}", indent, AUTHORS);
		println!("{}", indent);
		println!("{}---", indent);
		println!("{}", indent);
		let (stdout_flatpak,_,_) = flatpak_run(&vec!["--version".to_string()]);
		println!("{}{}", indent, stdout_flatpak);
		if config.wrap_pacman {
			println!("{}---", indent);
			pacman_exec(&args_pacman);
		}
		return;
	}

	// init flatpak metadata
	let mut flatpak = FlatpakMeta::default();
	// update flatpak app list
	match flatpak.get_apps() {
		Ok(apps) => apps,
		Err(e) => {
			println!("Error: {}", e);
			exit(exit_status::ERROR);
		}
	};

	// the targets for operations (often packages or files)
	let targets : Vec<&str> = args.targets
		.iter()
		.map(|s| s.as_str())
		.collect();
	
	// other operations
	if args.query {
		if config.wrap_pacman {
			let mut cmd = Command::new("pacman");
			cmd.args(&args_pacman)
				.stdin(Stdio::inherit())
				.stdout(Stdio::inherit());
			cmd.stderr(Stdio::piped());
			let mut child = cmd.spawn()
				.expect(&format!("{prefix} failed to execute pacman", prefix = text::ERROR_PREFIX.red().bold()));
			if let Some(mut err) = child.stderr.take() {
				err.read_to_string(&mut stderr_pacman)
					.expect(&format!("{prefix} failed to read stderr", prefix = text::ERROR_PREFIX.red().bold()));
			}
			status = child.wait()
				.expect(&format!("{prefix} failed to wait on pacman", prefix = text::ERROR_PREFIX.red().bold()));
		}
		if args.info {
			// show info for a package
			let results = flatpak.search_apps(&targets);
			if results.is_empty() && status.code().unwrap_or(exit_status::ERROR) > 0 {
				eprintln!("{}", stderr_pacman);
				exit(status.code().unwrap_or(exit_status::ERROR));
			} else if results.is_empty() {
				exit(status.code().unwrap_or(exit_status::ERROR));
			}
            for index in results {
                match flatpak.get_app_info_full(index) {
                    Ok(app) => app,
                    Err(e) => {
                        eprintln!("{} {}", text::ERROR_PREFIX.red().bold(), e);
                        exit(exit_status::ERROR);
                    }
                };
                print_app_info(&flatpak.apps[index]);
            }
		} else if args.owns {
			// which package owns this file
			if targets.len() == 0 {
				eprintln!("{}", stderr_pacman);
				exit(status.code().unwrap_or(exit_status::ERROR));
				//eprintln!("{} {}", text::ERROR_PREFIX.red().bold(), "no targets specified");	//dev
				//exit(1);
			}
			// look if already found (by pacman)
			if status.code().unwrap_or(exit_status::ERROR) == 0 {
				exit(status.code().unwrap_or(exit_status::ERROR));
			}
			for target in &targets {
				let idx = is_owned_by(&mut flatpak, target);
				let idx = match idx {
					Ok(idx) => idx,
                    Err(e) => {
                        eprintln!("{} {}", text::ERROR_PREFIX.red().bold(), e);
                        exit(exit_status::ERROR);
                    }
				};
				if idx >= 0 {
					println!("{}", flatpak.apps[idx as usize].extid);
				} else {
					eprintln!("{}", stderr_pacman);
					exit(status.code().unwrap_or(exit_status::ERROR));
					//eprintln!("Error : {} {}", "no package owns", target);	//dev
				}
			}//for target
		} else if args.list {
			// list files of a package
			let matches: Vec<usize> = flatpak.search_apps(&targets);
			if matches.len() == 0 {
				eprintln!("{}", stderr_pacman);
				exit(status.code().unwrap_or(exit_status::ERROR));
			}
			for i in matches {
				let files = match flatpak.get_app_files(i) {
					Ok(f) => f,
					Err(e) => {
                        eprintln!("{} {}", text::ERROR_PREFIX.red().bold(), e);
                        exit(exit_status::ERROR);
					}
				};
				for f in &files {
					println!("{} {}", flatpak.apps[i].extid.bold(), f);
				}
			}
		} else if args.search {
			let matches: Vec<usize>  = flatpak.search_apps_desc(&targets);
			if matches.len() == 0 {
				//eprintln!("{} {}", text::ERROR_PREFIX.red().bold(), "no package found");	//dev
				exit(exit_status::NOT_FOUND);
			}
			for i in matches {
				let app = &flatpak.apps[i];
				let installed = false;
				print_app_long(&app, installed);
			}
			
		} else {
			// just -Q
			let results = flatpak.search_apps(&targets);
			if results.is_empty() {
				eprintln!("{}", stderr_pacman);
				exit(status.code().unwrap_or(exit_status::ERROR));
			}
			for index in results {
                match flatpak.get_app_info(index) {
                    Ok(app) => app,
                    Err(e) => {
                        eprintln!("{} {}", text::ERROR_PREFIX.red().bold(), e);
                        exit(exit_status::ERROR);
                    }
                };
                let app : &FlatpakApp = &flatpak.apps[index];
				print_app_short(app);
			}
		}
	
	} else if args.sync {
		if args.search {
			// format: remote/print_app_short() [installed]
			pacman_exec(&args_pacman);
			
			let matches: Vec<FlatpakApp>  = match flatpak.search(targets) {
				Ok(app) => app,
				Err(e) => {
					eprintln!("{} {}", text::ERROR_PREFIX.red().bold(), e);
					exit(exit_status::ERROR);
				}
			};
			if matches.len() == 0 {
				//eprintln!("{} {}", text::ERROR_PREFIX.red().bold(), "no package found");
				exit(exit_status::NOT_FOUND);
			}
			for app in &matches {
				let installed = flatpak.apps.iter()
					.any(|a| a.id == app.id && a.branch == app.branch);
				print_app_long(&app, installed);
			}
			//search
		} else {
			if targets.len() == 0 {
				eprintln!("{} {}", text::ERROR_PREFIX.red().bold(), text::NO_TARGETS);
				exit(exit_status::NOT_FOUND);
			}
			//dev: test if it is a flatpak
			//dev: if both is available (flat and pacman), let choose
			//dev: for each package?
			let pkg: String = format!("^{}$", targets[0]);
			let (stdout_pac, stderr_pac, status_pac) = pacman_run(&vec!["-Ss".to_string(), pkg]);
			if !stdout_pac.is_empty() && status_pac == status_true {
				status = pacman_exec(&args_pacman);
				exit(status.code().unwrap_or(exit_status::ERROR));
			} else {
				//else try flatpak?
				println!("{:?}", stdout_pac);
				println!("{:?}", stderr_pac);
				println!("{:?}", status_pac);
			}
			println!("Operation not implemented.");
		}

	} else if args.remove {
		//let (_, _, status) = pacman_run(vec!["-Ss".to_string(), pkg]);
		pacman_exec(&args_pacman);
		println!("Operation not implemented.");
	} else if args.database {
		pacman_exec(&args_pacman);
		println!("Operation not implemented.");
	} else if args.deptest {
		pacman_exec(&args_pacman);
		println!("Operation not implemented.");
	} else if args.upgrade {
		pacman_exec(&args_pacman);
		println!("Operation not implemented.");
	} else if args.files {
		pacman_exec(&args_pacman);
		println!("Operation not implemented.");
	}

	//println!("{}", "Hello, world!".blue());
	//println!("pattern: {:?}, path: {:?}", args.pattern, args.path)
}
