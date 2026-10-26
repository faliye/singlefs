#!/usr/bin/env python3
"""Run isolated public-API probes; never edit product sources or existing tests."""
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]

def main():
    paths = sorted(p for name in ('singlefs-harness', 'singlefs-core', 'singlefs-format', 'singlefs-replay-prefix', 'singlefs-reference-filesystem', 'singlefs-checker') for p in (ROOT / 'crates' / name).rglob('*.rs') if 'audits' not in p.parts)
    before = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    (HERE / 'evidence/source-sha256.txt').write_text(''.join(f'{digest}  {name}\n' for name, digest in before.items()))
    with tempfile.TemporaryDirectory(prefix='singlefs-harness-audit-') as directory:
        project = Path(directory)
        (project / 'src').mkdir()
        (project / 'src/lib.rs').write_text((HERE / 'probes.rs').read_text())
        manifest = '[package]\nname="singlefs-harness-audit"\nversion="0.0.0"\nedition="2021"\n[dependencies]\n'
        manifest += ''.join(f'{name}={{path="{ROOT / "crates" / name}"}}\n' for name in ('singlefs-harness', 'singlefs-core'))
        manifest += '[profile.release]\noverflow-checks=true\ndebug-assertions=true\n'
        (project / 'Cargo.toml').write_text(manifest)
        env = os.environ.copy()
        env.pop('SINGLEFS_HARNESS_AUDIT_CONTROL', None)
        env['CARGO_TARGET_DIR'] = str(ROOT / 'target')
        for round_number in range(1, 3):
            print(f'ROUND {round_number}', flush=True)
            subprocess.run(['cargo', 'test', '--release', '--offline', '--manifest-path', str(project / 'Cargo.toml'), '--lib', '--', '--nocapture', '--test-threads=1'], cwd=ROOT, env=env, check=True)
    for p in paths:
        assert hashlib.sha256(p.read_bytes()).hexdigest() == before[str(p.relative_to(ROOT))], f'Source changed during audit: {p}'
    print('HARNESS_AUDIT_COMPLETE two rounds passed; product sources unchanged.', flush=True)

if __name__ == '__main__':
    main()
