// Mirrors src/helpers/squad_rules.rs — the backend is the source of truth,
// this just lets the UI give instant feedback before a round trip.

export const POSITION_LABEL: Record<number, string> = {
  1: "GK",
  2: "DEF",
  3: "MID",
  4: "FWD",
};

export const SQUAD_SIZE = 15;
export const STARTING_XI_SIZE = 11;
export const MAX_PER_CLUB = 3;
export const BUDGET_LIMIT = 1000; // tenths of a million -> 100.0m

export const REQUIRED_BY_POSITION: Record<number, number> = {
  1: 2,
  2: 5,
  3: 5,
  4: 3,
};

export const STARTING_RANGE_BY_POSITION: Record<number, [number, number]> = {
  1: [1, 1],
  2: [3, 5],
  3: [2, 5],
  4: [1, 3],
};

export function formatCost(now_cost: number): string {
  return (now_cost / 10).toFixed(1);
}
