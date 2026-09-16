//! Semantic validation for the typed Feast Frenzy content catalog.

use super::{GameData, GoalKind, PerkEffect};
use std::collections::HashSet;

impl GameData {
    pub fn validate(&self) -> Result<(), String> {
        self.validate_ids()?;
        self.validate_references()?;
        self.validate_goals()?;
        self.validate_balance()
    }

    fn validate_ids(&self) -> Result<(), String> {
        let collections = [
            (
                "customer type",
                self.customer_types
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "dish type",
                self.dish_types
                    .iter()
                    .map(|item| item.color.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "upgrade",
                self.upgrades
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "recipe",
                self.recipes
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "achievement",
                self.achievements
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "tutorial step",
                self.tutorial_steps
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "specialization",
                self.specializations
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "trait behavior",
                self.trait_behaviors
                    .iter()
                    .map(|item| item.trait_key.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "dining event",
                self.dining_events
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "prestige perk",
                self.prestige_perks
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "goal",
                self.goals
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<Vec<_>>(),
            ),
        ];
        for (kind, ids) in collections {
            if ids.iter().any(|id| id.trim().is_empty()) {
                return Err(format!("{kind} contains an empty id"));
            }
            let mut seen = HashSet::new();
            if ids.iter().any(|id| !seen.insert(*id)) {
                return Err(format!("duplicate {kind} id"));
            }
        }
        Ok(())
    }

    fn validate_goals(&self) -> Result<(), String> {
        if self
            .ui_text
            .entries
            .values()
            .any(|value| value.trim().is_empty())
        {
            return Err("ui text catalog contains an empty value".to_string());
        }
        for goal in &self.goals {
            if goal.min_day == 0 {
                return Err(format!("goal {} has an invalid minimum day", goal.id));
            }
            let valid = match goal.kind {
                GoalKind::ServeCourses { target } | GoalKind::ServeFreshDishes { target } => {
                    target > 0
                }
                GoalKind::CompleteDiningEvent
                | GoalKind::ProcessRegular
                | GoalKind::AttractClientele
                | GoalKind::Prestige => true,
            };
            if !valid {
                return Err(format!("goal {} has a non-positive target", goal.id));
            }
        }
        Ok(())
    }

    fn validate_references(&self) -> Result<(), String> {
        if self.customer_types.is_empty() || self.dish_types.is_empty() {
            return Err("customer and dish catalogs must not be empty".to_string());
        }
        let known_meat = |ingredient: &str| {
            self.customer_types
                .iter()
                .any(|item| ingredient == format!("{}-meat", item.id))
        };
        for customer in &self.customer_types {
            if customer.preferred_dishes.is_empty()
                || customer
                    .preferred_dishes
                    .iter()
                    .any(|dish| self.dish_type_by_color(dish).is_none())
            {
                return Err(format!(
                    "{} has an invalid preferred dish list",
                    customer.id
                ));
            }
            if customer.unlock_cost.keys().any(|ingredient| {
                ingredient != &self.balance.regular_ingredient_name && !known_meat(ingredient)
            }) {
                return Err(format!("{} unlocks with unknown ingredient", customer.id));
            }
            if !customer.initially_unlocked && customer.unlock_cost.is_empty() {
                return Err(format!(
                    "locked customer {} has no unlock cost",
                    customer.id
                ));
            }
        }
        const EFFECT_KEYS: [&str; 9] = [
            "capacity_gain_multiplier",
            "combo_multiplier",
            "cook_time_multiplier",
            "max_customers_bonus",
            "meat_yield_multiplier",
            "patience_multiplier",
            "recipe_value_multiplier",
            "satisfaction_decay_multiplier",
            "spawn_interval_multiplier",
        ];
        for upgrade in &self.upgrades {
            if upgrade
                .effects
                .keys()
                .any(|key| !EFFECT_KEYS.contains(&key.as_str()))
            {
                return Err(format!("{} contains an unread upgrade effect", upgrade.id));
            }
        }
        for recipe in &self.recipes {
            if recipe.ingredients.keys().any(|ingredient| {
                ingredient != &self.balance.regular_ingredient_name && !known_meat(ingredient)
            }) || recipe
                .customer_type
                .as_ref()
                .is_some_and(|id| self.customer_type_by_id(id).is_none())
                || recipe
                    .unlock_requirements
                    .keys()
                    .any(|id| self.customer_type_by_id(id).is_none())
            {
                return Err(format!("{} contains an invalid reference", recipe.id));
            }
        }
        for perk in &self.prestige_perks {
            if let PerkEffect::StartingMeat { meat, .. } = &perk.effect {
                if !known_meat(meat) {
                    return Err(format!("{} starts with unknown meat {meat}", perk.id));
                }
            }
        }
        Ok(())
    }

    fn validate_balance(&self) -> Result<(), String> {
        let balance = &self.balance;
        let positive = [
            balance.customer_spawn_interval,
            balance.customer_patience_time,
            balance.day_length_ms,
            balance.player_walk_speed,
            balance.customer_walk_speed,
            balance.restaurant_floor_width,
            balance.restaurant_floor_height,
            balance.player_interaction_range,
            balance.cinematic.reveal_ms,
        ];
        if positive.iter().any(|value| *value <= 0.0) {
            return Err("balance contains a non-positive required value".to_string());
        }
        let probabilities = [
            balance.returning_guest_chance,
            balance.fox_steal_chance,
            balance.can_wander_chance,
            balance.monkey_throw_chance,
            balance.vip_accept_chance,
        ];
        if probabilities
            .iter()
            .any(|value| !(0.0..=1.0).contains(value))
        {
            return Err("gameplay probabilities must be between 0 and 1".to_string());
        }
        if balance.min_courses == 0
            || balance.min_courses > balance.max_courses
            || balance.max_courses as usize > self.dish_types.len()
            || balance.dish_spoil_ms <= balance.dish_fresh_window_ms
            || !(0.0..1.0).contains(&balance.event_day_fraction)
            || balance.prestige_requirement_growth < 1.0
        {
            return Err(
                "balance course, freshness, event, or prestige invariant failed".to_string(),
            );
        }
        Ok(())
    }
}
