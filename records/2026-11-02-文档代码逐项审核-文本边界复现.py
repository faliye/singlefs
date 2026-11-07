"""Task-server-only audit of documented text-tool boundaries in owned scratch.

Exit zero confirms expected discrepancies, not product correctness. Faults are
injected only into this process; no source or real repository document changes.
"""
import errno
import hashlib
import importlib.util
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
from unittest.mock import patch


def main():
    root = Path(__file__).resolve().parents[1]
    directory = root / 'scripts/kb'
    paths = [directory / name for name in
             ('quote-rust-items.py', 'lib_atomic_replace.py', 'kb-sections.py')]
    before = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    for name in ('ATOMIC_REPLACE_BREAK', 'KB_SECTIONS_BREAK', 'KB_SECTIONS_NO_H1',
                 'KB_SECTIONS_NO_PREAMBLE', 'KB_SECTIONS_NO_FENCE'):
        assert not os.environ.get(name), name
    spec = importlib.util.spec_from_file_location('audit_atomic_boundaries', paths[1])
    atomic = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(atomic)
    with tempfile.TemporaryDirectory(prefix='audit-text-boundaries-') as scratch:
        work = Path(scratch)
        cases = [
            ('normal', b'/// doc\n#[must_use]\nfn chosen() -> u8 { 7 }\n', 0),
            ('missing', None, 2),
            ('invalid_utf8', b'\xff\nfn chosen() {}\n', 1),
            ('multiline_attribute', b'/// doc\n#[cfg(\n    target_os = "linux"\n)]\nfn chosen() {}\n', 0),
        ]
        for label, data, expected in cases:
            source = work / (label + '.rs')
            if data is not None:
                source.write_bytes(data)
            p = subprocess.run([sys.executable, '-B', str(paths[0]), f'{source}::chosen'],
                               cwd=root, text=True, capture_output=True, timeout=60)
            print(f'AUDIT_RUST case={label} exit={p.returncode} '
                  f'traceback={"Traceback" in p.stderr} doc={"/// doc" in p.stdout} '
                  f'attribute={"#[" in p.stdout}', flush=True)
            assert p.returncode == expected, (label, p.stdout, p.stderr)
            if label == 'normal':
                assert '/// doc' in p.stdout and '#[must_use]' in p.stdout
            if label == 'invalid_utf8':
                assert 'UnicodeDecodeError' in p.stderr and not p.stdout
            if label == 'multiline_attribute':
                assert 'fn chosen()' in p.stdout and '#[' not in p.stdout and '/// doc' not in p.stdout

        real_fsync = atomic.os.fsync
        for label in ('normal', 'file_fsync_error', 'directory_interrupt'):
            target = work / (label + '.md')
            target.write_text('OLD\n', encoding='utf-8')

            def fsync_with_fault(fd):
                is_directory = stat.S_ISDIR(os.fstat(fd).st_mode)
                if label == 'file_fsync_error' and not is_directory:
                    raise OSError(errno.EIO, 'audit injected file fsync error')
                if label == 'directory_interrupt' and is_directory:
                    raise KeyboardInterrupt('audit injected post-rename interruption')
                return real_fsync(fd)

            error = None
            try:
                with patch.object(atomic.os, 'fsync', fsync_with_fault):
                    atomic.replace_file_contents_by_rename(target, 'NEW\n')
            except BaseException as caught:
                error = type(caught).__name__
            content = target.read_text(encoding='utf-8').strip()
            leftovers = list(work.glob('.*.replacing'))
            print(f'AUDIT_ATOMIC case={label} exception={error} content={content} '
                  f'temporary_files={len(leftovers)}', flush=True)
            assert not leftovers
            expected = {'normal': (None, 'NEW'), 'file_fsync_error': ('ReplaceRefused', 'OLD'),
                        'directory_interrupt': ('KeyboardInterrupt', 'NEW')}[label]
            assert (error, content) == expected

        source = work / 'sections.md'
        source.write_text('INTRO\n\n# First\n\nBODY\n\n```\n## Fake\n```\n\n## Second\nTEXT\n', encoding='utf-8')
        outputs = {}
        for name in ('normal', 'KB_SECTIONS_BREAK', 'KB_SECTIONS_NO_H1',
                     'KB_SECTIONS_NO_PREAMBLE', 'KB_SECTIONS_NO_FENCE'):
            environment = os.environ.copy()
            if name != 'normal':
                environment[name] = 'raw-arguments' if name == 'KB_SECTIONS_BREAK' else '1'
            p = subprocess.run([sys.executable, '-B', str(paths[2]), str(source)], cwd=root,
                               env=environment, text=True, capture_output=True, timeout=60)
            outputs[name] = p.stdout
            print(f'AUDIT_SECTIONS setting={name} exit={p.returncode} '
                  f'output_lines={len(p.stdout.splitlines())}', flush=True)
            print(p.stdout + p.stderr, end='', flush=True)
            assert p.returncode == (2 if name == 'KB_SECTIONS_BREAK' else 0)
        for name in ('KB_SECTIONS_NO_H1', 'KB_SECTIONS_NO_PREAMBLE', 'KB_SECTIONS_NO_FENCE'):
            assert outputs[name] != outputs['normal']
    after = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    assert before == after, 'Source changed during audit; re-review required'
    for source, digest in after.items():
        print(f'AUDIT_TEXT source={Path(source).relative_to(root)} sha256={digest}')
    print('AUDIT_TEXT temporary_materials_removed=true')


if __name__ == '__main__':
    main()
