use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{de::Visitor, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    let data = fs::read(&path)
        .with_context(|| format!("Failed to open file {}", path.as_ref().display()))?;

    parse(&data).with_context(|| format!("Failed to load file {}", path.as_ref().display()))
}

pub fn load_from_git<P>(revision: &str, path: P) -> Result<Vec<Translation>>
where
    P: AsRef<Path>,
{
    use git2::Repository;

    let fun = || -> Result<Vec<Translation>> {
        let repo = Repository::discover(&path).context("Failed to discover git repository.")?;

        let revision = repo
            .revparse_single(revision)
            .with_context(|| format!("Failed to find revision {:?}.", revision))?;

        let blob = revision
            .peel_to_tree()?
            .get_path(path.as_ref())?
            .to_object(&repo)?
            .peel_to_blob()?;
        parse(blob.content())
    };

    fun().with_context(|| {
        format!(
            "Failed to extract file for path {:?} of git revision {:?}.",
            path.as_ref().display(),
            revision
        )
    })
}

fn parse(data: &[u8]) -> Result<Vec<Translation>> {
    use json_comments::StripComments;
    let enc = guess_encoding(data);
    let (data, encountered_malformeds) = enc.decode_with_bom_removal(data);

    if encountered_malformeds {
        log::warn!("Replaced some malformed characters in translation file.");
    }

    let data = StripComments::new(data.as_bytes());

    let result: DeserializationHelper =
        serde_json::from_reader(data).context("Failed to parse translation file")?;
    Ok(result.0)
}

fn guess_encoding(data: &[u8]) -> &'static encoding_rs::Encoding {
    use encoding_rs::Encoding;
    crate::config::get()
        .encoding()
        .unwrap_or_else(|| Encoding::for_bom(data).map_or(encoding_rs::UTF_8, |x| x.0))
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
    use super::*;

    #[test]
    fn read_from_file() {
        crate::config::init_test();
        let res = load_from_file("testdata/en.json").unwrap();
        println!("{:#?}", res);
        assert_eq!(res.len(), 20);
        assert_eq!(
            res[0].term,
            "globe.championship.congregation.burden.probable"
        );
        assert_eq!(res[0].translation, "colonial congregation sustain");
    }

    #[test]
    fn read_from_git_branch_tag_commit() {
        crate::config::init_test();
        let branch = load_from_git("foo", "testdata/en.json").unwrap();
        let tag = load_from_git("blabla", "testdata/en.json").unwrap();
        let commit = load_from_git("01452d761e", "testdata/en.json").unwrap();
        assert_eq!(branch, tag);
        assert_eq!(branch, commit);
    }

    #[test]
    fn decode_parse_encodings() {
        crate::config::init_test();
        let utf8 = include_bytes!("../../testdata/en-utf8.json");
        let utf8bom = include_bytes!("../../testdata/en-utf8-bom.json");
        let utf16be = include_bytes!("../../testdata/en-utf16be.json");
        let utf16le = include_bytes!("../../testdata/en-utf16le.json");

        let utf8bom = parse(utf8bom).unwrap();
        let utf16be = parse(utf16be).unwrap();
        let utf16le = parse(utf16le).unwrap();
        let utf8 = parse(utf8).unwrap();

        assert_eq!(utf8, utf8bom);
        assert_eq!(utf16be, utf16le);
        assert_eq!(utf16be, utf8);
    }
}
