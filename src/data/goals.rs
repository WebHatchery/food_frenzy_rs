//! Data-driven next-day objectives and their gameplay eligibility rules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDef {
    pub id: String,
    pub title: String,
    pub description: String,
    pub min_day: u32,
    pub kind: GoalKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GoalKind {
    ServeCourses { target: u32 },
    ServeFreshDishes { target: u32 },
    CompleteDiningEvent,
    ProcessRegular,
    AttractClientele,
    Prestige,
}
