use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

// FPL element_type: 1 = GK, 2 = DEF, 3 = MID, 4 = FWD
const GK: i32 = 1;
const DEF: i32 = 2;
const MID: i32 = 3;
const FWD: i32 = 4;

pub const SQUAD_SIZE: usize = 15;
pub const STARTING_XI_SIZE: usize = 11;
pub const MAX_PER_CLUB: usize = 3;

/// Budget, in tenths of a million (matches FPL's `now_cost` unit) — 100.0m.
pub const BUDGET_LIMIT: i32 = 1000;

const REQUIRED_GK: usize = 2;
const REQUIRED_DEF: usize = 5;
const REQUIRED_MID: usize = 5;
const REQUIRED_FWD: usize = 3;

const STARTING_GK: usize = 1;
const STARTING_DEF_RANGE: (usize, usize) = (3, 5);
const STARTING_MID_RANGE: (usize, usize) = (2, 5);
const STARTING_FWD_RANGE: (usize, usize) = (1, 3);

pub struct SquadPlayerInfo {
    pub id: i32,
    pub position: i32,
    pub team_id: i32,
    pub now_cost: i32,
}

pub fn validate_squad(
    players: &[SquadPlayerInfo],
    starting_ids: &HashSet<i32>,
    captain_id: i32,
    vice_captain_id: i32,
) -> Result<()> {
    if players.len() != SQUAD_SIZE {
        bail!("Squad must have exactly {} players", SQUAD_SIZE);
    }

    let all_ids: HashSet<i32> = players.iter().map(|p| p.id).collect();
    if all_ids.len() != players.len() {
        bail!("Squad contains duplicate players");
    }

    let total_cost: i32 = players.iter().map(|p| p.now_cost).sum();
    if total_cost > BUDGET_LIMIT {
        bail!(
            "Squad costs {:.1}m, budget is {:.1}m",
            total_cost as f32 / 10.0,
            BUDGET_LIMIT as f32 / 10.0
        );
    }

    let mut by_position: HashMap<i32, usize> = HashMap::new();
    let mut by_club: HashMap<i32, usize> = HashMap::new();
    for p in players {
        *by_position.entry(p.position).or_insert(0) += 1;
        *by_club.entry(p.team_id).or_insert(0) += 1;
    }

    check_count(&by_position, GK, REQUIRED_GK, "goalkeeper")?;
    check_count(&by_position, DEF, REQUIRED_DEF, "defender")?;
    check_count(&by_position, MID, REQUIRED_MID, "midfielder")?;
    check_count(&by_position, FWD, REQUIRED_FWD, "forward")?;

    if let Some((_, &count)) = by_club.iter().find(|&(_, &c)| c > MAX_PER_CLUB) {
        bail!(
            "Max {} players allowed from a single club (found {})",
            MAX_PER_CLUB,
            count
        );
    }

    if starting_ids.len() != STARTING_XI_SIZE {
        bail!("Starting XI must have exactly {} players", STARTING_XI_SIZE);
    }
    if !starting_ids.is_subset(&all_ids) {
        bail!("Starting XI must be chosen from the 15-player squad");
    }

    let mut starting_by_position: HashMap<i32, usize> = HashMap::new();
    for p in players.iter().filter(|p| starting_ids.contains(&p.id)) {
        *starting_by_position.entry(p.position).or_insert(0) += 1;
    }

    let gk_count = starting_by_position.get(&GK).copied().unwrap_or(0);
    if gk_count != STARTING_GK {
        bail!("Starting XI must include exactly 1 goalkeeper");
    }
    check_range(&starting_by_position, DEF, STARTING_DEF_RANGE, "defenders")?;
    check_range(&starting_by_position, MID, STARTING_MID_RANGE, "midfielders")?;
    check_range(&starting_by_position, FWD, STARTING_FWD_RANGE, "forwards")?;

    if captain_id == vice_captain_id {
        bail!("Captain and vice-captain must be different players");
    }
    if !all_ids.contains(&captain_id) {
        bail!("Captain must be part of the squad");
    }
    if !all_ids.contains(&vice_captain_id) {
        bail!("Vice-captain must be part of the squad");
    }
    if !starting_ids.contains(&captain_id) {
        bail!("Captain must be in the starting XI");
    }
    if !starting_ids.contains(&vice_captain_id) {
        bail!("Vice-captain must be in the starting XI");
    }

    Ok(())
}

fn check_count(
    by_position: &HashMap<i32, usize>,
    position: i32,
    required: usize,
    label: &str,
) -> Result<()> {
    let count = by_position.get(&position).copied().unwrap_or(0);
    if count != required {
        bail!(
            "Squad must have exactly {} {}s (found {})",
            required,
            label,
            count
        );
    }
    Ok(())
}

fn check_range(
    by_position: &HashMap<i32, usize>,
    position: i32,
    (min, max): (usize, usize),
    label: &str,
) -> Result<()> {
    let count = by_position.get(&position).copied().unwrap_or(0);
    if count < min || count > max {
        bail!(
            "Starting XI must have between {} and {} {} (found {})",
            min,
            max,
            label,
            count
        );
    }
    Ok(())
}
