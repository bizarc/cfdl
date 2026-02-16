# Multi-File Layout

This example demonstrates splitting model content by concern:

- `model.cfdl` for header + imports
- `structure.cfdl` for entities
- `contracts.cfdl` for streams/contracts/events

Compile:

```bash
./target/debug/cfdl compile examples/language_tutorial/multi_file --out /tmp/tutorial_multi_file.ir.json
```
