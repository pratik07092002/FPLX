"use client";

import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/components/AuthProvider";
import { api, ApiError, PlayerListItem } from "@/lib/api";
import {
  BUDGET_LIMIT,
  MAX_PER_CLUB,
  POSITION_LABEL,
  REQUIRED_BY_POSITION,
  SQUAD_SIZE,
  STARTING_RANGE_BY_POSITION,
  STARTING_XI_SIZE,
  formatCost,
} from "@/lib/squadRules";

const POSITION_TABS = [0, 1, 2, 3, 4] as const; // 0 = All

export default function BuildSquadPage() {
  const { token } = useAuth();
  const router = useRouter();

  const [players, setPlayers] = useState<PlayerListItem[]>([]);
  const [loadingPlayers, setLoadingPlayers] = useState(true);
  const [search, setSearch] = useState("");
  const [posFilter, setPosFilter] = useState<number>(0);

  const [squadIds, setSquadIds] = useState<number[]>([]);
  const [startingIds, setStartingIds] = useState<number[]>([]);
  const [captainId, setCaptainId] = useState<number | null>(null);
  const [viceCaptainId, setViceCaptainId] = useState<number | null>(null);

  const [submitting, setSubmitting] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    api
      .get<PlayerListItem[]>("/players")
      .then(setPlayers)
      .finally(() => setLoadingPlayers(false));
  }, []);

  const byId = useMemo(() => new Map(players.map((p) => [p.id, p])), [players]);
  const squad = useMemo(() => squadIds.map((id) => byId.get(id)).filter(Boolean) as PlayerListItem[], [squadIds, byId]);

  const budgetUsed = squad.reduce((sum, p) => sum + p.now_cost, 0);
  const countByPosition = countBy(squad, (p) => p.position);
  const countByClub = countBy(squad, (p) => p.team_id);

  const filteredPlayers = players.filter((p) => {
    if (posFilter !== 0 && p.position !== posFilter) return false;
    if (search) {
      const q = search.toLowerCase();
      const name = `${p.first_name} ${p.second_name}`.toLowerCase();
      if (!name.includes(q) && !p.team_name.toLowerCase().includes(q)) return false;
    }
    return true;
  });

  function canAdd(p: PlayerListItem): string | null {
    if (squadIds.includes(p.id)) return null;
    if (squadIds.length >= SQUAD_SIZE) return `Squad is full (${SQUAD_SIZE})`;
    if ((countByPosition[p.position] ?? 0) >= REQUIRED_BY_POSITION[p.position]) {
      return `Already have ${REQUIRED_BY_POSITION[p.position]} ${POSITION_LABEL[p.position]}s`;
    }
    if ((countByClub[p.team_id] ?? 0) >= MAX_PER_CLUB) {
      return `Max ${MAX_PER_CLUB} players from ${p.team_short_name}`;
    }
    if (budgetUsed + p.now_cost > BUDGET_LIMIT) return "Over budget";
    return null;
  }

  function toggleSquad(p: PlayerListItem) {
    if (squadIds.includes(p.id)) {
      setSquadIds((ids) => ids.filter((id) => id !== p.id));
      setStartingIds((ids) => ids.filter((id) => id !== p.id));
      if (captainId === p.id) setCaptainId(null);
      if (viceCaptainId === p.id) setViceCaptainId(null);
      return;
    }
    const blocker = canAdd(p);
    if (blocker) return;
    setSquadIds((ids) => [...ids, p.id]);
  }

  function toggleStarting(id: number) {
    setStartingIds((ids) => {
      if (ids.includes(id)) {
        if (captainId === id) setCaptainId(null);
        if (viceCaptainId === id) setViceCaptainId(null);
        return ids.filter((x) => x !== id);
      }
      if (ids.length >= STARTING_XI_SIZE) return ids;
      return [...ids, id];
    });
  }

  const startingByPosition = countBy(
    squad.filter((p) => startingIds.includes(p.id)),
    (p) => p.position
  );

  async function submit() {
    setFormError(null);

    if (squadIds.length !== SQUAD_SIZE) return setFormError(`Pick exactly ${SQUAD_SIZE} players`);
    if (startingIds.length !== STARTING_XI_SIZE) return setFormError(`Pick exactly ${STARTING_XI_SIZE} starters`);
    if (!captainId || !viceCaptainId) return setFormError("Pick a captain and vice-captain");

    setSubmitting(true);
    try {
      await api.post(
        "/fantasy/create-team",
        {
          players: squadIds,
          starting_ids: startingIds,
          captain_id: captainId,
          vice_captain_id: viceCaptainId,
        },
        token
      );
      setSuccess(true);
      setTimeout(() => router.push("/team"), 1200);
    } catch (e) {
      setFormError(e instanceof ApiError ? e.message : "Failed to create team");
    } finally {
      setSubmitting(false);
    }
  }

  if (!token) {
    return <p style={{ color: "var(--muted)" }}>Connect your wallet first.</p>;
  }

  if (loadingPlayers) {
    return <p style={{ color: "var(--muted)" }}>Loading players…</p>;
  }

  return (
    <div style={{ display: "grid", gridTemplateColumns: "1.4fr 1fr", gap: 28, alignItems: "start" }}>
      <section>
        <h1 style={{ fontSize: 28, marginBottom: 16 }}>Build your squad</h1>

        <div style={{ display: "flex", gap: 8, marginBottom: 14 }}>
          {POSITION_TABS.map((pos) => (
            <button
              key={pos}
              className="btn btn-ghost"
              style={{
                padding: "6px 14px",
                fontSize: 13,
                borderColor: posFilter === pos ? "var(--green)" : undefined,
                color: posFilter === pos ? "var(--green-bright)" : undefined,
              }}
              onClick={() => setPosFilter(pos)}
            >
              {pos === 0 ? "All" : POSITION_LABEL[pos]}
            </button>
          ))}
          <input
            type="text"
            placeholder="Search player or club…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{ marginLeft: "auto", width: 220 }}
          />
        </div>

        <div className="card" style={{ maxHeight: 640, overflowY: "auto" }}>
          {filteredPlayers.map((p) => {
            const inSquad = squadIds.includes(p.id);
            const blocker = canAdd(p);
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
                <span className="tag">{POSITION_LABEL[p.position]}</span>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 14, fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis" }}>
                    {p.first_name} {p.second_name}
                  </div>
                  <div style={{ fontSize: 12, color: "var(--muted-2)" }}>{p.team_short_name}</div>
                </div>
                <span className="mono" style={{ fontSize: 13, color: "var(--green-bright)", width: 48, textAlign: "right" }}>
                  {formatCost(p.now_cost)}
                </span>
                <button
                  className={inSquad ? "btn btn-danger" : "btn btn-ghost"}
                  style={{ padding: "6px 12px", fontSize: 12 }}
                  disabled={!inSquad && !!blocker}
                  title={blocker ?? undefined}
                  onClick={() => toggleSquad(p)}
                >
                  {inSquad ? "Remove" : blocker ? "Blocked" : "Add"}
                </button>
              </div>
            );
          })}
        </div>
      </section>

      <aside className="card" style={{ padding: 22, position: "sticky", top: 20 }}>
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
          <b>Squad</b>
          <span className="mono" style={{ color: "var(--green-bright)" }}>
            {squadIds.length}/{SQUAD_SIZE}
          </span>
        </div>

        <div
          style={{
            height: 8,
            borderRadius: 5,
            background: "var(--panel-2)",
            border: "1px solid var(--line)",
            overflow: "hidden",
            marginBottom: 6,
          }}
        >
          <div
            style={{
              height: "100%",
              width: `${Math.min(100, (budgetUsed / BUDGET_LIMIT) * 100)}%`,
              background: budgetUsed > BUDGET_LIMIT ? "var(--red)" : "linear-gradient(90deg, var(--green-deep), var(--green))",
            }}
          />
        </div>
        <div className="mono" style={{ fontSize: 12, color: "var(--muted-2)", marginBottom: 18 }}>
          {formatCost(budgetUsed)} / {formatCost(BUDGET_LIMIT)} CR
        </div>

        <div style={{ marginBottom: 18 }}>
          {[1, 2, 3, 4].map((pos) => (
            <div key={pos} style={{ fontSize: 12, color: "var(--muted)", marginBottom: 4 }}>
              {POSITION_LABEL[pos]}: {countByPosition[pos] ?? 0}/{REQUIRED_BY_POSITION[pos]}
            </div>
          ))}
        </div>

        <div style={{ marginBottom: 8, fontSize: 13, color: "var(--muted)" }}>
          Starting XI ({startingIds.length}/{STARTING_XI_SIZE}) — tap a squad player below to set lineup
        </div>

        <div style={{ maxHeight: 260, overflowY: "auto", marginBottom: 18 }}>
          {squad.map((p) => {
            const starting = startingIds.includes(p.id);
            return (
              <div
                key={p.id}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  padding: "6px 0",
                  borderBottom: "1px solid var(--line)",
                  opacity: starting ? 1 : 0.55,
                }}
              >
                <button
                  className="btn btn-ghost"
                  style={{ padding: "3px 8px", fontSize: 11 }}
                  onClick={() => toggleStarting(p.id)}
                >
                  {starting ? "Starting" : "Bench"}
                </button>
                <span style={{ fontSize: 13, flex: 1 }}>
                  {p.first_name[0]}. {p.second_name}
                </span>
                <span className="tag">{POSITION_LABEL[p.position]}</span>
              </div>
            );
          })}
        </div>

        <label style={{ fontSize: 12, color: "var(--muted)" }}>Captain</label>
        <select
          value={captainId ?? ""}
          onChange={(e) => setCaptainId(e.target.value ? Number(e.target.value) : null)}
          style={{ width: "100%", marginTop: 4, marginBottom: 12 }}
        >
          <option value="">— select —</option>
          {squad
            .filter((p) => startingIds.includes(p.id) && p.id !== viceCaptainId)
            .map((p) => (
              <option key={p.id} value={p.id}>
                {p.first_name} {p.second_name}
              </option>
            ))}
        </select>

        <label style={{ fontSize: 12, color: "var(--muted)" }}>Vice-captain</label>
        <select
          value={viceCaptainId ?? ""}
          onChange={(e) => setViceCaptainId(e.target.value ? Number(e.target.value) : null)}
          style={{ width: "100%", marginTop: 4, marginBottom: 18 }}
        >
          <option value="">— select —</option>
          {squad
            .filter((p) => startingIds.includes(p.id) && p.id !== captainId)
            .map((p) => (
              <option key={p.id} value={p.id}>
                {p.first_name} {p.second_name}
              </option>
            ))}
        </select>

        {formError && <p className="error-text" style={{ marginBottom: 12 }}>{formError}</p>}
        {success && <p style={{ color: "var(--green-bright)", marginBottom: 12 }}>Squad created — redirecting…</p>}

        <button className="btn btn-primary" style={{ width: "100%", justifyContent: "center" }} disabled={submitting} onClick={submit}>
          {submitting ? "Submitting…" : "Save squad"}
        </button>

        <p style={{ fontSize: 11, color: "var(--muted-2)", marginTop: 10 }}>
          Starting XI needs {STARTING_RANGE_BY_POSITION[2][0]}-{STARTING_RANGE_BY_POSITION[2][1]} DEF,{" "}
          {STARTING_RANGE_BY_POSITION[3][0]}-{STARTING_RANGE_BY_POSITION[3][1]} MID,{" "}
          {STARTING_RANGE_BY_POSITION[4][0]}-{STARTING_RANGE_BY_POSITION[4][1]} FWD, exactly 1 GK.
        </p>
      </aside>
    </div>
  );
}

function countBy<T>(items: T[], key: (item: T) => number): Record<number, number> {
  const out: Record<number, number> = {};
  for (const item of items) {
    const k = key(item);
    out[k] = (out[k] ?? 0) + 1;
  }
  return out;
}
