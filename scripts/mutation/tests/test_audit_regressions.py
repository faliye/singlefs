#!/usr/bin/env python3
# admission: always 每一趟都现判此刻的 lib_mutation_units.py、mutation-prune.py、broad-mutation.py、crates-mutation-rows.py 与 prove-red.sh，上一次的结论不替这一次作保
# run-condition: none 只在临时目录里造小仓与假 cargo（cargo、rustc 一律换成假的或打桩），判定库那几条只连临时 prune-server
"""变异那几份脚本的回归单测（键、判定库、证红）。起法（单独一个 --selftest，等于 python3 -m unittest discover -s scripts/mutation/tests -v）：
  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <prune-server 二进制> -- python3 scripts/mutation/tests/test_audit_regressions.py --selftest
门禁 code-tooling 的 research-script-selftests 格照这一句跑（.claude/hooks/selftest-registry.tsv 登记成 light）。
判定库那几条（Library 的往返、两个写者并单元、并完刷新缓存）只对临时 prune-server 跑（判定库只在服务里，scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service）：
不在临时服务底下，--selftest 开跑前调客户端 selftest_service_problem、退 2、一条都不跑；直接 unittest discover 起的，那几条照 SERVICE_REMEDY 判失败（不跳过），别的照跑。
"""
import importlib.util
import contextlib
import io
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest import mock

DIRECTORY = Path(__file__).resolve().parents[1]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


units = load('audit_mutation_units', DIRECTORY / 'lib_mutation_units.py')
prune = load('audit_mutation_prune', DIRECTORY / 'mutation-prune.py')
broad = load('audit_broad_mutation', DIRECTORY / 'broad-mutation.py')
rows = load('audit_crates_mutation_rows', DIRECTORY / 'crates-mutation-rows.py')


SERVICE_REMEDY = ('在临时 prune-server 底下起：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service '
                  '--server-binary <prune-server 二进制> -- python3 scripts/mutation/tests/test_audit_regressions.py --selftest')


def service_library_directory(test, directory):
    """判定库那几条的库目录（只当服务里的名字与命名空间）：这一趟连的不是临时服务就让这一条失败、给出起法，不跳过、不写真服务。"""
    client = units.Library.client()
    problem = client.selftest_service_problem() if client is not None else '这棵树里没有 scripts/prune-server/prune_server_client.py'
    if problem:
        test.fail(f'{problem}；{SERVICE_REMEDY}')
    return directory


def load_prove():
    source = (DIRECTORY / 'prove-red.sh').read_text().split("3<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0]
    source = source.rsplit('sys.exit(main())', 1)[0]
    namespace = {'__name__': 'audit_prove'}
    with mock.patch.object(sys, 'argv', ['prove-red.sh', str(DIRECTORY)]):
        exec(compile(source, str(DIRECTORY / 'prove-red.sh'), 'exec'), namespace)
    return namespace


class CacheTests(unittest.TestCase):
    def test_nonfinite_and_negative_arguments_rejected(self):
        for flag, value in (('--suspect-test-seconds', 'nan'), ('--suspect-test-seconds', '-1'),
                            ('--suspect-test-seconds', 'inf'), ('--stop-grace', '-1')):
            with self.subTest(flag=flag, value=value), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    broad.parse_arguments([flag, value])
                self.assertEqual(error.exception.code, 2)

    def test_zero_suspect_seconds_and_stop_grace_remain_supported(self):
        for arguments in (['--suspect-test-seconds', '0'],
                          ['--stop-grace', '0']):
            with self.subTest(arguments=arguments):
                broad.parse_arguments(arguments)

    def test_skip_changes_key(self):
        base = units.parse_cargo_arguments('--lib --')
        skipped = units.parse_cargo_arguments('--lib -- --skip target')
        self.assertNotEqual(base.switches, skipped.switches)
        self.assertEqual(skipped.switches, units.parse_cargo_arguments('--lib -- --skip=target').switches)

    def test_changed_filter_cannot_reuse_caught_row_or_baseline(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / 'src').mkdir()
            (root / 'Cargo.toml').write_text('[package]\nname="tiny"\nversion="0.1.0"\nedition="2021"\n')
            (root / 'src/lib.rs').write_text('pub fn value() -> u32 { 42 }\n#[test]\nfn catches() { assert_eq!(value(), 42); }\n')
            with mock.patch.object(rows, 'judging_units', return_value=('judging', [])), \
                    mock.patch.object(rows, 'key_toolchain_lines', return_value=[]):
                context = rows.KeyContext(d, {})
            def components(arguments):
                fields = ['mutation', 'src/lib.rs', '{ 42 }', '{ 43 }', arguments, 'catches']
                return context.components(rows.Row(1, *fields, fields))
            valid = components('-p tiny --lib -- catches')
            self.assertIsNotNone(valid.row_key)
            for arguments in ('-p tiny --lib -- nonexistent', '-p tiny --lib nonexistent'):
                with self.subTest(arguments=arguments):
                    excluded = components(arguments)
                    self.assertNotEqual(valid.row_key, excluded.row_key)
                    self.assertNotEqual(valid.baseline_key, excluded.baseline_key)
            self.assertEqual(valid.row_key, components('-p tiny --lib catches').row_key)

    def test_cargo_subcommand_is_not_a_filter(self):
        self.assertEqual(units.parse_cargo_arguments('test --release --bin demo').filters, [])
        self.assertEqual(units.parse_cargo_arguments('check --profile test --bin demo').filters, [])

    def test_cargo_config_precedence(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            config = root / '.cargo'
            config.mkdir()
            (config / 'config').write_text('[build]\nrustflags=["--cfg", "legacy"]\n')
            (config / 'config.toml').write_text('invalid [')
            env = {'HOME': d, 'CARGO_HOME': str(config)}
            for discover in (units.cargo_configuration_files, prune.cargo_config_files):
                self.assertEqual(discover(d, env), [str(config / 'config')])

    def test_single_config_and_relative_cargo_home(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for directory in (root / '.cargo', root / 'cargo-home'):
                directory.mkdir()
                (directory / 'config.toml').write_text('[build]\nrustflags=[]\n')
            env = {'HOME': d, 'CARGO_HOME': 'cargo-home'}
            expected = [str(root / '.cargo/config.toml'), str(root / 'cargo-home/config.toml')]
            self.assertEqual(units.cargo_configuration_files(d, env), expected)
            self.assertEqual(prune.cargo_config_files(d, env), expected)

    def test_library_roundtrips_units_compositions_and_records_over_service(self):
        with tempfile.TemporaryDirectory() as d:
            lib = units.Library(service_library_directory(self, d))
            reference = units.UnitReference('fn', 'src/lib.rs', 'answer', '1-3', 'fingerprint')
            lib.put_composition('active', [reference])
            lib.flush_units()
            lib.write('records', 'record', {'line': 'a\\b\n中文', 'verdict': 'caught'})
            fresh = units.Library(d)
            self.assertEqual(fresh.unit(units.unit_id(reference)), reference)
            self.assertEqual(fresh.composition('active')['units'], [units.unit_id(reference)])
            self.assertEqual(fresh.read('records', 'record'), {'line': 'a\\b\n中文', 'verdict': 'caught'})

    def test_unreadable_config_refuses_key(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / '.cargo').mkdir()
            (root / '.cargo/config.toml').write_text('invalid [')
            result = subprocess.CompletedProcess([], 0, stdout='fake version\n')
            with mock.patch.object(units.subprocess, 'run', return_value=result):
                with self.assertRaises(units.MutationUnitsError):
                    units.toolchain_lines(d, {'HOME': d})

    def test_concurrent_unit_merges_keep_both_writers(self):
        with tempfile.TemporaryDirectory() as d:
            first, second = units.Library(service_library_directory(self, d)), units.Library(d)
            one = units.UnitReference('fn', 'a.rs', 'one', '1', 'hash1')
            two = units.UnitReference('fn', 'a.rs', 'two', '2', 'hash2')
            first.put_composition('first', [one])
            second.put_composition('second', [two])
            start = threading.Barrier(2)
            errors = []
            def flush(lib):
                try:
                    start.wait(5)
                    lib.flush_units()
                except BaseException as error:
                    errors.append(error)
            threads = [threading.Thread(target=flush, args=(lib,)) for lib in (first, second)]
            for thread in threads:
                thread.start()
            for thread in threads:
                thread.join(10)
            self.assertFalse(any(thread.is_alive() for thread in threads))
            self.assertEqual(errors, [])
            fresh = units.Library(d)
            self.assertEqual(fresh.unit(units.unit_id(one)), one)
            self.assertEqual(fresh.unit(units.unit_id(two)), two)

    def test_idempotent_merge_refreshes_cached_unit_lookup(self):
        with tempfile.TemporaryDirectory() as d:
            first, second = units.Library(service_library_directory(self, d)), units.Library(d)
            reference = units.UnitReference('fn', 'a.rs', 'answer', '1-3', 'hash')
            identifier = units.unit_id(reference)
            self.assertIsNone(first.unit(identifier))
            first.put_composition('composition', [reference])
            second.merge_units(identifier[:2], {identifier: list(reference)})
            first.flush_units()
            self.assertEqual(first.unit(identifier), reference)

    def test_wrapper_digest_uses_supplied_path(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            executable = root / 'bin' / 'audit-wrapper'
            executable.parent.mkdir()
            executable.write_text('#!/bin/sh\nexit 0\n')
            executable.chmod(0o755)
            env = {'HOME': d, 'PATH': str(executable.parent), 'RUSTC_WRAPPER': executable.name}
            before = prune.build_environment_lines(d, env)
            executable.write_text('#!/bin/sh\nexit 1\n')
            self.assertNotEqual(before, prune.build_environment_lines(d, env))

    def test_wrapper_paths_with_spaces_and_relative_path_entries(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            executable = root / 'bin dir' / 'wrapper file'
            executable.parent.mkdir()
            executable.write_text('#!/bin/sh\nexit 0\n')
            executable.chmod(0o755)
            env = {'PATH': 'bin dir'}
            digest = prune.program_digest(str(executable), d, env)
            self.assertNotEqual(digest, '不在')
            self.assertEqual(prune.program_digest('bin dir/wrapper file', d, env), digest)
            self.assertEqual(prune.program_digest('wrapper file', d, env), digest)

    def test_ancestor_config_wrapper_uses_working_directory_for_path(self):
        with tempfile.TemporaryDirectory() as d:
            parent = Path(d)
            root = parent / 'project'
            (parent / '.cargo').mkdir()
            (root / 'bin').mkdir(parents=True)
            executable = root / 'bin/wrapper'
            executable.write_text('#!/bin/sh\nexit 0\n')
            executable.chmod(0o755)
            config = parent / '.cargo/config.toml'
            config.write_text('[build]\nrustc-wrapper="wrapper"\n')
            env = {'HOME': d, 'PATH': 'bin'}
            before = prune.build_environment_lines(str(root), env)
            self.assertNotIn('program build.rustc-wrapper 不在', before)
            executable.write_text('#!/bin/sh\nexit 1\n')
            self.assertNotEqual(before, prune.build_environment_lines(str(root), env))
            # A value containing a slash is relative to the config, not PATH/cwd.
            config.write_text('[build]\nrustc-wrapper="project/bin/wrapper"\n')
            expected = prune.program_digest(str(executable), str(root), env)
            self.assertIn('program build.rustc-wrapper ' + expected, prune.build_environment_lines(str(root), env))


class ProveTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.prove = load_prove()

    def test_empty_baseline_does_not_prove_mutant(self):
        self.assert_baseline_refused('running 0 tests\ntest result: ok. 0 passed; 0 failed\n')

    def test_not_run_baseline_does_not_prove_mutant(self):
        self.assert_baseline_refused('test tests::{expected} ... ok\nNOT_RUN_NO_GPU_ADAPTER test={expected} reason=no-adapter\n')

    def assert_baseline_refused(self, baseline):
        with tempfile.TemporaryDirectory() as d:
            copy = self.prove['selftest_copy'](d)
            rows, _ = self.prove['table_rows'](copy)
            row = next(r for r in rows.values() if '-p demo --lib' in r.arguments)
            calls = []
            def cargo(*args):
                calls.append(args)
                if len(calls) == 1:
                    return 0, baseline.format(expected=row.expected)
                return 101, f'test tests::{row.expected} ... FAILED\n'
            entry = subprocess.CompletedProcess([], 0, stdout='', stderr='')
            with mock.patch.dict(self.prove, run_cargo=cargo, rebuild_problem=lambda *a: None), \
                    mock.patch.object(subprocess, 'run', return_value=entry):
                code, lines = self.prove['prove_holding_the_lock'](copy, 'demo', [row.name], 'cargo', {})
            self.assertEqual(code, 2, lines)
            self.assertEqual(len(calls), 1)
            self.assertIn('--show-output', calls[0][1])

    def test_source_bytes_restored_after_exception(self):
        with tempfile.TemporaryDirectory() as d:
            copy = self.prove['selftest_copy'](d)
            rows, _ = self.prove['table_rows'](copy)
            row = next(r for r in rows.values() if '-p demo --lib' in r.arguments)
            source = Path(copy) / row.path
            data = source.read_bytes().replace(b'\n', b'\r\n')
            source.write_bytes(data)
            calls = []
            def cargo(*args):
                calls.append(args)
                if len(calls) == 1:
                    return 0, f'test tests::{row.expected} ... ok\n'
                raise RuntimeError('interrupted cargo')
            entry = subprocess.CompletedProcess([], 0, stdout='', stderr='')
            with mock.patch.dict(self.prove, run_cargo=cargo), mock.patch.object(subprocess, 'run', return_value=entry):
                with self.assertRaisesRegex(RuntimeError, 'interrupted cargo'):
                    self.prove['prove_holding_the_lock'](copy, 'demo', [row.name], 'cargo', {})
            self.assertEqual(source.read_bytes(), data)

    def test_crlf_multiline_anchor_still_matches(self):
        with tempfile.TemporaryDirectory() as d:
            copy = self.prove['selftest_copy'](d)
            source = Path(copy) / 'crates/demo/src/lib.rs'
            original = b'fn answer() -> u32 {\r\n    42\r\n}\r\n'
            source.write_bytes(original)
            (Path(copy) / 'crates/mutations.tsv').write_text(
                'multiline\tcrates/demo/src/lib.rs\t42\\n}\t43\\n}\t-p demo --lib\tanswer_is_42\n')
            calls = []
            def cargo(*args):
                calls.append(source.read_bytes())
                return (0, 'test tests::answer_is_42 ... ok\n') if len(calls) == 1 else (101, 'test tests::answer_is_42 ... FAILED\n')
            entry = subprocess.CompletedProcess([], 0, stdout='', stderr='')
            with mock.patch.dict(self.prove, run_cargo=cargo, rebuild_problem=lambda *a: None), \
                    mock.patch.object(subprocess, 'run', return_value=entry):
                code, lines = self.prove['prove_holding_the_lock'](copy, 'demo', ['multiline'], 'cargo', {})
            self.assertEqual(code, 0, lines)
            self.assertEqual(len(calls), 2)
            self.assertIn(b'43', calls[1])
            self.assertEqual(source.read_bytes(), original)

    def test_source_outside_copy_is_refused_before_running(self):
        for form in ('absolute', 'parent', 'symlink', 'hardlink'):
            with self.subTest(form=form), tempfile.TemporaryDirectory() as d:
                root = Path(d)
                copy = root / 'copy'
                (copy / 'crates').mkdir(parents=True)
                outside = root / 'outside.rs'
                outside.write_text('fn answer() -> u32 { 42 }\n')
                path = str(outside) if form == 'absolute' else '../outside.rs'
                if form in ('symlink', 'hardlink'):
                    path = 'crates/source.rs'
                    if form == 'symlink':
                        (copy / path).symlink_to(outside)
                    else:
                        os.link(outside, copy / path)
                (copy / 'crates/mutations.tsv').write_text(
                    f'escape\t{path}\t42\t43\t-p demo --lib\tanswer_is_42\n')
                calls = []
                def cargo(*args):
                    calls.append(outside.read_text())
                    return 0, 'test tests::answer_is_42 ... ok\ntest result: ok. 1 passed; 0 failed\n'
                fake_entry = subprocess.CompletedProcess([], 0, stdout='', stderr='')
                with mock.patch.dict(self.prove, run_cargo=cargo, rebuild_problem=lambda *a: None), \
                        mock.patch.object(subprocess, 'run', return_value=fake_entry):
                    code, lines = self.prove['prove_holding_the_lock'](str(copy), 'demo', ['escape'], 'cargo', {})
                self.assertEqual(code, 2, lines)
                self.assertEqual(calls, [])
                self.assertEqual(outside.read_text(), 'fn answer() -> u32 { 42 }\n')

    def test_internal_symlink_is_still_allowed(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            source = root / 'source.rs'
            source.write_text('fn answer() {}')
            (root / 'alias.rs').symlink_to(source)
            self.assertIsNone(self.prove['source_path_problem'](d, 'alias.rs'))


def selftest():
    """--selftest：这一趟连的不是临时服务就退 2、一条都不跑（不写真服务）；是就照 unittest discover 跑这一份，全过退 0、有一条没过退 1。"""
    client = units.Library.client()
    problem = client.selftest_service_problem() if client is not None else '这棵树里没有 scripts/prune-server/prune_server_client.py'
    if problem:
        print(f'  ✗ test_audit_regressions.py 自证没跑：{problem}')
        print(f'     → 怎么办：{SERVICE_REMEDY}')
        return 2
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__]))
    if not result.wasSuccessful():
        print(f'  ✗ test_audit_regressions.py：{result.testsRun} 条里 {len(result.failures)} 条失败、{len(result.errors)} 条出错')
        print('     → 怎么办：照上面 FAIL / ERROR 那几条的回溯修被测的脚本（或测的东西已经改名、改参数的，照此刻的写法改用例）')
        return 1
    print(f'  ✓ test_audit_regressions.py 自证 {result.testsRun} 条都过')
    return 0


if __name__ == '__main__':
    if sys.argv[1:] == ['--selftest']:
        sys.path.insert(0, str(DIRECTORY.parents[1] / '.claude' / 'scripts'))
        from project_preflight import preflight
        preflight(__file__)
        sys.exit(selftest())
    unittest.main()
