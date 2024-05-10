use std::collections::HashMap;
use lazy_static::lazy_static;
use toml;
use serde_json::{Value, Map};

lazy_static!(
    static ref TRANSLATIONS: HashMap<String, String> = {
        let translations: HashMap<String, String> = toml::from_str(
            include_str!("translations.toml")
        ).unwrap();
        translations
    };
);

// Function to translate keys
pub fn translate_keys(obj: &mut Value) {
    let translations = &*TRANSLATIONS;

    match obj {
        Value::Object(map) => {
            let mut new_map = Map::new();
            for (key, value) in map.iter() {
                let new_key = translations.get(key).unwrap_or(key).clone();
                new_map.insert(new_key, value.clone());
            }
            *map = new_map;
        }
        Value::Array(vec) => {
            for value in vec {
                translate_keys(value);
            }
        }
        _ => {}
    }
}
