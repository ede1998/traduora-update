use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{de::Visitor, Deserialize};

#[derive(Debug, Clone)]
pub struct Translation {
    pub term: String,
    pub translation: String,
}

impl Translation {
    pub fn new(term: String, translation: String) -> Self {
        Self { term, translation }
    }

    pub fn cmp_by_term(&self, other: &Self) -> std::cmp::Ordering {
        self.term.cmp(&other.term)
    }
}

pub fn load_from_file<P>(path: P) -> Result<Vec<Translation>>
where
    P: AsRef<Path>,
{
    let data = fs::read_to_string(&path)
        .with_context(|| format!("Failed to open file {}", path.as_ref().display()))?;

    Ok(serde_json::from_str::<DeserializationHelper>(&data)
        .with_context(|| {
            format!(
                "Failed to deserialize terms and translations from file {}.",
                path.as_ref().display()
            )
        })?
        .0)
}

struct DeserializationHelper(Vec<Translation>);

impl<'de> Deserialize<'de> for DeserializationHelper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TranslationVisitor;

        impl<'de> Visitor<'de> for TranslationVisitor {
            type Value = DeserializationHelper;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map of terms and translations")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut data = Vec::with_capacity(map.size_hint().unwrap_or(0));

                while let Some((term, translation)) = map.next_entry()? {
                    data.push(Translation::new(term, translation));
                }

                Ok(DeserializationHelper(data))
            }
        }

        deserializer.deserialize_map(TranslationVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::load_from_file;

    #[test]
    fn read_from_file() {
        let res = load_from_file("testdata/en.json").unwrap();
        println!("{:#?}", res);
        assert_eq!(res.len(), 20);
        assert_eq!(
            res[0].term,
            "globe.championship.congregation.burden.probable"
        );
        assert_eq!(res[0].translation, "colonial congregation sustain");
    }
}
