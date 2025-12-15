# rdx-core

Core primitives for the **Extended Convex Economy / Reaction–Diffusion** P2P evaluation scaffold:

- goods taxonomy as an `n`-vector (ex. AI-complementary human services)
- aggregated Cobb–Douglas preferences (`beta`)
- per-good pairwise parameters vs base good (`alpha_to_base`)
- dyadic two-good Pareto-optimal exchange oracle (default: Cobb–Douglas Walras equilibrium)
- P2P evaluation across all goods vs base and trade execution helpers
- codec boundary for P2P transmission of preference payloads (optional feature `mvcf`)
