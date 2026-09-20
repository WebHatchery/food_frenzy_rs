//! Serializable run state and persistent progression owned by the game layer.

use crate::data::{Achievement, CustomerSpecialTraits, GameData, Recipe, TutorialTrigger, Upgrade};
use macroquad_toolkit::timing::IntervalTimer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod cinematic;
mod day_cycle;
mod floaters;
mod progression;
mod tutorial;

pub use cinematic::{CinematicPhase, ProcessingCinematic};
pub use day_cycle::{DayCycle, DayStats};
pub use floaters::{
    FloaterAnchor, FloaterKind, Floaters, FLOATER_LIFETIME_MS, MAX_ACTIVE_FLOATERS,
};
pub use tutorial::TutorialProgress;

pub const INFINITE_INGREDIENTS: i64 = -1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Satisfaction {
    pub blue: f32,
    pub green: f32,
    pub yellow: f32,
    pub red: f32,
}

impl Satisfaction {
    pub fn total(&self) -> f32 {
        self.blue + self.green + self.yellow + self.red
    }

    pub fn get(&self, color: &str) -> Option<f32> {
        match color {
            "blue" => Some(self.blue),
            "green" => Some(self.green),
            "yellow" => Some(self.yellow),
            "red" => Some(self.red),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, color: &str) -> Option<&mut f32> {
        match color {
            "blue" => Some(&mut self.blue),
            "green" => Some(&mut self.green),
            "yellow" => Some(&mut self.yellow),
            "red" => Some(&mut self.red),
            _ => None,
        }
    }

    pub fn decay_all(&mut self, amount: f32) {
        for value in [
            &mut self.blue,
            &mut self.green,
            &mut self.yellow,
            &mut self.red,
        ] {
            *value = (*value - amount).max(0.0);
        }
    }
}

/// A cooked dish waiting on the pass. Ages in real time: fresh dishes pay a
/// bonus, spoiled ones are discarded (see `engine::freshness`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "PlatedDishCompat")]
pub struct PlatedDish {
    pub name: String,
    pub age_ms: f32,
}

impl PlatedDish {
    pub fn new(name: String) -> Self {
        Self { name, age_ms: 0.0 }
    }
}

/// Pre-freshness saves stored plated dishes as bare strings.
#[derive(Deserialize)]
#[serde(untagged)]
enum PlatedDishCompat {
    Current {
        name: String,
        #[serde(default)]
        age_ms: f32,
    },
    Legacy(String),
}

impl From<PlatedDishCompat> for PlatedDish {
    fn from(compat: PlatedDishCompat) -> Self {
        match compat {
            PlatedDishCompat::Current { name, age_ms } => Self { name, age_ms },
            PlatedDishCompat::Legacy(name) => Self::new(name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookingStation {
    pub color: String,
    pub is_cooking: bool,
    pub remaining_ms: f32,
    pub dishes: Vec<PlatedDish>,
}

impl CookingStation {
    pub fn new(color: String) -> Self {
        Self {
            color,
            is_cooking: false,
            remaining_ms: 0.0,
            dishes: Vec::new(),
        }
    }

    pub fn can_cook(&self, max_ready: usize) -> bool {
        !self.is_cooking && self.dishes.len() < max_ready
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerActor {
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    #[serde(default)]
    pub carried_station: Option<String>,
    #[serde(default)]
    pub task_label: String,
    #[serde(default)]
    pub clear_carry_on_arrival: bool,
    #[serde(default)]
    pub action_lock_ms: f32,
    #[serde(default)]
    pub lock_on_arrival_ms: f32,
}

impl Default for PlayerActor {
    fn default() -> Self {
        Self {
            x: -235.0,
            y: 104.0,
            target_x: -235.0,
            target_y: 104.0,
            carried_station: None,
            task_label: String::new(),
            clear_carry_on_arrival: false,
            action_lock_ms: 0.0,
            lock_on_arrival_ms: 0.0,
        }
    }
}

/// A telegraphed special-trait warning: the guest is about to act (steal,
/// tantrum, wander) unless the player answers within the window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitAlert {
    pub trait_key: String,
    pub remaining_ms: f32,
    /// Courses served when the alert armed; a wanderer settles if any course
    /// lands during the window.
    pub courses_served_at_arm: usize,
}

/// One course a guest ordered (a dish color plus a display label such as
/// "Entrée" / "Main" / "Dessert"), and whether it has been served yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub color: String,
    pub label: String,
    #[serde(default)]
    pub served: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: u32,
    pub guest_id: String,
    pub display_name: String,
    pub customer_type: String,
    pub satisfaction: Satisfaction,
    pub max_satisfaction: Satisfaction,
    pub deliciousness: f32,
    pub total_satisfaction: f32,
    pub overfed: bool,
    pub table_index: usize,
    pub arrived_at_ms: f64,
    #[serde(default)]
    pub floor_x: f32,
    #[serde(default)]
    pub floor_y: f32,
    #[serde(default)]
    pub target_x: f32,
    #[serde(default)]
    pub target_y: f32,
    #[serde(default)]
    pub is_seated: bool,
    /// Running tab the guest pays in Cash when they leave the restaurant.
    #[serde(default)]
    pub bill: i64,
    /// Countdown before a guest who finished their order gets up to pay and leave.
    #[serde(default)]
    pub depart_timer_ms: f32,
    /// The 1-3 courses this guest wants this visit.
    #[serde(default)]
    pub order: Vec<Course>,
    /// How many previous visits this guest was fully served (fattening progress).
    #[serde(default)]
    pub times_fed: u32,
    /// Active telegraphed-trait warning, if any.
    #[serde(default)]
    pub trait_alert: Option<TraitAlert>,
    /// Personality archetype id, copied from the guest record at spawn so the
    /// UI can flavor chatter without a `GuestState` lookup.
    #[serde(default)]
    pub personality: Option<String>,
    /// Time left eating the course just served; the next course should wait.
    #[serde(default)]
    pub eating_ms: f32,
    /// Time spent waiting for the next course after finishing the last one.
    #[serde(default)]
    pub waiting_ms: f32,
}

impl Customer {
    pub fn traits(&self, data: &GameData) -> CustomerSpecialTraits {
        data.customer_type_by_id(&self.customer_type)
            .and_then(|customer_type| customer_type.special_traits.clone())
            .unwrap_or_default()
    }

    pub fn refresh_totals(&mut self) {
        self.total_satisfaction = self.satisfaction.total();
        self.overfed = self.total_satisfaction > self.max_satisfaction.total();
    }

    /// Index of the next course this guest is waiting on for `color`, if any.
    pub fn next_course_for(&self, color: &str) -> Option<usize> {
        self.order
            .iter()
            .position(|course| !course.served && course.color == color)
    }

    /// True once the guest has an order and every course has been served.
    pub fn order_complete(&self) -> bool {
        !self.order.is_empty() && self.order.iter().all(|course| course.served)
    }

    pub fn courses_served(&self) -> usize {
        self.order.iter().filter(|course| course.served).count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestRecord {
    pub id: String,
    pub name: String,
    pub customer_type: String,
    pub visits: u32,
    pub feedings: u32,
    #[serde(default)]
    pub satisfied_visits: u32,
    pub processed_count: u32,
    pub last_seen_at: u64,
    /// Personality archetype id (see `assets/data/regulars.json`); colors the
    /// guest's arrival line and their Lounge farewell.
    #[serde(default)]
    pub personality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestState {
    pub guests: Vec<GuestRecord>,
}

impl GuestState {
    pub fn new() -> Self {
        Self { guests: Vec::new() }
    }

    fn random_id() -> String {
        format!("guest-{}", macroquad_toolkit::rng::gen_range(0, 999_999))
    }

    pub fn create_guest(
        &mut self,
        name: &str,
        customer_type: &str,
        personality: Option<String>,
    ) -> GuestRecord {
        let guest = GuestRecord {
            id: Self::random_id(),
            name: name.to_string(),
            customer_type: customer_type.to_string(),
            visits: 0,
            feedings: 0,
            satisfied_visits: 0,
            processed_count: 0,
            last_seen_at: 0,
            personality,
        };

        self.guests.push(guest.clone());
        guest
    }

    pub fn guest_by_id(&self, guest_id: &str) -> Option<&GuestRecord> {
        self.guests.iter().find(|guest| guest.id == guest_id)
    }

    pub fn get_returning_unlocked_guest(
        &self,
        unlocked_customer_types: &[String],
        excluded_ids: &[String],
    ) -> Option<GuestRecord> {
        let candidates = self.returning_guest_candidates(unlocked_customer_types, excluded_ids);
        macroquad_toolkit::rng::choose(&candidates).map(|guest| (*guest).clone())
    }

    pub fn returning_guest_candidates<'a>(
        &'a self,
        unlocked_customer_types: &[String],
        excluded_ids: &[String],
    ) -> Vec<&'a GuestRecord> {
        let excluded: HashSet<_> = excluded_ids.iter().collect();
        let unlocked: HashSet<_> = unlocked_customer_types.iter().collect();
        self.guests
            .iter()
            .filter(|guest| {
                guest.feedings > 0
                    && guest.processed_count == 0
                    && !excluded.contains(&guest.id)
                    && unlocked.contains(&guest.customer_type)
            })
            .collect()
    }

    pub fn record_guest_visit(&mut self, guest_id: &str) {
        self.record_guest_touch(guest_id, |guest| {
            guest.visits = guest.visits.saturating_add(1);
        });
    }

    pub fn record_guest_fed(&mut self, guest_id: &str) {
        self.record_guest_touch(guest_id, |guest| {
            guest.feedings = guest.feedings.saturating_add(1);
        });
    }

    pub fn record_guest_satisfied_visit(&mut self, guest_id: &str) {
        self.record_guest_touch(guest_id, |guest| {
            guest.satisfied_visits = guest.satisfied_visits.saturating_add(1);
        });
    }

    pub fn record_guest_processed(&mut self, guest_id: &str) {
        self.record_guest_touch(guest_id, |guest| {
            guest.processed_count = guest.processed_count.saturating_add(1);
        });
    }

    fn record_guest_touch(&mut self, guest_id: &str, update: impl FnOnce(&mut GuestRecord)) {
        if let Some(guest) = self.guests.iter_mut().find(|guest| guest.id == guest_id) {
            update(guest);
            // This field is an ordering marker, not a wall-clock save value.
            // A logical touch counter keeps replays and headless tests
            // deterministic while still making each visit observable.
            guest.last_seen_at = guest.last_seen_at.saturating_add(1);
        }
    }
}

impl Default for GuestState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionState {
    pub currency: i64,
    pub upgrades: Vec<Upgrade>,
    pub recipes: Vec<Recipe>,
    pub achievements: Vec<Achievement>,
    pub prestige_level: u32,
    pub prestige_points: i32,
    pub total_score: i64,
    pub processed_customer_counts: HashMap<String, u32>,
    pub processed_customer_types: Vec<String>,
    pub feeding_capacity_bonus: i64,
    pub crafted_recipe_counts: HashMap<String, u32>,
    pub total_dishes_served: i64,
    pub preferred_dishes_served: i64,
    pub overfed_customer_count: i64,
    pub customers_lost: i64,
    #[serde(default)]
    pub unlocked_customer_types: Vec<String>,
    /// Chosen house style (see `assets/data/specializations.json`); None until
    /// the player commits after their first processing. Reset by prestige.
    #[serde(default)]
    pub specialization: Option<String>,
    /// The chosen style's effect table, copied at pick time so `get_effect`
    /// needs no `GameData` access.
    #[serde(default)]
    pub specialization_effects: HashMap<String, f64>,
    /// Trait keys whose first-encounter hint has already been shown.
    #[serde(default)]
    pub seen_trait_hints: Vec<String>,
    #[serde(default)]
    pub days_completed: i64,
    #[serde(default)]
    pub regulars_processed: i64,
    #[serde(default)]
    pub events_completed: i64,
    #[serde(default)]
    pub fresh_dishes_served: i64,
    #[serde(default)]
    pub full_house_bonuses: i64,
    /// Prestige perk ids chosen so far (one per prestige).
    #[serde(default)]
    pub prestige_perks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub score: i64,
    pub combo: u32,
    pub chain: u32,
    pub customers: Vec<Customer>,
    pub ingredients: HashMap<String, i64>,
    pub cooking_stations: HashMap<String, CookingStation>,
    pub special_table_busy: bool,
    pub special_table_timer: f32,
    pub messages: Vec<String>,
    pub next_customer_id: u32,
    #[serde(default)]
    pub player: PlayerActor,
    /// Onboarding progress; persisted so the tutorial resumes across saves.
    #[serde(default)]
    pub tutorial: TutorialProgress,
    /// Presentation-only: floating gain numbers. Never saved.
    #[serde(skip)]
    pub floaters: Floaters,
    /// Presentation-only: the Last Meal Lounge sequence in progress, if any.
    /// Rewards are already applied when this is set, so dropping it on
    /// save/load loses nothing but the show.
    #[serde(skip)]
    pub processing_cinematic: Option<ProcessingCinematic>,
    /// Presentation-only management destination. The service floor remains
    /// intact beneath it so returning never loses the carried dish or guest.
    #[serde(skip)]
    pub show_management: bool,
    #[serde(skip)]
    pub management_tab: u8,
    #[serde(skip)]
    pub management_page: usize,
    #[serde(skip)]
    pub selected_recipe_id: Option<String>,
    #[serde(skip)]
    pub prestige_page: usize,
    #[serde(skip)]
    pub selected_prestige_perk: Option<String>,
    #[serde(skip)]
    pub specialization_page: usize,
    #[serde(skip)]
    pub show_pause_menu: bool,
    #[serde(skip)]
    pub show_help: bool,
    #[serde(skip)]
    pub show_history: bool,
    /// Presentation-only guest detail selection, so touch players can pin
    /// the same information that desktop players get by hovering.
    #[serde(skip)]
    pub selected_guest_id: Option<u32>,
    /// Armed when a guest arrives with an unserved order; consumed by the
    /// full-house bonus when every seated order completes at once.
    #[serde(default)]
    pub full_room_bonus_armed: bool,
    /// The soft day/shift clock and per-day ledger.
    #[serde(default)]
    pub day_cycle: DayCycle,
    /// The dining event currently in effect, if any.
    #[serde(default)]
    pub active_event: Option<ActiveEvent>,
    /// True while the prestige perk choice modal is up.
    #[serde(default)]
    pub pending_prestige: bool,
    /// Sound effects queued this frame; drained by the app each tick.
    #[serde(skip)]
    pub sfx_queue: Vec<SfxCue>,
}

/// A dining event in progress (definition lives in `GameData::dining_events`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEvent {
    pub event_id: String,
    pub remaining_ms: f32,
}

/// A sound-effect request queued by gameplay code and drained by the app
/// (gameplay has no audio access; see `audio::AudioBank`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfxCue {
    CookStart,
    DishReady,
    Serve,
    Cash,
    LoungeSting,
    DayEnd,
    Event,
}

/// Interval accumulators start with a placeholder duration: every call site
/// sets the real interval via `set_interval` immediately before ticking, so
/// the constructed value here never matters.
fn default_interval_timer() -> IntervalTimer {
    IntervalTimer::new(1.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timers {
    pub elapsed_ms: f64,
    pub next_spawn_ms: f64,
    pub spawn_step: usize,
    #[serde(default = "default_interval_timer")]
    pub decay_timer: IntervalTimer,
    #[serde(default = "default_interval_timer")]
    pub patience_timer: IntervalTimer,
    #[serde(default = "default_interval_timer")]
    pub trait_timer: IntervalTimer,
    #[serde(default = "default_interval_timer")]
    pub save_timer: IntervalTimer,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            elapsed_ms: 0.0,
            next_spawn_ms: 0.0,
            spawn_step: 0,
            decay_timer: default_interval_timer(),
            patience_timer: default_interval_timer(),
            trait_timer: default_interval_timer(),
            save_timer: default_interval_timer(),
        }
    }
}

impl Default for Timers {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new(data: &crate::data::GameData) -> Self {
        let mut stations = HashMap::new();
        for dish in &data.dish_types {
            stations.insert(dish.color.clone(), CookingStation::new(dish.color.clone()));
        }

        let mut ingredients = HashMap::new();
        ingredients.insert(
            data.balance.regular_ingredient_name.clone(),
            INFINITE_INGREDIENTS,
        );

        Self {
            score: 0,
            combo: 0,
            chain: 0,
            customers: Vec::new(),
            ingredients,
            cooking_stations: stations,
            special_table_busy: false,
            special_table_timer: 0.0,
            messages: vec![data.text("service_started").to_string()],
            next_customer_id: 1,
            player: PlayerActor {
                task_label: data.text("task_prep").to_string(),
                ..PlayerActor::default()
            },
            tutorial: TutorialProgress::default(),
            floaters: Floaters::default(),
            processing_cinematic: None,
            show_management: false,
            management_tab: 0,
            management_page: 0,
            selected_recipe_id: None,
            prestige_page: 0,
            selected_prestige_perk: None,
            specialization_page: 0,
            show_pause_menu: false,
            show_help: false,
            show_history: false,
            selected_guest_id: None,
            full_room_bonus_armed: false,
            day_cycle: DayCycle::default(),
            active_event: None,
            pending_prestige: false,
            sfx_queue: Vec::new(),
        }
    }

    pub fn queue_sfx(&mut self, cue: SfxCue) {
        self.sfx_queue.push(cue);
    }

    /// The active dining event's effect, resolved against the data table.
    pub fn active_event_effect<'data>(
        &self,
        data: &'data GameData,
    ) -> Option<&'data crate::data::EventEffect> {
        self.active_event
            .as_ref()
            .and_then(|active| data.dining_event_by_id(&active.event_id))
            .map(|event| &event.effect)
    }

    /// Report a tutorial-worthy gameplay event; advances the tutorial when it
    /// matches the current step.
    pub fn tutorial_observe(&mut self, trigger: TutorialTrigger, data: &GameData) {
        self.tutorial.observe(trigger, &data.tutorial_steps);
    }

    pub fn next_customer_id(&mut self) -> u32 {
        let id = self.next_customer_id;
        self.next_customer_id = self.next_customer_id.saturating_add(1);
        id
    }

    pub fn add_message(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
        if self.messages.len() > 18 {
            self.messages.drain(0..(self.messages.len() - 18));
        }
    }

    pub fn station_mut(&mut self, color: &str) -> Option<&mut CookingStation> {
        self.cooking_stations.get_mut(color)
    }

    pub fn has_ing(&self, key: &str, amount: i64) -> bool {
        if key == "regular" {
            return true;
        }

        self.ingredients
            .get(key)
            .is_some_and(|value| *value >= amount)
    }

    pub fn remove_ingredients(&mut self, key: &str, amount: i64) -> bool {
        if !self.has_ing(key, amount) {
            return false;
        }

        let current = self.ingredients.entry(key.to_string()).or_insert(0);
        if *current != INFINITE_INGREDIENTS {
            *current = current.saturating_sub(amount);
        }

        true
    }
}
