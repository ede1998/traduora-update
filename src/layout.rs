use druid::im;
use druid::widget::{
    Button, Checkbox, Controller, Flex, Label, List, Scroll, Tabs, TabsTransition,
};
use druid::{Data, Lens};
use druid::{Env, Widget, WidgetExt};

use crate::loader::{Modification, Translation};

#[derive(Data, Clone, Lens)]
pub struct TabData<T> {
    pub select_all_active: bool,
    pub entries: im::Vector<ModificationEntry<T>>,
}

impl<T> Default for TabData<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self {
            select_all_active: true,
            entries: im::Vector::default(),
        }
    }
}

impl<T> From<im::Vector<ModificationEntry<T>>> for TabData<T>
where
    T: Clone,
{
    fn from(m: im::Vector<ModificationEntry<T>>) -> Self {
        Self {
            select_all_active: true,
            entries: m,
        }
    }
}

#[derive(Data, Clone, Lens, Default)]
pub struct AppState {
    pub added: TabData<Added>,
    pub removed: TabData<Removed>,
    pub updated: TabData<Updated>,
}

#[derive(Clone, Debug, Data, Lens)]
pub struct ModificationEntry<T> {
    pub active: bool,
    pub term: String,
    pub translation: String,
    pub modification: T,
}

impl ModificationEntry<Updated> {
    pub fn updated(term: String, translation: String) -> Self {
        Self {
            active: true,
            term,
            modification: Updated,
            translation,
        }
    }
}

impl ModificationEntry<Removed> {
    pub fn removed(term: String, translation: String) -> Self {
        Self {
            active: true,
            term,
            modification: Removed,
            translation,
        }
    }
}

impl ModificationEntry<Added> {
    pub fn added(term: String, translation: String) -> Self {
        Self {
            active: true,
            term,
            modification: Added,
            translation,
        }
    }
}

trait DisplayString {
    fn display_string(&self) -> String;
}

impl<T> DisplayString for ModificationEntry<T> {
    fn display_string(&self) -> String {
        format!("{} ==> {}", self.term, self.translation)
    }
}

#[derive(Clone, Debug, Data)]
pub struct Removed;
#[derive(Clone, Debug, Data)]
pub struct Updated;
#[derive(Clone, Debug, Data)]
pub struct Added;

struct OmniSelector;

impl<T, W> Controller<TabData<T>, W> for OmniSelector
where
    T: Clone,
    W: Widget<TabData<T>>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut TabData<T>,
        env: &Env,
    ) {
        let old_value = data.select_all_active;
        child.event(ctx, event, data, env);
        if old_value == data.select_all_active {
            return;
        }
        for entry in data.entries.iter_mut() {
            entry.active = data.select_all_active;
        }
    }
}

fn build_item<T>() -> impl Widget<ModificationEntry<T>>
where
    T: druid::Data,
    ModificationEntry<T>: DisplayString,
{
    Flex::row()
        .with_child(Checkbox::new("").lens(ModificationEntry::<T>::active))
        .with_child(Label::new(|item: &ModificationEntry<T>, _env: &_| {
            item.display_string()
        }))
}

fn build_list<T>() -> impl Widget<TabData<T>>
where
    T: druid::Data,
    ModificationEntry<T>: DisplayString,
{
    Flex::column()
        .with_child(
            Checkbox::new(|is_active: &bool, _env: &_| {
                if *is_active {
                    "Deselect all"
                } else {
                    "Select all"
                }
                .into()
            })
            .lens(TabData::<T>::select_all_active)
            .controller(OmniSelector)
            .align_left(),
        )
        .with_default_spacer()
        .with_flex_child(
            Scroll::new(List::new(build_item).with_spacing(5.))
                .vertical()
                .expand_width()
                .lens(TabData::<T>::entries),
            1.,
        )
}

pub fn build_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_flex_child(
            Tabs::new()
                .with_transition(TabsTransition::Instant)
                .with_tab("Removed", build_list().lens(AppState::removed))
                .with_tab("Added", build_list().lens(AppState::added))
                .with_tab("Updated", build_list().lens(AppState::updated)),
            10.,
        )
        .with_child(Button::new("Update terms").padding(10.))
}

pub fn build_app_state(translations: &[Translation]) -> AppState {
    let added: im::Vector<_> = translations
        .iter()
        .filter_map(|t| {
            matches!(t.modification, Modification::Added)
                .then(|| ModificationEntry::added(t.term.clone(), t.translation.clone()))
        })
        .collect();
    let removed: im::Vector<_> = translations
        .iter()
        .filter_map(|t| {
            matches!(t.modification, Modification::Removed(_))
                .then(|| ModificationEntry::removed(t.term.clone(), t.translation.clone()))
        })
        .collect();
    let updated: im::Vector<_> = translations
        .iter()
        .filter_map(|t| {
            matches!(t.modification, Modification::Updated(_))
                .then(|| ModificationEntry::updated(t.term.clone(), t.translation.clone()))
        })
        .collect();

    AppState {
        added: added.into(),
        removed: removed.into(),
        updated: updated.into(),
    }
}
