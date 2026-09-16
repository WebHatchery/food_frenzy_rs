//! Data-integrity tests: strict parses of every embedded JSON asset plus
//! cross-reference checks, split out of `data.rs` to respect the file-size
//! limit. Toolkit loading degrades to defaults at runtime; these are the
//! loud counterpart so broken content fails CI instead of shipping empty.

use feast_frenzy::data::{GameData, TutorialTrigger, STATION_COLORS};

fn parsed() -> GameData {
    GameData::load_embedded_strict().expect("embedded game data must parse")
}

// Keep in sync with the `get_effect` call sites in engine.rs/gameplay.rs;
// an unread effect key is a balance change that silently does nothing.
const KNOWN_EFFECTS: [&str; 9] = [
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

fn meat_key(customer_type_id: &str) -> String {
    // Must match the key minted in `gameplay.rs` when a guest is processed.
    format!("{customer_type_id}-meat")
}

fn assert_unique_ids(kind: &str, ids: &[&str]) {
    let mut seen = std::collections::HashSet::new();
    for id in ids {
        assert!(seen.insert(*id), "duplicate {kind} id: {id}");
    }
}

#[test]
fn all_embedded_assets_parse_strictly() {
    let data = parsed();
    assert!(!data.customer_types.is_empty(), "no customer types");
    assert!(!data.dish_types.is_empty(), "no dish types");
    assert!(!data.upgrades.is_empty(), "no upgrades");
    assert!(!data.recipes.is_empty(), "no recipes");
    assert!(!data.achievements.is_empty(), "no achievements");
}

#[test]
fn ids_are_unique_across_each_asset() {
    let data = parsed();
    assert_unique_ids(
        "customer type",
        &data
            .customer_types
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
    );
    assert_unique_ids(
        "upgrade",
        &data
            .upgrades
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
    );
    assert_unique_ids(
        "recipe",
        &data
            .recipes
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
    );
    assert_unique_ids(
        "achievement",
        &data
            .achievements
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
    );
}

#[test]
fn dish_types_cover_exactly_the_station_colors() {
    let data = parsed();
    let colors: Vec<&str> = data
        .dish_types
        .iter()
        .map(|dish| dish.color.as_str())
        .collect();
    assert_eq!(colors.len(), STATION_COLORS.len());
    for color in STATION_COLORS {
        assert!(colors.contains(&color), "missing dish for station {color}");
    }
    for dish in &data.dish_types {
        assert!(
            dish.cook_time_ms > 0.0,
            "{}: cook time must be > 0",
            dish.color
        );
        assert!(
            !dish.examples.is_empty(),
            "{}: needs example names",
            dish.color
        );
    }
}

#[test]
fn customer_types_reference_real_dishes_and_meats() {
    let data = parsed();
    let valid_ingredients: Vec<String> = data
        .customer_types
        .iter()
        .map(|item| meat_key(&item.id))
        .chain(std::iter::once(
            data.balance.regular_ingredient_name.clone(),
        ))
        .collect();

    for customer_type in &data.customer_types {
        assert!(
            !customer_type.preferred_dishes.is_empty(),
            "{}: needs at least one preferred dish",
            customer_type.id
        );
        for dish_color in &customer_type.preferred_dishes {
            assert!(
                data.dish_type_by_color(dish_color).is_some(),
                "{}: unknown preferred dish {dish_color}",
                customer_type.id
            );
        }
        for ingredient in customer_type.unlock_cost.keys() {
            assert!(
                valid_ingredients.contains(ingredient),
                "{}: unlock cost references unknown ingredient {ingredient}",
                customer_type.id
            );
        }
        assert!(
            customer_type.initially_unlocked || !customer_type.unlock_cost.is_empty(),
            "{}: locked type must have an unlock cost or it is unreachable",
            customer_type.id
        );
    }
}

#[test]
fn recipes_reference_real_customer_types_and_meats() {
    let data = parsed();
    let valid_ingredients: Vec<String> = data
        .customer_types
        .iter()
        .map(|item| meat_key(&item.id))
        .chain(std::iter::once(
            data.balance.regular_ingredient_name.clone(),
        ))
        .collect();

    for recipe in &data.recipes {
        assert!(
            !recipe.ingredients.is_empty(),
            "{}: no ingredients",
            recipe.id
        );
        for ingredient in recipe.ingredients.keys() {
            assert!(
                valid_ingredients.contains(ingredient),
                "{}: unknown ingredient {ingredient}",
                recipe.id
            );
        }
        if let Some(customer_type) = &recipe.customer_type {
            assert!(
                data.customer_type_by_id(customer_type).is_some(),
                "{}: unknown customer type {customer_type}",
                recipe.id
            );
        }
        assert!(
            recipe.base_value > 0,
            "{}: base value must be > 0",
            recipe.id
        );
    }
}

#[test]
fn upgrade_effects_only_use_keys_the_engine_reads() {
    let data = parsed();
    for upgrade in &data.upgrades {
        assert!(!upgrade.effects.is_empty(), "{}: no effects", upgrade.id);
        for key in upgrade.effects.keys() {
            assert!(
                KNOWN_EFFECTS.contains(&key.as_str()),
                "{}: effect key {key} is never read by the engine",
                upgrade.id
            );
        }
        assert!(
            upgrade.max_level >= 1,
            "{}: max level must be >= 1",
            upgrade.id
        );
        assert!(
            upgrade.cost_growth >= 1.0,
            "{}: cost growth below 1.0 makes upgrades cheaper over time",
            upgrade.id
        );
    }
}

#[test]
fn achievements_and_balance_are_sane() {
    let data = parsed();
    for achievement in &data.achievements {
        assert!(
            achievement.max_progress > 0,
            "{}: max progress must be > 0",
            achievement.id
        );
    }
    let balance = &data.balance;
    assert!(balance.max_customers >= 1);
    assert!(balance.visits_until_ready >= 1);
    assert!(balance.min_courses >= 1);
    assert!(balance.min_courses <= balance.max_courses);
    assert!(balance.max_courses as usize <= STATION_COLORS.len());
    assert!(balance.prestige_score_requirement > 0);
    assert!(balance.special_table_process_time > 0.0);
    assert!(
        !balance.visits_until_ready_by_tier.is_empty(),
        "tier readiness ladder drives early pacing"
    );
    for window in balance.visits_until_ready_by_tier.windows(2) {
        assert!(
            window[0] <= window[1],
            "readiness ladder must not get cheaper at higher tiers"
        );
    }
    assert!(balance
        .visits_until_ready_by_tier
        .iter()
        .all(|visits| *visits >= 1));
}

#[test]
fn specializations_are_real_tradeoffs_on_known_effect_keys() {
    let data = parsed();
    assert!(data.specializations.len() >= 3, "need a real choice");
    assert_unique_ids(
        "specialization",
        &data
            .specializations
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
    );
    for spec in &data.specializations {
        assert!(
            spec.effects.len() >= 2,
            "{}: a specialization with one effect is a buff, not a trade-off",
            spec.id
        );
        for key in spec.effects.keys() {
            assert!(
                KNOWN_EFFECTS.contains(&key.as_str()),
                "{}: effect key {key} is never read by the engine",
                spec.id
            );
        }
    }
}

#[test]
fn trait_behaviors_cover_every_special_trait_flag() {
    // One entry per bool on CustomerSpecialTraits, keyed by field name.
    const TRAIT_FLAGS: [&str; 11] = [
        "low_appetite",
        "can_wander",
        "multiplies_on_process",
        "fast_spoilage",
        "can_steal_food",
        "can_eat_waste",
        "high_yield",
        "throws_food",
        "big_tipper",
        "influencer",
        "gourmand",
    ];
    let data = parsed();
    for flag in TRAIT_FLAGS {
        let behavior = data
            .trait_behavior(flag)
            .unwrap_or_else(|| panic!("missing trait behavior for {flag}"));
        assert!(
            !behavior.name.is_empty() && !behavior.hint.is_empty(),
            "{flag}"
        );
        if behavior.telegraphed {
            assert!(
                !behavior.telegraph.is_empty(),
                "{flag}: telegraphed traits need telegraph text"
            );
        }
    }
    assert_unique_ids(
        "trait behavior",
        &data
            .trait_behaviors
            .iter()
            .map(|item| item.trait_key.as_str())
            .collect::<Vec<_>>(),
    );
}

#[test]
fn freshness_and_streak_tuning_is_sane() {
    let data = parsed();
    let balance = &data.balance;
    assert!(balance.dish_fresh_window_ms > 0.0);
    assert!(
        balance.dish_spoil_ms > balance.dish_fresh_window_ms,
        "dishes must go stale before they spoil"
    );
    assert!(balance.fresh_bill_bonus_multiplier >= 1.0);
    assert!(balance.trait_telegraph_ms > 0.0);
    assert!(balance.combo_milestone_interval >= 2);
    assert!(balance.combo_milestone_cash > 0);
    assert!(balance.full_room_bonus_points > 0);
}

#[test]
fn course_pacing_tuning_is_sane() {
    let data = parsed();
    let balance = &data.balance;
    assert!(balance.course_eating_ms > 0.0);
    assert!(balance.course_wait_grace_ms > 0.0);
    assert!(
        balance.rushed_course_score_multiplier < 1.0,
        "rushing a course must cost renown"
    );
    assert!(
        balance.late_course_score_multiplier < 1.0,
        "leaving a guest hungry must cost renown"
    );
    assert!(
        balance.paced_course_score_multiplier >= 1.0,
        "good pacing must never pay worse than neutral"
    );
    assert!(balance.hangry_satisfaction_decay_per_s >= 0.0);
    assert!(
        balance.course_eating_ms + balance.course_wait_grace_ms < balance.customer_patience_time,
        "a full pacing cycle must fit well inside overall patience"
    );
}

#[test]
fn recipe_unlock_requirements_reference_real_customer_types() {
    let data = parsed();
    assert!(data.recipes.len() >= 12, "content expansion: 12+ recipes");
    for recipe in &data.recipes {
        assert!(
            !recipe.unlock_requirements.is_empty(),
            "{}: recipes without unlock requirements can never unlock",
            recipe.id
        );
        for customer_type in recipe.unlock_requirements.keys() {
            assert!(
                data.customer_type_by_id(customer_type).is_some(),
                "{}: unknown customer type {customer_type}",
                recipe.id
            );
        }
    }
}

#[test]
fn regulars_events_and_perks_are_well_formed() {
    let data = parsed();
    assert!(
        data.regulars.names.len() >= 20,
        "name pool too small - duplicate guests everywhere"
    );
    assert!(data.regulars.personalities.len() >= 3);
    assert_unique_ids(
        "personality",
        &data
            .regulars
            .personalities
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
    );
    for personality in &data.regulars.personalities {
        assert!(
            !personality.arrival.is_empty() && !personality.farewell.is_empty(),
            "{}",
            personality.id
        );
    }

    assert!(
        data.dining_events.len() >= 6,
        "need event variety beyond the starter set"
    );
    assert_unique_ids(
        "dining event",
        &data
            .dining_events
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
    );
    for event in &data.dining_events {
        assert!(event.duration_ms > 0.0, "{}", event.id);
        assert!(event.weight > 0, "{}: zero weight never fires", event.id);
        assert!(event.min_day >= 1, "{}", event.id);
    }

    assert!(
        data.prestige_perks.len() >= 6,
        "prestige needs a real choice beyond the starter set"
    );
    assert_unique_ids(
        "prestige perk",
        &data
            .prestige_perks
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
    );
    assert!(data.balance.day_length_ms > 0.0);
    assert!(data.balance.event_day_fraction > 0.0 && data.balance.event_day_fraction < 1.0);
    assert!(data.balance.regular_visits_threshold >= 1);
    assert!(data.balance.regular_yield_multiplier >= 1.0);
    assert!(data.balance.prestige_requirement_growth >= 1.0);
}

#[test]
fn fifth_clientele_tier_has_a_complete_content_path() {
    let data = parsed();
    let dragon = data
        .customer_type_by_id("dragon")
        .expect("fifth-tier dragon clientele is part of the content contract");
    assert_eq!(dragon.profile_tier, 5);
    assert!(!dragon.unlock_cost.is_empty());
    assert!(dragon
        .unlock_cost
        .keys()
        .all(|ingredient| ingredient.ends_with("-meat")));
    assert!(data
        .recipes
        .iter()
        .any(|recipe| recipe.customer_type.as_deref() == Some("dragon")));
    assert!(data.balance.visits_until_ready_by_tier.len() >= 5);
    assert!(
        std::path::Path::new("assets/images/characters/dragon.png").is_file(),
        "fifth-tier clientele needs a portrait asset"
    );
}

#[test]
fn achievements_cover_the_new_systems() {
    let data = parsed();
    assert!(
        data.achievements.len() >= 20,
        "content expansion: ~20 achievements"
    );
    // Ids the progression code updates; a typo here silently dead-ends one.
    for id in [
        "day-one",
        "week-of-service",
        "regular-goodbye",
        "event-weathered",
        "fresh-guarantee",
        "streak-chef",
        "master-larder",
        "full-house",
        "deep-menu",
    ] {
        assert!(
            data.achievements.iter().any(|item| item.id == id),
            "missing achievement {id}"
        );
    }
}

#[test]
fn tutorial_steps_are_present_and_end_with_acknowledged() {
    let data = parsed();
    assert!(
        data.tutorial_steps.len() >= 5,
        "tutorial must cover the full cook->serve->fatten->process loop"
    );
    assert_unique_ids(
        "tutorial step",
        &data
            .tutorial_steps
            .iter()
            .map(|step| step.id.as_str())
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        data.tutorial_steps.last().map(|step| step.trigger),
        Some(TutorialTrigger::Acknowledged),
        "final step must wait for the player to acknowledge it"
    );
    for step in &data.tutorial_steps {
        assert!(
            !step.title.is_empty() && !step.body.is_empty(),
            "{}",
            step.id
        );
    }
}
