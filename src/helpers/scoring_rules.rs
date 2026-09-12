// FPL element_type: 1 = GK, 2 = DEF, 3 = MID, 4 = FWD
pub struct FixtureStatLine {
    pub position: i32,
    pub minutes: i32,
    pub goals: i32,
    pub assists: i32,
    pub own_goals: i32,
    pub penalties_saved: i32,
    pub penalties_missed: i32,
    pub yellow_cards: i32,
    pub red_cards: i32,
    pub saves: i32,
    pub bonus: i32,
    pub defensive_contribution: i32,
    pub goals_conceded: i32,
}

/// Current Fantasy Premier League scoring rules, applied to one player's
/// single-fixture stat line.
pub fn calculate_points(s: &FixtureStatLine) -> i32 {
    let mut pts = 0;

    // Appearance
    if s.minutes >= 60 {
        pts += 2;
    } else if s.minutes > 0 {
        pts += 1;
    }

    // Goals scored
    pts += s.goals
        * match s.position {
            1 | 2 => 6,
            3 => 5,
            _ => 4,
        };

    // Assists
    pts += s.assists * 3;

    // Clean sheet (only counts with 60+ minutes played)
    if s.minutes >= 60 && s.goals_conceded == 0 {
        pts += match s.position {
            1 | 2 => 4,
            3 => 1,
            _ => 0,
        };
    }

    // Goals conceded penalty — GK/DEF only, -1 per 2 conceded, needs 60+ minutes
    if s.minutes >= 60 && matches!(s.position, 1 | 2) {
        pts -= s.goals_conceded / 2;
    }

    // Saves — GK only, 1pt per 3 saves
    if s.position == 1 {
        pts += s.saves / 3;
    }

    // Defensive contribution threshold (2025/26 rule)
    let dc_threshold = match s.position {
        2 => Some(10),
        3 | 4 => Some(12),
        _ => None,
    };
    if let Some(threshold) = dc_threshold {
        if s.defensive_contribution >= threshold {
            pts += 2;
        }
    }

    // Penalties
    pts += s.penalties_saved * 5;
    pts -= s.penalties_missed * 2;

    // Cards
    pts -= s.yellow_cards;
    pts -= s.red_cards * 3;

    // Own goals
    pts -= s.own_goals * 2;

    // Bonus — FPL already resolves this from BPS in the live feed, add as-is
    pts += s.bonus;

    pts
}
