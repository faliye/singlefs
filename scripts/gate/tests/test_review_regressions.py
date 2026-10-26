"""Isolated regressions for gate review; no real compiler or main index writes."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.dont_write_bytecode = True
GATE = Path(__file__).resolve().parents[1]


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), GATE / f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ReviewRegressions(unittest.TestCase):
    def test_git_quoted_and_renamed_patch_paths(self):
        rebuilt = load('lib_rebuilt_packages')
        writer = load('apply-writer-patch')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '-q', directory], check=True)
            relative = 'crates/singlefs-checker/src/中文.rs'
            source = root / relative
            source.parent.mkdir(parents=True)
            source.write_text('old\n')
            subprocess.run(['git', '-C', directory, 'add', '.'], check=True)
            source.write_text('new\n')
            patch = subprocess.check_output(['git', '-C', directory, 'diff'], text=True)
            (root / 'crates.patch').write_text(patch)
            with self.subTest(reader='rebuild'):
                self.assertEqual(rebuilt.paths_in_patch(patch), [relative])
            with self.subTest(reader='writer'):
                self.assertEqual(writer.touched_paths(directory), {relative})
            plain = '--- a/one.rs\n+++ b/one.rs\n@@ -1 +1 @@\n-old\n+++ b/fake.rs\n--- a/two.rs\n+++ b/two.rs\n@@ -1 +1 @@\n-old\n+new\n'
            self.assertEqual(rebuilt.paths_in_patch(plain), ['one.rs', 'two.rs'])
            renamed = 'diff --git a/old.rs b/new.rs\nsimilarity index 100%\nrename from old.rs\nrename to new.rs\n'
            with self.subTest(reader='rename'):
                self.assertEqual(rebuilt.paths_in_patch(renamed), ['old.rs', 'new.rs'])

    def test_gate_staged_accepts_relative_entry_and_repository_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'repo'
            root.mkdir()
            for args in [('init', '-q'), ('config', 'user.name', 'gate-test'),
                         ('config', 'user.email', 'gate-test@example.invalid')]:
                subprocess.run(['git', '-C', str(root), *args], check=True)
            (root / '.claude/scripts').mkdir(parents=True)
            (root / '.claude/scripts/gate.sh').write_text('#!/bin/bash\nexit 0\n')
            cwd = GATE.parents[1]
            environment = dict(os.environ, SINGLEFS_GATE_TARGET_DIR=str(Path(directory) / 'target'),
                               SINGLEFS_GATE_STAGED_LOG_DIRECTORY=str(Path(directory) / 'logs'))
            result = subprocess.run(['bash', 'scripts/gate/gate-staged.sh', os.path.relpath(root, cwd)],
                                    cwd=cwd, env=environment, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn('✓ 整轮全绿，refs/sop/staged-green 前移', result.stdout)

    def test_verus_mutation_requires_an_actual_failed_cell(self):
        verifier = load('verus-proof-check')
        self.assertFalse(verifier.selftest_break_caught(1, 'Traceback: unexpected exception'))
        self.assertFalse(verifier.selftest_break_caught(-9, ''))
        self.assertFalse(verifier.selftest_break_caught(2, 'bad arguments'))
        self.assertFalse(verifier.selftest_break_caught(0, '  ✗ proof-count 这一格判错：unexpected'))
        self.assertTrue(verifier.selftest_break_caught(1, '  ✗ proof-count 这一格判错：expected'))

    def setUp(self):
        # Every Git fixture owns its index, even when invoked from a private-index session.
        environment = {name: value for name, value in os.environ.items() if not name.startswith('GIT_')}
        patch = mock.patch.dict(os.environ, environment, clear=True)
        patch.start()
        self.addCleanup(patch.stop)

    def test_gate_staged_reports_failure_to_record_green_reference(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'repo'
            root.mkdir()
            real_git = shutil.which('git')
            def git(*args):
                return subprocess.check_output([real_git, '-C', str(root), *args], stderr=subprocess.DEVNULL)
            git('init', '-q')
            git('config', 'user.name', 'gate-test')
            git('config', 'user.email', 'gate-test@example.invalid')
            git('commit', '--allow-empty', '-qm', 'base')
            (root / '.claude/scripts').mkdir(parents=True)
            (root / '.claude/scripts/gate.sh').write_text('#!/bin/bash\nexit 0\n')
            binaries = Path(directory) / 'bin'
            binaries.mkdir()
            shim = binaries / 'git'
            shim.write_text('#!/bin/bash\nif [[ "$1" == "$FAIL_GIT_SUBCOMMAND" ]]; then exit 1; fi\n'
                            f'exec {shlex.quote(real_git)} "$@"\n')
            shim.chmod(0o755)
            for command in ('commit-tree', 'update-ref'):
                with self.subTest(command=command):
                    environment = dict(os.environ, PATH=str(binaries) + os.pathsep + os.environ['PATH'],
                                       FAIL_GIT_SUBCOMMAND=command, SINGLEFS_GATE_TARGET_DIR=str(Path(directory) / 'target'),
                                       SINGLEFS_GATE_STAGED_LOG_DIRECTORY=str(Path(directory) / 'logs'))
                    result = subprocess.run(['bash', str(GATE / 'gate-staged.sh'), str(root)], env=environment,
                                            capture_output=True, text=True, timeout=30)
                    self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                    self.assertNotIn('✓ 整轮全绿，refs/sop/staged-green 前移', result.stdout)

    def test_baseline_diff_does_not_mask_numeric_file_identities_or_values(self):
        script = (GATE / 'baseline-diff.sh').read_text()
        program = script.split('compare_outputs() {\n  python3 - "$@" <<\'PY\'\n', 1)[1].split('\nPY\n', 1)[0]
        with tempfile.TemporaryDirectory() as directory:
            base, work = Path(directory) / 'base.log', Path(directory) / 'work.log'
            for before, after in [('✗ e9.rs 不正确', '✗ e10.rs 不正确'),
                                  ('✗ checksum: 123', '✗ checksum: 456')]:
                with self.subTest(after=after):
                    base.write_text(before + '\n')
                    work.write_text(after + '\n')
                    result = subprocess.run([sys.executable, '-c', program, str(base), str(work),
                                             '/base', '/work', '1', '1', 'gate', 'base', '0'],
                                            capture_output=True, text=True)
                    self.assertEqual(result.returncode, 1, result.stdout + result.stderr)

    def test_literal_global_file_query_is_keyable_but_dynamic_globals_are_not(self):
        units = load('lib_function_units')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'sample.py'
            path.write_text('def entry():\n    return globals().get("__file__")\n')
            key, forms = units.fixture_key(directory, 'sample.py::entry')
            self.assertIsNotNone(key, forms)
            path.write_text('def entry(name):\n    return globals().get(name)\n')
            key, forms = units.fixture_key(directory, 'sample.py::entry')
            self.assertIsNone(key)
            self.assertIn('globals-lookup', forms)

    def test_target_selection_is_removed_only_from_execution_arguments(self):
        runner = load('test-targets-parallel')
        arguments = ['--workspace', '--exclude', 'other', '--tests', '--test=alpha', '--offline', '--features', 'extra']
        kept, selected = runner.execution_arguments(arguments)
        self.assertEqual(kept, ['--offline', '--features', 'extra'])
        self.assertTrue(selected)
        self.assertIn('--tests', arguments)

    def test_metadata_default_members_and_workspace_exclusions(self):
        runner = load('test-targets-parallel')
        metadata = {'packages': [{'id': name, 'name': name, 'targets': []} for name in ('first', 'second')],
                    'workspace_members': ['first', 'second'], 'workspace_default_members': ['second']}
        response = subprocess.CompletedProcess([], 0, json.dumps(metadata), '')
        with mock.patch.object(runner.subprocess, 'run', return_value=response):
            self.assertEqual(runner.read_metadata([])[0][2], ['second'])
            self.assertEqual(runner.read_metadata(['--workspace', '--exclude', 'sec*'])[0][2], ['first'])

    def test_version_qualified_package_keeps_doctests(self):
        runner = load('test-targets-parallel')
        commands = []
        def run(command, **kwargs):
            commands.append(command)
            return subprocess.CompletedProcess(command, 0)
        with mock.patch.object(runner, 'build', return_value=(0, [])), \
                mock.patch.object(runner, 'read_metadata', return_value=(({'demo-id': 'demo'}, {'demo': True}, ['demo'], None), None)), \
                mock.patch.object(runner.subprocess, 'run', side_effect=run), \
                contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(runner.run(1, ['-p', 'demo@1.0']), 0)
        self.assertEqual(commands, [['cargo', 'test', '-p', 'demo', '--doc']])

    def test_non_utf8_filename_still_checks_worktree_size(self):
        checker = load('large-file-check')
        with tempfile.TemporaryDirectory() as directory:
            subprocess.run(['git', 'init', '-q', directory], check=True)
            path = os.fsencode(directory) + b'/file-\xff'
            with open(path, 'wb') as handle:
                handle.write(b'x')
            subprocess.run(['git', '-C', directory, 'add', '.'], check=True)
            with open(path, 'wb') as handle:
                handle.write(b'x' * 33)
            _, oversized, _ = checker.judge_tracked(directory, 32)
            self.assertEqual(len(oversized), 1)

    def test_explicit_test_target_does_not_add_doctests(self):
        runner = load('test-targets-parallel')
        artifact = {'executable': '/fake/test', 'profile': {'test': True},
                    'target': {'kind': ['test'], 'name': 'alpha'}, 'package_id': 'demo-id'}
        commands = []
        def run(command, **kwargs):
            commands.append(command)
            return subprocess.CompletedProcess(command, 0)
        with mock.patch.object(runner, 'build', return_value=(0, [artifact])), \
                mock.patch.object(runner, 'read_metadata', return_value=(({'demo-id': 'demo'}, {'demo': True}, ['demo'], None), None)), \
                mock.patch.object(runner.subprocess, 'run', side_effect=run), \
                contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(runner.run(1, ['-p', 'demo', '--test', 'alpha']), 0)
        self.assertFalse(any('--doc' in command for command in commands))
        self.assertEqual(commands[0].count('--test'), 1)

    def test_metadata_uses_requested_manifest(self):
        runner = load('test-targets-parallel')
        response = subprocess.CompletedProcess([], 0, '{"packages": [], "workspace_members": []}', '')
        with mock.patch.object(runner.subprocess, 'run', return_value=response) as run:
            runner.read_metadata(['--manifest-path', 'another/Cargo.toml', '--offline'])
        command = run.call_args.args[0]
        self.assertIn('--manifest-path', command)
        self.assertEqual(command[command.index('--manifest-path') + 1], 'another/Cargo.toml')

    def test_deleted_checker_source_requires_report(self):
        writer = load('apply-writer-patch')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'crates/singlefs-checker/src').mkdir(parents=True)
            (root / 'crates/mutations.tsv').write_text('# mutations\n')
            (root / 'crates/singlefs-checker/src/lib.rs').write_text('pub fn check() {}\n')
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            patch = root / 'patch'
            patch.mkdir()
            (patch / 'crates.patch').write_text(
                'diff --git a/crates/singlefs-checker/src/lib.rs b/crates/singlefs-checker/src/lib.rs\n'
                'deleted file mode 100644\n--- a/crates/singlefs-checker/src/lib.rs\n+++ /dev/null\n'
                '@@ -1 +0,0 @@\n-pub fn check() {}\n')
            problems, _, _ = writer.plan(str(patch), str(root))
            self.assertFalse(any('git apply --check' in problem for problem in problems), problems)
            self.assertTrue(any(writer.CHECKER_SECTION in problem for problem in problems))

    def test_staged_tree_keeps_private_index_and_materializes_its_tree(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'repo'
            root.mkdir()
            environment = {key: value for key, value in os.environ.items() if not key.startswith('GIT_')}
            def git(*args, env=environment):
                return subprocess.check_output(['git', '-C', str(root), *args], env=env, stderr=subprocess.DEVNULL)
            git('init', '-q')
            git('config', 'user.name', 'gate-test')
            git('config', 'user.email', 'gate-test@example.invalid')
            (root / 'file').write_text('head\n')
            git('add', '.')
            git('commit', '-qm', 'base')
            private = dict(environment, GIT_INDEX_FILE=str(Path(directory) / 'private-index'))
            git('read-tree', 'HEAD', env=private)
            (root / 'file').write_text('staged\n')
            git('add', 'file', env=private)
            tree = git('write-tree', env=private).decode().strip()
            checkout = Path(directory) / 'checkout'
            try:
                result = subprocess.run(['bash', str(GATE / 'staged-tree-worktree.sh'), 'create', str(root),
                                         '--tree', tree, '--path', str(checkout)], env=private,
                                        capture_output=True, text=True, timeout=30)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertEqual((checkout / 'file').read_text(), 'staged\n')
                self.assertEqual(git('write-tree', env=private).decode().strip(), tree)
                checkout_tree = subprocess.check_output(['git', '-C', str(checkout), 'write-tree'], env=environment).decode().strip()
                self.assertEqual(checkout_tree, tree)
            finally:
                subprocess.run(['git', '-C', str(root), 'worktree', 'remove', '--force', str(checkout)],
                               env=environment, capture_output=True)

    def test_empty_index_still_checks_unpushed_blobs(self):
        checker = load('large-file-check')
        with tempfile.TemporaryDirectory() as directory:
            def git(*args):
                return subprocess.check_output(['git', '-C', directory, *args], stderr=subprocess.DEVNULL)
            git('init', '-q')
            git('config', 'user.name', 'gate-test')
            git('config', 'user.email', 'gate-test@example.invalid')
            git('commit', '--allow-empty', '-qm', 'base')
            git('branch', 'upstream')
            git('branch', '--set-upstream-to=upstream')
            path = Path(directory) / 'oversized'
            path.write_bytes(b'x' * 33)
            git('add', 'oversized')
            git('commit', '-qm', 'oversized')
            git('rm', '-q', 'oversized')
            git('commit', '-qm', 'empty tree')
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(checker.judge(directory, 32, 'test'), 1)

    def test_compiler_diagnostics_are_forwarded(self):
        runner = load('test-targets-parallel')
        message = {'reason': 'compiler-message', 'message': {'rendered': 'error: precise compiler diagnostic\n'}}
        result = subprocess.CompletedProcess([], 101, json.dumps(message) + '\n')
        stderr = io.StringIO()
        with mock.patch.object(runner.subprocess, 'run', return_value=result), contextlib.redirect_stderr(stderr):
            code, _ = runner.build([], [])
        self.assertEqual(code, 101)
        self.assertIn('error: precise compiler diagnostic', stderr.getvalue())

    def test_orphan_failed_results_are_rejected(self):
        rerun = load('rerun-failed-tests')
        commands, problems = rerun.failed_tests_by_target(
            'test lost_case ... FAILED\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n')
        self.assertFalse(commands)
        self.assertTrue(problems)

    def test_unrecognized_running_header_cannot_reuse_previous_target(self):
        rerun = load('rerun-failed-tests')
        commands, problems = rerun.failed_tests_by_target(
            'Running unittests src/lib.rs (target/lib)\n'
            'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'
            'Running custom-target (target/custom)\n'
            'test lost_case ... FAILED\n'
            'test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n')
        self.assertFalse(commands)
        self.assertTrue(problems)


if __name__ == '__main__':
    unittest.main()
