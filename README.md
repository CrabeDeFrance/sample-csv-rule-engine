# What it is

Sample implementation of a rule engine able to evaluate conditions on CSV files (based on evalexpr crate).

See test_data.csv & test_rules.json for sample data.

# Usage 

## test

```bash
cargo run -- -c ./test_data.csv -r ./test_rules.json
```

## perf

```bash
cargo bench
```
