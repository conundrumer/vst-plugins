use std::env;
use std::fs::File;
use std::error::Error;
use std::collections::BTreeMap;
use std::io::{BufReader, Read};
use std::str::FromStr;
use serde_json;

#[derive(Copy, Clone, Deserialize, Debug)]
pub enum EntryType { Mono, Poly, Drum, Param }

#[derive(Deserialize, Debug)]
pub struct JsonEntry<T: Ord> {
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub name: String,
    pub address: String,
    #[serde(default)]
    pub params: Vec<String>,
    pub keys: Option<BTreeMap<T, String>>
}

pub type Entry = JsonEntry<u8>;

pub fn load() -> Result<Vec<Entry>, Box<Error>> {
    let mut path = env::current_dir()?;

    path.push("oscify-config.json");

    let f = File::open(&path)?;
    let mut file = BufReader::new(&f);

    let mut s = String::new();

    file.read_to_string(&mut s)?;

    let config: Vec<JsonEntry<String>> = serde_json::from_str(&s)?;
    let config: Result<Vec<_>, _> = config.into_iter().map(try_from).collect();
    let config = config?;

    Ok(config)
}

fn try_from(entry: JsonEntry<String>) -> Result<Entry, Box<Error>> {
    let mut next_entry = JsonEntry {
        entry_type: entry.entry_type,
        name: entry.name,
        address: entry.address,
        params: entry.params,
        keys: None
    };
    if let Some(map) = entry.keys {
        let mut next_map = BTreeMap::new();
        for (key, value) in map {
            let key = u8::from_str(&key)?;
            next_map.insert(key, value);
        }
        next_entry.keys = Some(next_map);
    }
    Ok(next_entry)
}
