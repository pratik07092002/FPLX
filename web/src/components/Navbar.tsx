"use client";

import Link from "next/link";
import dynamic from "next/dynamic";
import { useAuth } from "./AuthProvider";

const WalletMultiButton = dynamic(
  async () => (await import("@solana/wallet-adapter-react-ui")).WalletMultiButton,
  { ssr: false }
);

export function Navbar() {
  const { token, isAuthenticating } = useAuth();

  return (
    <nav
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "20px 32px",
        borderBottom: "1px solid var(--line)",
      }}
    >
      <Link
        href="/"
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          fontFamily: "var(--disp)",
          fontSize: 20,
          fontWeight: 700,
        }}
      >
        <span
          style={{
            width: 9,
            height: 9,
            borderRadius: 2,
            background: "var(--green)",
            boxShadow: "0 0 12px 2px rgba(47,224,122,.6)",
          }}
        />
        FPLX
      </Link>

      {token && (
        <div style={{ display: "flex", gap: 26, fontSize: 14, color: "var(--muted)" }}>
          <Link href="/team">My Team</Link>
          <Link href="/team/build">Build Squad</Link>
          <Link href="/leagues">Leagues</Link>
          <Link href="/transfers">Transfers</Link>
        </div>
      )}

      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        {isAuthenticating && (
          <span className="mono" style={{ fontSize: 12, color: "var(--green-bright)" }}>
            signing in…
          </span>
        )}
        <WalletMultiButton />
      </div>
    </nav>
  );
}
