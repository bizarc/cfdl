---
id: install-python
title: Install for Python
slug: /docs/install/python
---

# Install for Python

`cfdl_sdk` embeds the compiler and engine in-process (a Rust extension
module) — no separate binary or server.

## pip (at launch)

```bash
pip install cfdl-sdk
```

## From a checkout (current pre-launch path)

```bash
git clone https://github.com/bizarc/cfdl
cd cfdl
pip install -e "python/[dev,viz]"
```

Editable installs build the native module via maturin; a Rust toolchain is
required for this path (prebuilt wheels are not).

## Supported platforms

CPython ≥ 3.10 (abi3 wheels): Linux x86_64/aarch64 (manylinux), macOS
universal2, Windows x64. `pandas>=2.0` is a hard dependency; the optional
`viz` extra adds matplotlib.

## Verify

```python
import cfdl_sdk
results = cfdl_sdk.run("examples/language_tutorial/minimal_model", rate=0.10)
print(results.metrics())
```

Next: [Python SDK usage](../python-sdk) · executed example
[notebooks](../python-sdk#notebooks)
