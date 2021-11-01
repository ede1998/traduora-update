use crate::loader::{Modification, Translation};

use anyhow::Context;
use traduora::api::TermId;
use traduora::{
    api::{
        terms::{CreateTerm, DeleteTerm},
        translations::EditTranslation,
    },
    auth::Authenticated,
    Query, Traduora,
};

fn update(
    term: TermId,
    translation: String,
    client: &Traduora<Authenticated>,
) -> Result<(), (String, anyhow::Error)> {
    use crate::config::*;
    let endpoint = EditTranslation::new(PROJECT_ID.into(), LOCALE.into(), term, translation);

    endpoint
        .query(client)
        .with_context(|| {
            format!(
                "Failed to update term {:?} to translation {:?}.",
                endpoint.term_id, endpoint.value
            )
        })
        .map_err(|e| (endpoint.value, e))?;

    Ok(())
}

fn remove(term: TermId, client: &Traduora<Authenticated>) -> anyhow::Result<()> {
    use crate::config::*;
    let endpoint = DeleteTerm::new(PROJECT_ID.into(), term);
    endpoint
        .query(client)
        .with_context(|| format!("Failed to delete term {:?}.", endpoint.term_id))?;

    Ok(())
}

fn add(
    term: String,
    translation: String,
    client: &Traduora<Authenticated>,
) -> Result<(), (String, String, anyhow::Error)> {
    use crate::config::*;
    let creator = CreateTerm::new(term, PROJECT_ID);
    let term = creator
        .query(client)
        .with_context(|| format!("Failed to create term {:?}.", creator.term))
        .map_err(|e| (creator.term.clone(), translation.clone(), e))?;

    let editor = EditTranslation::new(PROJECT_ID.into(), LOCALE.into(), term.id, translation);

    editor
        .query(client)
        .with_context(|| format!("Failed to set translation {:?} for new term.", editor.value))
        .map_err(|e| (creator.term, editor.value, e))?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    ClientCreation(anyhow::Error),
    Update(Vec<(String, String, anyhow::Error)>),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ClientCreation(e) => write!(f, "Failed to create client: {}", e),
            Error::Update(errs) => {
                writeln!(f, "Failed to create/update/delete {} terms:", errs.len())?;
                for e in errs {
                    writeln!(
                        f,
                        "    Term {:?} with translation {:?}. Reason: {}",
                        e.0, e.1, e.2
                    )?;
                }
                Ok(())
            }
        }
    }
}

pub type UpdateResult = Result<(), Error>;

pub fn run(translations: Vec<Translation>, mut progress: impl FnMut(usize, usize)) -> UpdateResult {
    let client = crate::config::create_client().map_err(Error::ClientCreation)?;
    let total = translations.len();

    let errors: Vec<_> = translations
        .into_iter()
        .enumerate()
        .filter_map(|(count, t)| {
            progress(count + 1, total);
            match t.modification {
                Modification::Removed(term_id) => remove(term_id, &client)
                    .err()
                    .map(|e| (t.term, t.translation, e)),
                Modification::Updated(term_id) => update(term_id, t.translation, &client)
                    .err()
                    .map(|(tl, e)| (t.term, tl, e)),
                Modification::Added => add(t.term, t.translation, &client).err(),
            }
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error::Update(errors))
    }
}
