#![allow(unused)]
// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use druid::lens::{Constant, Unit};
use druid::widget::{
    Axis, Button, Checkbox, Controller, Flex, Label, LensWrap, List, ListIter, Scroll, Switch,
    Tabs, TabsEdge, TabsTransition, TextBox,
};
use druid::{im, Color};
use druid::{AppLauncher, Env, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc};
use druid::{Data, Lens};

#[derive(Data, Clone, Lens)]
struct TabData<T> {
    select_all_active: bool,
    entries: im::Vector<ModificationEntry<T>>,
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

#[derive(Data, Clone, Lens, Default)]
struct AppState {
    added: TabData<Added>,
    removed: TabData<Removed>,
    updated: TabData<Updated>,
}

#[derive(Clone, Debug, Data, Lens)]
struct ModificationEntry<T> {
    pub active: bool,
    pub term: String,
    pub modification: T,
}

trait DisplayString {
    fn display_string(&self) -> String;
}

impl DisplayString for ModificationEntry<Removed> {
    fn display_string(&self) -> String {
        self.term.to_string()
    }
}

impl DisplayString for ModificationEntry<Updated> {
    fn display_string(&self) -> String {
        format!("{} ==> {}", self.term, self.modification.0)
    }
}

impl DisplayString for ModificationEntry<Added> {
    fn display_string(&self) -> String {
        format!("{} ==> {}", self.term, self.modification.0)
    }
}

#[derive(Clone, Debug, Data)]
struct Removed;
#[derive(Clone, Debug, Data)]
struct Updated(String);
#[derive(Clone, Debug, Data)]
struct Added(String);

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

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui);
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(build_test_data())
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

fn build_ui() -> impl Widget<AppState> {
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
fn build_test_data() -> AppState {
    fn added(term: &str, translation: &str) -> ModificationEntry<Added> {
        ModificationEntry {
            active: true,
            term: term.into(),
            modification: Added(translation.into()),
        }
    }
    fn chg(term: &str, translation: &str) -> ModificationEntry<Updated> {
        ModificationEntry {
            active: true,
            term: term.into(),
            modification: Updated(translation.into()),
        }
    }
    fn rm(term: &str) -> ModificationEntry<Removed> {
        ModificationEntry {
            active: true,
            term: term.into(),
            modification: Removed,
        }
    }
    let added = [
        added(
            "globe.championship.congregation.burden.probable",
            "colonial congregation sustain",
        ),
        added(
            "textbook.dot.thankfully.large-scale.universal",
            "subsequently refuge authorize",
        ),
        added(
            "defensive.aspire.flaw.trophy.strictly",
            "dealer sheer gravity",
        ),
        added(
            "risky.publishing.abstract.solidarity.successor",
            "inhabitant proceeds collision",
        ),
        added(
            "strengthen.albeit.pulse.mosque.amusing",
            "seize pond monster",
        ),
        added(
            "password.shipping.nearby.purely.coordination",
            "inhibit negotiation cheer",
        ),
        added(
            "oral.theoretical.specialized.lighting.militant",
            "alien prevalence wheat",
        ),
    ]
    .into_iter()
    .cycle()
    .take(500)
    .collect();
    let updated = [
        chg(
            "principal.transcript.pledge.prevention.ballet",
            "diversity permanently spin",
        ),
        chg(
            "strip.notably.hopeful.resume.rhetoric",
            "clinical remainder curriculum",
        ),
        chg(
            "nonsense.cease.revelation.interaction.complexity",
            "satisfaction disrupt noble",
        ),
        chg(
            "sigh.tribunal.fluid.detain.toll",
            "undergraduate collaboration proceedings",
        ),
        chg(
            "treasure.protein.erect.suck.monk",
            "trauma simultaneously contradiction",
        ),
        chg(
            "abstract.spite.magnitude.lane.adverse",
            "remains accomplish enact",
        ),
        chg(
            "specify.comparable.appoint.diminish.surplus",
            "firm joint badge",
        ),
        chg(
            "elementary.absorb.cocktail.healthcare.stem",
            "activation accent hatred",
        ),
    ]
    .into_iter()
    .collect();
    let removed = [
        rm("odds.niche.carve.ideology.embark"),
        rm("globe.voting.literary.faculty.militia"),
        rm("erupt.clerk.unveil.latter.ideological"),
        rm("pledge.empire.barely.contrary.accused"),
        rm("canvas.capability.subscriber.maintenance.economics"),
    ]
    .into_iter()
    .collect();
    AppState {
        added: TabData {
            entries: added,
            ..Default::default()
        },
        removed: TabData {
            entries: removed,
            ..Default::default()
        },
        updated: TabData {
            entries: updated,
            ..Default::default()
        },
    }
}
