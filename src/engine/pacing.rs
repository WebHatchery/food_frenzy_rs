//! Course pacing: after a course lands, the guest spends a while eating, then
//! wants the next course within a grace window. Serving into the eating
//! window is rushed (penalized renown); hitting the window between is
//! well-paced (bonus); past the grace the guest is kept waiting (penalized,
//! and their satisfaction bleeds until the course arrives).

use crate::data::GameBalance;
use crate::state::Customer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoursePacing {
    /// The guest's first course this visit: no rhythm to judge yet.
    FirstCourse,
    /// Served while they were still eating the previous course.
    Rushed,
    /// Served after eating, within the grace window.
    WellPaced,
    /// Served after the grace window had already run out.
    KeptWaiting,
}

/// Judge the pacing of serving this guest their *next* course right now.
pub fn classify_course_pacing(customer: &Customer, balance: &GameBalance) -> CoursePacing {
    if customer.courses_served() == 0 {
        CoursePacing::FirstCourse
    } else if customer.eating_ms > 0.0 {
        CoursePacing::Rushed
    } else if customer.waiting_ms <= balance.course_wait_grace_ms.max(0.0) {
        CoursePacing::WellPaced
    } else {
        CoursePacing::KeptWaiting
    }
}

pub fn pacing_score_multiplier(pacing: CoursePacing, balance: &GameBalance) -> f64 {
    match pacing {
        CoursePacing::FirstCourse => 1.0,
        CoursePacing::Rushed => balance.rushed_course_score_multiplier.clamp(0.0, 1.0),
        CoursePacing::WellPaced => balance.paced_course_score_multiplier.max(1.0),
        CoursePacing::KeptWaiting => balance.late_course_score_multiplier.clamp(0.0, 1.0),
    }
}

/// True while the guest has finished eating, still has courses coming, and
/// the grace window has expired — the state that bleeds satisfaction.
pub fn is_kept_waiting(customer: &Customer, balance: &GameBalance) -> bool {
    customer.is_seated
        && !customer.order_complete()
        && customer.courses_served() > 0
        && customer.eating_ms <= 0.0
        && customer.waiting_ms > balance.course_wait_grace_ms.max(0.0)
}
