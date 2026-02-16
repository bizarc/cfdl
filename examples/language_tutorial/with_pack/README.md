# With Pack

This example shows a slightly larger pack-enabled model using pack **contracts** for revenue and opex. In real models, individual revenue/opex items are often modeled as **streams**; see the Language Guide "When to use streams vs contracts."

Compile:

```bash
./target/debug/cfdl compile examples/language_tutorial/with_pack --out /tmp/tutorial_with_pack.ir.json --packs packs
```
