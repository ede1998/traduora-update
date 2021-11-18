use std::sync::Arc;

use druid::widget::{
    Button, Checkbox, Controller, Either, Flex, Label, LineBreaking, List, ProgressBar, Scroll,
    Spinner, Tabs, TabsTransition,
};
use druid::{im, theme, AppDelegate, ExtEventSink, LensExt, Selector, SingleUse, Target};
use druid::{Data, Lens};
use druid::{Env, Widget, WidgetExt};
use itertools::Itertools;
use traduora::api::TermId;

use crate::loader::{Modification, Translation};
use crate::modal_host::ModalHost;
use crate::updater::{Error as UpdateError, UpdateResult};

trait LensExtExt<A: ?Sized, B: ?Sized>: LensExt<A, B> {
    fn read_only<Get, C>(self, get: Get) -> druid::lens::Then<Self, ReadOnly<Get>, B>
    where
        Get: Fn(&B) -> C,
        Self: Sized,
    {
        self.then(ReadOnly::new(get))
    }
}

impl<A, B, L> LensExtExt<A, B> for L where L: Lens<A, B> {}

/// Lens that silently discards all writes.
#[derive(Clone, Copy, Debug)]
struct ReadOnly<Get> {
    get: Get,
}

impl<Get> ReadOnly<Get> {
    fn new<A: ?Sized, B: ?Sized>(get: Get) -> Self
    where
        Get: Fn(&A) -> B,
    {
        Self { get }
    }
}

impl<A: ?Sized, B, Get> Lens<A, B> for ReadOnly<Get>
where
    Get: Fn(&A) -> B,
{
    fn with<V, F: FnOnce(&B) -> V>(&self, data: &A, f: F) -> V {
        f(&(self.get)(data))
    }

    fn with_mut<V, F: FnOnce(&mut B) -> V>(&self, data: &mut A, f: F) -> V {
        let mut temp = (self.get)(data);
        f(&mut temp)
    }
}

#[derive(Data, Debug, Clone, Lens)]
pub struct TabData<T: Clone> {
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

#[derive(Data, Debug, Clone)]
enum Popup {
    Progressing(f64),
    Finished(Arc<UpdateResult>),
}

impl Popup {
    fn as_progressing(&self) -> Option<f64> {
        match self {
            Self::Progressing(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns `true` if the popup is [`Finished`].
    ///
    /// [`Finished`]: Popup::Finished
    fn is_finished(&self) -> bool {
        matches!(self, Self::Finished(..))
    }

    fn as_finished(&self) -> Option<&Arc<UpdateResult>> {
        if let Self::Finished(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl Default for Popup {
    fn default() -> Self {
        Self::Progressing(0.0)
    }
}

#[derive(Data, Debug, Clone, Lens, Default)]
pub struct AppState {
    pub added: TabData<Added>,
    pub removed: TabData<Removed>,
    pub updated: TabData<Updated>,
    popup: Popup,
}

impl AppState {
    fn extract_translations(&self) -> Vec<Translation> {
        fn extract<'a, T, I, F>(elements: I, construct: F) -> impl Iterator<Item = Translation> + 'a
        where
            T: 'a + Clone,
            I: IntoIterator<Item = &'a ModificationEntry<T>> + 'a,
            F: Fn(String, String, T) -> Translation + 'a,
        {
            elements.into_iter().cloned().filter_map(move |e| {
                e.active
                    .then(|| construct(e.term, e.translation, e.modification))
            })
        }
        let added = extract(&self.added.entries, |term, translation, _| {
            Translation::added(term, translation)
        });
        let removed = extract(&self.removed.entries, |term, translation, r| {
            Translation::removed(term, translation, r.0)
        });
        let updated = extract(&self.updated.entries, |term, translation, u| {
            Translation::updated(term, translation, u.0)
        });
        added.chain(removed).chain(updated).collect()
    }

    pub fn build(translations: impl IntoIterator<Item = Translation>) -> Self {
        fn new<T: Clone>() -> im::Vector<ModificationEntry<T>> {
            im::Vector::<ModificationEntry<T>>::new()
        }
        let (added, removed, updated) = translations.into_iter().fold(
            (new::<Added>(), new::<Removed>(), new::<Updated>()),
            |(mut added, mut removed, mut updated), t| {
                match t.modification {
                    Modification::Removed(id) => {
                        removed.push_back(ModificationEntry::removed(t.term, t.translation, id));
                    }
                    Modification::Added => {
                        added.push_back(ModificationEntry::added(t.term, t.translation));
                    }
                    Modification::Updated(id) => {
                        updated.push_back(ModificationEntry::updated(t.term, t.translation, id));
                    }
                }
                (added, removed, updated)
            },
        );

        Self {
            added: added.into(),
            removed: removed.into(),
            updated: updated.into(),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Data, Lens)]
pub struct ModificationEntry<T> {
    pub active: bool,
    pub term: String,
    pub translation: String,
    pub modification: T,
}

impl ModificationEntry<Updated> {
    pub fn updated(term: String, translation: String, id: TermId) -> Self {
        Self {
            active: true,
            term,
            modification: Updated(id),
            translation,
        }
    }
}

impl ModificationEntry<Removed> {
    pub fn removed(term: String, translation: String, id: TermId) -> Self {
        Self {
            active: true,
            term,
            modification: Removed(id),
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

impl DisplayString for (String, String, anyhow::Error) {
    fn display_string(&self) -> String {
        format!("{} ==> {}: {:?}", self.0, self.1, self.2)
    }
}

#[derive(Clone, Debug)]
pub struct Removed(pub TermId);

impl Data for Removed {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Debug)]
pub struct Updated(pub TermId);

impl Data for Updated {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

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
    let main_view = Flex::column()
        .with_flex_child(
            Tabs::new()
                .with_transition(TabsTransition::Instant)
                .with_tab("Removed", build_list().lens(AppState::removed))
                .with_tab("Added", build_list().lens(AppState::added))
                .with_tab("Updated", build_list().lens(AppState::updated)),
            10.,
        )
        .with_child(Button::new("Update terms").padding(10.).on_click(
            |ctx, data: &mut AppState, _env| {
                data.popup = Popup::default();
                let cmd = ModalHost::make_modal_command(build_popup);
                ctx.submit_command(cmd);
                wrapped_run(ctx.get_external_handle(), data);
            },
        ));

    ModalHost::new(main_view)
}

fn build_popup() -> impl Widget<AppState> {
    let progressing = Flex::column()
        .with_child(Label::new("Uploading terms."))
        .with_default_spacer()
        .with_child(Spinner::new())
        .with_default_spacer()
        .with_child(ProgressBar::new())
        .padding(16.0)
        .background(theme::BACKGROUND_DARK)
        .lens(AppState::popup.read_only(|p: &Popup| p.as_progressing().unwrap_or(0.)));

    let finished = Flex::column()
        .with_child(Label::new("Finished uploading terms."))
        .with_default_spacer()
        .with_flex_child(
            Scroll::new(
                Label::new(|data: &Arc<UpdateResult>, _: &_| match data.as_ref() {
                    Ok(_) => "No error occurred.".into(),
                    Err(UpdateError::ClientCreation(e)) => format!("{}", e),
                    Err(UpdateError::Update(errs)) => {
                        errs.iter().map(DisplayString::display_string).join("\n")
                    }
                })
                .with_line_break_mode(LineBreaking::WordWrap),
            ),
            1.,
        )
        .with_default_spacer()
        .with_child(Button::new("Ok").on_click(|ctx, _, _| {
            ctx.submit_command(ModalHost::DISMISS_MODAL);
        }))
        .padding(16.0)
        .background(theme::BACKGROUND_DARK)
        .lens(
            AppState::popup
                .read_only(|p: &Popup| p.as_finished().cloned().unwrap_or_else(|| Ok(()).into())),
        );

    Either::new(
        |data: &AppState, _| data.popup.is_finished(),
        finished,
        progressing,
    )
}

fn wrapped_run(sink: ExtEventSink, data: &AppState) {
    let translations = data.extract_translations();

    std::thread::spawn(move || {
        let result = crate::updater::run(translations, |current, max| {
            let current = current as f64;
            let max = max.max(1) as f64;
            let percentage = current / max;
            log::debug!("Sending update progress command: {} of {}", current, max);
            sink.submit_command(UPDATE_PROGRESS, percentage, Target::Auto)
                .expect("Failed to submit update progress command.");
        });
        log::info!("Sending finished update command: {:#?}", result);
        sink.submit_command(UPDATE_FINISHED, SingleUse::new(result), Target::Auto)
            .expect("Failed to submit update finished command.");
    });
}

const UPDATE_PROGRESS: Selector<f64> =
    Selector::new("me.erik-hennig.traduora-update.update-progress");

const UPDATE_FINISHED: Selector<SingleUse<UpdateResult>> =
    Selector::new("me.erik-hennig.traduora-update.update-finished");

pub struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _: &mut druid::DelegateCtx,
        _: Target,
        cmd: &druid::Command,
        data: &mut AppState,
        _: &Env,
    ) -> druid::Handled {
        log::debug!("Received command {:?}.", cmd);
        if let Some(progress) = cmd.get(UPDATE_PROGRESS) {
            data.popup = Popup::Progressing(*progress);
            druid::Handled::Yes
        } else if let Some(result) = cmd.get(UPDATE_FINISHED).and_then(SingleUse::take) {
            let load_result = crate::loader::load_data();
            log::info!(
                "Finished refreshing data. Error (if any): {:?}.",
                load_result.as_ref().err()
            );
            *data = AppState::build(load_result.unwrap_or_default());
            data.popup = Popup::Finished(result.into());
            druid::Handled::Yes
        } else {
            druid::Handled::No
        }
    }
}

#[derive(Debug, Clone, Data)]
pub struct AppStateError(Arc<anyhow::Error>);

impl From<anyhow::Error> for AppStateError {
    fn from(f: anyhow::Error) -> Self {
        Self(Arc::new(f))
    }
}

pub fn build_ui_startup_failed() -> impl Widget<AppStateError> {
    Flex::column()
        .with_child(Label::new(
            "Failed to start. Please fix the error and restart the application.",
        ))
        .with_default_spacer()
        .with_flex_child(
            Label::new(|state: &AppStateError, _: &_| format!("{:?}", state.0)),
            1.,
        )
}
