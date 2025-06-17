# What it is

Sample implementation of a rule engine able to evaluate conditions on CSV files (based on evalexpr crate).

See test_data.csv & test_rules.json for sample data.

# Usage 

## test

Simple usage, file or directory

```bash
cargo run -- -i ./test_data.csv -r ./test_rules.json
```

Watch a directory

```bash
cargo run -- -i ./test_data_dir -r ./test_rules.json -p 10
```


## perf

```bash
cargo bench
```
