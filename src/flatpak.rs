//! fetch data from flatpak (over its cli)
// flatpak.rs

use std::process::Command;
use std::io;

/// flatpak app metadata
#[derive(Debug, Default)]
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
#[derive(Default)]
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
	
	/// fetch a basic list off all apps from the flatpak cli
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
	
	/// get more detailed infos about a (flatpak) app
	pub fn get_app_info(&mut self, idx: usize) -> io::Result<&FlatpakApp> {	//dev: include usizeArr?
		if self.list_full.is_empty() {
			let flatpak_list_raw = Command::new("flatpak")
				.args(["list", "--columns=name,application,arch,branch,version,application"])
				.output()?;
			if !flatpak_list_raw.status.success() {
				return Err(io::Error::new(io::ErrorKind::Other, "command: 'flatpak list' failed")); //dev
			}
			self.list_full = String::from_utf8_lossy(&flatpak_list_raw.stdout).into();
		}
		
		// dev
		let searchterms = [&self.apps[idx].id, &self.apps[idx].arch, &self.apps[idx].branch];
		if let Some(matching) = self.list_full
			.lines()
			.find(|line| searchterms.iter().all(|k| line.contains(k.as_str())))
		{
			let columns: Vec<&str> = matching.split('\t').collect();
			if columns.len() >= 4 {
				self.apps[idx].name = columns[0].into();
				self.apps[idx].version = columns[4].into();
				//dev: test version
                if self.apps[idx].version.is_empty() {
                    self.apps[idx].version = "?".to_string();
                }
			} else {
				//dev failed
                println!("error stuff: app has to few columns");
			}
		} else {
			//dev failed
			println!("error stuff: app not found in str");
		}
		//println!("{:?}", matching);

		//dev: write the fields into the correct self.apps spots 
	

		//for app in &mut apps {
		//	app.extid = format!("{}/{}/{}", app.id, app.arch, app.branch);
		//}
		//self.apps = apps;
		
		Ok(&self.apps[idx])
	}
}

