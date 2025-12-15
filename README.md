# Reaction–Diffusion Exchange Economy for Human Services Complementing AI (Rust)

This workspace provides a **research-grade simulation and analysis scaffold** for modeling the *human-based economic services that complement AI* (data provenance, labeling, HITL operations, governance, safety, UX, domain oversight, physical-world verification, etc.) as an **n-good exchange economy** within the **Extended Convex Economy / Reaction–Diffusion** framework.

The goal is to treat each service category as a **good** in a conserved endowment vector, and to support **peer-to-peer (P2P) dyadic exchange evaluation** of *every good* against a **base good** (numeraire), using **aggregated Cobb–Douglas preferences** and a **Pareto-optimal two-good exchange oracle**.

## What this code is intended for

- Represent a 63-service taxonomy as a **goods vector** (plus a base good).
- Give each agent:
  - an endowment vector over goods,
  - an aggregated Cobb–Douglas preference profile (`beta`),
  - and per-good pairwise parameters against the base good (`alpha_to_base`).
- For each P2P encounter between two agents, evaluate **each candidate good A vs base good B**:
  1. compute a **quotient** (exchange rate) `Q_AB` (implemented as the equilibrium/clearing price ratio),
  2. compute a Pareto-optimal exchange point for (A,B),
  3. evaluate the utility change for both peers using their **full n-good Cobb–Douglas** utility,
  4. rank goods by mutual benefit and execute the best feasible trade.

The “reaction” part (endogenous transformation rules) is included as a module boundary (production rules are pluggable), but this workspace focuses primarily on the **diffusion / P2P exchange evaluation** logic, this is its exchange Pareto-optimal setup.

## Repository layout

- `crates/rdx-core`: core model types (goods list, preferences, P2P evaluation, Pareto oracle, simulation loop)
- `crates/rdx-cli`: CLI runner that generates a reproducible synthetic economy and outputs CSV traces

## Quickstart

```bash
cargo run -p rdx-cli --release -- --config config/example.json
```

Outputs are written to `out/`:
- `p2p_trades.csv`: executed P2P trades (good A, good B, agents, deltas, utilities)
- `prices.csv`: per-round implied exchange rates vs base (optional trace)
- `endowments_mean.csv`: mean holdings per good per round

## About the multivariate codec crate

This implementation uses `labormedia/multivariate-convex-function` for coding/decoding preference payloads.
Because public APIs may evolve, this project includes an **optional** feature flag:

- default: uses `serde_json` for encoding/decoding
- `--features mvcf`: switches the codec boundary to call into the external crate

After crate’s public API confirmation, encode/decode calls can be wired in `crates/rdx-core/src/codec.rs`.

## Notes on f(...) and Exchange(...)

These solution logics are interchangeable given a direct approach. The treatment is to consider the p2p exchange parameters extracted from each peer, alpha + endowments, to calculate the Pareto-optimal exchange quantities:

- `Q_AB = f(...)`
- `(e1_AB, e2_AB) = Exchange(..., Q_AB)`

as endogenous, producing the exact Pareto-optimal point.

In this scaffold, those are represented by a **`ParetoOracle` trait** with a default
`CobbDouglasWalrasOracle` implementation that computes a **Walrasian equilibrium** for the two-good
exchange (which is Pareto efficient). The oracle can be replaced with any alternative solver
while keeping the P2P evaluation pipeline stable.

## License

MIT.

## Pairing modes

The config supports two pairing modes:

- `against_base`: evaluate each candidate good `A` against the base good `B` only.
- `all_pairs_pruned`: evaluate all ordered pairs `(A,B)` but only inside a pruned candidate set
  per encounter (size `candidate_goods_k`, plus the base good). This approximates “evaluate across
  the whole goods vector” while keeping runtime tractable.

Example (in `config/example.json`):

```json
{
  "pairing_mode": "all_pairs_pruned",
  "candidate_goods_k": 12
}
```
