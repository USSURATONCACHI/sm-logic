use std::io;
use std::path::{Path, PathBuf};
use json::JsonValue;

type Error = ();

// C:\Users\redch\AppData\Roaming\Axolot Games\Scrap Mechanic\User\User_76561198288016737\Blueprints
/// Blueprint manager
pub struct BPManager {
	folder: PathBuf,
}

impl BPManager {
	pub fn from_folder<P>(blueprints_folder: P) -> Result<BPManager, String>
		where P: Into<PathBuf>,
	{
		let folder = blueprints_folder.into();

		if folder.exists() && folder.is_dir() {
			Ok(BPManager { folder })
		} else {
			Err(format!("Folder '{}' does not exists", folder.display()))
		}
	}

	pub fn folder(&self) -> &PathBuf {
		&self.folder
	}

	pub fn save<S>(&self, name: S, blueprint: JsonValue, overwrite_if_exists: bool) -> Result<(), String>
		where S: Into<String>
	{
		let name = name.into();

		match self.get_bp_folder(&name) {
			Some(path) => {
				if !overwrite_if_exists {
					return Err(format!("Blueprint '{}' already exists, overwriting is proibited", name));
				}
			}

			None => {

			}
		}

		;Ok(())
	}

	pub fn get_bp_folder<S>(&self, name: S) -> Option<PathBuf>
		where S: Into<String>
	{
		let bp_name = name.into();
		let all_dirs = std::fs::read_dir(self.folder()).unwrap();

		for dir in all_dirs {
			let dir = match dir {
				Ok(entry) => entry,
				Err(_) => continue,
			};

			if !dir.path().is_dir() {
				continue;
			}

			//let blueprint_file = dir.path().join("blueprint.json");
			let descr_file = dir.path().join("description.json");

			if !descr_file.is_file() || !descr_file.exists() {
				continue;
			}

			let description = std::fs::read_to_string(descr_file);
			let description = if description.is_err() {
				continue;
			} else {
				description.unwrap()
			};

			let description = json::parse(&description);
			let description = if description.is_err() {
				continue;
			} else {
				description.unwrap()
			};

			if description.contains("name") &&
				description["name"].eq(&bp_name) {
				return Some(dir.path());
			}
		}

		None
	}

	pub fn set_description<S1, S2>(&self, name: S1, description: S2) -> Result<(), Error>
		where S1: Into<String>, S2: Into<String>
	{
		todo!()
	}
}