use anyhow::Result;
use itertools::{merge_join_by, EitherOrBoth, Itertools};
use traduora::api::TermId;

use super::{local, remote};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modification {
    Removed(TermId),
    Updated(TermId),
    Added,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
    mut git: Vec<local::Translation>,
) -> Vec<Translation> {
    local.sort_unstable_by(local::Translation::cmp_by_term);
    remote.sort_unstable_by(remote::Translation::cmp_by_term);
    git.sort_unstable_by(local::Translation::cmp_by_term);
    merge_join_by(local, remote, |l, r| l.term.cmp(&r.term))
        .filter_map(|e| match e {
            EitherOrBoth::Both(local, remote) => (local.translation != remote.translation && !local.translation.is_empty())
                .then(|| Translation::updated(local.term, local.translation, remote.term_id)),
            EitherOrBoth::Left(local) => Some(Translation::added(local.term, local.translation)),
            EitherOrBoth::Right(remote) => Some(Translation::removed(
                remote.term,
                remote.translation,
                remote.term_id,
            )),
        })
        .merge_join_by(git, |t, g| t.term.cmp(&g.term))
        .filter_map(|e: EitherOrBoth<_, _>| {
            match e {
                // term does not exist in history and local file but on Traduora -> probably added from elsewhere
                EitherOrBoth::Left(Translation {
                    modification: Modification::Removed(_),
                    ..
                }) |
                // deleted in local translations and traduora, only exists in history -> we are done already
                EitherOrBoth::Right(_) => None,
                EitherOrBoth::Both(t, g) => match t.modification {
                    // term exists in git -> removal was explicit
                    Modification::Removed(_) => Some(t),
                    // Term exists locally and in git but not in Traduora -> term removed elsewhere
                    Modification::Added => None,
                    // Translations differ in Traduora and locally but git is same as local -> translation changed elsewhere
                    // Translations differ in Traduora and locally but git is different than local -> translation changed locally
                    Modification::Updated(_) => (t.translation != g.translation).then(|| t),
                },
                // term does not exist in git but was not removed, git is too old to know term -> no git data to double check with
                EitherOrBoth::Left(t) => Some(t),
            }
        })
        .collect()
}

pub fn load_data() -> Result<Vec<Translation>> {
    let translation_file = crate::config::get().translation_file();
    let revision = crate::config::get().revision();

    let local = local::load_from_file(translation_file)?;
    let remote = remote::fetch_from_traduora()?;
    let git = if revision.is_empty() {
        Vec::new()
    } else {
        local::load_from_git(revision, translation_file)?
    };
    Ok(merge(local, remote, git))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_not_remove_text_for_translation_update() {
        let remote = vec![remote::Translation {
            term_id: "example-id".into(),
            term: "foo.bar.baz".into(),
            translation: "hello world".into(),
        }];
        let local = vec![local::Translation {
            term: "foo.bar.baz".into(),
            translation: String::new(),
        }];
        let git = Vec::new();

        let result = merge(local, remote, git);

        const EXPECTED: &[Translation] = &[];
        assert_eq!(EXPECTED, result);
    }
}
