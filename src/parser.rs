//! # Examples
//!
//! ```
//! extern crate parser;
//! use parser::{
//!     Version, TaskConfiguration, BaseTaskConfiguration,
//!     CommandOptions, Group, ProblemMatcher, PresentationOptions,
//!     GroupKind, TaskType, parse};
//! # extern crate tempfile;
//! use std::io::{self, Write};
//! use std::fs::File;
//! # fn main() {
//! #     if let Err(_) = run() {
//! #         ::std::process::exit(1);
//! #     }
//! # }
//! # fn run() -> Result<(), io::Error> {
//! # let dir = tempfile::tempdir()?;
//! # let file_path = dir.path().join("tasks.json");
//! let mut file = File::create(&file_path)?;
//! let data = r#"
//! {
//!     "version": "2.0.0",
//!     "tasks": [
//!         {
//!             "label": "Emit Greeting",
//!             "group": "build",
//!             "type": "process",
//!             "command": "/bin/echo",
//!             "isBackground": false,
//!             "options": {
//!                 "cwd": "/tmp"
//!             },
//!             "args": ["Hello,", "world!"],
//!             "presentation": {},
//!             "problemMatcher": {}
//!         }
//!     ]
//! }"#;
//! writeln!(file, "{}", data);
//! assert_eq!(
//!     TaskConfiguration {
//!         version: Version(String::from("2.0.0")),
//!         tasks: vec![
//!             BaseTaskConfiguration {
//!                 label: String::from("Emit Greeting"),
//!                 command: String::from("/bin/echo"),
//!                 group: Group {
//!                     kind: GroupKind::Build,
//!                     is_default: false,
//!                 },
//!                 task_type: TaskType::Process,
//!                 is_background: false,
//!                 args: vec![String::from("Hello,"), String::from("world!")],
//!                 options: CommandOptions {
//!                     cwd: String::from("/tmp"),
//!                 },
//!                 presentation: PresentationOptions {
//!                 },
//!                 problem_matcher: ProblemMatcher {
//!                 },
//!             },
//!         ],
//!     },
//!     parse(file_path.as_path()).unwrap()
//! );
//! # Ok(())
//! # }
//! ```

use std::path;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use std::fs::File;
use std::io::BufReader;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};

extern crate void;
use void::Void;

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor impl
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, visitor: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TaskConfiguration {
    #[serde(default)]
    pub version: Version,
    pub tasks: Vec<BaseTaskConfiguration>,
}
impl Default for TaskConfiguration {
    fn default() -> Self {
        TaskConfiguration {
            version: Version::default(),
            tasks: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Version(pub String);
impl Default for Version {
    fn default() -> Self {
        Version("2.0.0".to_string())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct BaseTaskConfiguration {
    #[serde(rename = "type")]
    pub task_type: TaskType,
    #[serde(deserialize_with = "string_or_struct")]
    pub group: Group,
    pub label: String,
    pub command: String,
    pub is_background: bool,
    pub options: CommandOptions,
    pub args: Vec<String>,
    pub presentation: PresentationOptions,
    pub problem_matcher: ProblemMatcher,
}
impl Default for BaseTaskConfiguration {
    fn default() -> Self {
        BaseTaskConfiguration {
            task_type: TaskType::Shell,
            group: Group {
                kind: GroupKind::Build,
                is_default: false,
            },
            label: "".to_string(),
            command: "".to_string(),
            is_background: false,
            options: CommandOptions::default(),
            args: Vec::new(),
            presentation: PresentationOptions {},
            problem_matcher: ProblemMatcher {},
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub kind: GroupKind,
    pub is_default: bool,
}
impl FromStr for Group {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Group {
            kind: match s {
                "build" => GroupKind::Build,
                "test" => GroupKind::Test,
                _ => GroupKind::Build,
            },
            is_default: false,
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GroupKind {
    Build,
    Test,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Shell,
    Process,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(default)]
pub struct CommandOptions {
    pub cwd: String,
}
impl Default for CommandOptions {
    fn default() -> Self {
        CommandOptions {
            cwd: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PresentationOptions {}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ProblemMatcher {}

pub fn parse(config: &path::Path) -> Result<TaskConfiguration, serde_json::Error> {
    let file = File::open(&config).unwrap();
    let reader = BufReader::new(file);

    let tc: TaskConfiguration = serde_json::from_reader(reader)?;

    Ok(tc)
}

