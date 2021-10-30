use anyhow::{anyhow, Context, Result};
use druid::im;
use itertools::{EitherOrBoth, Itertools};
use traduora::api::{terms::Term, TermId};

use crate::layout::{Added, AppState, ModificationEntry, Removed, Updated};

use super::{fetch_from_traduora, load_from_file};

#[derive(Debug, Clone)]
pub struct Translation {
    pub term_id: Option<TermId>,
    pub term: String,
    pub translation: String,
}

impl Translation {
    pub fn new(term: String, translation: String) -> Self {
        Self {
            term,
            translation,
            term_id: None,
        }
    }

    fn cmp_by_term(&self, other: &Self) -> std::cmp::Ordering {
        self.term.cmp(&other.term)
    }
}

impl From<(Term, String)> for Translation {
    fn from((term, translation): (Term, String)) -> Self {
        Self {
            term_id: Some(term.id),
            term: term.value,
            translation,
        }
    }
}

pub fn build_app_state() -> Result<AppState> {
    let mut local = load_from_file("testdata/en.json")?;
    let mut remote = fetch_from_traduora()?;
    local.sort_unstable_by(Translation::cmp_by_term);
    remote.sort_unstable_by(Translation::cmp_by_term);

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut updated = Vec::new();
    for e in local
        .into_iter()
        .merge_join_by(remote, Translation::cmp_by_term)
    {
        match e {
            EitherOrBoth::Both(local, remote) => {
                if local.translation != remote.translation {
                    updated.push(ModificationEntry::new(
                        local.term,
                        local.translation,
                        Updated,
                    ));
                }
            }
            EitherOrBoth::Left(local) => {
                added.push(ModificationEntry::new(local.term, local.translation, Added));
            }
            EitherOrBoth::Right(remote) => {
                removed.push(ModificationEntry::new(
                    remote.term,
                    remote.translation,
                    Removed,
                ));
            }
        }
    }
    Ok((added, removed, updated).into())
}
