#!/usr/bin/env python3
"""Agent script regressions using isolated files, repositories and processes.

Run: python3 -B -m unittest discover -s scripts/agents/tests -v
"""
import hashlib
import importlib.util
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest import mock

HERE = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True


def load_script(name):
    specification = importlib.util.spec_from_file_location(name.replace('-', '_'), HERE / name)
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    return module


class ScriptCase(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory(prefix='singlefs-agents-tests-')
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.repo = self.root / 'repo'
        self.repo.mkdir()
        self.env = {key: value for key, value in os.environ.items()
                    if not key.endswith('_BREAK') and not key.startswith('GIT_')}
        self.env.update(SINGLEFS_WORK_ROOT=str(self.root / 'work'),
                        REMIND_STATE_ROOT=str(self.root / 'reminders'), CLAUDE_CODE_SESSION_ID='audit')
        self.draft = self.root / 'work/draft'

    def command(self, name, *arguments):
        return ['bash' if name.endswith('.sh') else sys.executable, str(HERE / name), *map(str, arguments)]

    def run_script(self, name, *arguments):
        return subprocess.run(self.command(name, *arguments), env=self.env, capture_output=True,
                              text=True, timeout=30)

    def spawn(self, command):
        process = subprocess.Popen(command, env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   text=True, start_new_session=True)
        def cleanup():
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.communicate(timeout=10)
        self.addCleanup(cleanup)
        return process


class SnapshotTests(ScriptCase):
    def setUp(self):
        super().setUp()
        self.git('init', '-q')
        self.git('config', 'user.name', 'Test')
        self.git('config', 'user.email', 'test@example.invalid')
        self.git('config', 'commit.gpgsign', 'false')
        script = self.repo / 'run.sh'
        script.write_text('#!/bin/sh\necho snapshot\n')
        script.chmod(0o755)
        self.git('add', 'run.sh')
        self.git('commit', '-qm', 'fixture')
        self.snapshot = self.repo / 'research/prompts/audit-snapshot'
        self.snapshot.mkdir(parents=True)
        self.manifest = self.snapshot / 'judged-sha256.txt'
        self.manifest.write_text(self.entry('run.sh', script.read_bytes()))
        self.output = self.root / 'output/tree'

    def git(self, *args):
        return subprocess.run(['git', '-C', str(self.repo), *args], env=self.env, check=True,
                              capture_output=True, text=True)

    def entry(self, path, content):
        return f'{hashlib.sha256(content).hexdigest()}  {path}\n'

    def build(self):
        return self.run_script('snapshot-tree.py', 'audit', '--repository', self.repo, '--out', self.output)

    def test_head_executable_remains_executable(self):
        result = self.build()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual((self.output / 'run.sh').stat().st_mode & 0o777, 0o755)
        result = subprocess.run([str(self.output / 'run.sh')], capture_output=True, text=True, timeout=10)
        self.assertEqual(result.stdout, 'snapshot\n')

    def test_worktree_fallback_preserves_execution_bit(self):
        script = self.repo / 'untracked.sh'
        script.write_text('#!/bin/sh\necho fallback\n')
        script.chmod(0o755)
        self.manifest.write_text(self.entry(script.name, script.read_bytes()))
        result = self.build()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertTrue(os.access(self.output / script.name, os.X_OK))

    def test_invalid_paths_rejected_before_any_output(self):
        source = self.root / 'payload'
        source.write_text('replacement')
        self.output.parent.mkdir()
        victim = self.output.parent / 'payload'
        victim.write_text('valuable')
        good = self.manifest.read_text()
        for path in ('../payload', str(source), 'nested/../../payload', '.'):
            with self.subTest(path=path):
                self.manifest.write_text(good + self.entry(path, source.read_bytes()))
                result = self.build()
                self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                self.assertFalse(self.output.exists())
                self.assertEqual(victim.read_text(), 'valuable')
                self.assertEqual(source.read_text(), 'replacement')

    def test_diff_paths_cannot_escape_temporary_tree(self):
        for old, new in (('../payload', 'run.sh'), ('run.sh', '../payload')):
            with self.subTest(old=old, new=new):
                (self.snapshot / 'judged-vs-head.diff').write_text(f'diff --git a/{old} b/{new}\n')
                self.assertEqual(self.build().returncode, 2)
                self.assertFalse(self.output.exists())

    def test_malformed_manifest_is_not_partial_success(self):
        with self.manifest.open('a') as handle:
            handle.write('not-a-hash  omitted.txt\n')
        self.assertEqual(self.build().returncode, 2)
        self.assertFalse(self.output.exists())


class BackupTests(ScriptCase):
    def copy(self, *names):
        return self.run_script('copy-originals.py', 'copy', self.draft, *names, '--root', self.repo)

    def test_normal_backup_is_independent(self):
        source = self.repo / 'file'
        source.write_text('original')
        self.assertEqual(self.copy('file').returncode, 0)
        source.write_text('changed')
        self.assertEqual((self.draft / 'originals/file').read_text(), 'original')
        self.assertEqual(self.run_script('copy-originals.py', 'check', self.draft).returncode, 1)

    def test_backup_parent_symlink_is_rejected(self):
        source = self.repo / 'file'
        source.write_text('original')
        self.draft.mkdir(parents=True)
        (self.draft / 'originals').symlink_to(self.repo, target_is_directory=True)
        self.assertEqual(self.copy('file').returncode, 3)
        self.assertFalse((self.repo / 'manifest.tsv').exists())
        self.assertEqual(source.read_text(), 'original')

    def test_nested_parent_symlink_is_rejected(self):
        (self.repo / 'nested').mkdir()
        (self.repo / 'nested/file').write_text('original')
        (self.draft / 'originals').mkdir(parents=True)
        (self.draft / 'originals/nested').symlink_to(self.repo / 'nested', target_is_directory=True)
        self.assertEqual(self.copy('nested/file').returncode, 3)

    def test_manifest_symlink_is_rejected(self):
        (self.repo / 'file').write_text('original')
        victim = self.root / 'victim'
        victim.write_text('valuable')
        (self.draft / 'originals').mkdir(parents=True)
        (self.draft / 'originals/manifest.tsv').symlink_to(victim)
        self.assertEqual(self.copy('file').returncode, 3)
        self.assertEqual(victim.read_text(), 'valuable')

    def test_source_hardlink_is_not_a_backup(self):
        source = self.repo / 'file'
        source.write_text('original')
        (self.draft / 'originals').mkdir(parents=True)
        os.link(source, self.draft / 'originals/file')
        self.assertEqual(self.copy('file').returncode, 3)

    def test_source_parent_cannot_escape_repository(self):
        (self.root / 'outside').write_text('original')
        (self.repo / 'external').symlink_to(self.root, target_is_directory=True)
        self.assertEqual(self.copy('external/outside').returncode, 3)

    def test_parallel_updates_keep_every_manifest_entry(self):
        names = [f'{index}.txt' for index in range(12)]
        children = []
        for name in names:
            (self.repo / name).write_text('original')
            children.append(self.spawn(self.command('copy-originals.py', 'copy', self.draft, name, '--root', self.repo)))
        for child in children:
            out, err = child.communicate(timeout=30)
            self.assertEqual(child.returncode, 0, out + err)
        rows = (self.draft / 'originals/manifest.tsv').read_text().splitlines()[1:]
        self.assertEqual({row.split('\t')[0] for row in rows}, set(names))
        for name in names:
            (self.repo / name).write_text('changed')
        result = self.run_script('copy-originals.py', 'check', self.draft)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(sum(line.startswith('  改过 ') for line in result.stdout.splitlines()), len(names))

    def test_hash_names_are_neither_comments_nor_root_header(self):
        for name in ('#notes.txt', '# root'):
            (self.repo / name).write_text('original')
            self.assertEqual(self.copy(name).returncode, 0)
            (self.repo / name).write_text('changed')
        result = self.run_script('copy-originals.py', 'check', self.draft)
        self.assertEqual(result.returncode, 1)
        self.assertIn('改过 #notes.txt', result.stdout)
        self.assertIn('改过 # root', result.stdout)

    def test_unrepresentable_names_refused_before_copy(self):
        for name in ('tab\tname', 'new\nline'):
            with self.subTest(name=name):
                (self.repo / name).write_text('original')
                self.assertEqual(self.copy(name).returncode, 3)
                self.assertFalse(self.draft.exists())

    def test_corrupt_manifest_is_not_ignored(self):
        (self.repo / 'file').write_text('original')
        self.assertEqual(self.copy('file').returncode, 0)
        with (self.draft / 'originals/manifest.tsv').open('a') as handle:
            handle.write('broken-record\n')
        result = self.run_script('copy-originals.py', 'check', self.draft)
        self.assertEqual(result.returncode, 3)
        self.assertNotIn('Traceback', result.stderr)

    def test_reserved_manifest_path_cannot_overwrite_backup(self):
        for name in ('manifest.tsv', 'manifest.tsv/file'):
            with self.subTest(name=name):
                source = self.repo / name
                source.parent.mkdir(exist_ok=True)
                source.write_text('original')
                result = self.copy(name)
                self.assertEqual(result.returncode, 3, result.stdout + result.stderr)
                self.assertFalse(self.draft.exists())
                self.assertEqual(source.read_text(), 'original')
                source.unlink()


class CitationTests(ScriptCase):
    def test_plain_snapshot_and_multiple_roots(self):
        plain = self.root / 'plain'
        (plain / 'kb').mkdir(parents=True)
        (plain / 'kb/规则.md').write_text('真实引文\n')
        report = self.root / 'report.md'
        for text, roots, code in (
                ('`规则.md:9999`', [plain], 1),
                ('`规则.md:1`：「真实引文」', [plain], 0),
                ('`规则.md:9999`', [self.repo, plain], 1),
                ('`规则.md:9999`', [plain, plain], 1)):
            with self.subTest(text=text, roots=roots):
                report.write_text(text)
                args = [arg for root in roots for arg in ('--root', root)]
                result = self.run_script('cite-check.py', report, *args)
                self.assertEqual(result.returncode, code, result.stdout + result.stderr)
                self.assertNotIn('路径认不出', result.stdout)
        (plain / '规则.md').write_text('同名文件\n')
        result = self.run_script('cite-check.py', report, '--root', plain)
        self.assertIn('2 个同名文件', result.stdout)

    def test_untracked_snapshot_inside_repository(self):
        subprocess.run(['git', 'init', '-q', str(self.repo)], env=self.env, check=True)
        plain = self.repo / 'snapshot'
        plain.mkdir()
        (plain / 'file.md').write_text('only one line\n')
        report = self.root / 'report.md'
        report.write_text('`file.md:9999`')
        self.assertEqual(self.run_script('cite-check.py', report, '--root', plain).returncode, 1)


class CostTests(ScriptCase):
    def test_invalid_calendar_dates_have_usage_errors(self):
        for flag, value in (('--since', '2026-02-30'), ('--until', '2026-13-01'), ('--until', '9999-12-31')):
            with self.subTest(flag=flag, value=value):
                result = self.run_script('agent-cost.py', 'usage', flag, value)
                self.assertEqual(result.returncode, 2)
                self.assertIn('不是可用的日期', result.stdout)
                self.assertNotIn('Traceback', result.stderr)

    def test_explicit_jobs_does_not_read_default(self):
        module = load_script('agent-cost.py')
        with mock.patch.object(module.heavy_threads, 'default_heavy_threads', side_effect=AssertionError('default read')), \
                mock.patch.object(module, 'collect_outcome', return_value=[]), mock.patch('sys.stdout'):
            self.assertEqual(module.main(['outcome', '--root', str(self.repo), '--repo', str(self.repo), '--jobs', '1']), 77)


class WatchTests(ScriptCase):
    def test_log_deleted_after_fd_scan_does_not_crash(self):
        module = load_script('agent-watch.py')
        path = self.root / 'progress.log'
        child = self.spawn([sys.executable, '-c',
                            'import sys,time; f=open(sys.argv[1],"w"); f.write("started\\n"); '
                            'f.flush(); print("ready",flush=True); time.sleep(30)', str(path)])
        self.assertEqual(child.stdout.readline().strip(), 'ready')
        original = module.written_files
        def disappear(pid):
            files = original(pid)
            if str(path) in files:
                path.unlink()
            return files
        thresholds = SimpleNamespace(process_report_minutes=0, process_stale_minutes=10000, process_max_minutes=10000)
        with mock.patch.object(module, 'written_files', side_effect=disappear):
            lines, alerts = module.process_alerts(os.getpid(), set(), thresholds)
        self.assertFalse(path.exists(), 'The disappearing-file race must actually be exercised')
        self.assertTrue(any(str(child.pid) in line for line in lines))
        self.assertEqual(alerts, [])

    def test_progress_deleted_before_read_is_reported(self):
        module = load_script('agent-watch.py')
        with mock.patch.object(module, 'progress_file_of', return_value=str(self.root / 'missing')):
            note = module.progress_note(SimpleNamespace(), module.datetime.now(module.timezone.utc))
        self.assertIn('已消失或不可读', note)


class ReminderTests(ScriptCase):
    def test_concurrent_starts_and_stop(self):
        # Long timer: inspect live registrations, then stop our processes without waiting for a reminder.
        self.env['REMIND_PERIOD_UNIT_SECONDS'] = '30'
        children = [self.spawn(self.command('remind.sh', 'start', '--every', '1', '--message', str(index), '--name', 'same'))
                    for index in range(8)]
        deadline = time.monotonic() + 15
        while sum(child.poll() is None for child in children) > 1 and time.monotonic() < deadline:
            time.sleep(0.02)
        live = [child for child in children if child.poll() is None]
        self.assertEqual(len(live), 1)
        listing = self.run_script('remind.sh', 'list')
        self.assertIn(f'进程 {live[0].pid}', listing.stdout)
        result = self.run_script('remind.sh', 'stop', '--name', 'same')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for child in children:
            out, err = child.communicate(timeout=10)
            self.assertIn(child.returncode, (0, 143))
            self.assertNotIn('REMIND ', out)
            self.assertEqual(err, '')
        self.assertFalse((self.root / 'reminders/audit/same.conf').exists())
        self.assertFalse((self.root / 'reminders/audit/same.pid').exists())

    def test_invalid_session_cannot_escape_state_root(self):
        self.env['CLAUDE_CODE_SESSION_ID'] = '../outside'
        self.assertEqual(self.run_script('remind.sh', 'stop').returncode, 2)
        self.assertFalse((self.root / 'outside').exists())

    def test_failed_timer_does_not_report_success(self):
        self.env['REMIND_PERIOD_UNIT_SECONDS'] = '-1'
        result = self.run_script('remind.sh', 'start', '--every', '1', '--message', 'test')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('REMIND ', result.stdout)


if __name__ == '__main__':
    unittest.main()
