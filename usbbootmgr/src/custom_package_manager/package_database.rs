use std::io::{ErrorKind, Read};
use std::{env, borrow::Borrow};
use std::ffi::OsString;
use std::fs::File;

use serde::de::Unexpected;
use serde::ser::SerializeSeq;
use serde::{Serialize, Deserialize, ser::SerializeStruct, de::{Visitor, Error}};

pub struct DatabaseFileDetails<'a> {
    /// Environment variable containing xdg dir that package database is located in.
    pub xdg_dir_env_var: &'a str,
    /// Environment variable containing location of user's home dir.
    pub home_dir_env_var: &'a str,
    /// The path to the xdg dir to be used in case xdg_dir_env_var does not exist,
    /// relative to the user's home dir.
    pub default_xdg_dir: &'a str,
    /// The path to the package database relative to the xdg dir.
    pub package_database_file: &'a str,
}

fn get_database_filename(file_details: DatabaseFileDetails) -> Result<OsString, ()> {
    let mut config_dir = match env::var_os(file_details.xdg_dir_env_var) {
        Some(x) => x,
        None => {
            let mut home_dir = env::var_os(file_details.home_dir_env_var).ok_or(())?;
            home_dir.push("/");
            home_dir.push(file_details.default_xdg_dir);
            home_dir
        },
    };
    config_dir.push("/");
    config_dir.push(file_details.package_database_file);
    Ok(config_dir)
}

pub struct DatabaseDetails<'a> {
    file_details: DatabaseFileDetails<'a>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Package {
    name: String,
    git_upstream_url: String,
}

struct DatabaseContents {
    packages: Vec<Package>,
}
impl Serialize for DatabaseContents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut root = serializer.serialize_struct("DatabaseContents", 1)?;
        root.serialize_field("packages", &SerializePackages(&self.packages))?;
        root.end()
    }
}
impl<'de> Deserialize<'de> for DatabaseContents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        struct DatabaseContentsVisitor;
        impl<'a> Visitor<'a> for DatabaseContentsVisitor {
            type Value = DatabaseContents;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "the contents of a package database")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'a>, {
                let mut packages: Option<DeserializePackages> = None;

                while let Some(key) = map.next_key()? {
                    let key: &str = key;

                    match key {
                        "packages" => {
                            if packages.is_some() {
                                return Err(A::Error::duplicate_field("packages"));
                            }
                            packages = Some(map.next_value()?);
                        },
                        _ => return Err(A::Error::unknown_field(key, FIELDS)),
                    }
                }
                let packages = packages.ok_or_else(|| A::Error::missing_field("packages"))?;
                Ok(DatabaseContents {
                    packages: packages.0,
                })
            }
        }

        const FIELDS: &[&str] = &["packages"];

        deserializer.deserialize_struct("DatabaseContents", FIELDS, DatabaseContentsVisitor)
    }
}

struct SerializePackages<'a>(&'a Vec<Package>);
impl Serialize for SerializePackages<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut elements = serializer.serialize_seq(Some(self.0.len()))?;
        for package in self.0 {
            elements.serialize_element(package)?;
        }
        elements.end()
    }
}
struct DeserializePackages(Vec<Package>);
impl<'de> Deserialize<'de> for DeserializePackages {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        struct DeserializePackagesVisitor;
        impl<'a> Visitor<'a> for DeserializePackagesVisitor {
            type Value = DeserializePackages;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a list of packages")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'a>, {
                let mut packages = match seq.size_hint() {
                    Some(hint) => Vec::with_capacity(hint),
                    None => Vec::new(),
                };
                while let Some(package) = seq.next_element()? {
                    packages.push(package);
                }
                Ok(DeserializePackages(packages))
            }
        }

        deserializer.deserialize_seq(DeserializePackagesVisitor)
    }
}

pub enum DatabaseError {
    /// There is another package with the same name as the package
    /// that is being added.
    PackageAlreadyExists,
}

/// This struct is meant to be dependent on a database file.
/// It is not meant to exist independently from a database file.
pub struct Database {
    contents: DatabaseContents,
    database_file_handle: File,
}
impl Database {
    pub fn new(details: DatabaseDetails) -> Result<Database, ()> {
        let database_filepath = get_database_filename(details.file_details)?;

        let mut create_new_options = File::options();
        create_new_options
            .read(true)
            .truncate(false)
            .write(true)
            .append(false)
            .create_new(true);

        let mut existing_file_options = create_new_options.clone();
        existing_file_options
            .create_new(false)
            .create(false);

        match create_new_options.open(&database_filepath) {
            Ok(file_handle) => {
                let contents = DatabaseContents {
                    packages: Vec::new(),
                };
                Ok(Database {
                    contents,
                    database_file_handle: file_handle,
                })
            },
            Err(err) => {
                match err.kind() {
                    ErrorKind::AlreadyExists => {
                        match existing_file_options.open(&database_filepath) {
                            Ok(mut file_handle) => {
                                let mut file_contents = String::new();
                                file_handle.read_to_string(&mut file_contents)
                                    .map_err(|_| ())?;
                                let contents = toml::from_str(&file_contents)
                                    .map_err(|_| ())?;
                                Ok(Database {
                                    contents,
                                    database_file_handle: file_handle,
                                })
                            },
                            Err(_) => Err(()),
                        }
                    },
                    _ => Err(()),
                }
            },
        }
    }
    /// This function may be a costly operation. So try to call it after
    /// all mutation is done.
    pub fn save() {
    }

    pub fn add_entry(&mut self, package: Package) -> Result<(), DatabaseError> {
        let package_already_exists = self.contents.packages.iter()
            .find(|x| x.name == package.name);
        if package_already_exists.is_some() {
            return Err(DatabaseError::PackageAlreadyExists);
        }
        self.contents.packages.push(package);
        Ok(())
    }
    pub fn remove_entry() {
    }
}
