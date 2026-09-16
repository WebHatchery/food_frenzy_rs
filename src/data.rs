//! Typed game content and balance data loaded from the embedded asset catalog.

use macroquad_toolkit::data_loader::{
    load_json_file_with_fallback_sync, parse_json_labeled, JsonFallbackPolicy,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CUSTOMER_TYPES_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/customer_types.json");
const DISH_TYPES_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/dish_types.json");
const UPGRADES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/upgrades.json");
const RECIPES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/recipes.json");
const ACHIEVEMENTS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/achievements.json");
const GAME_BALANCE_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/game_balance.json");
const TUTORIAL_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/tutorial.json");
const SPECIALIZATIONS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/specializations.json");
const TRAIT_BEHAVIORS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/trait_behaviors.json");
const REGULARS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/regulars.json");
const DINING_EVENTS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/dining_events.json");
const PRESTIGE_PERKS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/prestige_perks.json");

pub const STATION_COLORS: [&str; 4] = ["blue", "green", "yellow", "red"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomerSpecialTraits {
    #[serde(default)]
    pub low_appetite: bool,
    #[serde(default)]
    pub can_wander: bool,
    #[serde(default)]
    pub multiplies_on_process: bool,
    #[serde(default)]
    pub fast_spoilage: bool,
    #[serde(default)]
    pub can_steal_food: bool,
    #[serde(default)]
    pub can_eat_waste: bool,
    #[serde(default)]
    pub high_yield: bool,
    #[serde(default)]
    pub throws_food: bool,
    /// Tips double when they leave satisfied.
    #[serde(default)]
    pub big_tipper: bool,
    /// Fresh dishes served to them earn bonus renown.
    #[serde(default)]
    pub influencer: bool,
    /// Always orders the maximum number of courses and pays half again more.
    #[serde(default)]
    pub gourmand: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerType {
    pub id: String,
    pub name: String,
    pub preferred_dishes: Vec<String>,
    pub base_deliciousness: f32,
    pub description: String,
    #[serde(default)]
    pub initially_unlocked: bool,
    #[serde(default)]
    pub profile_tier: u32,
    #[serde(default)]
    pub unlock_cost: HashMap<String, i64>,
    #[serde(default)]
    pub special_traits: Option<CustomerSpecialTraits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DishType {
    pub color: String,
    pub name: String,
    pub cook_time_ms: f32,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upgrade {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub base_cost: i64,
    pub cost: i64,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub max_level: u32,
    pub cost_growth: f64,
    #[serde(default)]
    pub purchased: bool,
    #[serde(default)]
    pub effects: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub ingredients: HashMap<String, i64>,
    #[serde(default)]
    pub customer_type: Option<String>,
    pub unlocked: bool,
    pub unlock_condition: String,
    pub profit_multiplier: f64,
    pub base_value: i64,
    pub capacity_bonus: i64,
    /// Processed-count requirements (customer type id → count) that unlock
    /// this recipe; drives `ProgressionState::update_recipe_unlocks`.
    #[serde(default)]
    pub unlock_requirements: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked: bool,
    pub progress: i64,
    pub max_progress: i64,
    pub reward: i64,
}

/// What has to happen in play for a tutorial step to complete and advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TutorialTrigger {
    GuestSeated,
    CookingStarted,
    DishCarried,
    CourseServed,
    GuestDepartedFed,
    GuestReady,
    GuestProcessed,
    /// Completed by the player clicking "Got it" rather than a game event.
    Acknowledged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub body: String,
    pub trigger: TutorialTrigger,
}

/// A house style the player commits to after their first processing. Effects
/// use the same keys as upgrades and feed the same `get_effect` accumulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecializationDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub flavor: String,
    pub effects: HashMap<String, f64>,
}

/// Name pool and personality archetypes for persistent guests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegularsData {
    pub names: Vec<String>,
    pub personalities: Vec<PersonalityDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityDef {
    pub id: String,
    /// Shown when the guest walks in ("bounces in humming a little tune").
    pub arrival: String,
    /// The processing farewell line — the dark beat in the Lounge reveal.
    pub farewell: String,
    /// Idle table-talk lines shown in ambient speech bubbles.
    #[serde(default)]
    pub chatter: Vec<String>,
}

/// A timed dining situation (rush, inspector, critic, generous mood).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    pub id: String,
    pub name: String,
    pub announcement: String,
    pub description: String,
    pub duration_ms: f32,
    pub weight: u32,
    /// Earliest day this event can fire.
    pub min_day: u32,
    pub effect: EventEffect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventEffect {
    /// Spawn interval is multiplied (below 1.0 = faster arrivals).
    SpawnRush { multiplier: f32 },
    /// The Last Meal Lounge cannot be used.
    LoungeClosed,
    /// Serving courses earns multiplied renown.
    ServeRenownMultiplier { multiplier: f64 },
    /// Departure tips are multiplied.
    TipMultiplier { multiplier: f64 },
}

/// A permanent bonus chosen when prestiging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrestigePerkDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effect: PerkEffect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PerkEffect {
    KeepClientele,
    KeepSpecialization,
    StartingCash { amount: i64 },
    StartingMeat { meat: String, amount: i64 },
}

/// Player-facing behavior of a special trait: how it telegraphs, what the
/// counterplay is, and the one-time hint shown on first encounter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitBehavior {
    #[serde(rename = "trait")]
    pub trait_key: String,
    pub name: String,
    pub telegraph: String,
    pub counter: String,
    pub hint: String,
    pub telegraphed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBalance {
    pub customer_spawn_interval: f32,
    pub initial_spawn_delays: Vec<f32>,
    pub base_satisfaction_gain: f32,
    pub preferred_satisfaction_gain: f32,
    pub max_satisfaction_per_type: f32,
    pub max_feeding_capacity_bonus: i64,
    pub vip_deliciousness_threshold: f32,
    pub vip_satisfaction_threshold: f32,
    pub base_score_multiplier: f64,
    pub preferred_dish_score_multiplier: f64,
    pub vip_points_per_deliciousness: f64,
    pub max_customers: u32,
    pub overfeed_multiplier: f32,
    pub customer_patience_time: f32,
    pub trait_tick_interval: f32,
    pub satisfaction_decay_interval: f32,
    pub prestige_score_requirement: i64,
    pub regular_ingredient_name: String,
    pub satisfaction_decay_rate: f32,
    pub overfeed_multiplier_for_low_appetite: f32,
    pub special_table_process_time: f32,
    pub cooking_slots_limit: usize,
    pub starting_regular_ingredients: i64,
    #[serde(default = "default_dish_bill_value")]
    pub dish_bill_value: i64,
    #[serde(default = "default_preferred_bill_multiplier")]
    pub preferred_bill_multiplier: f64,
    #[serde(default = "default_satisfied_tip_rate")]
    pub satisfied_tip_rate: f64,
    #[serde(default = "default_content_dwell_ms")]
    pub content_dwell_ms: f32,
    #[serde(default = "default_min_courses")]
    pub min_courses: u32,
    #[serde(default = "default_max_courses")]
    pub max_courses: u32,
    #[serde(default = "default_visits_until_ready")]
    pub visits_until_ready: u32,
    /// Visits needed before a guest is Lounge-ready, indexed by profile tier
    /// (tier 1 uses index 0). Cheap early tiers land the hook fast; later
    /// tiers stay a longer investment. Falls back to `visits_until_ready`
    /// when empty.
    #[serde(default)]
    pub visits_until_ready_by_tier: Vec<u32>,
    /// Plated dishes served within this window pay the fresh bonus.
    #[serde(default = "default_dish_fresh_window_ms")]
    pub dish_fresh_window_ms: f32,
    /// Plated dishes older than this are discarded from the pass.
    #[serde(default = "default_dish_spoil_ms")]
    pub dish_spoil_ms: f32,
    #[serde(default = "default_fresh_bill_bonus_multiplier")]
    pub fresh_bill_bonus_multiplier: f64,
    /// How long a telegraphed trait warns before it resolves.
    #[serde(default = "default_trait_telegraph_ms")]
    pub trait_telegraph_ms: f32,
    /// Every Nth combo pays a streak bonus.
    #[serde(default = "default_combo_milestone_interval")]
    pub combo_milestone_interval: u32,
    #[serde(default = "default_combo_milestone_cash")]
    pub combo_milestone_cash: i64,
    /// Renown per seated guest when every table's order is complete at once.
    #[serde(default = "default_full_room_bonus_points")]
    pub full_room_bonus_points: i64,
    /// Length of one service day.
    #[serde(default = "default_day_length_ms")]
    pub day_length_ms: f32,
    /// How far into a day its dining event fires (0..1).
    #[serde(default = "default_event_day_fraction")]
    pub event_day_fraction: f32,
    /// Satisfied visits after which a guest counts as a regular.
    #[serde(default = "default_regular_visits_threshold")]
    pub regular_visits_threshold: u32,
    /// Meat-yield multiplier for processing a regular.
    #[serde(default = "default_regular_yield_multiplier")]
    pub regular_yield_multiplier: f32,
    /// Each prestige multiplies the next renown requirement by this.
    #[serde(default = "default_prestige_requirement_growth")]
    pub prestige_requirement_growth: f64,
    /// How long a guest spends eating a served course before wanting the next.
    #[serde(default = "default_course_eating_ms")]
    pub course_eating_ms: f32,
    /// After eating, how long a guest happily waits for the next course.
    #[serde(default = "default_course_wait_grace_ms")]
    pub course_wait_grace_ms: f32,
    /// Renown multiplier for serving a course while they are still eating.
    #[serde(default = "default_rushed_course_score_multiplier")]
    pub rushed_course_score_multiplier: f64,
    /// Renown multiplier for a well-paced course (after eating, within grace).
    #[serde(default = "default_paced_course_score_multiplier")]
    pub paced_course_score_multiplier: f64,
    /// Renown multiplier for a course that arrives after the grace window.
    #[serde(default = "default_late_course_score_multiplier")]
    pub late_course_score_multiplier: f64,
    /// Satisfaction lost per second while kept waiting past the grace window.
    #[serde(default = "default_hangry_satisfaction_decay_per_s")]
    pub hangry_satisfaction_decay_per_s: f32,
}

fn default_min_courses() -> u32 {
    1
}

fn default_max_courses() -> u32 {
    3
}

fn default_visits_until_ready() -> u32 {
    5
}

fn default_dish_bill_value() -> i64 {
    6
}

fn default_preferred_bill_multiplier() -> f64 {
    2.0
}

fn default_satisfied_tip_rate() -> f64 {
    0.5
}

fn default_content_dwell_ms() -> f32 {
    4_000.0
}

fn default_dish_fresh_window_ms() -> f32 {
    12_000.0
}

fn default_dish_spoil_ms() -> f32 {
    32_000.0
}

fn default_fresh_bill_bonus_multiplier() -> f64 {
    1.25
}

fn default_trait_telegraph_ms() -> f32 {
    5_000.0
}

fn default_combo_milestone_interval() -> u32 {
    5
}

fn default_combo_milestone_cash() -> i64 {
    12
}

fn default_full_room_bonus_points() -> i64 {
    40
}

fn default_day_length_ms() -> f32 {
    180_000.0
}

fn default_event_day_fraction() -> f32 {
    0.35
}

fn default_regular_visits_threshold() -> u32 {
    3
}

fn default_regular_yield_multiplier() -> f32 {
    1.5
}

fn default_prestige_requirement_growth() -> f64 {
    1.6
}

fn default_course_eating_ms() -> f32 {
    8_000.0
}

fn default_course_wait_grace_ms() -> f32 {
    14_000.0
}

fn default_rushed_course_score_multiplier() -> f64 {
    0.5
}

fn default_paced_course_score_multiplier() -> f64 {
    1.2
}

fn default_late_course_score_multiplier() -> f64 {
    0.75
}

fn default_hangry_satisfaction_decay_per_s() -> f32 {
    1.2
}

impl Default for GameBalance {
    fn default() -> Self {
        Self {
            customer_spawn_interval: 15_000.0,
            initial_spawn_delays: vec![4_000.0, 11_000.0],
            base_satisfaction_gain: 8.0,
            preferred_satisfaction_gain: 12.0,
            max_satisfaction_per_type: 40.0,
            max_feeding_capacity_bonus: 80,
            vip_deliciousness_threshold: 3.0,
            vip_satisfaction_threshold: 120.0,
            base_score_multiplier: 1.0,
            preferred_dish_score_multiplier: 2.0,
            vip_points_per_deliciousness: 140.0,
            max_customers: 2,
            overfeed_multiplier: 1.5,
            customer_patience_time: 135_000.0,
            trait_tick_interval: 8_000.0,
            satisfaction_decay_interval: 3_000.0,
            prestige_score_requirement: 50_000,
            regular_ingredient_name: "regular".to_string(),
            satisfaction_decay_rate: 0.5,
            overfeed_multiplier_for_low_appetite: 1.25,
            special_table_process_time: 3_000.0,
            cooking_slots_limit: 3,
            starting_regular_ingredients: -1,
            dish_bill_value: 6,
            preferred_bill_multiplier: 2.0,
            satisfied_tip_rate: 0.5,
            content_dwell_ms: 2_000.0,
            min_courses: 1,
            max_courses: 3,
            visits_until_ready: 5,
            visits_until_ready_by_tier: vec![2, 3, 4, 5],
            day_length_ms: default_day_length_ms(),
            event_day_fraction: default_event_day_fraction(),
            regular_visits_threshold: default_regular_visits_threshold(),
            regular_yield_multiplier: default_regular_yield_multiplier(),
            prestige_requirement_growth: default_prestige_requirement_growth(),
            course_eating_ms: default_course_eating_ms(),
            course_wait_grace_ms: default_course_wait_grace_ms(),
            rushed_course_score_multiplier: default_rushed_course_score_multiplier(),
            paced_course_score_multiplier: default_paced_course_score_multiplier(),
            late_course_score_multiplier: default_late_course_score_multiplier(),
            hangry_satisfaction_decay_per_s: default_hangry_satisfaction_decay_per_s(),
            dish_fresh_window_ms: default_dish_fresh_window_ms(),
            dish_spoil_ms: default_dish_spoil_ms(),
            fresh_bill_bonus_multiplier: default_fresh_bill_bonus_multiplier(),
            trait_telegraph_ms: default_trait_telegraph_ms(),
            combo_milestone_interval: default_combo_milestone_interval(),
            combo_milestone_cash: default_combo_milestone_cash(),
            full_room_bonus_points: default_full_room_bonus_points(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub customer_types: Vec<CustomerType>,
    pub dish_types: Vec<DishType>,
    pub upgrades: Vec<Upgrade>,
    pub recipes: Vec<Recipe>,
    pub achievements: Vec<Achievement>,
    pub tutorial_steps: Vec<TutorialStep>,
    pub specializations: Vec<SpecializationDef>,
    pub trait_behaviors: Vec<TraitBehavior>,
    pub regulars: RegularsData,
    pub dining_events: Vec<EventDef>,
    pub prestige_perks: Vec<PrestigePerkDef>,
    pub balance: GameBalance,
}

impl Default for RegularsData {
    fn default() -> Self {
        Self {
            names: vec!["Guest".to_string()],
            personalities: Vec::new(),
        }
    }
}

impl GameData {
    /// Parse the shipped catalog without consulting runtime override files.
    /// Integration tests use this strict path so a broken embedded asset can
    /// never be hidden by the normal runtime fallback policy.
    pub fn load_embedded_strict() -> Result<Self, String> {
        Ok(Self {
            customer_types: parse_json_labeled(
                "assets/data/customer_types.json",
                CUSTOMER_TYPES_JSON,
            )?,
            dish_types: parse_json_labeled("assets/data/dish_types.json", DISH_TYPES_JSON)?,
            upgrades: parse_json_labeled("assets/data/upgrades.json", UPGRADES_JSON)?,
            recipes: parse_json_labeled("assets/data/recipes.json", RECIPES_JSON)?,
            achievements: parse_json_labeled("assets/data/achievements.json", ACHIEVEMENTS_JSON)?,
            tutorial_steps: parse_json_labeled("assets/data/tutorial.json", TUTORIAL_JSON)?,
            specializations: parse_json_labeled(
                "assets/data/specializations.json",
                SPECIALIZATIONS_JSON,
            )?,
            trait_behaviors: parse_json_labeled(
                "assets/data/trait_behaviors.json",
                TRAIT_BEHAVIORS_JSON,
            )?,
            regulars: parse_json_labeled("assets/data/regulars.json", REGULARS_JSON)?,
            dining_events: parse_json_labeled(
                "assets/data/dining_events.json",
                DINING_EVENTS_JSON,
            )?,
            prestige_perks: parse_json_labeled(
                "assets/data/prestige_perks.json",
                PRESTIGE_PERKS_JSON,
            )?,
            balance: parse_json_labeled("assets/data/game_balance.json", GAME_BALANCE_JSON)?,
        })
    }

    pub fn load() -> Self {
        Self {
            customer_types: load_json_file_with_fallback_sync(
                "assets/data/customer_types.json",
                CUSTOMER_TYPES_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            dish_types: load_json_file_with_fallback_sync(
                "assets/data/dish_types.json",
                DISH_TYPES_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            upgrades: load_json_file_with_fallback_sync(
                "assets/data/upgrades.json",
                UPGRADES_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            recipes: load_json_file_with_fallback_sync(
                "assets/data/recipes.json",
                RECIPES_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            achievements: load_json_file_with_fallback_sync(
                "assets/data/achievements.json",
                ACHIEVEMENTS_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            tutorial_steps: load_json_file_with_fallback_sync(
                "assets/data/tutorial.json",
                TUTORIAL_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            specializations: load_json_file_with_fallback_sync(
                "assets/data/specializations.json",
                SPECIALIZATIONS_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            trait_behaviors: load_json_file_with_fallback_sync(
                "assets/data/trait_behaviors.json",
                TRAIT_BEHAVIORS_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            regulars: load_json_file_with_fallback_sync(
                "assets/data/regulars.json",
                REGULARS_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            dining_events: load_json_file_with_fallback_sync(
                "assets/data/dining_events.json",
                DINING_EVENTS_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            prestige_perks: load_json_file_with_fallback_sync(
                "assets/data/prestige_perks.json",
                PRESTIGE_PERKS_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
            balance: load_json_file_with_fallback_sync(
                "assets/data/game_balance.json",
                GAME_BALANCE_JSON,
                JsonFallbackPolicy::ReadOrParseError,
            )
            .unwrap_or_default(),
        }
    }

    pub fn customer_type_by_id(&self, id: &str) -> Option<&CustomerType> {
        self.customer_types.iter().find(|item| item.id == id)
    }

    pub fn dish_type_by_color(&self, color: &str) -> Option<&DishType> {
        self.dish_types.iter().find(|item| item.color == color)
    }

    pub fn specialization_by_id(&self, id: &str) -> Option<&SpecializationDef> {
        self.specializations.iter().find(|item| item.id == id)
    }

    pub fn trait_behavior(&self, trait_key: &str) -> Option<&TraitBehavior> {
        self.trait_behaviors
            .iter()
            .find(|item| item.trait_key == trait_key)
    }

    pub fn dining_event_by_id(&self, id: &str) -> Option<&EventDef> {
        self.dining_events.iter().find(|item| item.id == id)
    }

    pub fn personality_by_id(&self, id: &str) -> Option<&PersonalityDef> {
        self.regulars
            .personalities
            .iter()
            .find(|item| item.id == id)
    }

    pub fn prestige_perk_by_id(&self, id: &str) -> Option<&PrestigePerkDef> {
        self.prestige_perks.iter().find(|item| item.id == id)
    }
}
