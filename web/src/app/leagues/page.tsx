"use client";

import { useState } from "react";
import { useAuth } from "@/components/AuthProvider";
import { api, ApiError, LeaderboardEntry, LeagueResponse } from "@/lib/api";

type ContestType = "campaign" | "derby" | "round";

export default function LeaguesPage() {
  const { token } = useAuth();

  if (!token) return <p style={{ color: "var(--muted)" }}>Connect your wallet first.</p>;

  return (
    <section style={{ display: "grid", gap: 24 }}>
      <h1 style={{ fontSize: 28 }}>Leagues</h1>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 24 }}>
        <CreateLeagueCard token={token} />
        <JoinLeagueCard token={token} />
      </div>
      <LeaderboardCard />
    </section>
  );
}

function CreateLeagueCard({ token }: { token: string }) {
  const [name, setName] = useState("");
  const [leagueType, setLeagueType] = useState<"public" | "private">("public");
  const [contestType, setContestType] = useState<ContestType>("campaign");
  const [fixtureId, setFixtureId] = useState("");
  const [gameweek, setGameweek] = useState("");
  const [result, setResult] = useState<LeagueResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function submit() {
    setError(null);
    setResult(null);
    setSubmitting(true);
    try {
      const body: Record<string, unknown> = {
        league_name: name,
        league_type: leagueType,
        contest_type: contestType,
      };
      if (contestType === "derby") body.fixture_id = Number(fixtureId);
      if (contestType === "round") body.gameweek = Number(gameweek);

      const league = await api.post<LeagueResponse>("/fantasy/create-league", body, token);
      setResult(league);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : "Failed to create league");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="card" style={{ padding: 22 }}>
      <h3 style={{ fontSize: 16, marginBottom: 16 }}>Create a league</h3>

      <Field label="Name">
        <input type="text" value={name} onChange={(e) => setName(e.target.value)} style={{ width: "100%" }} />
      </Field>

      <Field label="Visibility">
        <select value={leagueType} onChange={(e) => setLeagueType(e.target.value as "public" | "private")} style={{ width: "100%" }}>
          <option value="public">Public</option>
          <option value="private">Private (join code)</option>
        </select>
      </Field>

      <Field label="Format">
        <select value={contestType} onChange={(e) => setContestType(e.target.value as ContestType)} style={{ width: "100%" }}>
          <option value="campaign">Campaign — season-long</option>
          <option value="derby">Derby — single fixture</option>
          <option value="round">Round — single gameweek</option>
        </select>
      </Field>

      {contestType === "derby" && (
        <Field label="Fixture ID">
          <input type="number" value={fixtureId} onChange={(e) => setFixtureId(e.target.value)} style={{ width: "100%" }} />
        </Field>
      )}

      {contestType === "round" && (
        <Field label="Gameweek">
          <input type="number" value={gameweek} onChange={(e) => setGameweek(e.target.value)} style={{ width: "100%" }} />
        </Field>
      )}

      {error && <p className="error-text" style={{ marginBottom: 10 }}>{error}</p>}

      {result && (
        <div className="mono" style={{ fontSize: 12, color: "var(--green-bright)", marginBottom: 10 }}>
          Created league #{result.id}
          {result.join_code && <> · code {result.join_code}</>}
        </div>
      )}

      <button className="btn btn-primary" disabled={!name || submitting} onClick={submit}>
        {submitting ? "Creating…" : "Create league"}
      </button>
    </div>
  );
}

function JoinLeagueCard({ token }: { token: string }) {
  const [leagueId, setLeagueId] = useState("");
  const [joinCode, setJoinCode] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function submit() {
    setError(null);
    setMessage(null);
    setSubmitting(true);
    try {
      await api.post(`/fantasy/leagues/${leagueId}/join`, { join_code: joinCode || null }, token);
      setMessage("Joined!");
    } catch (e) {
      setError(e instanceof ApiError ? e.message : "Failed to join league");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="card" style={{ padding: 22 }}>
      <h3 style={{ fontSize: 16, marginBottom: 16 }}>Join a league</h3>

      <Field label="League ID">
        <input type="number" value={leagueId} onChange={(e) => setLeagueId(e.target.value)} style={{ width: "100%" }} />
      </Field>

      <Field label="Join code (private leagues only)">
        <input type="text" value={joinCode} onChange={(e) => setJoinCode(e.target.value)} style={{ width: "100%" }} />
      </Field>

      <p style={{ fontSize: 12, color: "var(--muted-2)", marginBottom: 12 }}>
        Campaign leagues join on your existing squad. Derby/Round leagues need a one-shot entry —
        not yet available from this page.
      </p>

      {error && <p className="error-text" style={{ marginBottom: 10 }}>{error}</p>}
      {message && <p style={{ color: "var(--green-bright)", marginBottom: 10 }}>{message}</p>}

      <button className="btn btn-primary" disabled={!leagueId || submitting} onClick={submit}>
        {submitting ? "Joining…" : "Join league"}
      </button>
    </div>
  );
}

function LeaderboardCard() {
  const [leagueId, setLeagueId] = useState("");
  const [entries, setEntries] = useState<LeaderboardEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function load() {
    setError(null);
    setLoading(true);
    try {
      const data = await api.get<LeaderboardEntry[]>(`/fantasy/leagues/${leagueId}/leaderboard`);
      setEntries(data);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : "Failed to load leaderboard");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="card" style={{ padding: 22 }}>
      <h3 style={{ fontSize: 16, marginBottom: 16 }}>View leaderboard</h3>
      <div style={{ display: "flex", gap: 10, marginBottom: 16 }}>
        <input
          type="number"
          placeholder="League ID"
          value={leagueId}
          onChange={(e) => setLeagueId(e.target.value)}
          style={{ width: 160 }}
        />
        <button className="btn btn-ghost" disabled={!leagueId || loading} onClick={load}>
          {loading ? "Loading…" : "Load"}
        </button>
      </div>

      {error && <p className="error-text">{error}</p>}

      {entries && (
        <div>
          {entries.map((e, i) => (
            <div
              key={e.user_id}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 12,
                padding: "8px 0",
                borderBottom: "1px solid var(--line)",
              }}
            >
              <span className="mono" style={{ width: 24, color: "var(--muted-2)" }}>
                {i + 1}
              </span>
              <span style={{ flex: 1, fontSize: 14 }}>{e.user_name}</span>
              <span className="mono" style={{ color: "var(--green-bright)" }}>
                {e.total_points}
              </span>
            </div>
          ))}
          {entries.length === 0 && <p style={{ color: "var(--muted)" }}>No entrants yet.</p>}
        </div>
      )}
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={{ marginBottom: 14 }}>
      <label style={{ display: "block", fontSize: 12, color: "var(--muted)", marginBottom: 4 }}>{label}</label>
      {children}
    </div>
  );
}
