"""Regression tests for host cleanup boundaries; all deletions use temporary trees."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

sys.dont_write_bytecode = True
HOSTS = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('scratch_cleanup_review', HOSTS / 'clean-session-scratch.py')
cleanup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cleanup)


def load_host_module(name, filename):
    spec = importlib.util.spec_from_file_location(name, HOSTS / filename)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


stamp = load_host_module('stamp_review', 'source-stamp.py')
minimal = load_host_module('minimal_review', 'minimal-copy.py')
logs = load_host_module('logs_review', 'two_logs.py')
broadcast = load_host_module('broadcast_review', 'resource_release_broadcast.py')
environment_check = load_host_module('environment_review', 'test-environment-check.py')
dispatch = load_host_module('dispatch_review', 'host-dispatch.py')
resources = load_host_module('resources_review', 'host-resources.py')


class InputBoundaryTests(unittest.TestCase):
    def test_dispatch_refuses_invalid_wait_before_launch(self):
        for value in ('nan', 'inf', '-1', 'invalid'):
            with self.subTest(value=value), mock.patch.dict(os.environ, HOST_DISPATCH_WAIT_SECONDS=value), \
                 mock.patch.object(dispatch, 'run_request', return_value=0) as launch:
                self.assertEqual(dispatch.main(['--', 'true']), dispatch.EXIT_USAGE)
                launch.assert_not_called()

    def test_resource_probe_refuses_invalid_timing_before_collect(self):
        for variable in ('HOST_RESOURCES_SAMPLE_SECONDS', 'HOST_RESOURCES_HOST_TIMEOUT_SECONDS'):
            for value in ('nan', 'inf', '-1', 'invalid'):
                with self.subTest(variable=variable, value=value), mock.patch.dict(os.environ, {variable: value}), \
                     mock.patch.object(resources, 'collect', return_value={}) as collect, \
                     mock.patch.object(resources, 'render', return_value=''):
                    self.assertEqual(resources.main([], output=lambda line: None), resources.EXIT_USAGE)
                    collect.assert_not_called()

    def test_event_follow_refuses_nonfinite_since(self):
        for value in ('nan', 'inf', '-inf'):
            with self.subTest(value=value), mock.patch.object(broadcast.os, 'getppid', side_effect=[1, 2]):
                self.assertEqual(broadcast.command_follow(['--since', value]), 2)

    def test_ownership_break_switch_cannot_turn_dry_run_into_deletion(self):
        with tempfile.TemporaryDirectory(prefix='hosts-ownership-review-') as work:
            tree = Path(work) / 'mine'
            tree.mkdir()
            (tree / '.owner').write_text('session: review\ncreated: now\npurpose: test\n')
            environment = dict(os.environ, SINGLEFS_WORK_ROOT=work, WORK_DIRECTORY_OWNER_SESSION='review',
                               WORK_DIRECTORY_OWNERSHIP_BREAK='dry-run-deletes')
            environment.pop('WORK_DIRECTORY_OWNERSHIP_SELFTEST_MARK', None)
            result = subprocess.run(['bash', str(HOSTS / 'work-directory-ownership.sh'), 'sweep', '--dry-run'],
                                    env=environment, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            self.assertTrue(tree.exists())

    def test_environment_cleanup_refreshes_openers_for_each_deletion(self):
        environment = SimpleNamespace(command_runner=None, temporary_directories=['/unused'], recent_seconds=0,
                                      produced_after=0, previous_run_started=0)
        actions = [environment_check.CleanAction('fixture', [], lambda: True, SimpleNamespace(path=name))
                   for name in ('first', 'second')]
        def classify(path, scan, *args):
            return SimpleNamespace(verdict='in_use' if scan else 'residual', reason='fixture')
        with mock.patch.object(environment_check, 'scan_open_paths', side_effect=[{}, {'busy': True}]), \
             mock.patch.object(environment_check, 'classify_temporary_entry', side_effect=classify), \
             mock.patch.object(environment_check, 'delete_temporary_entry', return_value=True) as delete:
            self.assertEqual(environment_check.execute_clean(environment, actions), 0)
            self.assertEqual([call.args[0] for call in delete.call_args_list], ['first'])

    def test_minimal_copy_refuses_source_inside_destination(self):
        with tempfile.TemporaryDirectory(prefix='hosts-minimal-review-') as work:
            source = Path(work) / 'crates'
            (source / 'crates').mkdir(parents=True)
            (source / 'Cargo.toml').write_text('[workspace]\n')
            with mock.patch.object(minimal, 'copy_tree') as copy:
                self.assertEqual(minimal.execute([str(source), work]), minimal.EXIT_USAGE)
                copy.assert_not_called()

    def test_sync_refuses_overlapping_trees_before_rsync(self):
        with tempfile.TemporaryDirectory(prefix='hosts-sync-review-') as work:
            parent = Path(work)
            child = parent / 'child'
            child.mkdir()
            alias = parent / 'alias'
            alias.symlink_to(child, target_is_directory=True)
            for source, destination in ((parent, child), (child, parent), (child, child), (child, alias)):
                with self.subTest(source=source, destination=destination):
                    with mock.patch.object(stamp.subprocess, 'run') as run:
                        run.return_value.returncode = 0
                        with self.assertRaises(OSError):
                            stamp.sync(str(source), str(destination), str(parent / 'target'), (), lambda line: None)
                        run.assert_not_called()

    def test_log_drain_rejects_invalid_seconds_without_starting_command(self):
        with tempfile.TemporaryDirectory(prefix='hosts-logs-review-') as work:
            for value in ('nan', 'inf', '-1', 'not-a-number'):
                with self.subTest(value=value), mock.patch.dict(os.environ, TWO_LOGS_DRAIN_SECONDS=value):
                    with mock.patch.object(logs.subprocess, 'Popen') as start:
                        self.assertEqual(logs.run_wrapped(str(Path(work) / 'log'), ['true'], quiet=True), logs.EXIT_USAGE)
                        start.assert_not_called()

    def test_event_wait_rejects_nonfinite_timeout(self):
        for value in ('nan', 'inf', '-inf'):
            with self.subTest(value=value), mock.patch.object(broadcast, 'wait_for_events', return_value=[]) as wait:
                self.assertEqual(broadcast.command_wait(['--after', '0', '--timeout', value]), 2)
                wait.assert_not_called()


class CleanupRegressionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='hosts-review-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.work = self.root / 'work'
        self.work.mkdir()
        self.old = self.root / 'old'
        self.old.mkdir()
        self.repo = self.root / 'repo'
        self.repo.mkdir()
        self.git(self.repo, 'init', '-q')
        (self.repo / '.gitignore').write_text('private/\nsecret.env\n')
        self.git(self.repo, 'add', '.')
        self.git(self.repo, '-c', 'user.name=test', '-c', 'user.email=test@example.invalid', 'commit', '-qm', 'fixture')
        self.backup = self.root / 'backup'
        self.backup.mkdir()

    def git(self, path, *args):
        subprocess.run(['git', '-C', str(path), *args], check=True, capture_output=True)

    def clone(self, parent='mine', name='tree'):
        owner = self.work / parent
        owner.mkdir(exist_ok=True)
        (owner / '.owner').write_text('session: review-session\ncreated: now\npurpose: test\n')
        tree = owner / name
        self.git(self.repo, 'clone', '-q', str(self.repo), str(tree))
        return tree

    def run_cleanup(self, *extra):
        args = cleanup.parse(['--session', 'review-session', '--work-root', str(self.work),
                              '--tmp-root', str(self.old), '--repository', str(self.repo), *extra])
        with mock.patch.object(cleanup, 'claim_kept_directory', return_value=(str(self.backup), None)):
            return cleanup.run(args, say=lambda line: None)

    def test_keep_descendant_preserves_entire_candidate(self):
        tree = self.clone()
        secret = tree / 'secret.env'
        secret.write_text('keep this')
        self.assertEqual(self.run_cleanup('--delete', '--keep', str(secret)), 0)
        self.assertTrue(secret.exists(), 'deleting an ancestor must not bypass --keep')

    def test_keep_name_preserves_descendant(self):
        tree = self.clone()
        private = tree / 'private'
        private.mkdir()
        (private / 'secret.env').write_text('keep this')
        self.assertEqual(self.run_cleanup('--delete', '--keep', 'private'), 0)
        self.assertTrue(private.exists())

    def test_new_dirty_file_after_listing_is_preserved(self):
        tree = self.clone()
        original = cleanup.size_of
        def measure(path):
            result = original(path)
            (tree / 'new-work.txt').write_text('written after initial judgement')
            return result
        with mock.patch.object(cleanup, 'size_of', side_effect=measure):
            self.assertEqual(self.run_cleanup('--delete'), 0)
        self.assertTrue((tree / 'new-work.txt').exists())

    def test_same_named_trees_have_distinct_backups(self):
        for parent in ('first', 'second'):
            (self.clone(parent) / 'secret.env').write_text(parent)
        self.assertEqual(self.run_cleanup('--delete'), 0)
        self.assertEqual(sorted(p.read_text() for p in self.backup.rglob('secret.env')), ['first', 'second'])

    def test_invalid_cache_tag_does_not_discard_private_files(self):
        tree = self.clone()
        private = tree / 'private'
        private.mkdir()
        (private / 'CACHEDIR.TAG').write_text('not a cache signature')
        (private / 'valuable.txt').write_text('preserve')
        self.assertEqual(self.run_cleanup('--delete'), 0)
        self.assertEqual([p.read_text() for p in self.backup.rglob('valuable.txt')], ['preserve'])

    def test_break_switch_refused_outside_selftest(self):
        with mock.patch.object(cleanup, 'BROKEN', 'list-deletes'):
            tree = self.clone()
            self.assertEqual(self.run_cleanup(), 2)
            self.assertTrue(tree.exists())

    def test_session_path_traversal_refused(self):
        for session in ('..', '.', '/tmp', '../other', 'one/two'):
            with self.subTest(session=session):
                self.assertEqual(self.run_cleanup('--session', session), 2)

    def test_symlinked_legacy_session_not_scanned(self):
        project = self.old / 'project'
        project.mkdir()
        outside = self.root / 'other-session'
        outside.mkdir()
        (outside / 'copy').mkdir()
        (outside / 'copy' / 'Cargo.toml').write_text('[workspace]')
        (outside / 'copy' / 'crates').mkdir()
        (project / 'review-session').symlink_to(outside, target_is_directory=True)
        found, _ = cleanup.candidates('review-session', str(self.work), str(self.old))
        self.assertEqual(found, [])

    def test_git_tree_with_cache_tag_still_checks_dirty_files(self):
        tree = self.clone()
        (tree / 'CACHEDIR.TAG').write_bytes(cleanup.SCRATCH_FINDER.CACHEDIR_TAG_SIGNATURE)
        (tree / 'uncommitted.txt').write_text('valuable')
        self.assertEqual(self.run_cleanup('--delete'), 0)
        self.assertTrue((tree / 'uncommitted.txt').exists())


if __name__ == '__main__':
    unittest.main()
