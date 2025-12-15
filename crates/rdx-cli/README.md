# rdx-cli

CLI runner for the rdx-core simulation.

```bash
cargo run -p rdx-cli --release -- --config config/example.json --out-dir out
```

Outputs:
- `out/p2p_trades.csv` executed trades
- `out/endowments_mean.csv` mean holdings by good
- `out/config_used.json` parameters
