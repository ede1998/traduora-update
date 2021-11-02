use anyhow::Result;
use itertools::{EitherOrBoth, Itertools};
use traduora::api::TermId;

use super::{local, remote};

#[derive(Debug, Clone)]
pub enum Modification {
    Removed(TermId),
    Updated(TermId),
    Added,
}

#[derive(Debug, Clone)]
pub struct Translation {
    pub term: String,
    pub translation: String,
    pub modification: Modification,
}

impl Translation {
    pub fn added(term: String, translation: String) -> Self {
        Self {
            term,
            translation,
            modification: Modification::Added,
        }
    }

    pub fn removed(term: String, translation: String, term_id: TermId) -> Self {
        Self {
            term,
            translation,
            modification: Modification::Removed(term_id),
        }
    }
    pub fn updated(term: String, translation: String, term_id: TermId) -> Self {
        Self {
            term,
            translation,
            modification: Modification::Updated(term_id),
        }
    }
}

fn merge(
    mut local: Vec<local::Translation>,
    mut remote: Vec<remote::Translation>,
) -> Vec<Translation> {
    local.sort_unstable_by(local::Translation::cmp_by_term);
    remote.sort_unstable_by(remote::Translation::cmp_by_term);
    local
        .into_iter()
        .merge_join_by(remote, |l, r| l.term.cmp(&r.term))
        .filter_map(|e| match e {
            EitherOrBoth::Both(local, remote) => (local.translation != remote.translation)
                .then(|| Translation::updated(local.term, local.translation, remote.term_id)),
            EitherOrBoth::Left(local) => Some(Translation::added(local.term, local.translation)),
            EitherOrBoth::Right(remote) => Some(Translation::removed(
                remote.term,
                remote.translation,
                remote.term_id,
            )),
        })
        .collect()
}

pub fn load_data() -> Result<Vec<Translation>> {
    let local = local::load_from_file(crate::config::get().translation_file())?;
    let remote = remote::fetch_from_traduora()?;
    Ok(merge(local, remote))
}
