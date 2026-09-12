"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useAuth } from "@/components/AuthProvider";
import { api, ApiError, TeamPoints } from "@/lib/api";

export default function HomePage() {
  const { token } = useAuth();

  if (!token) {
    return <LoggedOutHero />;
  }

  return <Dashboard token={token} />;
}

function LoggedOutHero() {
  return (
    <section style={{ padding: "64px 0" }}>
      <div
        className="mono"
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 8,
          fontSize: 12,
          letterSpacing: "0.08em",
          color: "var(--green-bright)",
          border: "1px solid var(--line-strong)",
          padding: "6px 12px",
          borderRadius: 100,
          marginBottom: 22,
        }}
      >
        SOLANA WALLET AUTH
      </div>
      <h1 style={{ fontSize: 48, lineHeight: 1.05, maxWidth: "18ch" }}>
        Fantasy sport, <span style={{ color: "var(--green)" }}>settled on-chain.</span>
      </h1>
      <p style={{ marginTop: 20, fontSize: 16, color: "var(--muted)", maxWidth: "50ch" }}>
        Connect your Solana wallet in the top right to sign in — no email, no password. Your
        signature is the login.
      </p>
    </section>
  );
}

function Dashboard({ token }: { token: string }) {
  const [points, setPoints] = useState<TeamPoints | null>(null);
  const [noTeam, setNoTeam] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    api
      .get<TeamPoints>("/fantasy/my-points", token)
      .then((data) => {
        if (!cancelled) setPoints(data);
      })
      .catch((e) => {
        if (cancelled) return;
        if (e instanceof ApiError && e.status === 404) {
          setNoTeam(true);
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [token]);

  if (loading) {
    return <p style={{ color: "var(--muted)" }}>Loading…</p>;
  }

  if (noTeam) {
    return (
      <section className="card" style={{ padding: 40, textAlign: "center" }}>
        <h2 style={{ fontSize: 26 }}>Build your first squad</h2>
        <p style={{ marginTop: 12, color: "var(--muted)" }}>
          You&apos;re signed in, but you don&apos;t have a Campaign squad yet.
        </p>
        <Link href="/team/build" className="btn btn-primary" style={{ marginTop: 24 }}>
          Build squad →
        </Link>
      </section>
    );
  }

  return (
    <section>
      <h2 style={{ fontSize: 26, marginBottom: 20 }}>Gameweek {points?.gameweek}</h2>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(3, 1fr)",
          gap: 1,
          background: "var(--line)",
          border: "1px solid var(--line)",
          borderRadius: 10,
          overflow: "hidden",
        }}
      >
        <Stat label="Gross points" value={points?.gross_points ?? 0} />
        <Stat label="Transfer hits" value={-(points?.points_hit ?? 0)} />
        <Stat label="Total" value={points?.total_points ?? 0} highlight />
      </div>

      <div style={{ display: "flex", gap: 14, marginTop: 28 }}>
        <Link href="/team" className="btn btn-ghost">
          View squad
        </Link>
        <Link href="/transfers" className="btn btn-ghost">
          Make transfers
        </Link>
        <Link href="/leagues" className="btn btn-ghost">
          Leagues
        </Link>
      </div>
    </section>
  );
}

function Stat({ label, value, highlight }: { label: string; value: number; highlight?: boolean }) {
  return (
    <div style={{ background: "var(--panel)", padding: "18px 20px" }}>
      <b
        className="mono"
        style={{
          display: "block",
          fontSize: 24,
          color: highlight ? "var(--green-bright)" : "var(--text)",
        }}
      >
        {value}
      </b>
      <span style={{ fontSize: 12.5, color: "var(--muted-2)" }}>{label}</span>
    </div>
  );
}
