"""Task-server audit: real citation CLI plus real notation predicates in scratch.

Successful exit means observed discrepancies match the audit expectations.
"""
import hashlib
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    root = Path(__file__).resolve().parents[1]
    quote = root / 'scripts/kb/quote-kb.py'
    library = root / '.claude/gate.d/lib-forbidden-notations.py'
    before = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in (quote, library)}
    for name in os.environ:
        if name.startswith(('QUOTE_KB_', 'KB_SECTIONS_', 'LIB_FORBIDDEN_NOTATIONS_')):
            assert not os.environ[name], name
    with tempfile.TemporaryDirectory(prefix='audit-declarations-') as scratch:
        work = Path(scratch)
        kb = work / '.claude/kb'
        kb.mkdir(parents=True)
        (kb / 'b.md').write_text('## Included\n\nINCLUDED_BODY\n', encoding='utf-8')
        (kb / 'checks-owed.md').write_text('| C1 | required clause |\n', encoding='utf-8')
        (work / 'checklist.md').write_text(
            '### 小节清单：`.claude/kb/b.md`\n\n'
            '| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n'
            '| ## Included | 抄 | required |\n', encoding='utf-8')
        marker = '<!-- doc-lint:not-numbers C1 -->\n'
        cases = [('required', '', 5), ('declared_outside', marker, 0),
                 ('triple_fenced', '```\n' + marker + '```\n', 5),
                 ('tilde_fenced', '~~~\n' + marker + '~~~\n', 0),
                 ('long_fenced', '````\n```\n' + marker + '````\n', 0)]
        for label, prefix, expected in cases:
            cited = work / 'cited.md'
            cited.write_text(prefix + 'C1（实际条款）的正文必须核。\n', encoding='utf-8')
            destination = work / (label + '.md')
            p = subprocess.run([sys.executable, '-B', str(quote), '--checklist', 'checklist.md',
                                '--cited', 'cited.md', str(destination), '.claude/kb/b.md@## Included'],
                               cwd=work, capture_output=True, text=True, timeout=60)
            print(f'AUDIT_DECLARATION case={label} exit={p.returncode} output={destination.exists()}', flush=True)
            print(p.stdout + p.stderr, end='', flush=True)
            assert p.returncode == expected and destination.exists() == (expected == 0)
    spec = importlib.util.spec_from_file_location('audit_notations', library)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    # These are independent literal inputs; no reimplementation of the regex.
    cases = [('valid_compact', 'clock-times', '20261105T1234', True),  # clock-times:allow 自检输入就是要被判的时间戳写法
             ('invalid_compact', 'clock-times', '20269999T1234', False),
             ('invalid_hyphenated', 'clock-times', '2026-99-99T1234', True),  # clock-times:allow 自检输入就是要被判的时间戳写法
             ('short_batch', 'batch-codes', '批 12c', True),
             ('long_bare_batch', 'batch-codes', '123c', False),
             ('long_labeled_batch', 'batch-codes', '批 123c', True),
             ('inline_code', 'batch-codes', '`24c`', False),
             ('double_tick_code', 'batch-codes', '``24c``', True)]
    for label, form, value, expected_hit in cases:
        hits = module.all_hits(form, value, 'records/example.md')
        print(f'AUDIT_NOTATION case={label} hit={bool(hits)} shapes={[h[2] for h in hits]}', flush=True)
        assert bool(hits) == expected_hit, (label, hits)
    after = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in (quote, library)}
    assert before == after, 'Source changed during audit'
    for source, digest in after.items():
        print(f'AUDIT_DECLARATION source={Path(source).relative_to(root)} sha256={digest}')
    print('AUDIT_DECLARATION temporary_materials_removed=true')


if __name__ == '__main__':
    main()
