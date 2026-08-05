---
id: guide-multi-file
title: Multi-File Models
slug: /docs/guides/multi-file-models
generated: none
---

# Multi-File Models

Split a growing model by concern:

```
my-deal/
  model.cfdl        # header: version, model, use pack, imports
  time.cfdl         # timeline and phases
  structure.cfdl    # entities
  contracts.cfdl    # contracts, streams, events
```

`model.cfdl`:

```cfdl
version 0.1
model "my-deal"
use pack "cre" version "0.1.0"

import "time.cfdl"
import "structure.cfdl"
import "contracts.cfdl"
```

## Rules

- Import paths are **relative to the importing file**.
- No import cycles.
- Imports cannot escape the model root directory.

Violations produce compile diagnostics (unresolved import, cycle,
root-escape) with the offending path and span.

## Compiling

Point the CLI at the directory; it starts from `model.cfdl`:

```bash
cfdl compile my-deal --packs packs --out my-deal/ir.json
```

Worked example: `examples/language_tutorial/multi_file/` and the larger
[CRE multi-file example](/docs/examples/cre_multi_file).

## Reference links

- [Language Guide — multi-file models](/docs/language-guide)
