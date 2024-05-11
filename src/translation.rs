use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};
use serde_yaml::Value;

pub struct TranslationPlugin;

impl Plugin for TranslationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TranslationStorage::default())
            .insert_resource(SelectedLanguage("zh_cn".to_string()))
            .add_systems(Startup, load_translations);
    }
}

fn load_translations(mut storage: ResMut<TranslationStorage>) {
    let file = std::fs::File::open("lang/meta/languages.json")
        .expect(&format!("Failed to load translations"));
    let languages: Vec<String> =
        serde_json::from_reader(file).expect(&format!("Failed to load translations"));

    for language in languages {
        let mapping = load_translation(&language);
        storage.0.insert(language, mapping);
    }
}

fn flatten(prefix: Option<&str>, value: &Value, result: &mut HashMap<String, String>) {
    match value {
        Value::Mapping(map) => {
            for (k, v) in map {
                let key = match prefix {
                    Some(prefix) => {
                        let k = k.as_str().unwrap();
                        if k == "." {
                            prefix.to_string()
                        } else {
                            format!("{}.{}", prefix, k)
                        }
                    },
                    None => k.as_str().unwrap().to_string(),
                };
                flatten(Some(&key), v, result);
            }
        }
        Value::String(s) => {
            if let Some(prefix) = prefix {
                result.insert(prefix.to_string(), s.to_string());
            }
        }
        _ => {}
    }
}

fn load_translation(lang: &String) -> HashMap<String, String> {
    let file = std::fs::File::open(format!("lang/{}.yml", lang))
        .expect(&format!("Failed to load translation: {}", lang));
    let value: Value =
        serde_yaml::from_reader(file).expect(&format!("Failed to load translation: {}", lang));

    let mut mapping: HashMap<String, String> = HashMap::new();
    flatten(None, &value, &mut mapping);
    mapping
}

#[derive(Resource, Debug, Default)]
struct TranslationStorage(HashMap<String, HashMap<String, String>>);

impl TranslationStorage {
    pub fn translate(&self, lang: &str, key: &str) -> Option<&String> {
        self.0
            .get(lang)
            .and_then(|translation| translation.get(key))
    }
}

// TODO: EditorSetting
#[derive(Resource, Debug, Default)]
struct SelectedLanguage(String);

#[derive(SystemParam)]
pub struct Translator<'w> {
    translations: Res<'w, TranslationStorage>,
    language: Res<'w, SelectedLanguage>,
}

impl Translator<'_> {
    pub fn tr(&self, key: &str) -> String {
        match self.translations.translate(&self.language.0, key) {
            Some(result) => result.to_string(),
            None => key.to_string(),
        }
    }
}
