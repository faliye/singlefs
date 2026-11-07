"""Task-server audit of citation checker scope, using only temporary fake trees.

The whole-entry test keeps the real checker unchanged. Predicate probes load
the exact function prefix, omitting only preflight bootstrap, as the existing
repository regression test does; they do not establish external source facts.
"""
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    root = Path(__file__).resolve().parents[1]
    script = root / 'scripts/kb/verify-citations.sh'
    original = script.read_bytes()
    text = original.decode()
    for name in ('VERIFY_CITATIONS_ASSERTIONS', 'VERIFY_CITATIONS_BREAK', 'VERIFY_CITATIONS_SELFTEST_MARK'):
        assert not os.environ.get(name), name
    with tempfile.TemporaryDirectory(prefix='audit-citation-trees-') as scratch:
        work = Path(scratch)
        refs, kernel = work / 'refs', work / 'kernel'
        files = {
            refs / 'linux-6.17/fs/bcachefs/alloc_background.h': '#define BUCKET_GC_GEN_MAX 96U\n',
            refs / 'zfs/include/sys/spa.h': '#define SPA_BLKPTRSHIFT 7\n',
            kernel / 'fs/xfs/xfs_log_format.h': '#define XLOG_MAX_ICLOGS 8\n',
        }
        for path, value in files.items():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(value, encoding='utf-8')
        (work / 'tmp').mkdir()
        environment = os.environ.copy()
        environment.update(FS_REFS=str(refs), KERNEL_TREE=str(kernel), TMPDIR=str(work / 'tmp'))
        for label in ('normal', 'selftest_table_outside_selftest'):
            child_environment = environment.copy()
            if label != 'normal':
                child_environment['VERIFY_CITATIONS_ASSERTIONS'] = 'green'
            p = subprocess.run(['bash', str(script)], env=child_environment, cwd=root,
                               capture_output=True, text=True, timeout=120)
            summary = [line for line in p.stdout.splitlines() if '结果：' in line or '全部命中' in line]
            print(f'AUDIT_CITATIONS case={label} exit={p.returncode} summary={summary}', flush=True)
            assert p.returncode == (1 if label == 'normal' else 0), (p.stdout, p.stderr)
            if label != 'normal':
                assert '全部命中（4 条' in p.stdout
            assert not list((work / 'tmp').iterdir()), 'Text cache not removed'

        prefix = text.split('# ── 断言表')[0]
        prefix = '\n'.join(line for line in prefix.splitlines()
                           if not line.startswith(('source ', 'preflight ')))
        declaration = next(line for line in text.splitlines() if line.startswith('ck  D19 refs-linux')
                           and 'bch_extent_ptr' in line)
        header = refs / 'linux-6.17/fs/bcachefs/extents_format.h'
        for label, body, wanted_pass in (
                ('nonmatching_control', 'struct example { int field; };\n', 0),
                ('only_dev_no_offset_or_gen', 'struct example { unsigned dev:8; };\n', 1)):
            header.write_text(body, encoding='utf-8')
            program = prefix + '\n' + declaration + '\nprintf "AUDIT_SCORE pass=%s fail=%s\\n" "$pass" "$fail"\n'
            p = subprocess.run(['bash', '-c', program], env=environment, cwd=root,
                               capture_output=True, text=True, timeout=60)
            print(f'AUDIT_CITATION_PREDICATE case={label} exit={p.returncode}', flush=True)
            print(p.stdout + p.stderr, end='', flush=True)
            assert p.returncode == 0
            assert f'AUDIT_SCORE pass={wanted_pass} fail={1-wanted_pass}' in p.stdout
    assert script.read_bytes() == original, 'Checker changed during audit'
    print(f'AUDIT_CITATIONS sha256={hashlib.sha256(original).hexdigest()} temporary_materials_removed=true')


if __name__ == '__main__':
    main()
