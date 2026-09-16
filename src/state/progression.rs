use super::ProgressionState;
use crate::data::GameData;
use std::collections::HashMap;

impl ProgressionState {
    pub fn from_game_data(data: &GameData) -> Self {
        let mut state = Self {
            currency: 0,
            upgrades: data.upgrades.clone(),
            recipes: data.recipes.clone(),
            achievements: data.achievements.clone(),
            prestige_level: 0,
            prestige_points: 0,
            total_score: 0,
            processed_customer_counts: HashMap::new(),
            processed_customer_types: Vec::new(),
            feeding_capacity_bonus: 0,
            crafted_recipe_counts: HashMap::new(),
            total_dishes_served: 0,
            preferred_dishes_served: 0,
            overfed_customer_count: 0,
            customers_lost: 0,
            unlocked_customer_types: starting_customer_type_ids(data),
            specialization: None,
            specialization_effects: HashMap::new(),
            seen_trait_hints: Vec::new(),
            days_completed: 0,
            regulars_processed: 0,
            events_completed: 0,
            fresh_dishes_served: 0,
            full_house_bonuses: 0,
            prestige_perks: Vec::new(),
        };

        state.ensure_customer_unlocks(data);
        state.set_upgrade_costs();
        state
    }

    pub fn ensure_customer_unlocks(&mut self, data: &GameData) {
        for customer_type in &data.customer_types {
            if customer_type.initially_unlocked
                && !self
                    .unlocked_customer_types
                    .iter()
                    .any(|item| item == &customer_type.id)
            {
                self.unlocked_customer_types.push(customer_type.id.clone());
            }
        }

        if self.unlocked_customer_types.is_empty() {
            if let Some(customer_type) = data.customer_types.first() {
                self.unlocked_customer_types.push(customer_type.id.clone());
            }
        }
    }

    pub fn is_customer_unlocked(&self, customer_type_id: &str) -> bool {
        self.unlocked_customer_types
            .iter()
            .any(|item| item == customer_type_id)
    }

    pub fn unlock_customer_type(&mut self, customer_type_id: &str) -> bool {
        if self.is_customer_unlocked(customer_type_id) {
            return false;
        }

        self.unlocked_customer_types
            .push(customer_type_id.to_string());
        true
    }

    pub fn add_currency(&mut self, amount: i64) {
        if amount > 0 {
            self.currency = self.currency.saturating_add(amount);
        }
    }

    pub fn spend_currency(&mut self, amount: i64) -> bool {
        if self.currency < amount {
            return false;
        }

        self.currency -= amount;
        true
    }

    pub fn get_effect(&self, key: &str, fallback: f64) -> f64 {
        let mut total = fallback;
        for upgrade in &self.upgrades {
            if let Some(value) = upgrade.effects.get(key) {
                total += value * upgrade.level as f64;
            }
        }
        if let Some(value) = self.specialization_effects.get(key) {
            total += value;
        }

        if (fallback - 1.0).abs() < f64::EPSILON {
            total.max(0.25)
        } else {
            total
        }
    }

    /// Commit to a house style. One choice per run; prestige clears it.
    pub fn choose_specialization(&mut self, def: &crate::data::SpecializationDef) -> bool {
        if self.specialization.is_some() {
            return false;
        }
        self.specialization = Some(def.id.clone());
        self.specialization_effects = def.effects.clone();
        true
    }

    /// True the first time a trait key is reported; marks it seen.
    pub fn note_trait_encounter(&mut self, trait_key: &str) -> bool {
        if self.seen_trait_hints.iter().any(|seen| seen == trait_key) {
            return false;
        }
        self.seen_trait_hints.push(trait_key.to_string());
        true
    }

    pub fn set_upgrade_costs(&mut self) {
        for upgrade in &mut self.upgrades {
            let level = upgrade.level as f64;
            upgrade.cost = (((upgrade.base_cost as f64) * upgrade.cost_growth.powf(level)).ceil()
                as i64)
                .max(1);
        }
    }

    pub fn buy_upgrade(&mut self, upgrade_id: &str) -> bool {
        let Some(index) = self.upgrades.iter().position(|item| item.id == upgrade_id) else {
            return false;
        };

        if self.upgrades[index].level >= self.upgrades[index].max_level {
            return false;
        }

        let cost = self.upgrades[index].cost;
        if !self.spend_currency(cost) {
            return false;
        }

        let upgrade = &mut self.upgrades[index];
        upgrade.level = upgrade.level.saturating_add(1);
        if upgrade.level >= upgrade.max_level {
            upgrade.purchased = true;
            upgrade.level = upgrade.max_level;
        }
        self.set_upgrade_costs();
        true
    }

    pub fn record_score(&mut self, amount: i64) {
        let scored = amount.max(0);
        self.total_score = self.total_score.saturating_add(scored);
        self.update_achievement("busy-night", self.total_score);
        self.update_achievement("restaurant-empire", self.total_score);
    }

    pub fn record_served_dish(&mut self, is_preferred: bool, is_overfed: bool) {
        self.total_dishes_served = self.total_dishes_served.saturating_add(1);
        if is_preferred {
            self.preferred_dishes_served = self.preferred_dishes_served.saturating_add(1);
        }
        if is_overfed {
            self.overfed_customer_count = self.overfed_customer_count.saturating_add(1);
        }

        self.update_achievement("steady-service", self.total_dishes_served);
        self.update_achievement("favorite-service", self.preferred_dishes_served);
        self.update_achievement("overfed-specialist", self.overfed_customer_count);
    }

    pub fn record_processed_customer(&mut self, customer_type: &str, chain_length: u32) {
        let count = self
            .processed_customer_counts
            .entry(customer_type.to_string())
            .or_insert(0);
        *count = count.saturating_add(1);

        if !self
            .processed_customer_types
            .iter()
            .any(|item| item == customer_type)
        {
            self.processed_customer_types
                .push(customer_type.to_string());
        }

        self.update_recipe_unlocks();
        self.update_achievement("first-customer", 1);
        self.update_achievement("combo-master", i64::from(chain_length));
        self.update_achievement(
            "broad-menu",
            i64::try_from(self.processed_customer_types.len()).unwrap_or(0),
        );
    }

    /// Data-driven: a recipe unlocks once every processed-count requirement in
    /// its `unlock_requirements` table is met.
    fn update_recipe_unlocks(&mut self) {
        let counts = self.processed_customer_counts.clone();
        let mut unlocked_total = 0i64;
        for recipe in &mut self.recipes {
            if !recipe.unlocked
                && !recipe.unlock_requirements.is_empty()
                && recipe
                    .unlock_requirements
                    .iter()
                    .all(|(customer_type, needed)| {
                        counts.get(customer_type).copied().unwrap_or(0) >= *needed
                    })
            {
                recipe.unlocked = true;
            }
            if recipe.unlocked {
                unlocked_total += 1;
            }
        }
        self.update_achievement("deep-menu", unlocked_total);
    }

    pub fn record_day_completed(&mut self) {
        self.days_completed = self.days_completed.saturating_add(1);
        self.update_achievement("day-one", self.days_completed);
        self.update_achievement("week-of-service", self.days_completed);
    }

    pub fn record_regular_processed(&mut self) {
        self.regulars_processed = self.regulars_processed.saturating_add(1);
        self.update_achievement("regular-goodbye", self.regulars_processed);
    }

    pub fn record_event_completed(&mut self) {
        self.events_completed = self.events_completed.saturating_add(1);
        self.update_achievement("event-weathered", self.events_completed);
    }

    pub fn record_fresh_dish(&mut self) {
        self.fresh_dishes_served = self.fresh_dishes_served.saturating_add(1);
        self.update_achievement("fresh-guarantee", self.fresh_dishes_served);
    }

    pub fn record_full_house(&mut self) {
        self.full_house_bonuses = self.full_house_bonuses.saturating_add(1);
        self.update_achievement("full-house", self.full_house_bonuses);
    }

    pub fn record_combo_peak(&mut self, combo: u32) {
        self.update_achievement("streak-chef", i64::from(combo));
    }

    pub fn record_larder_total(&mut self, total_meat: i64) {
        self.update_achievement("master-larder", total_meat);
    }

    pub fn record_crafted_recipe(&mut self, recipe_id: &str, capacity_bonus: i64) {
        self.feeding_capacity_bonus = (self.feeding_capacity_bonus + capacity_bonus.max(0))
            .min(self.achievements_maximum_capacity_bonus());

        let next = self
            .crafted_recipe_counts
            .entry(recipe_id.to_string())
            .or_insert(0);
        *next = next.saturating_add(1);

        let total_crafted = self
            .crafted_recipe_counts
            .values()
            .map(|count| i64::from(*count))
            .sum::<i64>();

        self.update_achievement("recipe-merchant", total_crafted);
        self.update_achievement("capacity-planner", self.feeding_capacity_bonus);
    }

    fn achievements_maximum_capacity_bonus(&self) -> i64 {
        80
    }

    pub fn record_customer_lost(&mut self) {
        self.customers_lost = self.customers_lost.saturating_add(1);
    }

    fn update_achievement(&mut self, id: &str, progress: i64) {
        if let Some(achievement) = self.achievements.iter_mut().find(|item| item.id == id) {
            if achievement.progress < progress {
                achievement.progress = progress.min(achievement.max_progress);
            }

            if achievement.progress >= achievement.max_progress {
                achievement.unlocked = true;
            }
        }
    }

    pub fn can_prestige(&self, requirement: i64) -> bool {
        self.total_score >= requirement
    }

    pub fn prestige_reward(&self) -> i64 {
        let score_reward = (self.total_score / 10_000).max(0);
        let achievement_reward = self
            .achievements
            .iter()
            .filter(|item| item.unlocked)
            .count();
        let capacity_reward = (self.feeding_capacity_bonus / 20).max(0);
        (score_reward + achievement_reward as i64 + capacity_reward).max(1)
    }

    /// Reset for permanent multipliers, keeping whatever the chosen perk
    /// preserves. Pass `None` for a plain reset (old saves, tests).
    pub fn prestige(&mut self, data: &GameData, perk: Option<&crate::data::PrestigePerkDef>) {
        use crate::data::PerkEffect;

        let reward = self.prestige_reward();
        let kept_clientele = self.unlocked_customer_types.clone();
        let kept_specialization = (
            self.specialization.clone(),
            self.specialization_effects.clone(),
        );

        self.currency = self.currency.saturating_add(reward);
        self.prestige_level = self.prestige_level.saturating_add(1);
        self.prestige_points = self
            .prestige_points
            .saturating_add(i32::try_from(reward).unwrap_or(i32::MAX));

        self.total_score = 0;
        self.processed_customer_counts.clear();
        self.processed_customer_types.clear();
        self.feeding_capacity_bonus = 0;
        self.crafted_recipe_counts.clear();
        self.total_dishes_served = 0;
        self.preferred_dishes_served = 0;
        self.overfed_customer_count = 0;
        self.customers_lost = 0;
        self.unlocked_customer_types = starting_customer_type_ids(data);
        self.specialization = None;
        self.specialization_effects.clear();

        self.upgrades = data.upgrades.clone();
        self.recipes = data.recipes.clone();
        self.set_upgrade_costs();

        for achievement in &mut self.achievements {
            achievement.unlocked = false;
            achievement.progress = 0;
        }

        if let Some(item) = self
            .achievements
            .iter_mut()
            .find(|item| item.id == "fresh-start")
        {
            item.unlocked = true;
            item.progress = item.max_progress;
        }

        if let Some(perk) = perk {
            self.prestige_perks.push(perk.id.clone());
            match &perk.effect {
                PerkEffect::KeepClientele => {
                    self.unlocked_customer_types = kept_clientele;
                }
                PerkEffect::KeepSpecialization => {
                    let (specialization, effects) = kept_specialization;
                    self.specialization = specialization;
                    self.specialization_effects = effects;
                }
                PerkEffect::StartingCash { amount } => {
                    self.currency = self.currency.saturating_add((*amount).max(0));
                }
                // StartingMeat is applied by the caller: meat lives on
                // GameState, not progression.
                PerkEffect::StartingMeat { .. } => {}
            }
        }
    }
}

fn starting_customer_type_ids(data: &GameData) -> Vec<String> {
    let mut ids: Vec<String> = data
        .customer_types
        .iter()
        .filter(|customer_type| customer_type.initially_unlocked)
        .map(|customer_type| customer_type.id.clone())
        .collect();

    if ids.is_empty() {
        if let Some(customer_type) = data.customer_types.first() {
            ids.push(customer_type.id.clone());
        }
    }

    ids
}
