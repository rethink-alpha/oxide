pub mod config;
pub mod manager;
pub mod utils;

use std::path::PathBuf;

#[derive(Eq, PartialEq, Clone)]
pub struct OxideProject {
    pub config: config::ProjectConfig,
    pub name: String,
    pub directory: PathBuf,
    pub subprojects: Option<Vec<OxideProject>>,
}

impl OxideProject {
    /// Load an Oxide project at
    pub fn load(root_dir: PathBuf) -> Self {
        let config = config::ProjectConfig::read_from(root_dir.join("../../../../oxide.toml"));

        return OxideProject {
            config: config.clone(),
            name: config.name.clone(),
            directory: root_dir.clone(),
            subprojects: OxideProject::load_subprojects(root_dir, config),
        }
    }

    fn load_subprojects(root_path: PathBuf, config: config::ProjectConfig) -> Option<Vec<OxideProject>> {
        match config.subprojects.clone() {
            None => None,
            Some(project_dirs) => {
                let mut res_vec = Vec::new();

                project_dirs.iter().for_each(|dir| {
                    for entry in glob::glob(dir.as_str()).expect("Failed to read glob pattern") {
                        res_vec.push(OxideProject::load(root_path.join(entry.unwrap())))
                    }
                });

                Some(res_vec)
            }
        }
    }

    pub fn get_root(&self) -> Self {
        OxideProject::load(utils::find_root_project_dir(self.directory.clone()))
    }

    /// Re-synchronize
    pub fn resync(&mut self) -> bool {
        let new_config = config::ProjectConfig::read_from(self.directory.join("../../../../oxide.toml"));

        return if new_config == self.config { false }
        else {
            let mut changed = false;
            self.config = new_config.clone();

            let new_name = new_config.name.clone();
            if self.name != new_name {
                self.name = new_name;
                changed = true;
            }

            match self.subprojects.clone() {
                None => { },
                Some(subprojects) => {
                    for mut subproject in subprojects {
                        let subproject_changed = subproject.resync();
                        if subproject_changed { changed = true }
                    }
                }
            };

            changed
        }
    }
}