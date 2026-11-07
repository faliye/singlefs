"""Task-server audit of the real experiment-orphans gate on temporary trees."""
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    root = Path(__file__).resolve().parents[1]
    gate = root / '.claude/gate.d/doc-experiments.sh'
    sources = [gate, root / 'scripts/kb/lib_experiment_number_directories.py',
               root / '.claude/gate.d/lib/stage-cells.sh']
    before = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}
    for name in ('EXPERIMENT_PAGES_AND_PRODUCTS_BREAK', 'EXPERIMENT_NUMBER_DIRECTORIES_BREAK', 'STAGE_CELLS_BREAK'):
        assert not os.environ.get(name), name
    with tempfile.TemporaryDirectory(prefix='audit-orphan-body-') as scratch:
        for label in ('missing', 'actual_page', 'empty_page', 'directory_only', 'wrong_suffix'):
            work = Path(scratch) / label
            pages = work / '.claude/kb/experiments'
            pages.mkdir(parents=True)
            registry = work / '.claude/gate.d/experiment-number-directories.tsv'
            registry.parent.mkdir(parents=True)
            registry.write_text('research\te-number-names\t# isolated audit registry\n', encoding='utf-8')
            (work / 'research').mkdir()
            (work / 'research/e42-output.txt').write_text('sample output\n', encoding='utf-8')
            if label == 'actual_page':
                (pages / '42-sample.md').write_text('# E42 示例\n\n测什么：示例。\n判据：示例。\n口径：临时样本。\n', encoding='utf-8')
            elif label == 'empty_page':
                (pages / '42-empty.md').touch()
            elif label == 'directory_only':
                (pages / '42-placeholder').mkdir()
            elif label == 'wrong_suffix':
                (pages / '42.bin').write_bytes(b'\x00')
            p = subprocess.run(['bash', str(gate), '--check', 'experiment-orphans', str(work)],
                               cwd=root, capture_output=True, text=True, timeout=120)
            print(f'AUDIT_ORPHAN case={label} exit={p.returncode}', flush=True)
            print(p.stdout + p.stderr, end='', flush=True)
            assert p.returncode == (1 if label == 'missing' else 0), (label, p.stdout, p.stderr)
            if label != 'missing':
                assert '在 kb 里都有正文' in p.stdout
    after = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}
    assert before == after, 'Source changed during audit'
    for source, digest in after.items():
        print(f'AUDIT_ORPHAN source={Path(source).relative_to(root)} sha256={digest}')
    print('AUDIT_ORPHAN temporary_materials_removed=true')


if __name__ == '__main__':
    main()
