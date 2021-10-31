use anyhow::{Context, Result};
use itertools::{EitherOrBoth, Itertools};

use traduora::{
    api::{
        terms::{Term, Terms},
        translations::Translations,
        TermId,
    },
    Query,
};

#[derive(Debug, Clone)]
pub struct Translation {
    pub term_id: TermId,
    pub term: String,
    pub translation: String,
}

impl Translation {
    pub fn cmp_by_term(&self, other: &Self) -> std::cmp::Ordering {
        self.term.cmp(&other.term)
    }
}

impl From<(Term, String)> for Translation {
    fn from((term, translation): (Term, String)) -> Self {
        Self {
            term_id: term.id,
            term: term.value,
            translation,
        }
    }
}

pub fn fetch_from_traduora() -> Result<Vec<Translation>> {
    use crate::config::*;
    let client = create_client()?;

    let mut terms = Terms(PROJECT_ID.into())
        .query(&client)
        .with_context(|| format!("Failed to load terms for project {:?}", PROJECT_ID))?;

    let mut translations = Translations::new(PROJECT_ID.into(), LOCALE.into())
        .query(&client)
        .with_context(|| {
            format!(
                "Failed to load translations for locale {:?} in project {:?}",
                LOCALE, PROJECT_ID
            )
        })?;

    terms.sort_unstable_by(|t1, t2| t1.id.cmp(&t2.id));
    translations.sort_unstable_by(|t1, t2| t1.term_id.cmp(&t2.term_id));

    Ok(terms
        .into_iter()
        .merge_join_by(translations, |term, tl| term.id.cmp(&tl.term_id))
        .filter_map(|e| match e {
            EitherOrBoth::Both(term, translation) => Some((term, translation.value).into()),
            EitherOrBoth::Left(term) => Some((term, String::new()).into()),
            EitherOrBoth::Right(_) => None,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::fetch_from_traduora;

    #[test]
    fn fetch() {
        let res = fetch_from_traduora().unwrap();
        println!("{:#?}", res);
    }
}
