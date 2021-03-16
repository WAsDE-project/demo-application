#[macro_use]
extern crate log;

use anyhow::{anyhow, Result};
use regex::Regex;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

pub struct Manager {
    metadata: HashMap<String, Metadata>,
    attributes: Vec<String>,
    cache: Option<String>,
    modules: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct Metadata {
    id: String,
    attributes: Vec<String>,
    location: String,
    dependencies: HashMap<String, Dependency>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Dependency {
    pub id: String,
    #[serde(alias = "loadTime")]
    load_time: bool,
    location: String,
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.load_time == other.load_time && self.location == other.location
    }
}

impl Metadata {
    pub(crate) fn new(json: String) -> Result<Self> {
        let metadata: Metadata = serde_json::from_str(&json)?;

        Ok(metadata)
    }
}

impl Manager {
    pub fn new(metafile: &str, attributes: Vec<String>, cache: &Option<String>) -> Result<Self> {
        let metadata = Metadata::new(metafile.to_string())?;
        let modules = Manager::check_cached_modules(&cache.as_ref().unwrap()).unwrap_or_else(|_| vec![]);
        let mut metadata_map = HashMap::new();
        metadata_map.insert(metadata.id.clone(), metadata.clone());

        Ok(Self { metadata: metadata_map, attributes, cache: Some(cache.as_ref().unwrap().clone()), modules })
    }

    pub fn resolve_id(&mut self, caller_module: &str, id: &str) -> Result<Dependency> {
        let module = self.metadata.get(caller_module).ok_or(anyhow!(
            "Cannot find the desired module: `{}` defined in the metadata",
            caller_module
        ))?;

        let dependency = module.dependencies.get(id).ok_or(anyhow!(
            "Cannot find the desired module's dependency `{}` defined in the metadata of module {}",
            id, caller_module
        ))?;

        return Ok(dependency.clone());
    }

    fn load_metadata(&mut self, name: &str, location: &str) -> Result<Vec<u8>> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/{}", location, name);
        info!("loading metadata from {}", url);
        let attributes: Vec<&str> = self.attributes.iter().map(|a| a.as_str()).collect();
        let attributes: HashMap<&str, Vec<&str>> = [("attributes", attributes)].iter().cloned().collect();
        let file = client.post(&url).json(&attributes).send()?.bytes()?.to_vec();

        Ok(file)
    }

    pub fn get_attributes(&self) -> Vec<String> {
        self.attributes.clone()
    }

    fn analyse_module(
        &mut self,
        module: &Metadata,
        dependencies: &mut Vec<Dependency>,
    ) -> Result<()> {
        let deps: Vec<Dependency> = module
            .dependencies
            .iter()
            .filter(|d| d.1.load_time && !dependencies.contains(&d.1))
            .map(|dep| dep.1.clone())
            .collect();

        dependencies.extend(deps.clone());

        for d in deps {
            let dependency = self.load_metadata(&d.id, &d.location)?;
            let metadata: Metadata = serde_json::from_slice(&dependency)?;

            self.metadata
                .insert(d.id.clone(), metadata.clone());
            self.analyse_module(&metadata, dependencies)?;
        }

        Ok(())
    }

    fn gather_dependency_information(
        &mut self,
        caller_module: &str,
        location: &str,
        choices: &mut Vec<Dependency>,
    ) -> Result<()> {
        let module = self.metadata.get(caller_module).cloned();

        if module.is_none() {
            let metadata = self.load_metadata(caller_module, location)?;
            // println!("{}", String::from_utf8(metadata.clone()).unwrap());
            let metadata: Metadata = serde_json::from_slice(&metadata)?;
            self.metadata.insert(caller_module.to_string(), metadata.clone());
            self.analyse_module(&metadata, choices)?;

            return Ok(());
        };

        self.analyse_module(&module.unwrap(), choices)?;

        Ok(())
    }

    pub fn load_main(&mut self, name: &str) -> Result<Vec<u8>> {
        self.retrieve_binary(name)
    }

    fn retrieve_binary(&mut self, choice: &str) -> Result<Vec<u8>> {
        if self.modules.iter().any(|m| m == choice) {
            let binary = self.load_from_cache(&choice)?;
            Ok(binary)
        } else {
            let binary = self.load_from_registry(&choice)?;
            Ok(binary)
        }
    }

    fn load_all_dependencies(
        &mut self,
        dependencies: Vec<Dependency>,
    ) -> Result<Vec<(Dependency, Vec<u8>)>> {
        let mut dependency_tuples = vec![];

        for d in dependencies {
            dependency_tuples.push((d.clone(), self.retrieve_binary(&d.id)?));
        }

        Ok(dependency_tuples)
    }

    pub fn load(
        &mut self,
        standalone: Dependency
    ) -> Result<Vec<(Dependency, Vec<u8>)>> {
        let mut choices = vec![];
        // The main dependency that needs to be inserted to the very end.
        choices.push(standalone.clone());
        self.gather_dependency_information(&standalone.id, &standalone.location, &mut choices)?;
        let all_dependencies = self.load_all_dependencies(choices)?;

        Ok(all_dependencies)
    }

    fn load_from_cache(&self, name: &str) -> Result<Vec<u8>> {
        let filename = format!("{}/{}.wasm", self.cache.as_ref().unwrap(), name);
        let file = fs::read(filename)?;

        return Ok(file);
    }

    fn load_from_registry(&mut self, name: &str) -> Result<Vec<u8>> {
        let module = self
            .metadata
            .get(name)
            .ok_or(anyhow!(
                "Cannot find the desired module: `{}` defined in the metadata",
                name
            ))?
            .clone();
        info!("Fetching binary from {}", &module.location);
        let mut binary = reqwest::blocking::get(&module.location)?;
        let mut buffer: Vec<u8> = vec![];

        if !binary.status().is_success() {
            // TODO: This should be more specific.
            return Err(anyhow!(format!("Problem occurred while downloading the module {} from {}", name, module.location)));
        }

        binary.copy_to(&mut buffer)?;
        buffer.as_slice().to_vec();
        self.cache_module(&name, buffer.clone())?;

        Ok(buffer)
    }

    fn cache_module(&mut self, module: &str, binary: Vec<u8>) -> Result<()> {
        let cache_path = Path::new(&self.cache.as_ref().unwrap().clone()).to_owned();
        let filename = format!("{}.wasm", module);
        let full_pathname = format!("{}/{}", cache_path.display(), filename);
        let fullpath = Path::new(&full_pathname);

        if !cache_path.clone().exists() {
            fs::create_dir_all(cache_path.clone())?;
        }

        if cache_path.exists() && !fullpath.exists() {
            let mut file = fs::File::create(fullpath)?;
            file.write_all(binary.as_slice())?;

            self.modules.push(module.to_string());

            return Ok(());
        }

        Ok(())
    }

    fn check_cached_modules(cache: &str) -> Result<Vec<String>> {
        let pattern = Regex::new(r"^(?P<name>.+?).wasm")?;

        match fs::read_dir(cache) {
            Ok(modules) => {
                let modules = modules
                    .map(|module| -> Result<String> {
                        // TODO: This should be revised
                        // There could be problems with OsString and DirEntry.
                        let name = module?.file_name();
                        let name = name.to_str().unwrap();
                        let captures = pattern.captures(name).unwrap();
                        let data = captures.name("name").unwrap().as_str();

                        Ok(data.to_string())
                    })
                    .filter_map(Result::ok)
                    .collect::<Vec<_>>();

                Ok(modules)
            }
            Err(e) => {
                // QUESTION: Should we just create a cache directory here instead of throwing an error?
                Err(anyhow!("No existing cache detected. {}", e))
            }
        }
    }
}

#[test]
fn test_loading() -> Result<()> {
    let json = r#"
        {
            "cache": "./cache", 
            "modules": {
                "example_main": {
                    "location": "http://localhost:8080",
                    "dependencies": {
                        "marvin@0.0.1": {
                            "loadTime": true,
                            "id": "marvin@0.0.1",
                            "location": "http://localhost:8080/metadata_splitting"
                        }
                    }
                }
            }
        }
    "#;
    let mut manager = Manager::new(json, &Some("./cache".to_string()))?;
    manager.load("example_main".to_string(), "marvin@0.0.1".to_string())?;

    Ok(())
}
