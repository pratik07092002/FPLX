"use client";

import { useEffect, useMemo, useState } from "react";
import { useAuth } from "@/components/AuthProvider";
import { api, ApiError, MyTeamResponse, PlayerListItem, Team, TransferSummary } from "@/lib/api";
import { POSITION_LABEL, formatCost } from "@/lib/squadRules";

interface PendingTransfer {
  outId: number;
  inId: number | null;
}

export default function TransfersPage() {
  const { token } = useAuth();
  const [team, setTeam] = useState<MyTeamResponse | null>(null);
  const [players, setPlayers] = useState<PlayerListItem[]>([]);
  const [teams, setTeams] = useState<Team[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [pending, setPending] = useState<PendingTransfer[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [summary, setSummary] = useState<TransferSummary | null>(null);

  useEffect(() => {
    if (!token) return;
    let cancelled = false;

    Promise.all([
      api.get<MyTeamResponse>("/fantasy/my-team", token),
      api.get<PlayerListItem[]>("/players"),
      api.get<Team[]>("/teams"),
    ])
      .then(([teamData, playerData, teamsData]) => {
        if (cancelled) return;
        setTeam(teamData);
        setPlayers(playerData);
        setTeams(teamsData);
      })
      .catch((e) => !cancelled && setLoadError(e instanceof ApiError ? e.message : "Failed to load"))
      .finally(() => !cancelled && setLoading(false));

    return () => {
      cancelled = true;
    };
  }, [token]);

  const clubById = useMemo(() => new Map(teams.map((t) => [t.id, t])), [teams]);

  const squadIds = useMemo(() => new Set(team?.players.map((sp) => sp.id) ?? []), [team]);

  function addPending(outId: number) {
    setPending((p) => [...p, { outId, inId: null }]);
  }

  function removePending(outId: number) {
    setPending((p) => p.filter((t) => t.outId !== outId));
  }

  function setReplacement(outId: number, inId: number) {
    setPending((p) => p.map((t) => (t.outId === outId ? { ...t, inId } : t)));
  }

  async function submit() {
    if (!token) return;
    const ready = pending.filter((t) => t.inId !== null);
    if (ready.length === 0) return;

    setSubmitError(null);
    setSubmitting(true);
    try {
      const result = await api.post<TransferSummary>(
        "/fantasy/transfers",
        { transfers: ready.map((t) => ({ player_out_id: t.outId, player_in_id: t.inId })) },
        token
      );
      setSummary(result);
      setPending([]);
      const teamData = await api.get<MyTeamResponse>("/fantasy/my-team", token);
      setTeam(teamData);
    } catch (e) {
      setSubmitError(e instanceof ApiError ? e.message : "Transfer failed");
    } finally {
      setSubmitting(false);
    }
  }

  if (!token) return <p style={{ color: "var(--muted)" }}>Connect your wallet first.</p>;
  if (loading) return <p style={{ color: "var(--muted)" }}>Loading…</p>;
  if (loadError || !team) return <p className="error-text">{loadError ?? "No squad found."}</p>;

  return (
    <section>
      <h1 style={{ fontSize: 28, marginBottom: 20 }}>Transfers</h1>

      {summary && (
        <div className="card" style={{ padding: 16, marginBottom: 20, display: "flex", gap: 24 }}>
          <SummaryStat label="Made" value={summary.transfers_made} />
          <SummaryStat label="Free used" value={summary.free_transfers_used} />
          <SummaryStat label="Hits" value={summary.hits} />
          <SummaryStat label="Points cost" value={-summary.points_cost} />
          <SummaryStat label="Free remaining" value={summary.free_transfers_remaining} />
        </div>
      )}

      <div className="card">
        {team.players.map((p) => {
          const pendingTransfer = pending.find((t) => t.outId === p.id);
          const isCaptainOrVice = p.id === team.captain.id || p.id === team.vice_captain.id;
          const club = p.team ? clubById.get(p.team) : undefined;

          const replacements = players.filter(
            (candidate) => candidate.position === p.position && !squadIds.has(candidate.id)
          );

          return (
            <div key={p.id} style={{ padding: "12px 16px", borderBottom: "1px solid var(--line)" }}>
              <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                <span className="tag">{p.position ? POSITION_LABEL[p.position] : "—"}</span>
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: 14, fontWeight: 600 }}>
                    {p.first_name} {p.second_name}
                  </div>
                  <div style={{ fontSize: 12, color: "var(--muted-2)" }}>
                    {club?.name ?? "—"} · {formatCost(p.now_cost ?? 0)}m
                  </div>
                </div>
                {pendingTransfer ? (
                  <button className="btn btn-ghost" style={{ padding: "5px 12px", fontSize: 12 }} onClick={() => removePending(p.id)}>
                    Cancel
                  </button>
                ) : (
                  <button
                    className="btn btn-ghost"
                    style={{ padding: "5px 12px", fontSize: 12 }}
                    disabled={isCaptainOrVice}
                    title={isCaptainOrVice ? "Change captain/vice-captain first" : undefined}
                    onClick={() => addPending(p.id)}
                  >
                    Transfer out
                  </button>
                )}
              </div>

              {pendingTransfer && (
                <select
                  value={pendingTransfer.inId ?? ""}
                  onChange={(e) => setReplacement(p.id, Number(e.target.value))}
                  style={{ width: "100%", marginTop: 10 }}
                >
                  <option value="">— pick a replacement ({p.position ? POSITION_LABEL[p.position] : ""}) —</option>
                  {replacements.map((r) => (
                    <option key={r.id} value={r.id}>
                      {r.first_name} {r.second_name} ({r.team_short_name}) · {formatCost(r.now_cost)}m
                    </option>
                  ))}
                </select>
              )}
            </div>
          );
        })}
      </div>

      {submitError && <p className="error-text" style={{ marginTop: 16 }}>{submitError}</p>}

      <button
        className="btn btn-primary"
        style={{ marginTop: 20 }}
        disabled={pending.length === 0 || pending.some((t) => t.inId === null) || submitting}
        onClick={submit}
      >
        {submitting ? "Submitting…" : `Confirm ${pending.length} transfer${pending.length === 1 ? "" : "s"}`}
      </button>
    </section>
  );
}

function SummaryStat({ label, value }: { label: string; value: number }) {
  return (
    <div>
      <b className="mono" style={{ display: "block", fontSize: 20, color: "var(--green-bright)" }}>
        {value}
      </b>
      <span style={{ fontSize: 11, color: "var(--muted-2)" }}>{label}</span>
    </div>
  );
}
