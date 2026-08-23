#!/usr/bin/env python3
"""Every pack template must render and compile.

A template is what the editor inserts when a modeller reaches for a contract,
so one that does not compile is worse than none: it teaches a shape the
language rejects, and the modeller debugs the pack's own snippet.

Rendering uses the template's declared defaults, which is what the LSP does
when no parameter is supplied — so this also pins that the defaults are a
usable starting point rather than placeholders that fail on arrival. Both
defects it caught on its first run were real: a `principal = 0` default the
pack's own validation rejects (E6051, E8022, E9001), and terms whose declared
range outran the model it was placed in.

The model wrapped around each template is minimal — a timeline, one entity of
the pack's subject family, and any curve the body names.
"""

import tomllib, pathlib, re, subprocess, tempfile, os, sys

ENTITY = {'cre': 'entity asset tower : CRE.Asset.RealProperty',
          'opco': 'entity asset firm : OpCo.Asset.Enterprise',
          'energy': 'entity asset plant : Energy.Asset.GenerationFacility',
          'credit': 'entity asset pool : Credit.Asset.LoanPool'}
CURVES = {'draws': '2026-01: 1000000\n  2027-01: 500000',
          'sofr': '2026-01: 0.05\n  2027-01: 0.045',
          'occupancy': '2026-01: 1.0'}

fails = total = packs = 0
for pack in ('cre','opco','energy','credit'):
    tf = pathlib.Path('packs')/pack/'templates.toml'
    if not tf.exists():
        continue
    d = tomllib.loads(tf.read_text(encoding="utf-8"))
    packs += 1
    for t in d.get('templates', []):
        total += 1
        body = t['body']
        for k, v in t.get('defaults', {}).items():
            body = body.replace('${'+k+'}', v)
        left = re.findall(r'\$\{(\w+)\}', body)
        if left:
            print(f"  [{pack}] {t['id']}: UNRESOLVED {left}"); fails += 1; continue
        # curves the body references
        curves = ''
        for name in set(re.findall(r'curve_value\("(\w+)"|_curve = "(\w+)"', body)):
            nm = name[0] or name[1]
            if nm in CURVES:
                curves += f'curve {nm} step {{\n  {CURVES[nm]}\n}}\n\n'
        src = (f'version 0.1\nmodel "tpl-{t["id"].replace(".","-")}"\n'
               f'use pack "{pack}" version "0.1.0"\n'
               f'time calendar monthly from 2026-01 for 360 project 12\n\n'
               f'{ENTITY[pack]}\n\n{curves}{body}\n')
        with tempfile.TemporaryDirectory() as td:
            (pathlib.Path(td)/'model.cfdl').write_text(src, encoding="utf-8")
            r = subprocess.run(['./target/debug/cfdl','compile',td,'--out',
                                os.path.join(td,'ir.json'),'--packs','packs'],
                               capture_output=True, text=True, encoding="utf-8")
            if r.returncode != 0:
                msg = (r.stdout + r.stderr).strip().split('\n')
                msg = [m for m in msg if 'ERROR' in m][:2]
                print(f"  [{pack}] {t['id']}: {' | '.join(msg)}")
                fails += 1
if fails:
    print(f"\ncheck-pack-templates: {fails} template(s) do not compile", file=sys.stderr)
else:
    print(f"check-pack-templates: OK ({total} templates across {packs} packs render "
          "from their defaults and compile)")
sys.exit(1 if fails else 0)
