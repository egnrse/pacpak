//! fetch data from flatpak (over its cli)
// flatpak.rs

use std::process::Command;
use std::io;
use std::fs;
use chrono::{DateTime, Local};

/// string constants (eg. for errors or meta field values)
mod text {
	pub const VERSION_UNKOWN: &str = "?";
	//pub const NONE: &str = "None";
	pub const NOT_IMPLEMENTED: &str = "[not implemented]";
	pub const SKIPPED: &str = "[skipped]";
}

/// strings from flatpak commands (or their output)
pub mod flatpak_strings {
	// return string when no results where found for `flatpak search TEXT`
	pub const SEARCH_NO_RESULTS: &str = "No matches found";
}

/// flatpak app metadata
#[derive(Debug, Default, Clone)]
pub struct FlatpakApp {
	/// extended app id in the format: `appID/arch[itecture]/branch`
	pub extid: String,
	pub name: String,
	pub description: String,
	/// appID (in reverse dns form)
	pub id: String,
	/// architecture
	pub arch: String,
	pub branch: String,
	pub version: String,
	pub license: String,
	pub origin: String,
	pub collection: String,
	/// can be `system` or `user`
	pub installation: String,
	pub install_size: String,
	pub runtime: String,
	pub sdk: String,
	pub commit: String,
	pub parent: String,
	pub subject: String,
	pub build_date: String,
	
	// extra fields
	pub install_date: String,
	pub location: String,
	pub url: String,
	pub provides: String,
	pub packager: String,
	pub depends: String,
	//pub v: String,
}

/// flatpak meta object (houses all (app) metadata)
#[derive(Default, Clone)]
pub struct FlatpakMeta {
	/// information for all (flatpak) apps
	pub apps: Vec<FlatpakApp>,
	/// output of `flatpak list` with only a few columns
	pub list_small: String,
	/// output of `flatpak list` with many columns
	pub list_full: String,
}

impl FlatpakMeta {
	// can be instantiated with Default::default()
	
	/// fetch a basic list off all (installed flatpak) apps from the flatpak cli
	/// (overwrites the current self.apps vector)
	pub fn get_apps(&mut self) -> io::Result<&Vec<FlatpakApp>> {
		let flatpak_list_raw = Command::new("flatpak")
			.args(["list", "--columns=application,arch,branch,origin"])
			.output()?;
		if !flatpak_list_raw.status.success() {
			return Err(io::Error::new(io::ErrorKind::Other, "command: 'flatpak list' failed")); //dev
		}
		self.list_small = String::from_utf8_lossy(&flatpak_list_raw.stdout).into();
		let flatpak_list_str = &self.list_small;
		let mut apps: Vec<FlatpakApp> = flatpak_list_str
			.lines()
			.filter_map(|line| {
				let columns: Vec<&str> = line.split('\t').collect();
				if columns.len() == 4 {
					Some(FlatpakApp {
						id: columns[0].into(),
						arch: columns[1].into(),
						branch: columns[2].into(),
						origin: columns[3].into(),
						..Default::default()
					})
				} else {
					None
				}
			})
			.collect();

		for app in &mut apps {
			app.extid = format!("{}/{}/{}", app.id, app.arch, app.branch);
		}
		self.apps = apps;
		Ok(&self.apps)
	}
	
	/// searches for programs in self.apps, with a name/id similar to `input`
	/// returns a vector of indexes (for self.apps)
	pub fn search_apps(&mut self, input: &Vec<&str>) -> Vec<usize> {
		//println!("{:?}",input);
		let mut out : Vec<usize> = Vec::new();
		if input.len() > 0 && !input[0].is_empty() {
			for text in input { 
				let text = text.to_lowercase();
				
				for (i, app) in self.apps.iter().enumerate() {
					if app.extid.to_lowercase().contains(&text) || app.name.to_lowercase().contains(&text) {
						if !out.contains(&i) { out.push(i); }
						//println!("YES: {}",extid)
					}
				}// for (i, app)
			}// for text
			out.sort();
		} else {
			// output all indexes if no string was given
			out = (0..self.apps.len()).collect();
		}
		return out;
    }
	/// same as FlatpakMeta::search_apps(), but also searches in the description/origin
	/// returns a vector of indexes (for self.apps)
	pub fn search_apps_desc(&mut self, input: &Vec<&str>) -> Vec<usize> {
		// fetch some unfilled fields (for self)
		for i in 0..self.apps.len() {
			let app = &self.apps[i];
			if app.description.is_empty() || app.origin.is_empty() || app.branch.is_empty() {
				let _ = self.get_app_info_full(i);
			}
		}

		let mut out : Vec<usize> = Vec::new();
		if input.len() > 0 && !input[0].is_empty() {
			for text in input { 
				let text = text.to_lowercase();
				
				for (i, app) in self.apps.iter().enumerate() {
					if app.extid.to_lowercase().contains(&text) 
						|| app.name.to_lowercase().contains(&text) 
						|| app.description.to_lowercase().contains(&text)
						|| app.origin.to_lowercase().contains(&text) {
						if !out.contains(&i) { out.push(i); }
						//println!("YES: {}",extid)
					}
				}// for (i, app)
			}// for text
			out.sort();
		} else {
			// output all indexes if no string was given
			out = (0..self.apps.len()).collect();
		}
		return out;
    }
	/// get some basic infos about a (flatpak) app
	pub fn get_app_info(&mut self, idx: usize) -> io::Result<&FlatpakApp> {
		if self.list_full.is_empty() {
			let flatpak_list_raw = Command::new("flatpak")
				.args(["list", "--columns=name,application,arch,branch,version,application"])
				.output()?;
			if !flatpak_list_raw.status.success() {
				return Err(io::Error::new(io::ErrorKind::Other, "command: 'flatpak list' failed")); //dev
			}
			self.list_full = String::from_utf8_lossy(&flatpak_list_raw.stdout).into();
		}
		
		let searchterms = [&self.apps[idx].id, &self.apps[idx].arch, &self.apps[idx].branch];
		if let Some(matching) = self.list_full
			.lines()
			.find(|line| searchterms.iter().all(|k| line.contains(k.as_str())))
		{
			let columns: Vec<&str> = matching.split('\t').collect();
			if columns.len() >= 4 {
				self.apps[idx].name = columns[0].into();
				self.apps[idx].version = columns[4].into();
                if self.apps[idx].version.is_empty() {
                    self.apps[idx].version = "?".to_string();
                }
			} else {
				//dev failed
                eprintln!("error stuff: app has to few columns");
			}
		} else {
			//dev failed
			eprintln!("error stuff: app not found in str");
		}
		//println!("{:?}", matching);
		
		Ok(&self.apps[idx])
	}

	/// get detailed infos about a (flatpak) app
	pub fn get_app_info_full(&mut self, idx: usize) -> io::Result<&FlatpakApp> {
		let flatpak_info_raw = Command::new("flatpak")
			.args(["info", &self.apps[idx].extid])
			.output()?;
		if !flatpak_info_raw.status.success() {
			return Err(io::Error::new(io::ErrorKind::Other, "command: 'flatpak info' failed")); //dev
		}
		let info_str: String = String::from_utf8_lossy(&flatpak_info_raw.stdout).into();

		// before stuff
		let app = &mut self.apps[idx];
		app.depends = "flatpak ".to_string();

		let mut desc_read = false;	// detect multiline descriptions
		for line in info_str.lines() {

			if desc_read {
				// detect multiline descriptions
				if line.is_empty() {
					desc_read = false;
				} else {
					let pre_space = "		  ";	// space before the next line of description
					app.description += &format!("\n{}{}",pre_space,line.trim());
				}
			} else if let Some((key, value)) = line.split_once(':') {
				let key = key.trim();
				let value = value.trim();

				match key {
					"ID"     => app.id     = value.to_string(),
					//"Name"   => app.name   = value.to_string(),
					"Arch"   => app.arch   = value.to_string(),
					"Branch" => app.branch = value.to_string(),
					"Version" => app.version = value.to_string(),
					"License" => app.license = value.to_string(),
					"Origin" => app.origin = value.to_string(),
					"Collection" => app.collection = value.to_string(),
					"Installation" => app.installation = value.to_string(),
					"Installed" => app.install_size = value.to_string(),
					"Runtime" => {
						app.runtime = value.to_string();
						if !value.is_empty() {
							app.depends += value;
						}
					},
					"Sdk" => app.sdk = value.to_string(),
					
					"Commit" => app.commit = value.to_string(),
					"Parent" => app.parent = value.to_string(),
					"Subject" => app.subject = value.to_string(),
					"Date" => {
						app.build_date = value.to_string();
						// try to parse the date into the pacman format
						if let Ok(time) = DateTime::parse_from_str(&app.build_date, "%Y-%m-%d %H:%M:%S %z") {
							app.build_date = time.format("%a %d %b %Y %I:%M:%S %p %Z").to_string();
						}
					},
					_ => {}, // ignore unknown keys
				}//match
			} else if let Some((name, value)) = line.split_once('-') {
				app.name = name.trim().into();
				app.description = value.trim().into();
				desc_read = true;
			}
		}// for line
		
		// after stuff (or unimplemented fields)
		if app.version.is_empty() { app.version = text::VERSION_UNKOWN.to_string(); }
		if app.license.is_empty() { app.license = text::VERSION_UNKOWN.to_string(); }
		app.packager = text::NOT_IMPLEMENTED.to_string();
		app.url = text::NOT_IMPLEMENTED.to_string();
		app.provides = text::NOT_IMPLEMENTED.to_string();
		
		// calc / fetch other fields
		if true {
			let _ = self.get_location(idx);
			let app = &mut self.apps[idx];
			
			// get install date
			let meta = fs::metadata(&app.location);
			if let Err(e) = meta {
				return Err(io::Error::new(io::ErrorKind::Other, format!("command: 'flatpak info --show-location' failed: {}", e))); //dev
			}
			let modified_time = &meta?.modified()?;
			let datetime: DateTime<Local> = (*modified_time).into();
			app.install_date = datetime.format("%a %d %b %Y %I:%M:%S %p %Z").to_string();
		} else {
			app.location = text::SKIPPED.to_string();
			app.install_date = text::SKIPPED.to_string();
		}
		
		Ok(&self.apps[idx])
	}
	
	/// get the location of a (flatpak) app
	pub fn get_location(&mut self, idx: usize) -> io::Result<&FlatpakApp> {
		let location = Command::new("flatpak")
			.args(["info", "--show-location", &self.apps[idx].extid])
			.output()?;
		self.apps[idx].location = String::from_utf8_lossy(&location.stdout)
			.trim_end()
			.into();
		Ok(&self.apps[idx])
	}

	/// get a list of dependencies
	/// return a vector of self.apps indexes
	#[deprecated]
	pub fn get_dependencies(&mut self, idx: usize) -> io::Result<&FlatpakApp> {
		let depends_raw = Command::new("flatpak")
			.args(["info", "--show-runtime", "--show-extensions", &self.apps[idx].extid])
			.output()?;
		let depends = String::from_utf8_lossy(&depends_raw.stdout)
			.trim_end()
			.into();
		if depends == "-" {
			println!("test");
		}
		self.apps[idx].depends = depends;
		Ok(&self.apps[idx])
	}
	
	/// get a list of all files that belong to a (flatpak) app
	/// returns a vector of paths (as string)
	pub fn get_app_files(&mut self, idx: usize) -> io::Result<Vec<String>> {
		let mut out:Vec<String> = Vec::new();
		// fetch app location, if needed
		if self.apps[idx].location.is_empty() {
			let _ = self.get_location(idx);
		}
		out.push(self.apps[idx].location.clone());
		
		// list files/folders recursively
		let mut rec_files = Self::rec_file_explorer(&self.apps[idx].location)?;
		out.append(&mut rec_files);
		
		Ok(out)
	}
	/// list files/folders recursively
	fn rec_file_explorer(path: &str) -> io::Result<Vec<String>> {
		let mut out = Vec::new();
		let directory = fs::read_dir(path)?;
		for entry in directory {
			let entry = entry?;
			let path = entry.path();
			let path_string = path.display().to_string();
			out.push(path_string.clone());
			
			if path.is_dir() {
				let mut rec_out = Self::rec_file_explorer(&path_string)?;
				out.append(&mut rec_out);
			}
		}//for entry
		Ok(out)
	}

	// ====== OTHER FUNCTIONS ======
	
	/// search for flatpaks (including not installed)
	/// returns a vector of results
	pub fn search(self: &FlatpakMeta, input: Vec<&str>) -> io::Result<Vec<FlatpakApp>> {
		let mut args = vec!["search".to_string(), "--columns=name,application,branch,version,remotes,description,application".to_string()];
		args.extend(input.iter().map(|s| s.to_string()));
		let flatpak_search_raw = Command::new("flatpak")
			.args(&args)
			.output()?;
		if !flatpak_search_raw.status.success() {
			return Err(io::Error::new(io::ErrorKind::Other, "command: 'flatpak search' failed")); //dev
		}
		let search_str: String = String::from_utf8_lossy(&flatpak_search_raw.stdout).into();

		let mut results : Vec<FlatpakApp> = vec![];
		if search_str == format!("{}\n", flatpak_strings::SEARCH_NO_RESULTS) {
			// ignore if no results where found
		} else {
			for line in search_str.lines() {
				let columns: Vec<&str> = line.split('\t').collect();
				if columns.len() == 7 {
					let mut app: FlatpakApp = FlatpakApp::default();
					app.name = columns[0].into();
					app.id = columns[1].into();
					app.branch = columns[2].into();
					app.version = columns[3].into();
					app.origin = columns[4].into();
					app.description = columns[5].into();
					
					app.extid = format!("{}/{}/{}", app.id, app.arch, app.branch);
					results.push(app);
				}
				else {
					//dev failed
					eprintln!("error stuff: app has to few columns");
					println!("{}", line);
				}
			}//for line
		}//if search_str


		//println!("{}", search_str);	//dev
		Ok(results)
	}
}

