use anyhow::{bail, Result};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::datamodels::transfers_data_model::{TransferItem, TransferSummary};
use crate::helpers::squad_rules::{self, SquadPlayerInfo};
use crate::models::fantasy_team_model;

pub const MAX_BANKED_FREE_TRANSFERS: i32 = 5;
pub const POINTS_PER_HIT: i32 = 4;

struct SquadSlot {
    position: i32,
    team_id: i32,
    now_cost: i32,
    is_starting: bool,
    bench_order: Option<i16>,
}

struct TeamRow {
    id: Uuid,
    captain_id: i32,
    vice_captain_id: i32,
    free_transfers: i32,
    last_transfer_gameweek: Option<i32>,
}

pub async fn is_gameweek_locked(pool: &PgPool, gameweek: i32) -> Result<bool> {
    let rec = sqlx::query!(
        r#"SELECT EXISTS(SELECT 1 FROM fixtures WHERE gameweek = $1 AND started = true) AS "locked!""#,
        gameweek
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.locked)
}

async fn get_team_row(pool: &PgPool, user_id: Uuid) -> Result<TeamRow> {
    let rec = sqlx::query!(
        r#"
        SELECT id, captain_id, vice_captain_id, free_transfers, last_transfer_gameweek
        FROM fantasy_teams
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(TeamRow {
        id: rec.id,
        captain_id: rec.captain_id,
        vice_captain_id: rec.vice_captain_id,
        free_transfers: rec.free_transfers,
        last_transfer_gameweek: rec.last_transfer_gameweek,
    })
}

async fn get_squad_slots(pool: &PgPool, team_id: Uuid) -> Result<HashMap<i32, SquadSlot>> {
    let rows = sqlx::query!(
        r#"
        SELECT p.id, p.position, p.team_id, p.now_cost, fp.is_starting, fp.bench_order
        FROM fantasy_team_players fp
        JOIN players p ON p.id = fp.player_id
        WHERE fp.fantasy_team_id = $1
        "#,
        team_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.id,
                SquadSlot {
                    position: r.position,
                    team_id: r.team_id,
                    now_cost: r.now_cost,
                    is_starting: r.is_starting,
                    bench_order: r.bench_order,
                },
            )
        })
        .collect())
}

/// This gameweek's total points deducted from transfer hits.
pub async fn get_points_hit(pool: &PgPool, team_id: Uuid, gameweek: i32) -> Result<i32> {
    let rec = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(points_cost), 0)::INTEGER AS "total!"
        FROM fantasy_transfers
        WHERE fantasy_team_id = $1 AND gameweek = $2
        "#,
        team_id,
        gameweek
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.total)
}

pub async fn make_transfers(
    pool: &PgPool,
    user_id: Uuid,
    gameweek: i32,
    items: &[TransferItem],
) -> Result<TransferSummary> {
    if items.is_empty() {
        bail!("No transfers submitted");
    }

    if is_gameweek_locked(pool, gameweek).await? {
        bail!(
            "Gameweek {} has kicked off — transfers reopen once the next gameweek is current",
            gameweek
        );
    }

    let team = get_team_row(pool, user_id).await?;
    let mut slots = get_squad_slots(pool, team.id).await?;

    if slots.len() != squad_rules::SQUAD_SIZE {
        bail!("Squad is in an unexpected state — contact support");
    }

    // Replenish free transfers for gameweeks passed since the last transfer.
    let free_transfers_available = match team.last_transfer_gameweek {
        Some(last_gw) if gameweek > last_gw => {
            (team.free_transfers + (gameweek - last_gw)).min(MAX_BANKED_FREE_TRANSFERS)
        }
        _ => team.free_transfers,
    };

    let out_ids: HashSet<i32> = items.iter().map(|t| t.player_out_id).collect();
    if out_ids.len() != items.len() {
        bail!("Cannot transfer the same player out twice in one request");
    }

    for item in items {
        if item.player_out_id == team.captain_id {
            bail!("Change captain before transferring them out");
        }
        if item.player_out_id == team.vice_captain_id {
            bail!("Change vice-captain before transferring them out");
        }
    }

    let in_ids: Vec<i32> = items.iter().map(|t| t.player_in_id).collect();
    let incoming_info = fantasy_team_model::get_squad_player_info(pool, &in_ids).await?;
    let incoming_by_id: HashMap<i32, SquadPlayerInfo> =
        incoming_info.into_iter().map(|p| (p.id, p)).collect();

    for item in items {
        let out_slot = slots
            .remove(&item.player_out_id)
            .ok_or_else(|| anyhow::anyhow!("Player {} is not in your squad", item.player_out_id))?;

        if slots.contains_key(&item.player_in_id) {
            bail!("Player {} is already in your squad", item.player_in_id);
        }

        let incoming = incoming_by_id
            .get(&item.player_in_id)
            .ok_or_else(|| anyhow::anyhow!("Player {} does not exist", item.player_in_id))?;

        if incoming.position != out_slot.position {
            bail!(
                "Replacement must be the same position as the player leaving (player {} is not)",
                item.player_in_id
            );
        }

        slots.insert(
            item.player_in_id,
            SquadSlot {
                position: incoming.position,
                team_id: incoming.team_id,
                now_cost: incoming.now_cost,
                is_starting: out_slot.is_starting,
                bench_order: out_slot.bench_order,
            },
        );
    }

    let total_cost: i32 = slots.values().map(|s| s.now_cost).sum();
    if total_cost > squad_rules::BUDGET_LIMIT {
        bail!(
            "Squad would cost {:.1}m, budget is {:.1}m",
            total_cost as f32 / 10.0,
            squad_rules::BUDGET_LIMIT as f32 / 10.0
        );
    }

    let mut by_club: HashMap<i32, usize> = HashMap::new();
    for s in slots.values() {
        *by_club.entry(s.team_id).or_insert(0) += 1;
    }
    if let Some((_, &count)) = by_club.iter().find(|&(_, &c)| c > squad_rules::MAX_PER_CLUB) {
        bail!(
            "Max {} players allowed from a single club (found {})",
            squad_rules::MAX_PER_CLUB,
            count
        );
    }

    let transfers_made = items.len();
    let free_transfers_used = transfers_made.min(free_transfers_available.max(0) as usize);
    let hits = transfers_made - free_transfers_used;
    let points_cost = hits as i32 * POINTS_PER_HIT;
    let free_transfers_remaining = (free_transfers_available - transfers_made as i32).max(0);

    let mut tx = pool.begin().await?;

    for (idx, item) in items.iter().enumerate() {
        sqlx::query!(
            "DELETE FROM fantasy_team_players WHERE fantasy_team_id = $1 AND player_id = $2",
            team.id,
            item.player_out_id
        )
        .execute(&mut *tx)
        .await?;

        let incoming_slot = slots
            .get(&item.player_in_id)
            .expect("incoming slot was just inserted above");

        sqlx::query!(
            r#"
            INSERT INTO fantasy_team_players
            (fantasy_team_id, player_id, is_starting, bench_order)
            VALUES ($1,$2,$3,$4)
            "#,
            team.id,
            item.player_in_id,
            incoming_slot.is_starting,
            incoming_slot.bench_order,
        )
        .execute(&mut *tx)
        .await?;

        let this_cost = if idx < free_transfers_used { 0 } else { POINTS_PER_HIT };

        sqlx::query!(
            r#"
            INSERT INTO fantasy_transfers
            (fantasy_team_id, gameweek, player_out_id, player_in_id, points_cost)
            VALUES ($1,$2,$3,$4,$5)
            "#,
            team.id,
            gameweek,
            item.player_out_id,
            item.player_in_id,
            this_cost,
        )
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query!(
        r#"
        UPDATE fantasy_teams
        SET free_transfers = $1, last_transfer_gameweek = $2
        WHERE id = $3
        "#,
        free_transfers_remaining,
        gameweek,
        team.id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(TransferSummary {
        gameweek,
        transfers_made,
        free_transfers_used,
        hits,
        points_cost,
        free_transfers_remaining,
    })
}
