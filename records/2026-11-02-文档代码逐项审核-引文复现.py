"""Run through task-server: real quote CLI against owned temporary materials.

Exit zero means the documented checklist guarantee was contradicted as expected.
No repository document is written by the target tool.
"""
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    root = Path(__file__).resolve().parents[1]
    sources = [root / 'scripts/kb/quote-kb.py', root / 'scripts/kb/kb-sections.py']
    before = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}
    environment = os.environ.copy()
    for name in ('QUOTE_KB_BREAK', 'QUOTE_KB_CORRUPT', 'KB_SECTIONS_BREAK',
                 'KB_SECTIONS_NO_H1', 'KB_SECTIONS_NO_PREAMBLE', 'KB_SECTIONS_NO_FENCE'):
        if environment.get(name):
            raise AssertionError(f'Mutation switch must be unset: {name}')
    cases = [
        ('complete_control', '## Shared\n\nBODY_B\n',
         ['.claude/kb/a.md@## Shared', '.claude/kb/b.md@## Shared'], 0, True),
        ('missing_control', '## Other\n\nBODY_B\n',
         ['.claude/kb/b.md@## Other'], 4, False),
        ('other_file_same_heading', '## Shared\n\nBODY_B\n',
         ['.claude/kb/b.md@## Shared'], 0, False),
        ('heading_without_body', '## Shared\n\nBODY_B\n',
         ['.claude/kb/a.md:1-1', '.claude/kb/b.md@## Shared'], 0, False),
        ('heading_inside_other_file_code', '## Other\n\n```text\n## Shared\n```\n\nBODY_B\n',
         ['.claude/kb/b.md@## Other'], 0, False),
    ]
    with tempfile.TemporaryDirectory(prefix='audit-quote-') as scratch:
        for label, body_b, specs, observed_code, a_present in cases:
            work = Path(scratch) / label
            kb = work / '.claude/kb'
            kb.mkdir(parents=True)
            (kb / 'a.md').write_text('## Shared\n\nBODY_A_REQUIRED\n', encoding='utf-8')
            (kb / 'b.md').write_text(body_b, encoding='utf-8')
            b_heading = body_b.splitlines()[0]
            (work / 'checklist.md').write_text(
                '### 小节清单：`.claude/kb/a.md`\n\n'
                '| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n'
                '| ## Shared | 抄 | required |\n\n'
                '### 小节清单：`.claude/kb/b.md`\n\n'
                '| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n'
                f'| {b_heading} | 抄 | required |\n', encoding='utf-8')
            (work / 'cited.md').write_text(
                'Sources: `.claude/kb/a.md` and `.claude/kb/b.md`.\n', encoding='utf-8')
            command = [sys.executable, '-B', str(sources[0]), '--checklist', 'checklist.md',
                       '--cited', 'cited.md', 'output.md', *specs]
            result = subprocess.run(command, cwd=work, env=environment, text=True,
                                    capture_output=True, timeout=60)
            output = work / 'output.md'
            content = output.read_text(encoding='utf-8') if output.exists() else ''
            present = 'BODY_A_REQUIRED' in content
            print(f'AUDIT_QUOTE case={label} exit={result.returncode} '
                  f'output_exists={output.exists()} required_A_body_present={present}', flush=True)
            print(result.stdout + result.stderr, end='', flush=True)
            if result.returncode != observed_code or present != a_present:
                raise AssertionError(f'Unexpected result for {label}; re-audit source')
            if label == 'missing_control' and output.exists():
                raise AssertionError('Refusal unexpectedly wrote output')
    after = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}
    if before != after:
        raise AssertionError('Source changed during audit')
    for path, digest in after.items():
        print(f'AUDIT_QUOTE source={Path(path).relative_to(root)} sha256={digest}')
    print('AUDIT_QUOTE cases=5 controls=2 discrepancies=3 temporary_materials_removed=true')


if __name__ == '__main__':
    main()
