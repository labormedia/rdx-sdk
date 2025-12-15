use rand::prelude::*;
use crate::model::{Agent, SimConfig, TradeEvent, PairingMode};
use crate::preferences::{beta_from_alpha_to_base, cd_utility};
use crate::trade::{best_trade_against_base, best_trade_over_all_pairs_pruned, apply_trade, default_oracle};

#[derive(Clone, Debug)]
pub struct SimState {
    pub agents: Vec<Agent>,
    pub events: Vec<TradeEvent>,
}

pub fn init_agents(cfg: &SimConfig) -> SimState {
    let goods_qty = cfg.base_goods_quantity;
    let n = cfg.base_goods.len();
    
    assert_eq!(goods_qty, n, "[Safe Panic] Goods Quantity mismatch in configuration");
    assert!(n >= 2, "[Safe Panic] Goods quantity is less than 2");
    
    let mut rng = StdRng::seed_from_u64(cfg.seed);

    let mut agents = Vec::with_capacity(cfg.num_agents);
    for _ in 0..cfg.num_agents {
        // endowments: positive, comparable scale
        let e = (0..n)
            .map(|_| rng.gen_range(0.5..2.0) * cfg.initial_endowment_scale)
            .collect::<Vec<f64>>();

        // alpha_to_base: only meaningful for k != base, set base to 0.5 convention
        let mut alpha_to_base = vec![0.5; n];
        for k in 0..n {
            if k == cfg.base_good { continue; }
            alpha_to_base[k] = rng.gen_range(cfg.alpha_low..cfg.alpha_high);
        }

        let beta = beta_from_alpha_to_base(&alpha_to_base, cfg.base_good, 1e-6);

        agents.push(Agent { e, beta, alpha_to_base });
    }

    SimState { agents, events: Vec::new() }
}

/// Run diffusion rounds with P2P encounters. Reaction rules can be plugged in before calling `run`.
pub fn run(cfg: &SimConfig, state: &mut SimState) {
    let mut rng = StdRng::seed_from_u64(cfg.seed ^ 0xA5A5_A5A5_A5A5_A5A5);
    let oracle = default_oracle();

    for t in 0..cfg.rounds {
        for _ in 0..cfg.p2p_encounters_per_round {
            let i = rng.gen_range(0..state.agents.len());
            let mut j = rng.gen_range(0..state.agents.len());
            while j == i {
                j = rng.gen_range(0..state.agents.len());
            }

            let (ai, aj) = {
                let (left, right) = state.agents.split_at_mut(j.max(i));
                if i < j {
                    (&mut left[i], &mut right[0])
                } else {
                    (&mut right[0], &mut left[j])
                }
            };

            // Snapshot utilities pre-trade for logging
            let ui0 = cd_utility(&ai.beta, &ai.e, cfg.min_qty);
            let uj0 = cd_utility(&aj.beta, &aj.e, cfg.min_qty);

            let cand = match cfg.pairing_mode {
                PairingMode::AgainstBase => best_trade_against_base(
                    ai, aj, cfg.base_good, cfg.min_qty, cfg.oracle_bisect_iters, &oracle
                ),
                PairingMode::AllPairsPruned => best_trade_over_all_pairs_pruned(
                    ai, aj, cfg.base_good, cfg.candidate_goods_k, cfg.min_qty, cfg.oracle_bisect_iters, &oracle
                ),
            };

            if let Some(mut cand) = cand {
                // Apply (conservative step cap): scale deltas to avoid huge jumps.
                let cap = cfg.trade_step_cap_frac.clamp(0.0, 1.0);
                if cap < 1.0 {
                    cand.delta_a_i *= cap;
                    cand.delta_b_i *= cap;
                }

                apply_trade(ai, aj, &cand, cfg.min_qty);

                // Utilities post trade
                let ui1 = cd_utility(&ai.beta, &ai.e, cfg.min_qty);
                let uj1 = cd_utility(&aj.beta, &aj.e, cfg.min_qty);

                state.events.push(TradeEvent {
                    round: t,
                    i,
                    j,
                    good_a: cand.good_a,
                    good_b: cand.good_b,
                    q_ab: cand.q_ab,
                    delta_a_i: cand.delta_a_i,
                    delta_b_i: cand.delta_b_i,
                    delta_u_i: ui1 - ui0,
                    delta_u_j: uj1 - uj0,
                });
            }
        }
    }
}

pub fn mean_endowments(state: &SimState) -> Vec<f64> {
    let n = state.agents[0].e.len();
    let mut mean = vec![0.0; n];
    for ag in state.agents.iter() {
        for k in 0..n {
            mean[k] += ag.e[k];
        }
    }
    for k in 0..n {
        mean[k] /= state.agents.len() as f64;
    }
    mean
}
