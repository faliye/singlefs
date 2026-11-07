"""KB review regressions; all writes use isolated temporary files/repositories."""
import contextlib
import importlib.util
import io
import json
import os
import re
import random
import stat
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest
import zlib
from types import SimpleNamespace
from unittest import mock

sys.dont_write_bytecode = True
KB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(KB))


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), KB / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ReviewRegressions(unittest.TestCase):
    def test_gate_can_import_consolidation_by_path(self):
        # 与 doc-experiments.sh 相同：按路径导入，不能靠测试进程的 sys.path。
        source = '''
import importlib.util, sys
spec = importlib.util.spec_from_file_location('consolidate', sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
assert module.fenced_flags(['~~~', '## inside', '~~~']) == [True, True, True]
text, report = module.migrate_document('## topic\\nbody\\n', '', False)
assert text == '## topic\\nbody\\n'
'''
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([sys.executable, '-I', '-c', source, str(KB / 'doc-consolidate.py')],
                                    cwd=directory, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_fence_state_matrix(self):
        module = load('lib_markdown')
        cases = [
            (['```', '~~~', '# inside', '```', '# outside'], [True, True, True, True, False]),
            (['````', '```', '# inside', '`````', '# outside'], [True, True, True, True, False]),
            (['```', '```not-a-close', '# inside', '```'], [True] * 4),
            (['~~~', '~~~not-a-close', '# inside', '~~~'], [True] * 4),
            (['```inline```', '# outside'], [False, False]),
            (['   ~~~lang', '# inside', '  ~~~', '# outside'], [True, True, True, False]),
            (['    ```', '# outside'], [False, False]),
            (['```lang', '# inside', ''], [True, True, True]),
        ]
        for lines, expected in cases:
            with self.subTest(lines=lines):
                self.assertEqual(module.fenced_flags(lines), expected)

    def setUp(self):
        environment = {k: v for k, v in os.environ.items() if not k.startswith('GIT_')}
        patch = mock.patch.dict(os.environ, environment, clear=True)
        patch.start()
        self.addCleanup(patch.stop)

    def test_replace_rechecks_source(self):
        for name in ('replace-once', 'replace-batch'):
            with self.subTest(tool=name), tempfile.TemporaryDirectory() as directory:
                module = load(name)
                path = Path(directory) / 'a.txt'
                path.write_text('old\n')
                def concurrent_change(*args, **kwargs):
                    path.write_text('old\nother writer\n')
                    return None if name == 'replace-once' else []
                with mock.patch.object(module, 'clock_refusal' if name == 'replace-once' else 'clock_refusals', concurrent_change), contextlib.redirect_stdout(io.StringIO()):
                    result = (module.replace_once(str(path), 'old', 'new')[0] if name == 'replace-once'
                              else module.run([{'file': str(path), 'old': 'old', 'new': 'new'}], False))
                self.assertNotEqual(result, 0)
                self.assertEqual(path.read_text(), 'old\nother writer\n')

    def test_batch_aliases_are_sequential(self):
        module = load('replace-batch')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'a.txt'
            alias = Path(directory) / 'alias.txt'
            path.write_text('old\n')
            alias.symlink_to(path)
            with contextlib.redirect_stdout(io.StringIO()):
                result = module.run([{'file': str(path), 'old': 'old', 'new': 'middle'},
                                     {'file': str(alias), 'old': 'middle', 'new': 'new'}], False)
            self.assertEqual(result, 0)
            self.assertEqual(path.read_text(), 'new\n')
            self.assertTrue(alias.is_symlink())

    def test_batch_guards_follow_the_same_snapshot_and_edit_order(self):
        module = load('replace-batch')
        with tempfile.TemporaryDirectory() as directory:
            path = str(Path(directory) / 'a.txt')
            originals = {path: b'head\r\nold\r\ntail\r\n'}
            edits = [{'file': path, 'old': 'old\r\n', 'new': 'middle\r\nextra\r\n'},
                     {'file': path, 'old': 'extra\r\n', 'new': 'last\r\n'}]
            # 文件此刻已经变了；检查仍需针对待写计划定位，写前复核另外负责拒绝覆盖。
            Path(path).write_text('a different version\n')
            for wrapper, checker in ((module.clock_refusals, 'clock_refusal'),
                                     (module.session_name_refusals, 'session_name_refusal')):
                seen = []
                def inspect(path, new, line, root):
                    seen.append((new, line))
                    return ['refused'] if new == 'last\r\n' else None
                with mock.patch.object(module, checker, inspect):
                    self.assertEqual(wrapper(edits, originals=originals), [['refused']])
                self.assertEqual(seen, [('middle\r\nextra\r\n', 2), ('last\r\n', 3)])

    def test_batch_invalid_counts_and_empty_anchor(self):
        module = load('replace-batch')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'a.txt'
            spec = Path(directory) / 'spec.json'
            for old, count in [('', 4), ('old', True), ('old', 1.0), ('old', 0), ('old', -1), ('old', '1')]:
                path.write_text('old')
                spec.write_text(json.dumps([{'file': str(path), 'old': old, 'new': 'x', 'count': count}]))
                with self.subTest(old=old, count=count), contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(module.main([str(spec)]), 2)
                    self.assertEqual(path.read_text(), 'old')

    def test_rust_lexical_boundaries(self):
        module = load('quote-rust-items')
        cases = [
            ('/*\nfn wanted() {}\n*/\nfn wanted() {\n work();\n}', 'wanted', (3, 5)),
            ('fn wanted() {\n let s = r#"a " } still text"#;\n work();\n}', 'wanted', (0, 3)),
            ('const VALUE: [u8; 2] = [\n 1, 2\n];', 'VALUE', (0, 2)),
            ('const VALUE: u8 = {\n 1\n} + 2\n;', 'VALUE', (0, 3)),
            ("fn wanted() {\n let c = '\\u{7d}';\n work();\n}", 'wanted', (0, 3)),
            ('pub const unsafe fn wanted() {\n work();\n}\nconst AFTER: u8 = 1;', 'wanted', (0, 2)),
            ('pub const unsafe fn wanted() {\n work();\n}', 'wanted', (0, 2)),
        ]
        for text, name, bounds in cases:
            with self.subTest(text=text):
                self.assertEqual(module.find_item(text.split('\n'), name), (bounds, 0))

    def test_rust_const_generic_blocks_do_not_end_the_item(self):
        module = load('quote-rust-items')
        cases = [
            ('pub struct Packet<const N: usize = {\n    4\n}> {\n    bytes: [u8; N],\n}', 'Packet'),
            ('struct Packet<const N: usize = {\n    1 << 2\n}>([u8; N]);', 'Packet'),
            ('fn wanted() -> Packet<{\n    4\n}> {\n    Packet([0; 4])\n}', 'wanted'),
            ('fn wanted<T>() -> Vec<T> {\n    if 1 < 2 { Vec::new() } else { Vec::new() }\n}', 'wanted'),
            ('const VALUE: bool =\n    1 < 2;', 'VALUE'),
            ('static VALUE: usize =\n    1 << 2;', 'VALUE'),
            ('fn wanted<T>(index: usize) -> (value: usize)\n    requires\n        index < 4,\n    ensures\n        value <= 4,\n{\n    index\n}', 'wanted'),
        ]
        for source, name in cases:
            with self.subTest(source=source):
                lines = source.splitlines()
                self.assertEqual(module.find_item(lines + ['fn after() {}'], name),
                                 ((0, len(lines) - 1), 0))

    def test_rust_quote_really_rereads(self):
        module = load('quote-rust-items')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'a.rs'
            path.write_text('fn a() { old(); }\n')
            original = module.find_item
            def change(lines, name):
                result = original(lines, name)
                path.write_text('fn a() { new(); }\n')
                return result
            with mock.patch.object(module, 'find_item', change), contextlib.redirect_stderr(io.StringIO()):
                code, output = module.quote([str(path) + '::a'])
            self.assertEqual(code, 5)
            self.assertEqual(output, '')

    def test_numeric_loss_check_does_not_accept_decimal_prefix(self):
        module = load('decision-slim-check')
        for replacement in ('64.5', '64,000'):
            self.assertTrue(module.missing_tokens('64', replacement, []))

    def test_spec_rejects_malformed_entries(self):
        module = load('kb-spec-check')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'spec.json'
            for data in ([{'file': 'a', 'old': 'x', 'new': 'y'}, 42], [{'file': 'a', 'old': 1, 'new': 'y'}]):
                path.write_text(json.dumps(data))
                with self.subTest(data=data):
                    self.assertFalse(module.load_spec(str(path)))

    def test_indented_fences_hide_headings(self):
        module = load('kb-sections')
        self.assertEqual(module.fenced_lines(['   ```text', '## fake', '   ```']), {1, 2, 3})
        self.assertEqual(module.fenced_lines(['```inline```', '## real']), set())
        quote = load('quote-kb')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'sample.md'
            path.write_text('## first\n```inline```\n## second\nbody\n')
            self.assertEqual(quote.pick(str(path) + '@## first')[2], ['## first', '```inline```'])

    def test_dirty_index_cannot_hide_behind_restored_worktree(self):
        module = load('stale-candidates')
        with tempfile.TemporaryDirectory() as directory:
            def git(*args):
                return subprocess.check_output(['git', '-C', directory, *args], stderr=subprocess.DEVNULL)
            git('init', '-q')
            git('config', 'user.name', 'test')
            git('config', 'user.email', 'test@example.invalid')
            path = Path(directory) / 'a.md'
            path.write_text('old\n')
            git('add', '.')
            git('commit', '-qm', 'base')
            path.write_text('new\n')
            git('add', '.')
            path.write_text('old\n')
            self.assertEqual(module.worktree_changes(directory), ['a.md'])

    def test_nested_date_sections_keep_all_bodies(self):
        module = load('doc-consolidate')
        text = '## 2026-01-01\n### 2026-01-02\nfirst 11\n### 2026-01-02\nsecond 22\n'
        report = {'sections_merged': 0}
        result = '\n'.join(module.merge_same_date_headings(text.split('\n'), report))
        self.assertIn('first 11', result)
        self.assertIn('second 22', result)

    def test_generated_date_merges_preserve_bodies_and_are_idempotent(self):
        module = load('doc-consolidate')
        random_source = random.Random(37)
        for case in range(500):
            lines = ['# document']
            for number in range(random_source.randrange(3, 35)):
                level = random_source.choice([2, 3, 4])
                day = random_source.choice(['2026-01-01', '2026-01-02', 'ordinary'])
                lines += ['#' * level + ' ' + day + ' title', f'BODY_{case}_{number}']
                if random_source.randrange(4) == 0:
                    lines += ['````text', '```', '## 2026-01-01 fake', '````']
            expected = Counter(line for line in lines if line.startswith('BODY_'))
            merged = module.merge_same_date_headings(lines, {'sections_merged': 0})
            with self.subTest(case=case):
                self.assertEqual(Counter(line for line in merged if line.startswith('BODY_')), expected)
                self.assertEqual(module.merge_same_date_headings(merged, {'sections_merged': 0}), merged)

    def test_document_ledger_read_error_refuses_before_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, output = root / 'input.md', root / 'output.md'
            source.write_text('# sample\nbody\n')
            invalid = root / 'invalid-ledger.tsv'
            invalid.write_bytes(b'\xff\xfe')
            for ledger in (root, invalid):
                with self.subTest(ledger=ledger.name):
                    output.write_text('keep output\n')
                    result = subprocess.run([sys.executable, '-B', str(KB / 'doc-consolidate.py'),
                                             'migrate', str(source), '--out', str(output)],
                                            env=dict(os.environ, DOC_CONSOLIDATE_LEDGER=str(ledger)),
                                            capture_output=True, text=True)
                    self.assertEqual(result.returncode, 4, result.stdout + result.stderr)
                    self.assertEqual(output.read_text(), 'keep output\n')

    def test_claim_numbers_across_worktrees(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'repo'
            other = Path(directory) / 'other'
            root.mkdir()
            def git(*args):
                return subprocess.check_output(['git', '-C', str(root), *args], stderr=subprocess.DEVNULL)
            git('init', '-q')
            git('config', 'user.name', 'test')
            git('config', 'user.email', 'test@example.invalid')
            registry = root / '.claude/gate.d/experiment-number-directories.tsv'
            registry.parent.mkdir(parents=True)
            registry.write_text('research\te-number-names\t# isolated test registry\n')
            git('add', '.')
            git('commit', '-qm', 'base')
            git('worktree', 'add', '-qb', 'other', str(other))
            def start(tree, number):
                return subprocess.Popen(['bash', str(KB / 'claim-experiment.sh'), number, 'sample'],
                                        env=dict(os.environ, CLAIM_ROOT=str(tree)), stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE, text=True)
            processes = [start(root if i % 2 else other, 'E19') for i in range(8)]
            codes = []
            for process in processes:
                output = process.communicate(timeout=30)
                codes.append(process.returncode)
                self.assertIn(process.returncode, (0, 1), output)
            self.assertEqual(codes.count(0), 1)
            process = start(root, 'E019')
            output = process.communicate(timeout=30)
            self.assertEqual(process.returncode, 1, output)

    def test_claim_does_not_skip_worktree_with_newline_in_path(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'repo'
            other = Path(directory) / 'other\nworktree'
            root.mkdir()
            def git(*args):
                return subprocess.run(['git', '-C', str(root), *args], check=True, capture_output=True)
            git('init')
            registry = root / '.claude/gate.d/experiment-number-directories.tsv'
            registry.parent.mkdir(parents=True)
            registry.write_text('research\te-number-names\t# isolated test registry\n')
            git('add', '.')
            git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'commit', '-m', 'base')
            git('worktree', 'add', '-qb', 'other', str(other))
            occupied = other / 'research/prompts/e19-note.md'
            occupied.parent.mkdir(parents=True)
            occupied.write_text('occupied\n')
            for arguments in (['E19', 'sample'], ['--next']):
                result = subprocess.run(['bash', str(KB / 'claim-experiment.sh'), *arguments],
                                        env=dict(os.environ, CLAIM_ROOT=str(root)), capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertFalse((root / 'research/prompts/e19-preregistration.md').exists())

    def test_archive_preserves_staged_changes_restored_to_base(self):
        module = load('archive-past-rounds')
        with tempfile.TemporaryDirectory() as directory:
            def git(*args):
                return subprocess.check_output(['git', '-C', directory, *args], stderr=subprocess.DEVNULL).decode().strip()
            git('init', '-q')
            git('config', 'user.name', 'test')
            git('config', 'user.email', 'test@example.invalid')
            path = Path(directory) / 'research/prompts/e999-note.txt'
            path.parent.mkdir(parents=True)
            path.write_text('base\n')
            git('add', '.')
            git('commit', '-qm', 'base')
            base = git('rev-parse', 'HEAD')
            path.write_text('head\n')
            git('commit', '-qam', 'head')
            path.write_text('base\n')
            with mock.patch.object(module, 'base_of', return_value=base):
                self.assertNotIn('research/prompts/e999-note.txt', module.past_round_files(directory))
            path.write_text('staged\n')
            git('add', '.')
            path.write_text('head\n')
            with mock.patch.object(module, 'base_of', return_value='HEAD'):
                self.assertNotIn('research/prompts/e999-note.txt', module.past_round_files(directory))

    def test_git_hidden_changes_are_not_archived_or_accepted_as_basis(self):
        archive = load('archive-past-rounds')
        basis = load('lib_basis_paths')
        for flag in ('assume-unchanged', 'skip-worktree'):
            with self.subTest(flag=flag), tempfile.TemporaryDirectory() as directory:
                def git(*args):
                    return subprocess.run(['git', '-C', directory, *args], check=True, capture_output=True)
                git('init')
                relative = 'research/results/e999-note.txt'
                path = Path(directory) / relative
                path.parent.mkdir(parents=True)
                path.write_text('committed\n')
                git('add', '.')
                git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'commit', '-m', 'base')
                git('update-index', '--' + flag, relative)
                path.write_text('uncommitted evidence\n')
                index_before = (Path(directory) / '.git/index').read_bytes()
                with mock.patch.object(archive, 'base_of', return_value='HEAD'):
                    self.assertNotIn(relative, archive.past_round_files(directory))
                for citation in (relative, relative + ':1'):
                    problems, untracked = basis.basis_check(directory, citation)
                    self.assertTrue(problems)
                    self.assertEqual(untracked, [])
                self.assertEqual((Path(directory) / '.git/index').read_bytes(), index_before)
                self.assertEqual(path.read_text(), 'uncommitted evidence\n')
                git('update-index', '--no-' + flag, relative)
                self.assertTrue(basis.basis_check(directory, relative)[0])
                path.write_text('committed\n')
                self.assertEqual(basis.basis_check(directory, relative), ([], []))
                with mock.patch.object(archive, 'base_of', return_value='HEAD'):
                    self.assertIn(relative, archive.past_round_files(directory))

    def test_archive_preserves_dirty_paths_with_control_characters(self):
        archive = load('archive-past-rounds')
        with tempfile.TemporaryDirectory() as directory:
            def git(*args):
                return subprocess.run(['git', '-C', directory, *args], check=True, capture_output=True)
            git('init')
            paths = ['research/results/e999-' + marker + '.txt' for marker in ('tab\tname', 'line\nname', 'cr\rname')]
            for relative in paths:
                path = Path(directory) / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('original\n')
            git('add', '.')
            git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'commit', '-m', 'base')
            with mock.patch.object(archive, 'base_of', return_value='HEAD'):
                self.assertEqual(set(archive.past_round_files(directory)), set(paths))
                for relative in paths:
                    (Path(directory) / relative).write_text('modified\n')
                self.assertEqual(archive.past_round_files(directory), [])
                git('add', '.')
                for relative in paths:
                    (Path(directory) / relative).write_text('original\n')
                self.assertEqual(archive.past_round_files(directory), [])

    def test_archive_rechecks_contents_after_candidate_selection(self):
        archive = load('archive-past-rounds')
        with tempfile.TemporaryDirectory() as directory:
            def git(*args):
                return subprocess.run(['git', '-C', directory, *args], check=True, capture_output=True)
            git('init')
            relative = 'research/results/e999-note.txt'
            path = Path(directory) / relative
            path.parent.mkdir(parents=True)
            path.write_text('committed\n')
            git('add', '.')
            git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'commit', '-m', 'base')
            original_selection = archive.past_round_files
            def concurrent_write(root, **kwargs):
                selected = original_selection(root, **kwargs)
                path.write_text('new evidence\n')
                return selected
            with mock.patch.dict(os.environ, {'GATE_BASE': 'HEAD'}), \
                    mock.patch.object(archive, 'past_round_files', side_effect=concurrent_write), \
                    contextlib.redirect_stdout(io.StringIO()):
                result = archive.run(directory, True)
            self.assertNotEqual(result, 0)
            self.assertEqual(path.read_text(), 'new evidence\n')

    def test_batch_target_refuses_overlapping_choices(self):
        module = load('batch-target')
        for arguments in (['--path', 'docs', '--match', 'mine', 'docs/a.md'],
                          ['--match', 'one', 'a.md', '--match', 'two', 'a.md']):
            with self.subTest(arguments=arguments), self.assertRaises(module.Refusal):
                module.build('/unused', arguments)

    def test_fences_preserve_nested_markers_and_history_text(self):
        doc = load('doc-consolidate')
        history = load('history-consolidate')
        lines = ['````text', '```', '## fake', '````', 'outside']
        self.assertEqual(doc.fenced_flags(lines), [True, True, True, True, False])
        source = '## D1（样本）\n### 2026-01-01\n#### one\n~~~text\n## D2（假的）\n### 2026-01-02\n~~~\n'
        _, sections = history.parse_history(source)
        self.assertEqual(len(sections), 1)
        self.assertEqual(len(sections[0].days), 1)
        self.assertIn('## D2（假的）', sections[0].days[0].entries[0].body)

    def test_failed_count_command_cannot_pass(self):
        source = (KB / 'verify-citations.sh').read_text().split('# ── 断言表')[0]
        source = '\n'.join(line for line in source.splitlines()
                           if not line.startswith(('source ', 'preflight ')))
        source += "\nckn D1 refs-linux test 0 \"printf 0; : '%ROOT%'; false\"\n[[ $fail == 1 && $pass == 0 ]]\n"
        # bash -c 起的截段里 BASH_SOURCE 是空的，脚本按自己所在目录找 lib_work_root.py 取共享数据根那一步找不到；
        # 两棵树的根照文件头「两棵树的根」用 FS_REFS、KERNEL_TREE 给定，这一条只判 ckn 的计数，不读树
        with tempfile.TemporaryDirectory() as directory:
            environment = dict(os.environ, FS_REFS=os.path.join(directory, 'refs'), KERNEL_TREE=os.path.join(directory, 'kern'))
            for name in ('VERIFY_CITATIONS_BREAK', 'VERIFY_CITATIONS_ASSERTIONS', 'VERIFY_CITATIONS_SELFTEST_MARK'):
                environment.pop(name, None)
            result = subprocess.run(['bash', '-c', source], capture_output=True, text=True, env=environment)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_pdf_rejects_truncated_compressed_evidence(self):
        module = load('pdf-text')
        payload = zlib.compress(b'BT (apparently complete evidence) Tj ET')
        def body(data):
            return b'<< /Filter /FlateDecode >>\nstream\n' + data + b'\nendstream'
        self.assertEqual(module.stream_bytes(body(payload)), b'BT (apparently complete evidence) Tj ET')
        with self.assertRaises(ValueError):
            module.stream_bytes(body(payload[:-5]))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'truncated.pdf'
            path.write_bytes(b'%PDF-1.4\n1 0 obj\n' + body(payload[:-5]) + b'\nendobj\n')
            with self.assertRaises(ValueError):
                module.pdf_text(path)

    def test_pdf_quote_operators_and_escaped_line_breaks(self):
        module = load('pdf-text')
        self.assertEqual(module.unescape(b'(a\\\n b)'), b'a b')
        self.assertEqual(module.extract_text(b"BT (one) Tj (two) ' ET", {}), 'one\ntwo\n')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'a.pdf'
            path.write_bytes(b"%PDF-1.4\n1 0 obj\n<< >>\nstream\nBT (only) ' ET\nendstream\nendobj\n")
            self.assertIn('only', module.pdf_text(path))

    def test_fetch_records_and_checks_digest_without_network(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binaries = root / 'bin'
            binaries.mkdir()
            curl = binaries / 'curl'
            curl.write_text("#!/usr/bin/env python3\nimport pathlib,sys\npathlib.Path(sys.argv[sys.argv.index('-o')+1]).write_bytes(b'%PDF-1.4\\nsynthetic PDF\\n')\n")
            curl.chmod(0o755)
            environment = dict(os.environ, FS_REFS=str(root / 'refs'), PATH=str(binaries) + os.pathsep + os.environ['PATH'])
            command = ['bash', str(KB / 'fetch-refs.sh')]
            result = subprocess.run(command + ['--check'], env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertFalse((root / 'refs').exists())
            result = subprocess.run(command + ['--typo'], env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertFalse((root / 'refs').exists())
            result = subprocess.run(command, env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            result = subprocess.run(command + ['--check'], env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            path = root / 'refs/docs/naclcrypto.pdf'
            # PDF 丢失但旧摘要仍在时，只能恢复同一份证据，不能重设摘要。
            digest_path = path.with_suffix('.pdf.sha256')
            previous_digest = digest_path.read_bytes()
            curl_source = curl.read_text()
            path.unlink()
            curl.write_text(curl_source.replace('synthetic PDF', 'different PDF'))
            result = subprocess.run(command, env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertFalse(path.exists())
            self.assertEqual(digest_path.read_bytes(), previous_digest)
            curl.write_text(curl_source)
            result = subprocess.run(command, env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            path.write_bytes(path.read_bytes() + b'changed')
            result = subprocess.run(command + ['--check'], env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn('sha256', result.stderr)
            # 没摘要的旧证据不能被变更后的远端副本覆盖。
            previous = path.read_bytes()
            path.with_suffix('.pdf.sha256').unlink()
            result = subprocess.run(command, env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(path.read_bytes(), previous)
            self.assertFalse(path.with_suffix('.pdf.sha256').exists())

    def test_source_bytes_keep_crlf(self):
        for name in ('replace-once', 'replace-batch'):
            module = load(name)
            with tempfile.TemporaryDirectory() as directory, self.subTest(tool=name):
                path = Path(directory) / 'a.txt'
                path.write_bytes(b'old\r\nnext\r\n')
                with contextlib.redirect_stdout(io.StringIO()):
                    code = (module.replace_once(str(path), 'old', 'new')[0] if name == 'replace-once'
                            else module.run([{'file': str(path), 'old': 'old', 'new': 'new'}], False))
                self.assertEqual(code, 0)
                self.assertEqual(path.read_bytes(), b'new\r\nnext\r\n')

    def test_insert_matches_logical_lines_and_preserves_endings(self):
        module = load('insert-row')
        for ending in (b'\n', b'\r\n', b'\r'):
            for trailing in (True, False):
                with self.subTest(ending=ending, trailing=trailing), tempfile.TemporaryDirectory() as directory:
                    path = Path(directory) / 'a.txt'
                    path.write_bytes(b'head' + ending + b'anchor' + (ending if trailing else b''))
                    with contextlib.redirect_stdout(io.StringIO()):
                        self.assertEqual(module.insert(str(path), '^anchor$', 'after', 'new', '^new$', False), 0)
                    expected = ending.join([b'head', b'anchor', b'new']) + (ending if trailing else b'')
                    self.assertEqual(path.read_bytes(), expected)
                    with contextlib.redirect_stdout(io.StringIO()):
                        self.assertEqual(module.insert(str(path), '^anchor$', 'after', 'new', '^new$', False), 2)

    def test_basis_path_followed_by_chinese_still_checks_dirty_file(self):
        basis = load('lib_basis_paths')
        with tempfile.TemporaryDirectory() as directory:
            def git(*args):
                subprocess.run(['git', '-C', directory, *args], check=True, capture_output=True)
            git('init')
            for relative in ('scripts/x.py', 'records/中文说明.md'):
                path = Path(directory) / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('committed\n')
            git('add', '.')
            git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'commit', '-m', 'base')
            for relative in ('scripts/x.py', 'records/中文说明.md'):
                (Path(directory) / relative).write_text('dirty\n')
                for citation in (f'依据{relative}中的实现', f'依据{relative}:1中的实现', f'依据 {relative}.'):
                    self.assertEqual(basis.paths_cited(citation)[0][0], relative)
                    self.assertTrue(basis.basis_check(directory, citation)[0])

    def test_basis_paths_include_chinese_filenames(self):
        module = load('lib_basis_paths')
        self.assertEqual(module.paths_cited('`.claude/kb/decisions/001-样本.md:3-4`'),
                         [('.claude/kb/decisions/001-样本.md', 3, 4)])
        self.assertEqual(module.paths_cited('依据scripts/kb/foo.py'), [('scripts/kb/foo.py', None, None)])

    def test_basis_git_failure_cannot_be_overridden_by_line_exception(self):
        module = load('lib_basis_paths')
        with tempfile.TemporaryDirectory() as directory:
            state = module.CommittedState.__new__(module.CommittedState)
            state.root, state.usable = directory, True
            state.cache = {('head', 'docs/a.md'): [b'unchanged']}
            error = subprocess.CompletedProcess([], 128, b'', b'index read error')
            with mock.patch.object(state, '_git', return_value=error), \
                    mock.patch.object(state, 'index_lines', return_value=[b'unchanged']), \
                    mock.patch.object(state, 'worktree_lines', return_value=[b'unchanged']):
                problems, _ = module.basis_check(directory, '`docs/a.md:1`', committed=state)
            self.assertTrue(problems)
            self.assertIn('git diff', problems[0])

    def test_basis_corrupt_index_is_not_an_untracked_file(self):
        module = load('lib_basis_paths')
        with tempfile.TemporaryDirectory() as directory:
            def git(*args):
                return subprocess.run(['git', '-C', directory, *args], check=True, capture_output=True)
            git('init')
            git('-c', 'user.name=test', '-c', 'user.email=test@example.com',
                'commit', '--allow-empty', '-m', 'init')
            path = Path(directory) / 'docs/new.md'
            path.parent.mkdir()
            path.write_text('staged evidence\n')
            git('add', '.')
            (Path(directory) / '.git/index').write_bytes(b'corrupt index')
            problems, untracked = module.basis_check(directory, 'docs/new.md')
            self.assertTrue(problems)
            self.assertEqual(untracked, [])

    def test_basis_missing_blob_is_not_a_missing_path(self):
        module = load('lib_basis_paths')
        with tempfile.TemporaryDirectory() as directory:
            def git(*args):
                return subprocess.run(['git', '-C', directory, *args], check=True, capture_output=True)
            git('init')
            path = Path(directory) / 'docs/a.md'
            path.parent.mkdir()
            path.write_text('original evidence\n')
            git('add', '.')
            git('-c', 'user.name=test', '-c', 'user.email=test@example.com', 'commit', '-m', 'init')
            blob = git('rev-parse', 'HEAD:docs/a.md').stdout.decode().strip()
            git('rm', '--cached', 'docs/a.md')
            (Path(directory) / '.git/objects' / blob[:2] / blob[2:]).unlink()
            problems, untracked = module.basis_check(directory, 'docs/a.md')
            self.assertTrue(problems)
            self.assertEqual(untracked, [])

    def test_relabel_does_not_apply_old_offsets_to_new_content(self):
        module = load('relabel-item')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'a.md'
            text = '未定项 1\n'
            changed = 'other writer: ' + text
            path.write_text(text)
            def references(*args):
                yield (1, text, re.search(r'(未定项) (1)', text), '未', 'D90', None, 'D90')
                path.write_text(changed)
            library = SimpleNamespace(load_map=lambda: ({'D90': {1: '已'}}, {'D90': 'sample'}),
                                      self_decisions=lambda: {}, scanned_files=lambda: ['a.md'],
                                      references=references)
            with mock.patch.object(module, 'load_library', return_value=library), contextlib.redirect_stdout(io.StringIO()):
                code, _ = module.relabel(directory, 'D90', 1, False)
            self.assertNotEqual(code, 0)
            self.assertEqual(path.read_text(), changed)

    def test_pdf_page_contents_share_text_state(self):
        module = load('pdf-text')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'a.pdf'
            for contents in (b'[2 0 R 3 0 R]', b'4 0 R'):
                path.write_bytes(b'%PDF-1.4\n1 0 obj\n<< /Type /Page /Contents ' + contents +
                                 b' >>\nendobj\n2 0 obj\n<< >>\nstream\nBT\nendstream\nendobj\n'
                                 b'3 0 obj\n<< >>\nstream\n(hello) Tj ET\nendstream\nendobj\n'
                                 b'4 0 obj\n[2 0 R 3 0 R]\nendobj\n')
                self.assertEqual(module.pdf_text(path), 'hello\n')

    def test_pdf_nested_strings_and_non_text_operators(self):
        module = load('pdf-text')
        self.assertEqual(module.extract_text(b'BT (hello (world) ET) Tj ET', {}), 'hello (world) ET\n')
        self.assertEqual(module.extract_text(b'% BT (fake) Tj\nBT <414> Tj ET', {}), 'A@\n')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'a.pdf'
            path.write_bytes(b'%PDF-1.4\n1 0 obj\n<< >>\nstream\nBT(hello)Tj ET\nendstream\nendobj\n')
            self.assertEqual(module.pdf_text(path), 'hello\n')

    def test_history_code_is_not_rewritten_as_prose(self):
        module = load('history-consolidate')
        body = ['~~~text', '> 快查·改后：code', '2026-01-01 literal', '', '', '~~~']
        entry, _ = module.transform_entry(module.Entry('title', body), '2026-01-01', None,
                                         module.new_report(), 'sample', None)
        self.assertEqual(entry.body, body)

    def test_batch_write_each_switch_is_refused_at_entry_and_dry_run_never_writes(self):
        # 审核 A17 的三场景（records/2026-11-02-文档代码逐项审核-复现.py）：普通入口带 REPLACE_BATCH_WRITE_EACH=1 退 2、一个字节不写；
        # 进程内（自证里能设它的地方）设着它时 --dry-run 也只核不写，后一处锚点不中时前一处同样不写
        environment = {key: value for key, value in os.environ.items()
                       if key not in ('REPLACE_BATCH_BREAK', 'REPLACE_BATCH_WRITE_EACH', 'ATOMIC_REPLACE_BREAK')}
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / 'target.txt'
            for label, edits, write_each, wanted_code in (
                    ('正常干跑', [{'file': str(target), 'old': 'original', 'new': 'replacement'}], False, 0),
                    ('带开关干跑', [{'file': str(target), 'old': 'original', 'new': 'replacement'}], True, 2),
                    ('带开关干跑、后一处锚点不中', [{'file': str(target), 'old': 'original', 'new': 'replacement'},
                                       {'file': str(target), 'old': 'absent', 'new': 'other'}], True, 2)):
                with self.subTest(case=label):
                    target.write_text('original\n', encoding='utf-8')
                    specification = Path(directory) / 'spec.json'
                    specification.write_text(json.dumps(edits), encoding='utf-8')
                    run_environment = dict(environment, REPLACE_BATCH_WRITE_EACH='1') if write_each else environment
                    result = subprocess.run([sys.executable, '-B', str(KB / 'replace-batch.py'), '--dry-run', str(specification)],
                                            cwd=directory, env=run_environment, capture_output=True, text=True, timeout=60)
                    self.assertEqual(result.returncode, wanted_code, result.stdout + result.stderr)
                    self.assertEqual(target.read_text(encoding='utf-8'), 'original\n')
                    if write_each:
                        self.assertIn('REPLACE_BATCH_WRITE_EACH', result.stderr)
            module = load('replace-batch')
            for label, edits, wanted_code in (
                    ('进程内设着开关、全中的干跑', [{'file': str(target), 'old': 'original', 'new': 'replacement'}], 0),
                    ('进程内设着开关、后一处锚点不中的干跑', [{'file': str(target), 'old': 'original', 'new': 'replacement'},
                                             {'file': str(target), 'old': 'absent', 'new': 'other'}], 1)):
                with self.subTest(case=label), mock.patch.dict(os.environ, {'REPLACE_BATCH_WRITE_EACH': '1'}), \
                        contextlib.redirect_stdout(io.StringIO()):
                    target.write_text('original\n', encoding='utf-8')
                    self.assertEqual(module.run(edits, dry_run=True), wanted_code)
                    self.assertEqual(target.read_text(encoding='utf-8'), 'original\n')

    def test_append_validators_follow_the_markdown_fence_state(self):
        # 审核 A44 的六种输入（records/2026-11-02-文档代码逐项审核-追加标题复现.py）：两个追加工具都按 lib_markdown.py 的围栏认、标题去 0–3 个前导空格
        append = load('append-record')
        ledger = load('ledger-append-section')
        cases = [
            ('paragraph', 'ordinary paragraph\n', False),
            ('heading', '## real heading\n', True),
            ('simple_fence', '```\n## code\n```\n', False),
            ('shorter_inner_fence', '````\n```\n## still code\n````\n', False),
            ('indented_heading', '  ## real heading\n', True),
            ('indented_code_then_heading', '    ```\n## real heading\n', True),
        ]
        for tool, problem_of in (('append-record', append.text_problem),
                                 ('ledger-append-section', lambda body: ledger.body_problem('title', body, (('status', 'open'),)))):
            for label, body, refused in cases:
                with self.subTest(tool=tool, case=label):
                    self.assertEqual(problem_of(body) is not None, refused, problem_of(body))

    def test_ledger_lock_keeps_both_appends_when_both_would_pass_the_digest_check(self):
        # 审核 A45 的交错（records/2026-11-02-文档代码逐项审核-账本并发复现.py）：甲过了写前复核、停在改名之前，乙这时开跑。
        # 有账本锁：乙卡在锁上，甲等不到乙过复核、照常换上，乙读新账本取下一个号，两份正文都在。
        # 对照（弄坏开关 no-ledger-lock）：乙读到同一版、复核也过，两个都退 0 而只剩一份正文——证明这条测试判得出覆盖。
        for broken, both_bodies_kept in (('', True), ('no-ledger-lock', False)):
            with self.subTest(broken=broken or '（不设开关）'), tempfile.TemporaryDirectory() as scratch:
                module = load('ledger-append-section')
                real_replace = module.replace_file_contents_by_rename
                first_checked, second_checked = threading.Event(), threading.Event()
                worker = threading.local()

                def controlled_replace(path, text, check_before_rename=None):
                    def checked_then_wait():
                        check_before_rename()
                        if worker.label == '甲':
                            first_checked.set()
                            second_checked.wait(timeout=3)
                        else:
                            second_checked.set()
                    return real_replace(path, text, check_before_rename=checked_then_wait)

                root, ledger_path = module.make_root(scratch, 'repo', module.sample_ledger(['一'], ['一']))

                def append(label):
                    worker.label = label
                    with contextlib.redirect_stdout(io.StringIO()):
                        return module.append_section(ledger_path, '来自' + label, '来自%s的正文\n' % label,
                                                     [('判定', '新登'), ('历史', '—'), ('开着的', '整节')], notation_root=root)

                with mock.patch.object(module, 'replace_file_contents_by_rename', controlled_replace), \
                        mock.patch.object(module, 'BROKEN', broken):
                    with ThreadPoolExecutor(max_workers=2) as pool:
                        first = pool.submit(append, '甲')
                        self.assertTrue(first_checked.wait(timeout=30), '甲没走到写前复核之后')
                        second = pool.submit(append, '乙')
                        codes = [first.result(timeout=60), second.result(timeout=60)]
                final = Path(ledger_path).read_text(encoding='utf-8')
                headings, indexed, _, _ = module.scan(final.split('\n'))
                self.assertEqual(codes, [0, 0])
                if both_bodies_kept:
                    self.assertIn('来自甲的正文', final)
                    self.assertIn('来自乙的正文', final)
                    self.assertEqual((headings, indexed), ([1, 2, 3], [1, 2, 3]))
                else:
                    self.assertEqual(('来自甲的正文' in final) + ('来自乙的正文' in final), 1)
                    self.assertEqual((headings, indexed), ([1, 2], [1, 2]))

    def test_interruption_after_rename_is_reported_as_already_replaced(self):
        # 审核 A49（受控故障）与 A19：改名之后、目录 fsync 时到的 Ctrl-C 要让调用方分得出新内容已经换上；
        # 改名之前 fsync 临时文件失败（EIO）照旧是 ReplaceRefused、原文不动
        atomic = load('lib_atomic_replace')
        replace_once = load('replace-once')
        original_fsync = os.fsync

        def interrupted_on_directory(descriptor):
            if stat.S_ISDIR(os.fstat(descriptor).st_mode):
                raise KeyboardInterrupt('注入：目录 fsync 时按了 Ctrl-C')
            return original_fsync(descriptor)

        def failing_fsync(descriptor):
            raise OSError(5, '注入：fsync 失败')

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'target.md'
            path.write_text('OLD\n', encoding='utf-8')
            with mock.patch.object(os, 'fsync', interrupted_on_directory):
                with self.assertRaises(atomic.ReplacedThenInterrupted) as raised:
                    atomic.replace_file_contents_by_rename(str(path), 'NEW\n')
            self.assertIsInstance(raised.exception, KeyboardInterrupt)
            self.assertEqual(path.read_text(encoding='utf-8'), 'NEW\n')
            self.assertEqual(atomic.leftover_temporary_files(directory), [])
            path.write_text('OLD\n', encoding='utf-8')
            with mock.patch.object(os, 'fsync', interrupted_on_directory):
                code, message = replace_once.replace_once(str(path), 'OLD', 'NEW')
            self.assertEqual(code, 5, message)
            self.assertEqual(path.read_text(encoding='utf-8'), 'NEW\n')
            path.write_text('OLD\n', encoding='utf-8')
            with mock.patch.object(os, 'fsync', failing_fsync):
                with self.assertRaises(atomic.ReplaceRefused):
                    atomic.replace_file_contents_by_rename(str(path), 'NEW\n')
            self.assertEqual(path.read_text(encoding='utf-8'), 'OLD\n')
            self.assertEqual(atomic.leftover_temporary_files(directory), [])

    def test_basis_line_reference_to_a_file_deleted_in_the_worktree_is_refused(self):
        # 审核 A48：与 HEAD 不同的那一版缺了（工作区删掉）不跳过，写了行号的引用照样拒
        module = load('lib_basis_paths')
        with tempfile.TemporaryDirectory() as directory:
            def git(*args):
                return subprocess.run(['git', '-C', directory, *args], check=True, capture_output=True)
            git('init')
            path = Path(directory) / 'docs/a.md'
            path.parent.mkdir()
            path.write_text('first\nsecond\n')
            git('add', '.')
            git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'commit', '-m', 'base')
            self.assertEqual(module.basis_check(directory, '`docs/a.md:1`')[0], [])
            path.unlink()
            problems, untracked = module.basis_check(directory, '`docs/a.md:1`')
            self.assertTrue(problems)
            self.assertEqual(untracked, [])


if __name__ == '__main__':
    unittest.main()
