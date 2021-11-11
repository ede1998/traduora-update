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
    let project_id = crate::config::get().project_id();
    let locale = crate::config::get().locale();

    let mut terms = Terms(project_id.clone())
        .query(&client)
        .with_context(|| format!("Failed to load terms for project {:?}", project_id))?;

    let mut translations = Translations::new(project_id.clone(), locale.clone())
        .query(&client)
        .with_context(|| {
            format!(
                "Failed to load translations for locale {:?} in project {:?}",
                locale, project_id
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

    #[ignore = "needs access to a traduora instance"]
    #[test]
    fn fetch() {
        crate::config::init().unwrap();
        let res = fetch_from_traduora().unwrap();
        println!("{:#?}", res);
    }
}
