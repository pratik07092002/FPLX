const API_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

export class ApiError extends Error {
  status: number;
  constructor(message: string, status: number) {
    super(message);
    this.status = status;
  }
}

interface ApiResponse<T> {
  success: boolean;
  status_code: number;
  message: string;
  data: T | null;
}

async function request<T>(
  path: string,
  options: RequestInit = {},
  token?: string | null
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string> | undefined),
  };

  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const res = await fetch(`${API_URL}${path}`, { ...options, headers });

  let body: ApiResponse<T> | null = null;
  try {
    body = await res.json();
  } catch {
    // non-JSON response (e.g. network-level failure page)
  }

  if (!res.ok || !body?.success) {
    throw new ApiError(body?.message ?? `Request failed (${res.status})`, res.status);
  }

  return body.data as T;
}

export const api = {
  get: <T>(path: string, token?: string | null) => request<T>(path, { method: "GET" }, token),

  post: <T>(path: string, body?: unknown, token?: string | null) =>
    request<T>(path, { method: "POST", body: body ? JSON.stringify(body) : undefined }, token),
};

// ---------- Types matching the Rust API ----------

export interface Team {
  id: number;
  name: string;
  short_name: string;
  position: number;
}

export interface PlayerListItem {
  id: number;
  first_name: string;
  second_name: string;
  photo: string | null;
  team_id: number;
  team_name: string;
  team_short_name: string;
  position: number; // 1 GK, 2 DEF, 3 MID, 4 FWD
  now_cost: number; // tenths of a million
  form: number | null;
  total_points: number | null;
  news: string | null;
}

// The shape /fantasy/my-team's players actually come back in — Rust's
// FPLPlayer struct, flattened, which carries a bare `team` id (no name/
// short_name joined in) rather than PlayerListItem's catalog shape.
export interface SquadPlayer {
  id: number;
  first_name: string | null;
  second_name: string | null;
  photo: string | null;
  team: number | null;
  position: number | null;
  now_cost: number | null;
  total_points: number | null;
  news: string | null;
}

export interface SquadPlayerView extends SquadPlayer {
  is_starting: boolean;
  bench_order: number | null;
}

export interface MyTeamResponse {
  team_id: string;
  captain: SquadPlayer;
  vice_captain: SquadPlayer;
  players: SquadPlayerView[];
}

export interface PlayerPoints {
  player_id: number;
  first_name: string;
  second_name: string;
  position: number;
  points: number;
  is_captain: boolean;
  is_vice_captain: boolean;
  is_starting: boolean;
}

export interface TeamPoints {
  team_id: string;
  gameweek: number;
  gross_points: number;
  points_hit: number;
  total_points: number;
  players: PlayerPoints[];
}

export interface LeagueResponse {
  id: number;
  league_name: string;
  league_type: "public" | "private";
  contest_type: "campaign" | "derby" | "round";
  fixture_id: number | null;
  gameweek: number | null;
  join_code: string | null;
}

export interface LeaderboardEntry {
  user_id: string;
  user_name: string;
  total_points: number;
}

export interface TransferSummary {
  gameweek: number;
  transfers_made: number;
  free_transfers_used: number;
  hits: number;
  points_cost: number;
  free_transfers_remaining: number;
}
