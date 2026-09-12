"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useAuth } from "@/components/AuthProvider";
import { api, ApiError, MyTeamResponse, SquadPlayerView, Team, TeamPoints } from "@/lib/api";
import { POSITION_LABEL, formatCost } from "@/lib/squadRules";

export default function MyTeamPage() {
  const { token } = useAuth();
  const [team, setTeam] = useState<MyTeamResponse | null>(null);
  const [teams, setTeams] = useState<Team[]>([]);
  const [points, setPoints] = useState<TeamPoints | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!token) return;
    let cancelled = false;

    Promise.all([
      api.get<MyTeamResponse>("/fantasy/my-team", token),
      api.get<TeamPoints>("/fantasy/my-points", token),
      api.get<Team[]>("/teams"),
    ])
      .then(([teamData, pointsData, teamsData]) => {
        if (cancelled) return;
        setTeam(teamData);
        setPoints(pointsData);
        setTeams(teamsData);
      })
      .catch((e) => {
        if (cancelled) return;
        setError(e instanceof ApiError ? e.message : "Failed to load team");
      })
      .finally(() => !cancelled && setLoading(false));

    return () => {
      cancelled = true;
    };
  }, [token]);

  const clubById = new Map(teams.map((t) => [t.id, t]));
  const pointsById = new Map(points?.players.map((p) => [p.player_id, p.points]) ?? []);

  if (!token) return <p style={{ color: "var(--muted)" }}>Connect your wallet first.</p>;
  if (loading) return <p style={{ color: "var(--muted)" }}>Loading…</p>;

  if (error || !team) {
    return (
      <section className="card" style={{ padding: 40, textAlign: "center" }}>
        <h2 style={{ fontSize: 24 }}>No squad yet</h2>
        <p style={{ marginTop: 10, color: "var(--muted)" }}>{error}</p>
        <Link href="/team/build" className="btn btn-primary" style={{ marginTop: 20 }}>
          Build squad →
        </Link>
      </section>
    );
  }

  const starters = team.players.filter((p) => p.is_starting);
  const bench = team.players.filter((p) => !p.is_starting).sort((a, b) => (a.bench_order ?? 0) - (b.bench_order ?? 0));

  return (
    <section>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: 20 }}>
        <h1 style={{ fontSize: 28 }}>My Squad</h1>
        {points && (
          <div className="mono" style={{ fontSize: 14, color: "var(--muted)" }}>
            GW{points.gameweek} ·{" "}
            <span style={{ color: "var(--green-bright)", fontSize: 20 }}>{points.total_points}</span> pts
            {points.points_hit > 0 && <span style={{ color: "var(--red)" }}> (−{points.points_hit} hit)</span>}
          </div>
        )}
      </div>

      <h3 style={{ fontSize: 15, color: "var(--muted)", marginBottom: 10 }}>Starting XI</h3>
      <PlayerGrid players={starters} team={team} pointsById={pointsById} clubById={clubById} />

      <h3 style={{ fontSize: 15, color: "var(--muted)", margin: "24px 0 10px" }}>Bench</h3>
      <PlayerGrid players={bench} team={team} pointsById={pointsById} clubById={clubById} />
    </section>
  );
}

function PlayerGrid({
  players,
  team,
  pointsById,
  clubById,
}: {
  players: SquadPlayerView[];
  team: MyTeamResponse;
  pointsById: Map<number, number>;
  clubById: Map<number, Team>;
}) {
  return (
    <div className="card">
      {players.map((p) => {
        const isCaptain = p.id === team.captain.id;
        const isVice = p.id === team.vice_captain.id;
        const club = p.team ? clubById.get(p.team) : undefined;
        return (
          <div
            key={p.id}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              padding: "10px 16px",
              borderBottom: "1px solid var(--line)",
            }}
          >
            <span className="tag">{p.position ? POSITION_LABEL[p.position] : "—"}</span>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: 14, fontWeight: 600 }}>
                {p.first_name} {p.second_name}
                {isCaptain && (
                  <span className="tag" style={{ marginLeft: 8, color: "var(--amber)", borderColor: "var(--amber)" }}>
                    C
                  </span>
                )}
                {isVice && <span className="tag" style={{ marginLeft: 8 }}>VC</span>}
              </div>
              <div style={{ fontSize: 12, color: "var(--muted-2)" }}>{club?.name ?? "—"}</div>
            </div>
            <span className="mono" style={{ fontSize: 12, color: "var(--muted)" }}>
              {formatCost(p.now_cost ?? 0)}m
            </span>
            <span className="mono" style={{ fontSize: 16, color: "var(--green-bright)", width: 36, textAlign: "right" }}>
              {pointsById.get(p.id) ?? 0}
            </span>
          </div>
        );
      })}
    </div>
  );
}
