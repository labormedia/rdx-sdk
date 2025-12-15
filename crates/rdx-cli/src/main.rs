use anyhow::Context;
use clap::Parser;
use rdx_core::model::SimConfig;
use rdx_core::sim::{init_agents, run, mean_endowments};
use std::fs;

#[derive(Parser, Debug)]
#[command(name="rdx-cli", about="Reaction–Diffusion P2P exchange simulator for AI-complementary human services")]
struct Args {
    /// Path to JSON config
    #[arg(long, default_value="config/example.json")]
    config: String,

    /// Output directory
    #[arg(long, default_value="out")]
    out_dir: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let cfg_str = fs::read_to_string(&args.config)
        .with_context(|| format!("failed reading config: {}", args.config))?;
    let cfg: SimConfig = serde_json::from_str(&cfg_str)
        .with_context(|| "invalid config json")?;

    fs::create_dir_all(&args.out_dir)?;

    // init and run
    if cfg.base_goods.len() < 2 {
        anyhow::bail!("config.goods must contain at least 2 entries");
    }
    if cfg.base_goods_quantity != cfg.base_goods.len() {
        anyhow::bail!("base_goods_quantity out of bounds");
    }
    let goods = &cfg.base_goods;
    let mut state = init_agents(&cfg);
    run(&cfg, &mut state);

    // write events csv
    let events_path = format!("{}/p2p_trades.csv", args.out_dir);
    let mut wtr = csv::Writer::from_path(&events_path)?;
    wtr.write_record(&[
        "round","i","j","good_a","good_a_name","good_b","good_b_name",
        "q_ab","delta_a_i","delta_b_i","delta_u_i","delta_u_j"
    ])?;
    for ev in state.events.iter() {
        wtr.write_record(&[
            ev.round.to_string(),
            ev.i.to_string(),
            ev.j.to_string(),
            ev.good_a.to_string(),
            goods[ev.good_a].clone(),
            ev.good_b.to_string(),
            goods[ev.good_b].clone(),
            format!("{:.10}", ev.q_ab),
            format!("{:.10}", ev.delta_a_i),
            format!("{:.10}", ev.delta_b_i),
            format!("{:.10}", ev.delta_u_i),
            format!("{:.10}", ev.delta_u_j),
        ])?;
    }
    wtr.flush()?;

    // write mean endowments
    let mean = mean_endowments(&state);
    let mean_path = format!("{}/endowments_mean.csv", args.out_dir);
    let mut wtr2 = csv::Writer::from_path(&mean_path)?;
    wtr2.write_record(&["good","name","mean_qty"])?;
    for k in 0..mean.len() {
        wtr2.write_record(&[
            k.to_string(),
            goods[k].to_string(),
            format!("{:.10}", mean[k]),
        ])?;
    }
    wtr2.flush()?;

    // persist config used
    fs::write(format!("{}/config_used.json", args.out_dir), serde_json::to_string_pretty(&cfg)?)?;

    println!("Done. Wrote:");
    println!(" - {}", events_path);
    println!(" - {}", mean_path);
    println!(" - {}/config_used.json", args.out_dir);

    Ok(())
}
