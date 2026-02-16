# CFDL Language Tutorial Examples

These examples map directly to `docs/LANGUAGE_GUIDE.md`.

## Examples

- `minimal_model/` - smallest practical model
- `first_stream/` - stream scheduling and expression basics
- `simple_contract/` - single pack-based contract
- `with_pack/` - multiple pack-based contracts
- `multi_file/` - import-based project layout

## Run

Build CLI once from repo root:

```bash
cargo build -p cfdl-cli
```

Compile a tutorial model:

```bash
./target/debug/cfdl compile examples/language_tutorial/minimal_model --out /tmp/tutorial.ir.json
```

For pack examples, include packs path:

```bash
./target/debug/cfdl compile examples/language_tutorial/with_pack --out /tmp/tutorial_pack.ir.json --packs packs
```
