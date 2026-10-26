#!/usr/bin/env python3
"""Device script regressions: temporary files and fake privileged commands only.

Run: python3 scripts/devices/tests/test_devices.py
"""
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

HERE = Path(__file__).resolve().parent.parent


def source(name):
    return (HERE / name).read_text()


def section(name, start, end):
    return source(name).split(start, 1)[1].split(end, 1)[0]


class DeviceTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix="singlefs-device-tests.")
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.env = dict(os.environ, TMPDIR=str(self.root))
        for key in list(self.env):
            if key.startswith(('VM_', 'SNAPSHOT_CHECK_', 'E152_', 'SINGLEFS_')):
                self.env.pop(key)

    def bash(self, code, args=()):
        return subprocess.run(['bash', '-uo', 'pipefail', '-c', code, '_', *map(str, args)],
                              cwd=self.root, env=self.env, capture_output=True, text=True, timeout=20)

    def fixture(self, names):
        devices = self.root / 'scripts/devices'
        devices.mkdir(parents=True, exist_ok=True)
        for name in [*names, 'dm-owned.sh']:
            shutil.copy2(HERE / name, devices / name)
        (self.root / '.env').write_text('SUDO_PASS_A=fake-test-password\n')
        mock = self.root / 'mock'
        mock.mkdir(exist_ok=True)
        sudo = mock / 'sudo'
        sudo.write_text('''#!/bin/bash
read -r ignored
shift 3
printf '%s\\n' "$*" >> "$MOCK_LOG"
case "$1:${2:-}" in
 true:) exit 0;;
 losetup:--find) echo /dev/fake-test-loop; exit 0;;
 dmsetup:create|mdadm:--create) exit 1;;
 dd:*|losetup:-d|dmsetup:remove|mdadm:--stop) exit 0;;
 *) exit 1;;
esac
''')
        sudo.chmod(0o755)
        self.env.update(PATH=str(mock) + ':' + os.environ['PATH'], MOCK_LOG=str(self.root / 'calls'))
        return devices

    def test_creation_failure_never_removes_unowned_devices(self):
        names = ['e129-tear-injector.sh', 'e129-thin-neighbour.sh',
                 'e129-thin-rmw.sh', 'stripe-map-probe.sh', 'e34-iomin-probe.sh']
        devices = self.fixture(names)
        for name in names:
            with self.subTest(script=name):
                (self.root / 'calls').write_text('')
                result = self.bash('bash "$1"', [devices / name])
                calls = (self.root / 'calls').read_text()
                self.assertNotIn('dmsetup remove', calls)
                self.assertNotIn('mdadm --stop', calls)
                if name != 'stripe-map-probe.sh':
                    self.assertNotEqual(result.returncode, 0, result.stdout)
                if name == 'e129-thin-neighbour.sh':
                    self.assertNotIn('name=verdict', result.stdout)

    def test_dm_ownership_and_remove_failure_retry(self):
        result = self.bash('''
source "$1"
S() {
  printf '%s\\n' "$*"
  [[ "$*" != 'dmsetup create foreign' ]] || return 1
  [[ "$*" != 'dmsetup remove owned' || "$fail_remove" == 0 ]]
}
fail_remove=1
create_dm foreign && exit 90
remove_dm foreign
create_dm owned || exit 91
remove_dm owned && exit 92
fail_remove=0
remove_dm owned || exit 93
remove_dm owned
''', [HERE / 'dm-owned.sh'])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('dmsetup remove foreign', result.stdout)
        self.assertEqual(result.stdout.count('dmsetup remove owned'), 2)

    def test_e152_reaches_staging_from_repository_root(self):
        devices = self.fixture(['e152-run.sh', 'vm-bench.sh'])
        (self.root / 'scripts/experiment-apparatus').mkdir()
        hosts = self.root / 'scripts/hosts'
        hosts.mkdir()
        (hosts / 'capped.sh').write_text('exit 0\n')
        (devices / 'vm-kernel.sh').write_text('echo /fake/kernel\n')
        (devices / 'e152-stage-root.sh').write_text('echo reached-staging >&2; exit 17\n')
        result = self.bash('bash "$1" "$2"', [devices / 'e152-run.sh', self.root / 'out'])
        self.assertIn('reached-staging', result.stderr)
        self.assertNotIn('cannot stat', result.stderr)

    def test_snapshot_rejects_malformed_record(self):
        directory = self.root / 'research/prompts/r-snapshot'
        directory.mkdir(parents=True)
        (self.root / 'good').write_bytes(b'ok')
        manifest = directory / 'files-sha256.txt'
        good = hashlib.sha256(b'ok').hexdigest() + '  good\n'
        # Use the actual checking function without preflight's repository dependencies.
        code = 'die() { echo "$1"; exit "${3:-2}"; }\ncheck_snapshot() {' + section(
            'snapshot-check.sh', 'check_snapshot() {', '\nselftest() {')
        code += '\ncheck_snapshot "$1" r\n'
        for contents, expected in [(good, 0), (good + 'broken critical\n', 2), ('broken\n', 2)]:
            manifest.write_text(contents)
            self.assertEqual(self.bash(code, [self.root]).returncode, expected)

    def test_e34_read_error_and_short_read_are_not_measurements(self):
        helper = 'get_mark() {' + section('e34-iomin-probe.sh', 'get_mark() {', '# 标记在哪条腿')
        consumer = section('e34-iomin-probe.sh', '  local got intact\n', '\n}\n\nfor round')
        for size, rc, expected in [(512, 0, 0), (0, 0, 4), (256, 0, 4), (512, 1, 4)]:
            with self.subTest(size=size, rc=rc):
                (self.root / 'payload').write_bytes(b'x' * size)
                code = '''W="$1"; MD_DEV=/fake; v_off=0; vmark=x
scen=test; round=1; legA=0; legV=1; do_tear=1; fail_mark=V
S() { cp "$W/payload" "$W/read.bin"; return "$2_rc"; }
'''.replace('"$2_rc"', str(rc))
                code += 'emit() { echo "$*"; }\n' + helper + consumer
                result = self.bash(code, [self.root])
                self.assertEqual(result.returncode, expected, result.stderr)
                self.assertEqual('name=scenario' in result.stdout, expected == 0)

    def test_md_stop_requires_ownership(self):
        helper = 'stop_array() {' + section('e34-iomin-probe.sh', 'stop_array() {', '\nMD_DEV=')
        result = self.bash('S() { echo "$*"; }; MD_DEV=/fake; MD_OWNED=0\n' + helper +
                           '\nstop_array; MD_OWNED=1; stop_array; stop_array')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.count('mdadm --stop'), 1)

    def test_vm_arguments_roundtrip_without_execution(self):
        # Exercise the actual serializer, and parse with POSIX sh as the guest does.
        serializer = section(
            'vm-bench.sh', '  local arg\n', '  # N 块盘：')
        args = ['a b', '', '*.img', "a'b", 'line\nnext', '$(touch SHOULD_NOT_EXIST)', '`touch ALSO_NOT`', '\\']
        code = 'ird="$1"; shift\n' + serializer + '''
/bin/sh -c 'root=$1; set -- /dev/vda; . "$root/args"; printf "%s\\0" "$@"' _ "$ird" > "$ird/actual"
'''
        result = self.bash(code, [self.root, *args])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / 'actual').read_bytes(), b'\0'.join(x.encode() for x in ['/dev/vda', *args]) + b'\0')
        self.assertFalse((self.root / 'SHOULD_NOT_EXIST').exists())
        self.assertFalse((self.root / 'ALSO_NOT').exists())

    def test_geometry_complete_and_incomplete_logs(self):
        z64 = 'lbs=4096 pbs=4096 zoned=host-managed chunk_sectors=131072 nr_zones=8 bytes=536870912'
        z128 = z64.replace('131072', '262144').replace('nr_zones=8', 'nr_zones=4')
        plain = 'lbs=512 pbs=512 zoned=none chunk_sectors=0 nr_zones=NA bytes=536870912'
        tail = 'KEEP="' + source('vm-geom.sh').split('KEEP="', 1)[1]
        cases = [(1, [z64]*3, 0), (0, [z64,z128,plain], 0),
                 (1, [z64], 1), (0, [z64,plain], 1), (0, [z64]*3, 1),
                 (1, [z64,z64,z64.replace('lbs=4096','lbs=NA')], 1)]
        for uniform, geometries, expected in cases:
            rows = [f'GEOM dev=nvme0n{i+1} {g}' for i,g in enumerate(geometries)]
            (self.root / 'console').write_text('\n'.join(rows) + f'\nGEOM done emitted={len(rows)}\n')
            code = 'LOGF="$1/console"; UNIFORM="$2"\nok() { :; }; say() { :; }; die() { echo "$1"; exit 1; }\n' + tail
            result = self.bash(code, [self.root, uniform])
            self.assertEqual(result.returncode, expected, result.stdout + result.stderr)

    def test_neighbour_comparison_detects_short_prefix(self):
        expression = next(line for line in source('e129-thin-neighbour.sh').splitlines() if line.startswith('print(next('))
        expression = expression.split('")"', 1)[0]  # remove shell command-substitution terminator
        for length in [0, 256, 512, 513]:
            result = subprocess.run(['python3', '-c', f'got=bytes({length}); want=bytes(512)\n' + expression],
                                    capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip() == '-1', length == 512)

    def test_neighbour_requires_all_cells(self):
        guard = section('e129-thin-neighbour.sh', '# 每轮两格必须齐全；没有测量不等于没有失败。\n', '# ── 判决。两条，')
        for rounds in [0, 4, 5]:
            rows = [f'name=neighbour round={r} inject={i} verdict=intact' for r in range(rounds) for i in [0,1]]
            (self.root / 'spool').write_text('\n'.join(rows) + '\n')
            result = self.bash('SPOOL="$1/spool"; ROUNDS=5; emit() { echo "$*"; }\n' + guard, [self.root])
            self.assertEqual(result.returncode, 0 if rounds == 5 else 5)


    def test_e125_existing_device_is_never_registered(self):
        helper = 'mk() {' + section('e125-zoned-wp-probe.sh', 'mk() {', '# blkzone report')
        cleanup = 'cleanup() {' + section('e125-zoned-wp-probe.sh', 'cleanup() {', 'trap cleanup EXIT')
        code = '''CFG="$1/cfg"; WORK="$1/work"; ASKPASS="$1/askpass"; LOG="$1/log"; DEVS=()
mkdir -p "$CFG/existing" "$WORK"
S() { echo "$*" >> "$LOG"; case "$1" in mkdir) command "$@";; *) return 1;; esac; }
''' + helper + cleanup + '''
mk existing blocksize=4096 && exit 90
[[ ${#DEVS[@]} == 0 ]] || exit 91
cleanup
'''
        result = self.bash(code, [self.root])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / 'log').read_text().splitlines(), ['mkdir ' + str(self.root / 'cfg/existing')])
        self.assertTrue((self.root / 'cfg/existing').is_dir())
        # A newly created configfs directory is ours even if attribute setup fails.
        result = self.bash('CFG="$1/cfg"; DEVS=(); S() { if [[ "$1" == mkdir ]]; then command "$@"; else return 1; fi; }\n' + helper +
                           '\nmk fresh blocksize=4096 && exit 90; [[ "${DEVS[*]}" == fresh ]]', [self.root])
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_thin_cleanup_preserves_resources_until_success(self):
        for name in ['e129-thin-rmw.sh', 'e129-thin-neighbour.sh']:
            helper = 'teardown_pool() {' + section(name, 'teardown_pool() {', 'trap cleanup EXIT')
            for failure in ['mapping', 'meta', 'data']:
                with self.subTest(script=name, failure=failure):
                    code = '''W="$1/work"; mkdir -p "$W"; touch "$W/meta.img" "$W/data.img"
SN=s; TH=t; POOL=p; DATA=d; LOOP_M=/dev/meta; LOOP_D=/dev/data; failure="$2"
remove_dm() { [[ "$failure" != mapping ]]; }
S() { [[ "$*" != "losetup -d /dev/$failure" ]]; }
''' + helper + '''
teardown_pool && exit 90
[[ -f "$W/meta.img" && -f "$W/data.img" && "$LOOP_D" == /dev/data ]] || exit 91
if [[ "$failure" != data ]]; then [[ "$LOOP_M" == /dev/meta ]] || exit 92; fi
failure=none
teardown_pool || exit 93
[[ -z "$LOOP_M" && -z "$LOOP_D" && ! -e "$W/meta.img" && ! -e "$W/data.img" ]]
'''
                    result = self.bash(code, [self.root, failure])
                    self.assertEqual(result.returncode, 0, result.stderr)
            result = self.bash('W="$1/retained"; mkdir -p "$W"; touch "$W/data.img"; SN=s; TH=t; POOL=p; DATA=d; LOOP_M=m; LOOP_D=d\n' + helper +
                               '\nremove_dm() { return 1; }; trap cleanup EXIT; exit 0', [self.root])
            self.assertNotEqual(result.returncode, 0)
            self.assertTrue((self.root / 'retained/data.img').exists())

    def test_rmw_requires_successful_write(self):
        body = 'one_cell() {' + source('e129-thin-rmw.sh').split('one_cell() {', 1)[1]
        for fail_write in [0, 1]:
            code = '''W="$1"; TH=thin; POOL=pool; SN=snap; LOOP_D=/fake
BLK_BYTES=65536; VOL_SECT=16384; ROUNDS=1; N=0; SPOOL="$W/spool"; : > "$SPOOL"; fail_write="$2"
emit() { echo "$*"; echo "$*" >> "$SPOOL"; N=$((N+1)); }
setup_pool() { echo 0 > "$W/counter"; }; teardown_pool() { :; }; create_dm() { :; }
cat() { case "$1" in */minimum_io_size) echo 65536;; */physical_block_size) echo 512;; *) command cat "$@";; esac; }
written_sectors() { command cat "$W/counter"; }
S() {
 if [[ "$1" == dd && " $* " == *' bs=512 '* ]]; then
  if [[ "$shared" == 0 && "$fail_write" == 1 ]]; then return 1; fi
  if [[ "$shared" == 1 ]]; then echo 129 > "$W/counter"; else echo 1 > "$W/counter"; fi
 fi
 return 0
}
''' + body
            result = self.bash(code, [self.root, fail_write])
            self.assertEqual(result.returncode, 4 if fail_write else 0, result.stderr)
            self.assertEqual('name=verdict' in result.stdout, not fail_write)

    def test_e152_publish_failure_is_not_success(self):
        tail = 'cat "$WORK/summary.txt" >>' + source('e152-run.sh').split('cat "$WORK/summary.txt" >>', 1)[1]
        for failure in ['none', 'append', 'rename']:
            code = '''WORK="$1"; PARTIAL="$1/report.partial"; OUTPUT="$1/report"
echo summary > "$WORK/summary.txt"; echo data > "$PARTIAL"
fail() { echo "$1" >&2; exit 1; }
'''
            if failure == 'append':
                code += 'cat() { return 1; }\n'
            if failure == 'rename':
                code += 'OUTPUT="$1/missing/report"\n'
            result = self.bash(code + tail, [self.root])
            self.assertEqual(result.returncode, 0 if failure == 'none' else 1, result.stderr)
            self.assertEqual('✓ E152' in result.stderr, failure == 'none')
            if failure != 'none':
                self.assertTrue((self.root / 'report.partial').exists())

    def test_iops_rejects_unsupported_combinations_before_vm_start(self):
        # Run the actual entry point: rejection must precede KVM/dependency checks.
        for disks, recording in [('2', ''), ('1', str(self.root))]:
            self.env.update(VM_IOPS='100', VM_DISKS=disks, VM_BLKLOGWRITES_DIR=recording)
            result = self.bash('bash "$1" /fake/bench', [HERE / 'vm-bench.sh'])
            self.assertEqual(result.returncode, 1)
            self.assertIn('VM_IOPS', result.stderr)
            self.assertNotIn('/dev/kvm', result.stderr)
        guard = section('vm-bench.sh', '# 限速模型目前只实现单盘；拒绝不支持的组合，不能静默减少磁盘或关闭录制。\n', '# 工作目录登记簿。')
        self.assertEqual(self.bash('VM_IOPS=100; VM_DISKS=1; VM_BLKLOGWRITES_DIR=""; die() { exit 1; }\n' + guard).returncode, 0)

    def test_stage_root_extracts_only_verified_packages(self):
        # Build real tiny debs: the unlisted one would overwrite the approved zpool.
        self.assertIsNotNone(shutil.which('dpkg-deb'))
        packages = self.root / 'packages'
        (packages / 'debs').mkdir(parents=True)
        for name, payload in [('a-approved', 'approved'), ('z-extra', 'unverified')]:
            directory = self.root / name
            (directory / 'DEBIAN').mkdir(parents=True)
            (directory / 'usr/sbin').mkdir(parents=True)
            (directory / 'DEBIAN/control').write_text(f'Package: {name}\nVersion: 1\nArchitecture: all\nMaintainer: Test <t@example.invalid>\nDescription: fixture\n')
            (directory / 'usr/sbin/zpool').write_text(payload)
            subprocess.run(['dpkg-deb', '--build', '--root-owner-group', str(directory), str(packages / 'debs' / (name + '.deb'))],
                           check=True, stdout=subprocess.DEVNULL, timeout=10)
        approved = packages / 'debs/a-approved.deb'
        digest = hashlib.sha256(approved.read_bytes()).hexdigest()
        block = 'mkdir -p "$PACKAGES/debs"' + section('e152-stage-root.sh', 'mkdir -p "$PACKAGES/debs"', '# ── 二、')
        code = 'PACKAGES="$1"; PACKAGE_TABLE="unused a-approved.deb $2"; fail() { echo "$1" >&2; exit 1; }\n' + block
        result = self.bash(code, [packages, digest])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((packages / 'root/usr/sbin/zpool').read_text(), 'approved')
        self.assertNotEqual(self.bash(code, [packages, '0'*64]).returncode, 0)


    def test_tear_readback_errors_and_short_reads(self):
        body = 'one_round() {' + source('e129-tear-injector.sh').split('one_round() {', 1)[1]
        (self.root / 'pattern.bin').write_bytes(b'A' * 8192)
        for failure in ['none', 'read', 'short', 'compare']:
            code = '''W="$1"; failure="$2"; LOOP=/fake/loop; DM_DEV=/fake/dm; BOUNDARY=4096; ROUNDS=1; N=0; SPOOL="$W/spool"; : > "$SPOOL"
emit() { echo "$*"; echo "$*" >> "$SPOOL"; N=$((N+1)); }
python3() { [[ "$failure" != compare ]] || return 1; command python3 "$@"; }
S() {
 case "$*" in
 *rb_span.bin*) head -c 4096 "$W/pattern.bin" > "$W/rb_span.bin"; head -c 4096 /dev/zero >> "$W/rb_span.bin";;
 *rb_good.bin*) head -c 4096 "$W/pattern.bin" > "$W/rb_good.bin";;
 *rb_err.bin*)
  if [[ "$failure" == read || "$failure" == short ]]; then
   : > "$W/rb_err.bin"; [[ "$failure" == short ]]; return $?
  fi
  head -c 8192 /dev/zero > "$W/rb_err.bin";;
 *'bs=8192'*|*'seek=1'*) return 1;;
 esac
 return 0
}
''' + body
            result = self.bash(code, [self.root, failure])
            self.assertEqual(result.returncode, 0 if failure == 'none' else 4, result.stderr)
            self.assertEqual('name=verdict' in result.stdout, failure == 'none')

    def test_neighbour_comparator_errors_abort(self):
        fragment = '    first_diff="$(python3' + section(
            'e129-thin-neighbour.sh', '    first_diff="$(python3', '  emit "name=neighbour')
        fragment = fragment.rsplit('  fi', 1)[0]
        for result_value, status, expected in [('-1', 0, 0), ('12', 0, 0), ('', 1, 4), ('', 0, 4), ('junk', 0, 4)]:
            code = '''W="$1"; B_OFF=32768
emit() { echo "$*"; }
python3() { printf '%s' "$2_value"; return "$3_status"; }
'''.replace('"$2_value"', '"$result_value"').replace('"$3_status"', '"$status"')
            result = self.bash('result_value="$2"; status="$3"\n' + code + fragment + '\necho "verdict=$verdict"', [self.root, result_value, status])
            self.assertEqual(result.returncode, expected, result.stderr)
            self.assertEqual('verdict=' in result.stdout, expected == 0)

    def test_kernel_cache_validation_and_atomic_copy(self):
        helpers = 'kernel_readable() {' + section('vm-kernel.sh', 'kernel_readable() {', '# 已经有可读的就直接用')
        original = self.root / 'source'
        original.write_bytes(b'kernel-fixture')
        cached = self.root / 'cached'
        for data, expected in [(b'', 1), (b'ker', 1), (b'kernel-fixture', 0)]:
            cached.write_bytes(data)
            result = self.bash(helpers + '\ncached_kernel_matches "$1" "$2"', [cached, original])
            self.assertEqual(result.returncode, expected)
        self.assertNotEqual(self.bash(helpers + '\nkernel_readable "$1"', [self.root]).returncode, 0)
        # Run actual entry point with empty cache and a nonexistent release; no sudo needed.
        self.env['SINGLEFS_KERNEL_DIR'] = str(self.root)
        (self.root / 'singlefs-vmlinuz-review-nonexistent').touch()
        result = self.bash('bash "$1" --release review-nonexistent', [HERE / 'vm-kernel.sh'])
        self.assertNotEqual(result.returncode, 0)
        copy = 'COPY="$(mktemp' + source('vm-kernel.sh').split('COPY="$(mktemp', 1)[1]
        for fail_copy in [1, 0]:
            cached.write_bytes(b'previous-valid-cache')
            code = '''DEST_DIR="$1"; DEST="$1/cached"; SRC="$1/source"; PASS=fake; fail_copy="$2"
die() { echo "$*" >&2; exit 1; }
sudo() {
 read -r ignored; shift 3
 case "$1" in
 cp) if [[ "$fail_copy" == 1 ]]; then head -c 3 "$2" > "$3"; return 1; fi; command "$@";;
 chown) return 0;;
 stat) command "$@";;
 *) return 1;;
 esac
}
''' + helpers + copy
            result = self.bash(code, [self.root, fail_copy])
            self.assertEqual(result.returncode, fail_copy, result.stderr)
            self.assertEqual(cached.read_bytes(), b'previous-valid-cache' if fail_copy else original.read_bytes())
            self.assertEqual(list(self.root.glob('.singlefs-vmlinuz.*')), [])


if __name__ == '__main__':
    unittest.main(verbosity=2)
