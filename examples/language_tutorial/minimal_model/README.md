# Minimal Model

This is the smallest practical CFDL model:

- required header statements
- one entity
- one stream

This example uses a **standalone stream** for rent (guidance: if in doubt, start with a stream).

Compile:

```bash
./target/debug/cfdl compile examples/language_tutorial/minimal_model --out /tmp/tutorial_minimal.ir.json
```
