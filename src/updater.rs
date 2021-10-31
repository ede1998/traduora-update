use crate::layout::{Added, AppState, ModificationEntry, Removed, Updated};

use anyhow::{Context, Result};
use traduora::{
    api::{
        terms::{CreateTerm, DeleteTerm},
        translations::EditTranslation,
    },
    auth::Authenticated,
    Query, Traduora,
};

fn update(entry: &ModificationEntry<Updated>, client: &Traduora<Authenticated>) -> Result<()> {
    use crate::config::*;
    let endpoint = EditTranslation::new(
        PROJECT_ID.into(),
        LOCALE.into(),
        entry.modification.0.clone(),
        entry.translation.clone(),
    );

    endpoint.query(client).with_context(|| {
        format!(
            "Failed to update term {:?} to translation {:?}.",
            entry.term, entry.translation
        )
    })?;

    Ok(())
}

fn remove(entry: &ModificationEntry<Removed>, client: &Traduora<Authenticated>) -> Result<()> {
    use crate::config::*;
    let endpoint = DeleteTerm::new(PROJECT_ID.into(), entry.modification.0.clone());
    endpoint
        .query(client)
        .with_context(|| format!("Failed to delete term {:?}.", entry.term))?;

    Ok(())
}

fn add(entry: &ModificationEntry<Added>, client: &Traduora<Authenticated>) -> Result<()> {
    use crate::config::*;
    let endpoint = CreateTerm::new(entry.term.clone(), PROJECT_ID);
    let term = endpoint
        .query(client)
        .with_context(|| format!("Failed to create term {:?}.", entry.term))?;

    let endpoint = EditTranslation::new(
        PROJECT_ID.into(),
        LOCALE.into(),
        term.id,
        entry.translation.clone(),
    );

    endpoint.query(client).with_context(|| {
        format!(
            "Failed to set translation {:?} for new term.",
            entry.translation
        )
    })?;

    Ok(())
}

#[derive(Debug)]
pub enum Error<'a> {
    ClientCreation(anyhow::Error),
    Update(Vec<(&'a str, &'a str, anyhow::Error)>),
}

impl<'a> std::error::Error for Error<'a> {}

impl<'a> std::fmt::Display for Error<'a> {
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

pub fn run(state: &AppState) -> std::result::Result<(), Error> {
    let client = crate::config::create_client().map_err(Error::ClientCreation)?;
    let added = state.added.entries.iter().filter_map(|e| {
        e.active
            .then(|| add(e, &client))
            .and_then(|r| r.err())
            .map(|err| (e.term.as_str(), e.translation.as_str(), err))
    });
    let updated = state.updated.entries.iter().filter_map(|e| {
        e.active
            .then(|| update(e, &client))
            .and_then(|r| r.err())
            .map(|err| (e.term.as_str(), e.translation.as_str(), err))
    });
    let removed = state.removed.entries.iter().filter_map(|e| {
        e.active
            .then(|| remove(e, &client))
            .and_then(|r| r.err())
            .map(|err| (e.term.as_str(), e.translation.as_str(), err))
    });
    let data: Vec<_> = added.chain(removed).chain(updated).collect();

    if data.is_empty() {
        Ok(())
    } else {
        Err(Error::Update(data))
    }
}
