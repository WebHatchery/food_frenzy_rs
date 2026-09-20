//! Player-facing actions: cooking, serving, clientele growth, crafting, and
//! prestige. UI code calls these explicit mutations through commands.

use crate::data::{EventEffect, GameData, PerkEffect, TutorialTrigger};
use crate::engine::{
    can_process_customer, classify_course_pacing, classify_dish_age, cooking_time_ms,
    freshness_bill_multiplier, is_regular, overfeed_multiplier, pacing_score_multiplier,
    prestige_requirement, recipe_capacity_gain, recipe_value_multiplier, serving_bill,
    serving_gain, serving_points, vip_meat_gain, vip_points, visits_until_ready_for, CoursePacing,
    Freshness,
};
use crate::state::{
    FloaterAnchor, FloaterKind, GameState, GuestState, ProcessingCinematic, ProgressionState,
    INFINITE_INGREDIENTS,
};

pub fn dish_display_name(data: &GameData, color: &str) -> String {
    data.dish_type_by_color(color)
        .map(|dish| dish.name.clone())
        .unwrap_or_else(|| data.text("ui_dish_unknown").to_string())
}

pub fn try_prestige(
    data: &GameData,
    progression: &mut ProgressionState,
    game_state: &mut GameState,
) {
    let requirement = prestige_requirement(data, progression);
    if !progression.can_prestige(requirement) {
        game_state.add_message(data.text_format(
            "message_prestige_needs",
            [("requirement", requirement.to_string())].as_slice(),
        ));
        return;
    }
    if data.prestige_perks.is_empty() {
        progression.prestige(data, None);
        announce_prestige(data, game_state, progression);
    } else {
        // Opens the perk-choice modal; `confirm_prestige` finishes the job.
        game_state.pending_prestige = true;
        game_state.prestige_page = 0;
        game_state.selected_prestige_perk = None;
    }
}

/// Apply the chosen perk and run the reset (the perk modal's commit path).
pub fn confirm_prestige(
    perk_id: &str,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    let requirement = prestige_requirement(data, progression);
    let Some(perk) = data.prestige_perk_by_id(perk_id) else {
        return;
    };
    if !progression.can_prestige(requirement) {
        return;
    }
    game_state.pending_prestige = false;
    game_state.prestige_page = 0;
    game_state.selected_prestige_perk = None;
    progression.prestige(data, Some(perk));
    if let PerkEffect::StartingMeat { meat, amount } = &perk.effect {
        let entry = game_state.ingredients.entry(meat.clone()).or_insert(0);
        if *entry != INFINITE_INGREDIENTS {
            *entry = entry.saturating_add((*amount).max(0));
        }
    }
    game_state.add_message(data.text_format(
        "message_perk_secured",
        [("perk", perk.name.clone())].as_slice(),
    ));
    announce_prestige(data, game_state, progression);
}

fn announce_prestige(data: &GameData, game_state: &mut GameState, progression: &ProgressionState) {
    game_state.add_message(data.text_format(
        "message_prestige_complete",
        [("level", progression.prestige_level.to_string())].as_slice(),
    ));
    game_state.floaters.spawn(
        data.text_format(
            "message_prestige_level",
            [("level", progression.prestige_level.to_string())].as_slice(),
        ),
        FloaterKind::Renown,
        FloaterAnchor::Header,
    );
}

pub fn start_cooking(
    station_color: &str,
    data: &GameData,
    progression: &ProgressionState,
    game_state: &mut GameState,
) -> bool {
    let cooking_slot_limit = data.balance.cooking_slots_limit;
    let Some(station) = game_state.station_mut(station_color) else {
        return false;
    };

    if station.can_cook(cooking_slot_limit) {
        station.is_cooking = true;
        station.remaining_ms = cooking_time_ms(data, progression, station_color);
        game_state.queue_sfx(crate::state::SfxCue::CookStart);
        game_state.tutorial_observe(TutorialTrigger::CookingStarted, data);
        true
    } else {
        false
    }
}

pub fn serve_customer(
    station_color: &str,
    customer_id: u32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) -> bool {
    let Some(plan) = prepare_serve(station_color, customer_id, data, game_state) else {
        return false;
    };
    let total_gain = award_course_score(&plan, data, game_state, progression);
    let served = settle_course(&plan, data, game_state);
    record_course(
        &plan,
        &served,
        total_gain,
        data,
        game_state,
        progression,
        guest_state,
    );
    show_course_feedback(&plan, total_gain, data, game_state);
    game_state.queue_sfx(crate::state::SfxCue::Serve);
    game_state.tutorial_observe(TutorialTrigger::CourseServed, data);
    award_streak_bonuses(data, game_state, progression, plan.floor_x, plan.floor_y);
    true
}

struct ServePlan {
    customer_index: usize,
    course_index: usize,
    dish_name: String,
    freshness: Freshness,
    satisfaction_gain: f32,
    preferred: bool,
    pacing: CoursePacing,
    customer_traits: crate::data::CustomerSpecialTraits,
    is_overfed: bool,
    floor_x: f32,
    floor_y: f32,
}

struct ServedCourse {
    customer_name: String,
    guest_id: String,
    course_label: String,
    courses_done: usize,
    courses_total: usize,
    running_tab: i64,
}

fn prepare_serve(
    station_color: &str,
    customer_id: u32,
    data: &GameData,
    game_state: &mut GameState,
) -> Option<ServePlan> {
    let customer_index = game_state
        .customers
        .iter()
        .position(|customer| customer.id == customer_id)?;
    let customer = &game_state.customers[customer_index];
    if !customer.is_seated {
        game_state.add_message(data.text_format(
            "message_guest_walking",
            [("name", customer.display_name.clone())].as_slice(),
        ));
        return None;
    }
    let Some(course_index) = customer.next_course_for(station_color) else {
        game_state.add_message(
            data.text_format(
                "message_did_not_order",
                [
                    ("name", customer.display_name.clone()),
                    ("dish", dish_display_name(data, station_color)),
                ]
                .as_slice(),
            ),
        );
        return None;
    };
    let dish = game_state
        .station_mut(station_color)
        .and_then(|station| (!station.dishes.is_empty()).then(|| station.dishes.remove(0)))?;
    let freshness = classify_dish_age(dish.age_ms, &data.balance);
    let customer = &game_state.customers[customer_index];
    let traits = customer.traits(data);
    let (satisfaction_gain, delicious_gain, preferred) =
        if data.customer_type_by_id(&customer.customer_type).is_some() {
            serving_gain(data, &customer.customer_type, &traits, station_color)
        } else {
            (data.balance.base_satisfaction_gain, 0.0, false)
        };
    let pacing = classify_course_pacing(customer, &data.balance);
    let customer = &mut game_state.customers[customer_index];
    if let Some(existing) = customer.satisfaction.get_mut(station_color) {
        let max_total = customer.max_satisfaction.get(station_color).unwrap_or(40.0);
        let limit = max_total * overfeed_multiplier(data, &traits);
        *existing = (*existing + satisfaction_gain).min(limit);
    }
    customer.deliciousness =
        (customer.deliciousness + delicious_gain).min(data.balance.max_deliciousness);
    customer.refresh_totals();
    Some(ServePlan {
        customer_index,
        course_index,
        dish_name: dish.name,
        freshness,
        satisfaction_gain,
        preferred,
        pacing,
        customer_traits: traits,
        is_overfed: customer.overfed,
        floor_x: customer.floor_x,
        floor_y: customer.floor_y,
    })
}

fn award_course_score(
    plan: &ServePlan,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) -> i64 {
    let mut score_gain = serving_points(data, plan.satisfaction_gain, plan.preferred) as f64;
    score_gain *= pacing_score_multiplier(plan.pacing, &data.balance);
    if let Some(EventEffect::ServeRenownMultiplier { multiplier }) =
        game_state.active_event_effect(data)
    {
        score_gain *= multiplier.max(1.0);
    }
    if plan.customer_traits.influencer && plan.freshness == Freshness::Fresh {
        score_gain *= data.balance.influencer_score_multiplier;
    }
    add_score(data, game_state, progression, score_gain, true)
}

fn settle_course(plan: &ServePlan, data: &GameData, game_state: &mut GameState) -> ServedCourse {
    let fresh_multiplier = freshness_bill_multiplier(plan.freshness, &data.balance);
    let bill_gain = ((serving_bill(data, plan.preferred, &plan.customer_traits) as f64)
        * fresh_multiplier)
        .round() as i64;
    let customer = &mut game_state.customers[plan.customer_index];
    customer.order[plan.course_index].served = true;
    customer.eating_ms = data.balance.course_eating_ms.max(0.0);
    customer.waiting_ms = 0.0;
    customer.bill = customer.bill.saturating_add(bill_gain);
    ServedCourse {
        customer_name: customer.display_name.clone(),
        guest_id: customer.guest_id.clone(),
        course_label: customer.order[plan.course_index].label.clone(),
        courses_done: customer.courses_served(),
        courses_total: customer.order.len(),
        running_tab: customer.bill,
    }
}

fn record_course(
    plan: &ServePlan,
    served: &ServedCourse,
    total_gain: i64,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) {
    progression.record_served_dish(plan.preferred, plan.is_overfed);
    if plan.freshness == Freshness::Fresh {
        progression.record_fresh_dish();
        game_state.day_cycle.stats.fresh_dishes += 1;
    }
    game_state.day_cycle.stats.renown_earned += total_gain;
    guest_state.record_guest_fed(&served.guest_id);
    game_state.combo = game_state.combo.saturating_add(1);
    game_state.chain = game_state.chain.saturating_add(1);
    progression.record_combo_peak(game_state.combo);
    game_state.day_cycle.record_combo(game_state.combo);
    game_state.add_message(
        data.text_format(
            "message_serve_result",
            [
                ("name", served.customer_name.clone()),
                ("course", served.course_label.clone()),
                ("dish", plan.dish_name.clone()),
                ("points", total_gain.to_string()),
                ("tab", served.running_tab.to_string()),
                ("done", served.courses_done.to_string()),
                ("total", served.courses_total.to_string()),
            ]
            .as_slice(),
        ),
    );
}

fn show_course_feedback(
    plan: &ServePlan,
    total_gain: i64,
    data: &GameData,
    game_state: &mut GameState,
) {
    let flavor = match (plan.preferred, plan.freshness) {
        (true, Freshness::Fresh) => data.text("ui_flavor_loved_fresh").to_string(),
        (true, _) => data.text("ui_flavor_loved").to_string(),
        (false, Freshness::Fresh) => data.text("ui_flavor_fresh").to_string(),
        (false, _) => String::new(),
    };
    let pacing = if plan.pacing == CoursePacing::WellPaced {
        data.text("ui_well_paced")
    } else {
        ""
    };
    game_state.floaters.spawn_at(
        data.text_format(
            "ui_serve_floater",
            [
                ("points", total_gain.to_string()),
                ("flavor", flavor),
                ("pacing", pacing.to_string()),
            ]
            .as_slice(),
        ),
        FloaterKind::Renown,
        plan.floor_x,
        plan.floor_y,
    );
    let (floater_key, message_key) = match plan.pacing {
        CoursePacing::Rushed => (Some("ui_rushed_floater"), Some("message_rushed")),
        CoursePacing::KeptWaiting => (Some("ui_waiting_floater"), Some("message_kept_waiting")),
        _ => (None, None),
    };
    if let Some(key) = floater_key {
        game_state.floaters.spawn_at(
            data.text(key),
            FloaterKind::Alert,
            plan.floor_x,
            plan.floor_y,
        );
    }
    if let Some(key) = message_key {
        game_state.add_message(
            data.text_format(
                key,
                [(
                    "name",
                    game_state.customers[plan.customer_index]
                        .display_name
                        .clone(),
                )]
                .as_slice(),
            ),
        );
    }
}

/// Combo-milestone cash and the full-house renown bonus (every seated order
/// complete at once). Called after each successful serve.
fn award_streak_bonuses(
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    floor_x: f32,
    floor_y: f32,
) {
    let interval = data.balance.combo_milestone_interval.max(2);
    if game_state.combo > 0 && game_state.combo.is_multiple_of(interval) {
        let bonus = data.balance.combo_milestone_cash * i64::from(game_state.combo / interval);
        progression.add_currency(bonus);
        game_state.floaters.spawn_at(
            data.text_format(
                "message_streak_floater",
                [
                    ("combo", game_state.combo.to_string()),
                    ("bonus", bonus.to_string()),
                ]
                .as_slice(),
            ),
            FloaterKind::Cash,
            floor_x,
            floor_y,
        );
        game_state.add_message(
            data.text_format(
                "message_streak",
                [
                    ("combo", game_state.combo.to_string()),
                    ("bonus", bonus.to_string()),
                ]
                .as_slice(),
            ),
        );
    }

    if game_state.full_room_bonus_armed
        && !game_state.customers.is_empty()
        && game_state
            .customers
            .iter()
            .all(|customer| customer.order_complete())
    {
        game_state.full_room_bonus_armed = false;
        progression.record_full_house();
        let points = data.balance.full_room_bonus_points * game_state.customers.len().max(1) as i64;
        let awarded = add_score(data, game_state, progression, points as f64, false);
        game_state.floaters.spawn(
            data.text_format(
                "message_full_house_floater",
                [("points", awarded.to_string())].as_slice(),
            ),
            FloaterKind::Renown,
            FloaterAnchor::Header,
        );
        game_state.add_message(data.text("message_full_house"));
    }
}

pub fn invite_customer_to_vip(
    customer_id: u32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) -> bool {
    if game_state.special_table_busy {
        game_state.add_message(data.text("message_lounge_occupied"));
        return false;
    }
    if matches!(
        game_state.active_event_effect(data),
        Some(EventEffect::LoungeClosed)
    ) {
        game_state.add_message(data.text("message_inspector"));
        return false;
    }

    let Some(index) = game_state
        .customers
        .iter()
        .position(|customer| customer.id == customer_id)
    else {
        return false;
    };
    if !game_state.customers[index].is_seated {
        game_state.add_message(data.text_format(
            "message_lounge_needs_seat",
            [("name", game_state.customers[index].display_name.clone())].as_slice(),
        ));
        return false;
    }
    if !can_process_customer(&game_state.customers[index], data) {
        let customer = &game_state.customers[index];
        let needed = visits_until_ready_for(data, &customer.customer_type);
        game_state.add_message(
            data.text_format(
                "message_not_plump",
                [
                    ("name", customer.display_name.clone()),
                    (
                        "remaining",
                        needed.saturating_sub(customer.times_fed).to_string(),
                    ),
                    ("fed", customer.times_fed.to_string()),
                    ("needed", needed.to_string()),
                ]
                .as_slice(),
            ),
        );
        return false;
    }

    if !crate::engine::chance(data.balance.vip_accept_chance) {
        game_state.add_message(data.text_format(
            "message_vip_declined",
            [("name", game_state.customers[index].display_name.clone())].as_slice(),
        ));
        return false;
    }

    let customer = game_state.customers.remove(index);
    let was_regular = is_regular(&customer, data);
    let farewell = guest_state
        .guest_by_id(&customer.guest_id)
        .and_then(|guest| guest.personality.as_deref())
        .and_then(|personality| data.personality_by_id(personality))
        .map(|personality| personality.farewell.clone());
    let meat_gain = vip_meat_gain(&customer, data, progression);
    let meat_type = format!("{}-meat", customer.customer_type);
    let entry = game_state.ingredients.entry(meat_type.clone()).or_insert(0);
    if *entry != INFINITE_INGREDIENTS {
        *entry = entry.saturating_add(meat_gain);
    }
    let larder_total: i64 = game_state
        .ingredients
        .iter()
        .filter(|(name, amount)| {
            name.as_str() != data.balance.regular_ingredient_name && **amount > 0
        })
        .map(|(_, amount)| *amount)
        .sum();
    progression.record_larder_total(larder_total);
    if was_regular {
        progression.record_regular_processed();
    }
    game_state.day_cycle.stats.guests_processed += 1;
    game_state.day_cycle.stats.meat_gained += meat_gain;

    let chain_value = game_state.chain.saturating_add(1);
    game_state.chain = chain_value;
    game_state.combo = game_state.combo.saturating_add(1);
    guest_state.record_guest_processed(&customer.guest_id);

    let meal_points = ((vip_points(&customer, data) as f64) + (meat_gain as f64 * 10.0))
        * data.balance.base_score_multiplier;
    let awarded = add_score(data, game_state, progression, meal_points, true);
    let cash_gain = (awarded / 5).max(0);
    progression.add_currency(cash_gain);
    progression.record_processed_customer(&customer.customer_type, chain_value);
    game_state.special_table_busy = true;
    game_state.special_table_timer = data.balance.special_table_process_time;
    game_state.add_message(
        data.text_format(
            "message_lounge_started",
            [
                ("name", customer.display_name.clone()),
                ("meat", meat_gain.to_string()),
                ("meat_type", meat_type.clone()),
                ("points", awarded.to_string()),
            ]
            .as_slice(),
        ),
    );
    game_state.day_cycle.stats.renown_earned += awarded;
    game_state.day_cycle.stats.cash_earned += cash_gain;
    // Rewards are already banked above; the cinematic only stages the moment.
    let mut cinematic = ProcessingCinematic::new(
        customer.display_name.clone(),
        customer.customer_type.clone(),
        meat_gain,
        meat_type,
        awarded,
        cash_gain,
        (customer.floor_x, customer.floor_y),
    )
    .with_timing(data.balance.cinematic.clone());
    cinematic.farewell = farewell;
    game_state.processing_cinematic = Some(cinematic);
    game_state.queue_sfx(crate::state::SfxCue::LoungeSting);
    game_state.tutorial_observe(TutorialTrigger::GuestProcessed, data);

    true
}

pub fn attract_customer_type(
    customer_type_id: &str,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    let Some(customer_type) = data.customer_type_by_id(customer_type_id) else {
        game_state.add_message(data.text("message_unknown_clientele"));
        return;
    };
    if progression.is_customer_unlocked(customer_type_id) {
        game_state.add_message(data.text_format(
            "message_already_visits",
            [("name", customer_type.name.clone())].as_slice(),
        ));
        return;
    }

    let missing = missing_ingredients(game_state, &customer_type.unlock_cost);
    if !missing.is_empty() {
        game_state.add_message(
            data.text_format(
                "message_attract_missing",
                [
                    ("name", customer_type.name.clone()),
                    ("missing", missing.join(", ")),
                ]
                .as_slice(),
            ),
        );
        return;
    }

    if !spend_ingredients(
        data,
        game_state,
        &customer_type.unlock_cost,
        "attracting clientele",
    ) {
        return;
    }

    if progression.unlock_customer_type(customer_type_id) {
        game_state.add_message(data.text_format(
            "message_clientele_unlocked",
            [("name", customer_type.name.clone())].as_slice(),
        ));
    }
}

pub fn craft_recipe(
    recipe_id: &str,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    let Some(index) = progression
        .recipes
        .iter()
        .position(|recipe| recipe.id == recipe_id)
    else {
        game_state.add_message(data.text("message_unknown_recipe"));
        return;
    };

    let recipe = progression.recipes[index].clone();
    if !recipe.unlocked {
        game_state.add_message(data.text_format(
            "message_recipe_locked",
            [("recipe", recipe.name.clone())].as_slice(),
        ));
        return;
    }

    let required: Vec<_> = recipe
        .ingredients
        .iter()
        .filter(|(ingredient, _)| ingredient != &&data.balance.regular_ingredient_name)
        .map(|(ingredient, amount)| (ingredient.clone(), *amount))
        .collect();
    if !required
        .iter()
        .all(|(ingredient, amount)| game_state.has_ing(ingredient, *amount))
    {
        game_state.add_message(data.text_format(
            "message_recipe_missing",
            [("recipe", recipe.name.clone())].as_slice(),
        ));
        return;
    }

    if !spend_ingredient_pairs(data, game_state, &required, "crafting") {
        return;
    }

    let points = recipe.base_value as f64
        * recipe.profit_multiplier
        * recipe_value_multiplier(progression)
        * data.balance.base_score_multiplier;
    let awarded = add_score(data, game_state, progression, points, false);
    let cash_gain = awarded / 4;
    progression.add_currency(cash_gain);
    game_state.day_cycle.stats.renown_earned += awarded;
    game_state.day_cycle.stats.cash_earned += cash_gain;
    let bonus_capacity = recipe_capacity_gain(progression, recipe.capacity_bonus);
    progression.record_crafted_recipe(&recipe.id, bonus_capacity);
    game_state.add_message(
        data.text_format(
            "message_recipe_crafted",
            [
                ("recipe", recipe.name.clone()),
                ("points", awarded.to_string()),
            ]
            .as_slice(),
        ),
    );
    game_state.floaters.spawn(
        data.text_format(
            "message_crafted_floater",
            [
                ("recipe", recipe.name.clone()),
                ("points", awarded.to_string()),
                ("cash", cash_gain.to_string()),
            ]
            .as_slice(),
        ),
        FloaterKind::Renown,
        FloaterAnchor::Header,
    );
}

fn missing_ingredients(
    game_state: &GameState,
    cost: &std::collections::HashMap<String, i64>,
) -> Vec<String> {
    cost.iter()
        .filter_map(|(ingredient, amount)| {
            if game_state.has_ing(ingredient, *amount) {
                None
            } else {
                let current = game_state.ingredients.get(ingredient).copied().unwrap_or(0);
                Some(format!("{ingredient} {current}/{amount}"))
            }
        })
        .collect()
}

fn spend_ingredients(
    data: &GameData,
    game_state: &mut GameState,
    cost: &std::collections::HashMap<String, i64>,
    context: &str,
) -> bool {
    let pairs: Vec<_> = cost
        .iter()
        .map(|(ingredient, amount)| (ingredient.clone(), *amount))
        .collect();
    spend_ingredient_pairs(data, game_state, &pairs, context)
}

fn spend_ingredient_pairs(
    data: &GameData,
    game_state: &mut GameState,
    pairs: &[(String, i64)],
    context: &str,
) -> bool {
    for (ingredient, amount) in pairs {
        if !game_state.remove_ingredients(ingredient, *amount) {
            game_state.add_message(
                data.text_format(
                    "message_missing_ingredients",
                    [
                        ("ingredient", ingredient.clone()),
                        ("context", context.to_string()),
                    ]
                    .as_slice(),
                ),
            );
            return false;
        }
    }
    true
}

fn add_score(
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    points: f64,
    apply_combo: bool,
) -> i64 {
    let combo_multiplier = if apply_combo {
        let combo_boost = progression.get_effect("combo_multiplier", 1.0);
        1.0 + (f64::from(game_state.combo) * data.balance.combo_score_step * combo_boost)
    } else {
        1.0
    };
    let prestige_multiplier =
        1.0 + (f64::from(progression.prestige_points) * data.balance.prestige_point_multiplier);
    let awarded = (points * combo_multiplier * prestige_multiplier).max(0.0);

    let next_score = awarded.floor() as i64;
    if next_score > 0 {
        progression.record_score(next_score);
        game_state.score = game_state.score.saturating_add(next_score);
    }

    next_score
}
