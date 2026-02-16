---
id: getting-started
title: Getting Started
---

# Getting Started

## Build CLI

```bash
cargo build -p cfdl-cli
```

## Compile first model

```bash
./target/debug/cfdl compile examples/language_tutorial/minimal_model --out /tmp/tutorial.ir.json
```

## Run compiled IR

```bash
./target/debug/cfdl run /tmp/tutorial.ir.json --out /tmp/tutorial.results.json --rate 0.10
```

## Next step

- Continue to the full [Language Guide](language-guide).
