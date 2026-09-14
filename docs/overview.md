# FPLX — What This Project Is

*A plain-language guide for anyone on the team who doesn't read code.*

## The idea in one sentence

FPLX is a fantasy sports platform — pick real players, score points based on how they actually perform — where entry fees and payouts move through a Solana wallet instead of a credit card, and matches are settled automatically instead of trusted to whoever runs the app.

## The problem it's solving

Fantasy sports platforms (FPL, Dream11, and the rest) all work the same way: you build a squad, you win or lose based on real-world results, and if money is involved, you have to trust the company running the app to hold your money and pay out honestly. That trust is the whole weak point — it's why paid fantasy contests are legally messy in a lot of places, and why "the app just didn't pay out" is a real complaint people have about this category.

FPLX's bet is that moving the money part onto Solana (a fast, cheap blockchain) removes the "trust us" requirement. Entry fees sit in an on-chain escrow, not a company bank account. Nobody can quietly not pay out.

## How someone actually uses it

1. **Sign in with a crypto wallet.** No email, no password. You connect a Solana wallet (Phantom or Solflare today), the app asks your wallet to sign a one-time message, and that signature *is* your login. Nobody ever sees a password because there isn't one.
2. **Build a squad.** Same shape as Fantasy Premier League: 15 players, a fixed budget (100.0m in-game credits), rules about how many players you can have from one real club, and you pick a captain (whose points count double) and a vice-captain (backup captain).
3. **Pick a competition format.** This is where FPLX differs from official FPL — see below.
4. **Watch it score live.** As real matches happen, the platform pulls live stats from the Premier League's own data feed and recalculates your points — goals, assists, clean sheets, cards, bonus points — using the same scoring rules real FPL uses.
5. **Make transfers.** Between gameweeks, swap players in and out. You get one free swap per week (it banks up if you don't use it, up to five), and extra swaps cost you 4 points each — exactly like the real game.

## The three ways to play

This is the core product idea from this phase of the build — most fantasy platforms only offer one of these:

| Format | How long it lasts | What you build |
|---|---|---|
| **Campaign** | The whole season | One persistent squad you tweak week to week with transfers — this is the "classic FPL" experience |
| **Derby** | One single match | An 11-player team picked only from the two clubs playing in that match — a quick, single-game contest |
| **Round** | One gameweek | A fresh 15-player squad just for that week's fixtures, no transfers, start from scratch next week |

You can play all three at once with completely separate squads — a Campaign team for the season, a Derby entry for tonight's match, and a Round entry for this week, all under one wallet login.

## What's actually built right now

- Wallet login (real, tested with actual cryptographic signatures — not a placeholder)
- Live Premier League data sync (teams, players, fixtures, live match stats — polls the same data FPL's own live tracker uses)
- A real scoring engine implementing FPL's current point rules
- Squad building with full rule validation (budget, quotas, captain rules) for all three formats
- Transfers with the free-transfer/points-hit economy
- Leagues: create public or private contests of any of the three formats, join them, see a leaderboard
- A working website (black-and-green themed) where you can do all of the above
- A production-ready deployment setup (Docker) so this can actually be hosted somewhere, not just run on one laptop

## What's deliberately not built yet

- **Nothing is actually on-chain yet.** Wallets are used for login (real cryptographic signatures), but entry fees and payouts described in the pitch are not wired to an actual Solana program yet — that's the next big technical step if this is going to compete for something like Colosseum, where "on-chain" needs to be real, not just a wallet-login gimmick.
- **No cricket, no other leagues yet.** Right now it's Premier League football only. The architecture (the three-contest-format system) was built to extend to other sports later, but the data pipeline and scoring rules are Premier-League-specific today.
- **No mobile app.** There's a Flutter app in the repo from an earlier phase, but it's on hold — all current work is the website.
- **A few product gaps in the website itself:** you can't yet see a list of "leagues I've joined" in one place, and joining a Derby or Round contest (which needs you to submit a one-off squad) isn't wired into the website UI yet — you can do it via the API, just not by clicking through the site.

## Why this matters for the Colosseum pitch

Colosseum is Solana's accelerator/hackathon program — it rewards projects that are genuinely built *on* Solana, not projects that just bolted a "Connect Wallet" button onto an otherwise ordinary web app. Right now FPLX is honestly in the second category: it uses a Solana wallet for identity, which is a real and working piece, but the part that would make it a genuine Solana application — entry fees actually held in an on-chain escrow, payouts happening as a program instruction instead of a database update — hasn't been built yet. That's the single highest-leverage thing to do next if the goal is to be taken seriously as a Solana-native product rather than a normal fantasy app with a crypto login screen.

## Where things stand

All of this lives on a branch called `colosseum-2026`, not yet merged into the main codebase — it's been built feature by feature, with everything tested against a real database (and a completely fresh one, to catch the kind of bug that only shows up on day one of a new deployment) before moving to the next piece.
