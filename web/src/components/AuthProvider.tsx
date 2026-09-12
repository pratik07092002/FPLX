"use client";

import { createContext, useCallback, useContext, useEffect, useRef, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { api, ApiError } from "@/lib/api";

interface AuthState {
  token: string | null;
  walletAddress: string | null;
  isAuthenticating: boolean;
  error: string | null;
  logout: () => void;
}

const AuthContext = createContext<AuthState>({
  token: null,
  walletAddress: null,
  isAuthenticating: false,
  error: null,
  logout: () => {},
});

export function useAuth() {
  return useContext(AuthContext);
}

function toBase64(bytes: Uint8Array): string {
  let binary = "";
  bytes.forEach((b) => (binary += String.fromCharCode(b)));
  return btoa(binary);
}

function tokenKey(wallet: string) {
  return `fplx_token_${wallet}`;
}

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const { publicKey, signMessage, connected, disconnect } = useWallet();
  const [token, setToken] = useState<string | null>(null);
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const attemptedFor = useRef<string | null>(null);

  const authenticate = useCallback(async () => {
    if (!publicKey || !signMessage) return;
    const addr = publicKey.toBase58();

    if (attemptedFor.current === addr) return;
    attemptedFor.current = addr;

    const existing = localStorage.getItem(tokenKey(addr));
    if (existing) {
      setToken(existing);
      setWalletAddress(addr);
      return;
    }

    setIsAuthenticating(true);
    setError(null);

    try {
      const { nonce } = await api.post<{ nonce: string }>("/auth/nonce", { wallet: addr });
      const signature = await signMessage(new TextEncoder().encode(nonce));
      const { token: jwt } = await api.post<{ token: string }>("/auth/verify", {
        wallet: addr,
        signature: toBase64(signature),
      });

      localStorage.setItem(tokenKey(addr), jwt);
      setToken(jwt);
      setWalletAddress(addr);
    } catch (e) {
      attemptedFor.current = null;
      setError(e instanceof ApiError ? e.message : "Failed to sign in with wallet");
    } finally {
      setIsAuthenticating(false);
    }
  }, [publicKey, signMessage]);

  useEffect(() => {
    if (connected && publicKey && signMessage) {
      authenticate();
    }
    if (!connected) {
      attemptedFor.current = null;
      setToken(null);
      setWalletAddress(null);
    }
  }, [connected, publicKey, signMessage, authenticate]);

  const logout = useCallback(() => {
    if (walletAddress) {
      localStorage.removeItem(tokenKey(walletAddress));
    }
    setToken(null);
    setWalletAddress(null);
    attemptedFor.current = null;
    disconnect().catch(() => {});
  }, [walletAddress, disconnect]);

  return (
    <AuthContext.Provider value={{ token, walletAddress, isAuthenticating, error, logout }}>
      {children}
    </AuthContext.Provider>
  );
}
