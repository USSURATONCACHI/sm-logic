use std::io;
use std::path::PathBuf;

use json::{JsonValue, object};
use uuid::Uuid;

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

	pub fn save<S>(&self, name: S, blueprint: JsonValue, overwrite_if_exists: bool) -> io::Result<bool>
		where S: Into<String>
	{
		let name = name.into();

		match self.get_bp_folder(&name) {
			Some(path) => {
				if !overwrite_if_exists {
					return Ok(false);
				}
				let folder_name = path.file_name().unwrap().to_str().unwrap();
				println!("Overwriting '{}'", folder_name);
				self.generate_bp(folder_name.into(), name, blueprint)?;
				Ok(true)
			}

			None => {
				self.generate_bp(Uuid::new_v4().to_string().into(), name, blueprint)?;
				Ok(true)
			}
		}
	}

	fn generate_bp(&self, folder_name: PathBuf, name: String, bp: JsonValue) -> io::Result<()> {
		let blueprint_path = self.folder.join(folder_name.clone()).join("blueprint.json");
		let blueprint = bp.to_string();

		let description_path = self.folder.join(folder_name.clone()).join("description.json");
		let description = object! {
			"description" : "#{STEAM_WORKSHOP_NO_DESCRIPTION}",
		   "localId" : folder_name.to_str().unwrap(),
		   "name" : name,
		   "type" : "Blueprint",
		   "version" : 0
		}.to_string();

		if !self.folder.join(folder_name.clone()).exists() {
			std::fs::create_dir(self.folder.join(self.folder.join(folder_name.clone())))?;
		}

		std::fs::write(blueprint_path, blueprint)?;
		std::fs::write(description_path, description)?;
		Ok(())
	}

	pub fn get_bp_folder<S>(&self, name: S) -> Option<PathBuf>
		where S: Into<String>
	{
		let bp_name = name.into();
		let all_dirs = std::fs::read_dir(self.folder()).unwrap();

		for dir in all_dirs {
			let dir = match dir {
				Ok(entry) => entry,
				Err(_) => {
					continue;
				},
			};

			if !dir.path().is_dir() {
				continue;
			}

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
			let mut description = if description.is_err() {
				continue;
			} else {
				description.unwrap()
			};

			if description.has_key("name") &&
				description["name"].is_string() {
				let check_name = description["name"].take_string().unwrap();

				if check_name.eq(&bp_name) {
					return Some(dir.path());
				}
			}
		}

		None
	}

	pub fn set_description<S1, S2>(&self, name: S1, description: S2) -> Result<(), String>
		where S1: Into<String>, S2: Into<String>
	{
		let name = name.into();
		let description = description.into();

		match self.get_bp_folder(&name) {
			Some(folder) => {
				let descr_path = folder.join("description.json");
				let description = object! {
					"description" : description,
				   	"localId" : folder.file_name().unwrap().to_str().unwrap(),
				   	"name" : name,
				   	"type" : "Blueprint",
				   	"version" : 0
				}.to_string();

				std::fs::write(descr_path, description).unwrap();

				Ok(())
			}

			None => Err(format!("Blueprint '{}' does not exists", name))
		}
	}
}