#!/usr/bin/env python3
"""里程碑开工时清一次、完结时查一次：测试环境有没有上一轮的残留，宿主盘有没有异常。

用法：
    test-environment-check.py [check] [--temporary-directory 目录]... [--report 路径]
                              [--minimum-free-percent 10] [--minimum-free-gibibytes 50]
                              [--recent-minutes 10] [--kernel-log-since 时间] [--smart]
    test-environment-check.py clean [--temporary-directory 目录]... [--yes]
    test-environment-check.py --selftest

check 逐类判，每类一行 ✓ / ✗ / !（没查成）/ -（没要求跑）：
    残留测试设备   loop 设备背后的文件在临时目录下；device-mapper 目标名以 sfs_ / singlefs_ 开头，
                   或它压在这样的 loop 上；/proc/mounts 里源是这些设备、源或挂载点在临时目录的 singlefs-* 下；
                   命令行里带 singlefs 的 qemu-system* 进程
    临时目录残留   临时目录下的 singlefs-*，不在白名单、创建者已经不在、没有进程开着它下面的文件
    宿主盘         /、/home、/tmp 与临时目录所在的文件系统变成只读；剩余空间低于阈值；
                   ext4 的 errors_count 不为 0；本次开机以来内核日志里宿主盘栈上的块设备错误；
                   给了 --smart 时再用 sudo 跑 smartctl -H 或 nvme smart-log
clean 只清前两类判红的那些：卸载、dmsetup remove、losetup -d、删文件与目录；不给 --yes 只打印计划。
qemu 进程不自动杀，列出 pid 交人判。白名单与在用的一律不碰，删之前断言路径就在临时目录下、以 singlefs- 开头。

为什么要它：用户 2026-09-19 定「每一轮的里程碑都应该要有一个干净的环境……每次里程碑完结后要检查挂载的盘有没有异常」。
那天现查 /tmp 下 82 个 singlefs-*：37 个 replay 复跑目录共 41G（replay.sh 只在全绿时删），
40 个 4 GiB 稀疏测试盘镜像（crates 测试靠 Drop 删，进程被杀就漏下）。

退出码：0 干净；1 有残留或异常（clean --yes 时：有一步没做成）；2 用法错；3 有一类没查成（与 1 同时成立时取 1）。
破坏开关（只给 --selftest 用，证明自检会红）：环境变量 TEST_ENVIRONMENT_CHECK_BREAK 取
aliveowner / allowlist / openfile / pidreuse / recent / readonly / freespace / kernellog / loopbacking / deleteguard 之一。
"""
import argparse
import contextlib
import dataclasses
import datetime
import io
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time

REPOSITORY_ROOT = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..'))
SCRIPT_RELATIVE_PATH = 'research/scripts/test-environment-check.py'
BREAK_SWITCH_VARIABLE = 'TEST_ENVIRONMENT_CHECK_BREAK'
BREAK_SWITCH_NAMES = ('aliveowner', 'allowlist', 'openfile', 'pidreuse', 'recent',
                      'readonly', 'freespace', 'kernellog', 'loopbacking', 'deleteguard')
TOKYO_TIMEZONE = datetime.timezone(datetime.timedelta(hours=9))
GIBIBYTE = 1 << 30
ENTRY_PREFIX = 'singlefs-'
DEVICE_MAPPER_TEST_PREFIXES = ('sfs_', 'singlefs_')
LOOP_BLOCK_MAJOR = 7
# 进程起始时间晚于条目树里最新的修改时间这么多秒以上，就判那个 pid 已经被别的进程复用
PROCESS_IDENTIFIER_REUSE_SLACK_SECONDS = 2.0

# 有意跨轮复用的缓存：不判残留、clean 不碰。每条写明谁在用它。
ALLOWLIST = (
    (re.compile(r'^singlefs-crates-mutation-target(-w[0-9]+)?$'),
     '门禁 59 号的 cargo 编译产物目录，跨轮复用；带 -w<片号> 的是它分片并发跑时每片各自的那一个'
     '（.claude/gate.d/59-crates-mutation-replay.sh 的 prepare_shard）'),
    (re.compile(r'^singlefs-mutate-target$'),
     'mutate.sh 的 cargo 编译产物目录，跨轮复用（research/scripts/mutate.sh 第 54 行）'),
    (re.compile(r'^singlefs-e152-packages$'),
     'E152 六家文件系统的下载包缓存（research/scripts/e152-stage-root.sh 第 16 行）'),
    (re.compile(r'^singlefs-pdftext-[0-9]+$'),
     '引文核对的 PDF 抽文本缓存，名字里是 uid 不是 pid（research/scripts/verify-citations.sh 的 PDFTXT_CACHE）'),
    (re.compile(r'^singlefs-vmlinuz(-.+)?$'),
     '给 QEMU 用的可读内核副本（research/scripts/vm-kernel.sh 第 19、36 行）'),
)

# 名字里带创建者 pid 的形态：按 pid 判创建者在不在。第一个捕获组是 pid。
PROCESS_IDENTIFIER_NAME_PATTERNS = (
    (re.compile(r'^singlefs-replay-([0-9]+)$'), 'research/scripts/replay.sh 第 16 行'),
    (re.compile(r'^singlefs-step[0-9]+-.+-([0-9]+)-[0-9]+-dev[0-9]+\.img$'),
     'crates/singlefs-harness/tests/ 的 image_path'),
    (re.compile(r'^singlefs-core-fake-sysfs-([0-9]+)$'), 'crates/singlefs-core/src/block_device.rs 第 562 行'),
    (re.compile(r'^singlefs-core-.+-([0-9]+)-[0-9]+\.img$'), 'crates/singlefs-core/src/block_device.rs 第 427 行'),
)


def break_switch_is(name):
    return os.environ.get(BREAK_SWITCH_VARIABLE, '') == name


def format_bytes(byte_count):
    if byte_count >= GIBIBYTE:
        return f'{byte_count / GIBIBYTE:.1f} GiB'
    if byte_count >= 1 << 20:
        return f'{byte_count / (1 << 20):.1f} MiB'
    if byte_count >= 1 << 10:
        return f'{byte_count / (1 << 10):.1f} KiB'
    return f'{byte_count} B'


def format_time(epoch_seconds):
    """本机时钟是 UTC，用户看的是东京时间：两个都写。"""
    universal = datetime.datetime.fromtimestamp(epoch_seconds, datetime.timezone.utc)
    tokyo = universal.astimezone(TOKYO_TIMEZONE)
    return f'{universal:%Y-%m-%d %H:%M} UTC（东京 {tokyo:%m-%d %H:%M}）'


def default_temporary_directories():
    directories = ['/tmp']
    environment_directory = os.environ.get('TMPDIR', '')
    if environment_directory and os.path.realpath(environment_directory) != os.path.realpath('/tmp'):
        directories.append(environment_directory)
    return directories


def is_under_directory(path, directory):
    resolved_directory = directory.rstrip('/') or '/'
    if resolved_directory == '/':
        return path.startswith('/')
    return path == resolved_directory or path.startswith(resolved_directory + '/')


def read_text_or_none(path):
    try:
        with open(path, encoding='utf-8', errors='replace') as handle:
            return handle.read()
    except OSError:
        return None


# ── 进程：在不在、什么时候起的、开着哪些文件 ─────────────────────────────
# 一律按 /proc/<pid>/ 逐个读，不用按模式匹配的进程查找（本机钩子拒绝，而且模式串会命中自己）。

def list_process_identifiers():
    identifiers = []
    for name in os.listdir('/proc'):
        if name.isdigit():
            identifiers.append(int(name))
    return identifiers


def read_process_command_line(process_identifier):
    try:
        with open(f'/proc/{process_identifier}/cmdline', 'rb') as handle:
            raw = handle.read()
    except OSError:
        return None
    return [part.decode('utf-8', errors='replace') for part in raw.split(b'\0') if part]


def read_boot_time_epoch():
    for line in (read_text_or_none('/proc/stat') or '').splitlines():
        if line.startswith('btime '):
            return int(line.split()[1])
    raise RuntimeError('/proc/stat 里没有 btime 行')


def read_process_start_epoch(process_identifier):
    """进程起始的挂钟时间（秒）；进程不在返回 None。"""
    text = read_text_or_none(f'/proc/{process_identifier}/stat')
    if text is None:
        return None
    # 第二个字段是带括号的进程名，里面可以有空格：从最后一个右括号之后切
    fields_after_name = text[text.rfind(')') + 2:].split()
    start_ticks = int(fields_after_name[19])  # 整行第 22 个字段
    return read_boot_time_epoch() + start_ticks / os.sysconf('SC_CLK_TCK')


def process_owner_state(process_identifier, latest_modification_epoch):
    """返回 ('alive' | 'dead' | 'reused', 说明)。

    reused：这个 pid 现在的进程起在条目最后一次被修改之后，它不可能是创建者。
    """
    if break_switch_is('aliveowner'):
        return 'dead', f'创建者 pid {process_identifier} 已不在'
    start_epoch = read_process_start_epoch(process_identifier)
    if start_epoch is None:
        return 'dead', f'创建者 pid {process_identifier} 已不在'
    if (not break_switch_is('pidreuse')
            and start_epoch > latest_modification_epoch + PROCESS_IDENTIFIER_REUSE_SLACK_SECONDS):
        return 'reused', (f'pid {process_identifier} 现在的进程起于 {format_time(start_epoch)}，'
                          f'晚于条目最后修改，是别的进程复用了这个号，创建者已不在')
    command_line = read_process_command_line(process_identifier) or ['?']
    command_text = ' '.join(command_line).replace('\n', ' ')
    return 'alive', f'创建者 pid {process_identifier} 还在（{command_text[:100]}）'


@dataclasses.dataclass
class OpenPathScan:
    opened_paths: list          # [(路径, pid, 怎么开着的)]，只留临时目录下的
    unreadable_process_count: int   # 读不了 fd 的用户态进程（别的用户的）


def scan_open_paths(temporary_directories):
    """每个进程的 fd、cwd、exe 里落在临时目录下的路径。读不了的进程（别的用户的）只计数。"""
    resolved_directories = [os.path.realpath(directory) for directory in temporary_directories]
    opened_paths = []
    unreadable_process_count = 0
    own_process_identifier = os.getpid()
    for process_identifier in list_process_identifiers():
        if process_identifier == own_process_identifier:
            continue
        links = []
        for kind in ('cwd', 'exe'):
            links.append((f'/proc/{process_identifier}/{kind}', kind))
        try:
            descriptor_names = os.listdir(f'/proc/{process_identifier}/fd')
        except PermissionError:
            if read_process_command_line(process_identifier):   # 内核线程命令行为空、也不开文件，不计
                unreadable_process_count += 1
            continue
        except OSError:
            continue  # 进程在读的过程中退出了
        for descriptor_name in descriptor_names:
            links.append((f'/proc/{process_identifier}/fd/{descriptor_name}', f'fd {descriptor_name}'))
        for link_path, kind in links:
            try:
                target = os.readlink(link_path)
            except OSError:
                continue
            target = target.removesuffix(' (deleted)')
            if any(is_under_directory(target, directory) for directory in resolved_directories):
                opened_paths.append((target, process_identifier, kind))
    return OpenPathScan(opened_paths=opened_paths, unreadable_process_count=unreadable_process_count)


def find_openers(entry_path, open_path_scan):
    if break_switch_is('openfile'):
        return []
    resolved_entry = os.path.realpath(entry_path) if not os.path.islink(entry_path) else entry_path
    openers = []
    for opened_path, process_identifier, kind in open_path_scan.opened_paths:
        if is_under_directory(opened_path, resolved_entry):
            openers.append((process_identifier, kind, opened_path))
    return openers


# ── 第二类：临时目录下的 singlefs-* ──────────────────────────────────────

@dataclasses.dataclass
class TemporaryEntry:
    path: str
    kind: str                       # 目录 / 文件 / 符号链接
    actual_bytes: int               # 按 st_blocks 算的实际占用，稀疏文件不按表观算
    apparent_bytes: int
    latest_modification_epoch: float
    unreadable_count: int           # 遍历时读不了的子项个数
    verdict: str                    # residual / in_use / allowlisted
    reason: str


def measure_entry(path):
    """(类型, 实际占用, 表观大小, 树里最新的修改时间, 读不了的子项数)。不跟符号链接。"""
    top_status = os.lstat(path)
    if stat.S_ISLNK(top_status.st_mode):
        return '符号链接', top_status.st_blocks * 512, top_status.st_size, top_status.st_mtime, 0
    if not stat.S_ISDIR(top_status.st_mode):
        return '文件', top_status.st_blocks * 512, top_status.st_size, top_status.st_mtime, 0
    actual_bytes = top_status.st_blocks * 512
    apparent_bytes = top_status.st_size
    latest_modification_epoch = top_status.st_mtime
    unreadable_count = 0

    def count_unreadable(error):
        nonlocal unreadable_count
        unreadable_count += 1

    for directory_path, directory_names, file_names in os.walk(path, onerror=count_unreadable):
        for child_name in directory_names + file_names:
            try:
                child_status = os.lstat(os.path.join(directory_path, child_name))
            except OSError:
                unreadable_count += 1
                continue
            actual_bytes += child_status.st_blocks * 512
            apparent_bytes += child_status.st_size
            latest_modification_epoch = max(latest_modification_epoch, child_status.st_mtime)
    return '目录', actual_bytes, apparent_bytes, latest_modification_epoch, unreadable_count


def match_allowlist(name):
    if break_switch_is('allowlist'):
        return None
    for pattern, reason in ALLOWLIST:
        if pattern.match(name):
            return reason
    return None


def match_process_identifier(name):
    for pattern, source in PROCESS_IDENTIFIER_NAME_PATTERNS:
        matched = pattern.match(name)
        if matched:
            return int(matched.group(1)), source
    return None, None


def describe_openers(openers):
    shown = [f'pid {process_identifier} 的 {kind} 开着 {opened_path}'
             for process_identifier, kind, opened_path in openers[:3]]
    more = f' 等 {len(openers)} 处' if len(openers) > 3 else ''
    return '；'.join(shown) + more


def classify_temporary_entry(path, open_path_scan, now_epoch, recent_seconds, produced_after=0.0):
    name = os.path.basename(path)
    kind, actual_bytes, apparent_bytes, latest_modification_epoch, unreadable_count = measure_entry(path)

    def build(verdict, reason):
        return TemporaryEntry(path=path, kind=kind, actual_bytes=actual_bytes, apparent_bytes=apparent_bytes,
                              latest_modification_epoch=latest_modification_epoch,
                              unreadable_count=unreadable_count, verdict=verdict, reason=reason)

    allowlist_reason = match_allowlist(name)
    if allowlist_reason is not None:
        return build('allowlisted', f'白名单：{allowlist_reason}')
    openers = find_openers(path, open_path_scan)
    process_identifier, source = match_process_identifier(name)
    if process_identifier is not None:
        owner_state, owner_description = process_owner_state(process_identifier, latest_modification_epoch)
        if owner_state == 'alive':
            return build('in_use', f'{owner_description}；名字形态出自 {source}')
        if openers:
            return build('in_use', f'{owner_description}，但 {describe_openers(openers)}')
        if produced_after and latest_modification_epoch >= produced_after:
            # 这一次跑自己产生的：门禁 59 号让每条变异点名的测试判红，那些测试红在断言上就 panic，
            # 写在断言之后的清理走不到（C448）。它确实是垃圾，但不是「上一轮没清干净」，不该让整道红。
            return build('produced_by_this_run', f'{owner_description}，也没有进程开着它'
                         f'（名字形态出自 {source}）；这一次跑自己产生的，本次跑完再清')
        return build('residual', f'{owner_description}，也没有进程开着它（名字形态出自 {source}）')
    if openers:
        return build('in_use', describe_openers(openers))
    age_seconds = now_epoch - latest_modification_epoch
    if age_seconds < recent_seconds and not break_switch_is('recent'):
        return build('in_use', f'名字里没有 pid、没有进程开着它，但 {int(age_seconds // 60)} 分钟前还在改，可能正在跑')
    return build('residual', '名字里没有 pid，没有进程开着它下面的文件，'
                             f'最近一次修改在 {int(age_seconds // 60)} 分钟前')


def scan_temporary_directories(temporary_directories, recent_seconds, produced_after=0.0):
    """返回 (条目列表, 打开文件扫描结果, 读不了的临时目录列表)。"""
    open_path_scan = scan_open_paths(temporary_directories)
    now_epoch = time.time()
    entries = []
    unreadable_directories = []
    seen_real_paths = set()
    for directory in temporary_directories:
        try:
            names = sorted(os.listdir(directory))
        except OSError as error:
            unreadable_directories.append(f'{directory}（{error.strerror}）')
            continue
        for name in names:
            if not name.startswith(ENTRY_PREFIX):
                continue
            path = os.path.join(directory, name)
            real_path = os.path.join(os.path.realpath(directory), name)
            if real_path in seen_real_paths:
                continue  # TMPDIR 与 /tmp 指向同一处时只算一次
            seen_real_paths.add(real_path)
            try:
                entries.append(classify_temporary_entry(path, open_path_scan, now_epoch, recent_seconds, produced_after))
            except FileNotFoundError:
                continue  # 扫描的过程中被别的进程删掉了
    return entries, open_path_scan, unreadable_directories


# ── 运行环境：真跑时取系统的，自检时换成造出来的 ──────────────────────────

@dataclasses.dataclass
class Environment:
    temporary_directories: list
    system_block_root: str = '/sys/block'
    mounts_path: str = '/proc/mounts'
    ext4_system_root: str = '/sys/fs/ext4'
    recent_seconds: int = 600
    minimum_free_percent: float = 10.0
    minimum_free_gibibytes: float = 50.0
    kernel_log_since: str = ''
    smart_requested: bool = False
    # 这个时刻之后才出现的临时条目单列成「这一次跑自己产生的」、不判红（C448）；0 表示不分界、照旧全判
    produced_after: float = 0.0
    # 可注入的读取器；None 表示用真系统
    kernel_log_reader: object = None       # () -> (行列表 | None, 说明)
    host_device_resolver: object = None    # (mounts 行) -> (设备名集合, 主次设备号集合)
    filesystem_space_reader: object = None  # (路径) -> (总字节, 可用字节)
    device_mapper_major: object = None
    command_runner: object = None           # (命令列表) -> (退出码, 输出)；clean 动设备用
    process_identifier_filter: object = None  # 只看这些 pid 里的 qemu；None 表示全部进程


@dataclasses.dataclass
class MountLine:
    source: str
    mount_point: str
    filesystem_type: str
    options: list


def decode_mount_field(field):
    """/proc/mounts 里空格、制表符写成八进制转义。"""
    return re.sub(r'\\([0-7]{3})', lambda matched: chr(int(matched.group(1), 8)), field)


def read_mount_lines(environment):
    text = read_text_or_none(environment.mounts_path) or ''
    mount_lines = []
    for raw_line in text.splitlines():
        fields = raw_line.split()
        if len(fields) < 4:
            continue
        mount_lines.append(MountLine(source=decode_mount_field(fields[0]),
                                     mount_point=decode_mount_field(fields[1]),
                                     filesystem_type=fields[2], options=fields[3].split(',')))
    return mount_lines


# ── 第一类：残留测试设备 ────────────────────────────────────────────────

@dataclasses.dataclass
class DeviceScan:
    loop_devices_checked: int
    loop_devices_attached: int
    residual_loops: list          # [(设备名, 背后文件)]
    device_mapper_checked: int
    residual_device_mappers: list  # [(dm-N, 目标名, 为什么)]
    mounts_checked: int
    residual_mounts: list          # [MountLine]
    processes_checked: int
    residual_emulators: list       # [(pid, 父 pid, 命令行)]


def scan_loop_devices(environment):
    resolved_directories = [os.path.realpath(directory) for directory in environment.temporary_directories]
    checked = 0
    attached = 0
    residual = []
    for device_name in sorted(os.listdir(environment.system_block_root)):
        if not re.match(r'^loop[0-9]+$', device_name):
            continue
        checked += 1
        backing_text = read_text_or_none(os.path.join(environment.system_block_root, device_name,
                                                      'loop', 'backing_file'))
        if backing_text is None:
            continue  # 没挂文件
        attached += 1
        backing_path = backing_text.strip().removesuffix(' (deleted)')
        if break_switch_is('loopbacking'):
            continue
        if any(is_under_directory(backing_path, directory) for directory in resolved_directories):
            residual.append((device_name, backing_text.strip()))
    return checked, attached, residual


def scan_device_mappers(environment, residual_loop_names):
    checked = 0
    residual = []
    for device_name in sorted(os.listdir(environment.system_block_root)):
        if not re.match(r'^dm-[0-9]+$', device_name):
            continue
        checked += 1
        target_name = (read_text_or_none(os.path.join(environment.system_block_root, device_name,
                                                      'dm', 'name')) or '').strip()
        slaves_directory = os.path.join(environment.system_block_root, device_name, 'slaves')
        try:
            slave_names = sorted(os.listdir(slaves_directory))
        except OSError:
            slave_names = []
        residual_slaves = [slave for slave in slave_names if slave in residual_loop_names]
        if target_name.startswith(DEVICE_MAPPER_TEST_PREFIXES):
            residual.append((device_name, target_name, '目标名是测试脚本的前缀'))
        elif residual_slaves:
            residual.append((device_name, target_name, f'压在残留的 {"、".join(residual_slaves)} 上'))
    return checked, residual


def mount_is_residual(mount_line, environment, residual_loop_names, residual_device_mappers):
    resolved_directories = [os.path.realpath(directory) for directory in environment.temporary_directories]
    residual_device_paths = {f'/dev/{name}' for name in residual_loop_names}
    for device_name, target_name, residual_reason in residual_device_mappers:
        residual_device_paths.add(f'/dev/{device_name}')
        residual_device_paths.add(f'/dev/mapper/{target_name}')
    if mount_line.source in residual_device_paths:
        return True
    for path in (mount_line.source, mount_line.mount_point):
        for directory in resolved_directories:
            if is_under_directory(path, directory) and path != directory:
                relative = path[len(directory.rstrip('/')) + 1:]
                if relative.startswith(ENTRY_PREFIX):
                    return True
    return False


def scan_emulator_processes(process_identifier_filter):
    checked = 0
    residual = []
    for process_identifier in list_process_identifiers():
        if process_identifier_filter is not None and process_identifier not in process_identifier_filter:
            continue
        command_line = read_process_command_line(process_identifier)
        if not command_line:
            continue
        checked += 1
        if not os.path.basename(command_line[0]).startswith('qemu-system'):
            continue
        if any('singlefs' in argument for argument in command_line[1:]):
            status_text = read_text_or_none(f'/proc/{process_identifier}/status') or ''
            parent_match = re.search(r'^PPid:\s+([0-9]+)', status_text, re.MULTILINE)
            parent_identifier = int(parent_match.group(1)) if parent_match else -1
            residual.append((process_identifier, parent_identifier, ' '.join(command_line).replace('\n', ' ')))
    return checked, residual


def scan_devices(environment):
    loop_checked, loop_attached, residual_loops = scan_loop_devices(environment)
    residual_loop_names = {device_name for device_name, backing in residual_loops}
    device_mapper_checked, residual_device_mappers = scan_device_mappers(environment, residual_loop_names)
    mount_lines = read_mount_lines(environment)
    residual_mounts = [mount_line for mount_line in mount_lines
                       if mount_is_residual(mount_line, environment, residual_loop_names, residual_device_mappers)]
    processes_checked, residual_emulators = scan_emulator_processes(environment.process_identifier_filter)
    return DeviceScan(loop_devices_checked=loop_checked, loop_devices_attached=loop_attached,
                      residual_loops=residual_loops, device_mapper_checked=device_mapper_checked,
                      residual_device_mappers=residual_device_mappers, mounts_checked=len(mount_lines),
                      residual_mounts=residual_mounts, processes_checked=processes_checked,
                      residual_emulators=residual_emulators)


# ── 第三类：宿主盘 ──────────────────────────────────────────────────────

def host_target_paths(environment):
    paths = ['/', '/home', '/tmp']
    for directory in environment.temporary_directories:
        resolved = os.path.realpath(directory)
        if resolved not in paths:
            paths.append(resolved)
    return [path for path in paths if os.path.exists(path)]


def covering_mount(path, mount_lines):
    """挂载点是 path 最长前缀的那一条；同一挂载点挂了几次取最后一次（最上面那层）。"""
    best = None
    for mount_line in mount_lines:
        if is_under_directory(path, mount_line.mount_point):
            if best is None or len(mount_line.mount_point) >= len(best.mount_point):
                best = mount_line
    return best


def resolve_host_devices(covering_mounts):
    """宿主挂载栈上的块设备名与主次设备号：分区、它的整盘、dm / md 的下层、nvme 控制器。"""
    names = set()
    pending = []
    for mount_line in covering_mounts:
        if mount_line.source.startswith('/dev/'):
            pending.append(os.path.basename(os.path.realpath(mount_line.source)))
    while pending:
        name = pending.pop()
        if name in names:
            continue
        names.add(name)
        class_path = f'/sys/class/block/{name}'
        if os.path.exists(os.path.join(class_path, 'partition')):
            pending.append(os.path.basename(os.path.dirname(os.path.realpath(class_path))))
        try:
            pending.extend(os.listdir(os.path.join(class_path, 'slaves')))
        except OSError:
            pass
        controller_match = re.match(r'^(nvme[0-9]+)n[0-9]+', name)
        if controller_match:
            names.add(controller_match.group(1))
    major_minor_numbers = set()
    for name in names:
        number_text = read_text_or_none(f'/sys/class/block/{name}/dev')
        if number_text:
            major_minor_numbers.add(number_text.strip())
    return names, major_minor_numbers


def read_filesystem_space(path):
    statistics = os.statvfs(path)
    return statistics.f_blocks * statistics.f_frsize, statistics.f_bavail * statistics.f_frsize


def read_device_mapper_major():
    for line in (read_text_or_none('/proc/devices') or '').splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[1] == 'device-mapper':
            return int(fields[0])
    return None


def read_kernel_error_log(since_text):
    """本次开机以来优先级 err 及以上的内核日志；读不到返回 (None, 原因)。"""
    command = ['journalctl', '-k', '-b', '-p', 'err', '--no-pager', '-o', 'short-iso']
    if since_text:
        command += ['--since', since_text]
    if shutil.which('journalctl'):
        completed = subprocess.run(command, capture_output=True, text=True, timeout=120)
        permission_hint = re.search(r'insufficient permissions|not seeing messages', completed.stderr)
        if completed.returncode == 0 and not permission_hint:
            lines = [line for line in completed.stdout.splitlines() if line.strip() and not line.startswith('-- ')]
            return lines, f'journalctl -k -b -p err{" --since " + since_text if since_text else ""}'
        journal_problem = (completed.stderr.strip().splitlines() or [f'退出码 {completed.returncode}'])[0]
    else:
        journal_problem = 'journalctl 不在 PATH'
    if since_text:
        return None, f'journalctl 读不了（{journal_problem}），而 dmesg 不支持 --since'
    if shutil.which('dmesg'):
        completed = subprocess.run(['dmesg', '--level=err,crit,alert,emerg'], capture_output=True, text=True,
                                   timeout=120)
        if completed.returncode == 0:
            return [line for line in completed.stdout.splitlines() if line.strip()], 'dmesg --level=err,crit,alert,emerg'
        dmesg_problem = (completed.stderr.strip().splitlines() or [f'退出码 {completed.returncode}'])[0]
    else:
        dmesg_problem = 'dmesg 不在 PATH'
    return None, f'权限或工具：journalctl（{journal_problem}）；dmesg（{dmesg_problem}）'


BLOCK_ERROR_PATTERN = re.compile(
    r'I/O error|Buffer I/O|blk_update_request|print_req_error|critical (medium|target) error|'
    r'[Mm]edium [Ee]rror|Unrecovered read error|EXT4-fs|XFS|BTRFS|F2FS|jbd2|nvme|\bata[0-9]|\bsd[a-z]+\b|'
    r'device-mapper|\bloop[0-9]+|\bdm-[0-9]+|\bmd[0-9]+|read-only|reset controller|I/O timeout')
DEVICE_NAME_PATTERN = re.compile(
    r'\b(nvme[0-9]+n[0-9]+(?:p[0-9]+)?|nvme[0-9]+|sd[a-z]+[0-9]*|vd[a-z]+[0-9]*|xvd[a-z]+[0-9]*|'
    r'mmcblk[0-9]+(?:p[0-9]+)?|loop[0-9]+|dm-[0-9]+|md[0-9]+)\b')
MAJOR_MINOR_PATTERN = re.compile(r'\b([0-9]{1,4}):([0-9]{1,7})\b')


def classify_kernel_log_line(line, host_device_names, host_major_minor_numbers, device_mapper_major):
    """返回 host（宿主盘栈上的错误）/ other（块设备错误，但不是测试设备也认不出是谁）/
    test（loop、dm 等测试设备上的）/ unrelated（不是块设备的错误）。"""
    message = re.sub(r'^\S+\s+\S+\s+kernel:\s*', '', line)   # 去掉 short-iso 的时间与主机名
    if not BLOCK_ERROR_PATTERN.search(message):
        return 'unrelated'
    device_names = set(DEVICE_NAME_PATTERN.findall(message))
    major_minor_numbers = {f'{major}:{minor}' for major, minor in MAJOR_MINOR_PATTERN.findall(message)}
    if break_switch_is('kernellog'):
        return 'test'
    if device_names & host_device_names or major_minor_numbers & host_major_minor_numbers:
        return 'host'
    test_major_numbers = {str(LOOP_BLOCK_MAJOR)}
    if device_mapper_major is not None:
        test_major_numbers.add(str(device_mapper_major))
    host_uses_device_mapper = any(name.startswith('dm-') for name in host_device_names)
    names_are_test = bool(device_names) and all(re.match(r'^(loop|dm-)[0-9]+$', name) for name in device_names)
    numbers_are_test = bool(major_minor_numbers) and all(
        number.split(':')[0] in test_major_numbers for number in major_minor_numbers)
    if names_are_test or numbers_are_test:
        return 'test'
    if message.startswith('device-mapper:') and not host_uses_device_mapper and not device_names:
        return 'test'
    return 'other'


# ── sudo：口令从仓根 .env 的 SUDO_PASS_A / SUDO_PASS_B 读，走 stdin，不打印、不进命令行 ────

def read_environment_file_values():
    values = {}
    text = read_text_or_none(os.path.join(REPOSITORY_ROOT, '.env')) or ''
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith('#') or '=' not in line:
            continue
        key, value = line.removeprefix('export ').split('=', 1)
        value = value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in '\'"':
            value = value[1:-1]
        values[key.strip()] = value
    return values


def find_sudo_password():
    """返回能用的口令；没有返回 None。每个候选只拿 `sudo -S true` 试一次，输出全丢掉。"""
    if not shutil.which('sudo'):
        return None
    values = read_environment_file_values()
    for key in ('SUDO_PASS_A', 'SUDO_PASS_B'):
        candidate = values.get(key, '')
        if not candidate:
            continue
        completed = subprocess.run(['sudo', '-S', '-p', '', 'true'], input=candidate + '\n',
                                   capture_output=True, text=True, timeout=60)
        if completed.returncode == 0:
            return candidate
    return None


def make_sudo_runner(password):
    def run_with_sudo(command):
        completed = subprocess.run(['sudo', '-S', '-p', '', *command], input=password + '\n',
                                   capture_output=True, text=True, timeout=300)
        return completed.returncode, (completed.stdout + completed.stderr).strip()
    return run_with_sudo


def whole_disk_names(host_device_names):
    """宿主设备名里的整盘（有 /sys/block/<名字> 的，dm / md / loop 除外）。"""
    disks = []
    for name in sorted(host_device_names):
        if re.match(r'^(dm-|md|loop)', name):
            continue
        if os.path.isdir(f'/sys/block/{name}'):
            disks.append(name)
    return disks


def check_smart(host_device_names):
    """返回 (状态, 一句话, 明细行)。状态 green / red / unchecked。"""
    disks = whole_disk_names(host_device_names)
    if not disks:
        return 'unchecked', '宿主挂载栈里没认出整盘', []
    smart_tool = shutil.which('smartctl')
    nvme_tool = shutil.which('nvme')
    if not smart_tool and not nvme_tool:
        return 'unchecked', 'smartctl 与 nvme 都不在 PATH', []
    password = find_sudo_password()
    if password is None:
        return 'unchecked', 'sudo 不通（.env 里的 SUDO_PASS_A / SUDO_PASS_B 都试过）', []
    run_with_sudo = make_sudo_runner(password)
    detail_lines = []
    worst = 'green'
    for disk in disks:
        if smart_tool:
            return_code, output = run_with_sudo([smart_tool, '-H', f'/dev/{disk}'])
            if return_code & 0b11:
                detail_lines.append(f'{disk}：smartctl -H 没跑成（退出码 {return_code}）')
                worst = 'unchecked' if worst == 'green' else worst
            elif return_code & 0b1000 or not re.search(r'PASSED|: OK', output):
                detail_lines.append(f'{disk}：smartctl -H 不是 PASSED（退出码 {return_code}）：{output[-200:]}')
                worst = 'red'
            else:
                detail_lines.append(f'{disk}：smartctl -H PASSED')
        elif disk.startswith('nvme'):
            return_code, output = run_with_sudo([nvme_tool, 'smart-log', f'/dev/{disk}'])
            warning = re.search(r'critical_warning\s*:\s*(\S+)', output)
            media_errors = re.search(r'media_errors\s*:\s*([0-9,]+)', output)
            if return_code != 0 or not warning:
                detail_lines.append(f'{disk}：nvme smart-log 没跑成（退出码 {return_code}）')
                worst = 'unchecked' if worst == 'green' else worst
            elif warning.group(1) not in ('0', '0x0') or (media_errors and media_errors.group(1) != '0'):
                detail_lines.append(f'{disk}：critical_warning={warning.group(1)}，'
                                    f'media_errors={media_errors.group(1) if media_errors else "?"}')
                worst = 'red'
            else:
                detail_lines.append(f'{disk}：critical_warning=0，media_errors=0')
        else:
            detail_lines.append(f'{disk}：不是 nvme 盘而 smartctl 不在 PATH，没查')
            worst = 'unchecked' if worst == 'green' else worst
    headline = {'green': f'查了 {len(disks)} 块盘，都正常',
                'red': f'查了 {len(disks)} 块盘，有异常',
                'unchecked': f'查了 {len(disks)} 块盘，有的没查成'}[worst]
    return worst, headline, detail_lines


# ── 每类一个判定 ────────────────────────────────────────────────────────

@dataclasses.dataclass
class CategoryResult:
    title: str
    status: str            # green / red / unchecked / skipped
    headline: str
    detail_lines: list
    next_step: str = ''


CLEAN_COMMAND = f'python3 {SCRIPT_RELATIVE_PATH} clean'


def judge_devices(device_scan):
    results = []
    loop_next = ('先确认它们不是正在跑的脚本在用（research/scripts/e*-probe.sh、e129-*.sh 跑完会自己拆），'
                 f'再跑 {CLEAN_COMMAND} 看计划，确认后加 --yes')
    if device_scan.residual_loops:
        results.append(CategoryResult('残留测试设备·loop', 'red',
                                      f'{len(device_scan.residual_loops)} 个 loop 设备背后的文件在临时目录下',
                                      [f'/dev/{name} ← {backing}' for name, backing in device_scan.residual_loops],
                                      loop_next))
    else:
        results.append(CategoryResult('残留测试设备·loop', 'green',
                                      f'查了 {device_scan.loop_devices_checked} 个 loop 设备'
                                      f'（{device_scan.loop_devices_attached} 个挂着文件），没有背后文件在临时目录下的', []))
    if device_scan.residual_device_mappers:
        results.append(CategoryResult('残留测试设备·device-mapper', 'red',
                                      f'{len(device_scan.residual_device_mappers)} 个测试用的 device-mapper 目标还在',
                                      [f'{device_name}（{target_name}）：{why}'
                                       for device_name, target_name, why in device_scan.residual_device_mappers],
                                      loop_next))
    else:
        results.append(CategoryResult('残留测试设备·device-mapper', 'green',
                                      f'查了 {device_scan.device_mapper_checked} 个 dm 设备，'
                                      '没有 sfs_ / singlefs_ 开头或压在残留 loop 上的', []))
    if device_scan.residual_mounts:
        results.append(CategoryResult('残留测试设备·挂载', 'red',
                                      f'{len(device_scan.residual_mounts)} 条挂载的源或挂载点是测试设备或临时目录',
                                      [f'{mount_line.source} 挂在 {mount_line.mount_point}（{mount_line.filesystem_type}）'
                                       for mount_line in device_scan.residual_mounts], loop_next))
    else:
        results.append(CategoryResult('残留测试设备·挂载', 'green',
                                      f'查了 {device_scan.mounts_checked} 条挂载，没有测试设备或临时目录下的', []))
    if device_scan.residual_emulators:
        results.append(CategoryResult(
            '残留测试设备·qemu 进程', 'red',
            f'{len(device_scan.residual_emulators)} 个命令行里带 singlefs 的 qemu-system* 进程',
            [f'pid {process_identifier}（父 pid {parent_identifier}）：{command_text[:160]}'
             for process_identifier, parent_identifier, command_text in device_scan.residual_emulators],
            '确认它不是正在跑的门禁 55 号或 vm-bench.sh 之后，按列出的 pid 另起一条 kill <pid> 停掉'
            '（clean 不自动杀进程）；父 pid 是 1 多半是跑它的脚本已经没了'))
    else:
        results.append(CategoryResult('残留测试设备·qemu 进程', 'green',
                                      f'查了 {device_scan.processes_checked} 个进程的命令行，'
                                      '没有带 singlefs 的 qemu-system*', []))
    return results


def describe_entry(entry):
    unreadable = f'，{entry.unreadable_count} 个子项读不了' if entry.unreadable_count else ''
    return (f'{entry.path}  {entry.kind}  实际 {format_bytes(entry.actual_bytes)}'
            f'（表观 {format_bytes(entry.apparent_bytes)}）  最后修改 {format_time(entry.latest_modification_epoch)}'
            f'{unreadable}  —— {entry.reason}')


def judge_temporary_entries(entries, open_path_scan, unreadable_directories, temporary_directories):
    residual = [entry for entry in entries if entry.verdict == 'residual']
    kept = [entry for entry in entries if entry.verdict != 'residual']
    kept_lines = [f'不判红：{describe_entry(entry)}' for entry in kept]
    produced_by_this_run = [entry for entry in entries if entry.verdict == 'produced_by_this_run']
    if open_path_scan.unreadable_process_count:
        kept_lines.append(f'注意：{open_path_scan.unreadable_process_count} 个进程的打开文件读不了（别的用户的），'
                          '「没有进程开着」只对读得到的进程成立')
    scope = '、'.join(temporary_directories)
    if residual:
        actual_total = sum(entry.actual_bytes for entry in residual)
        apparent_total = sum(entry.apparent_bytes for entry in residual)
        return CategoryResult('临时目录残留', 'red',
                              f'{scope} 下 {len(residual)} 项残留，实际占用 {format_bytes(actual_total)}'
                              f'（表观 {format_bytes(apparent_total)}）；共查 {len(entries)} 项 singlefs-*，其中这一次跑自己产生的 {len(produced_by_this_run)} 项不判红',
                              [describe_entry(entry) for entry in residual] + kept_lines,
                              f'跑 {CLEAN_COMMAND} 看计划，确认后加 --yes；某项其实是该留的缓存，'
                              '就把它连同用它的脚本与行号加进本脚本的 ALLOWLIST')
    if unreadable_directories:
        return CategoryResult('临时目录残留', 'unchecked', f'读不了 {"、".join(unreadable_directories)}',
                              kept_lines, '确认临时目录存在且当前用户读得了，或用 --temporary-directory 指到对的目录')
    allowlisted_count = sum(1 for entry in entries if entry.verdict == 'allowlisted')
    in_use_count = sum(1 for entry in entries if entry.verdict == 'in_use')
    return CategoryResult('临时目录残留', 'green',
                          f'查了 {scope} 下 {len(entries)} 项 singlefs-*（白名单 {allowlisted_count} 项、'
                          f'在用 {in_use_count} 项、这一次跑自己产生的 {len(produced_by_this_run)} 项），没有残留', kept_lines)


def judge_readonly(covering_pairs):
    detail_lines = []
    readonly_lines = []
    for path, mount_line in covering_pairs:
        if mount_line is None:
            readonly_lines.append(f'{path}：/proc/mounts 里找不到罩着它的挂载')
            continue
        is_readonly = 'ro' in mount_line.options and not break_switch_is('readonly')
        description = (f'{path} 在 {mount_line.source} 上（{mount_line.filesystem_type}，挂在 {mount_line.mount_point}，'
                       f'{"只读" if is_readonly else "可写"}）')
        (readonly_lines if is_readonly else detail_lines).append(description)
    if readonly_lines:
        return CategoryResult('宿主盘·只读挂载', 'red', f'{len(readonly_lines)} 处本该可写的挂载不可写或找不到',
                              readonly_lines + detail_lines,
                              '文件系统被内核改成只读多半是出了 I/O 错：先看 journalctl -k -b -p err 里宿主盘的行，'
                              '别再往这块盘写，交用户判要不要 fsck、换盘')
    return CategoryResult('宿主盘·只读挂载', 'green', f'查了 {len(covering_pairs)} 个路径，罩着它们的挂载都可写', [])


def judge_free_space(environment, covering_pairs):
    reader = environment.filesystem_space_reader or read_filesystem_space
    detail_lines = []
    short_lines = []
    seen_mount_points = set()
    for path, mount_line in covering_pairs:
        key = mount_line.mount_point if mount_line is not None else path
        if key in seen_mount_points:
            continue
        seen_mount_points.add(key)
        total_bytes, available_bytes = reader(path)
        percent_floor = total_bytes * environment.minimum_free_percent / 100
        absolute_floor = environment.minimum_free_gibibytes * GIBIBYTE
        required_bytes = max(percent_floor, absolute_floor)   # 两条都得满足：取严
        if break_switch_is('freespace'):
            required_bytes = 0
        description = (f'{key}：可用 {format_bytes(available_bytes)} / 共 {format_bytes(total_bytes)}'
                       f'（{available_bytes * 100 / max(total_bytes, 1):.0f}%），下限 {format_bytes(int(required_bytes))}')
        (short_lines if available_bytes < required_bytes else detail_lines).append(description)
    if short_lines:
        return CategoryResult('宿主盘·剩余空间', 'red', f'{len(short_lines)} 个文件系统剩余空间低于下限',
                              short_lines + detail_lines,
                              f'先跑 {CLEAN_COMMAND} 清残留；还不够就看临时目录与 target/ 下最大的几项，'
                              '或用 --minimum-free-percent / --minimum-free-gibibytes 改下限')
    return CategoryResult('宿主盘·剩余空间', 'green',
                          f'查了 {len(seen_mount_points)} 个文件系统，剩余都不低于 {environment.minimum_free_percent:g}%'
                          f' 且不低于 {environment.minimum_free_gibibytes:g} GiB', detail_lines)


def judge_ext4_errors(environment, host_device_names):
    checked = []
    error_lines = []
    for name in sorted(host_device_names):
        counter_text = read_text_or_none(os.path.join(environment.ext4_system_root, name, 'errors_count'))
        if counter_text is None:
            continue
        checked.append(name)
        if counter_text.strip() != '0':
            first_error = (read_text_or_none(os.path.join(environment.ext4_system_root, name,
                                                          'first_error_time')) or '?').strip()
            error_lines.append(f'{name}：errors_count={counter_text.strip()}，first_error_time={first_error}')
    if error_lines:
        return CategoryResult('宿主盘·ext4 错误计数', 'red', f'{len(error_lines)} 个 ext4 文件系统记过错',
                              error_lines, '看 journalctl -k 里 EXT4-fs error 的行与 /sys/fs/ext4/<设备>/first_error_*，'
                                           '交用户判要不要停机 fsck')
    if not checked:
        return CategoryResult('宿主盘·ext4 错误计数', 'skipped', '宿主挂载栈里没有 ext4', [])
    return CategoryResult('宿主盘·ext4 错误计数', 'green',
                          f'查了 {len(checked)} 个 ext4（{"、".join(checked)}），errors_count 都是 0', [])


def judge_kernel_log(environment, host_device_names, host_major_minor_numbers):
    reader = environment.kernel_log_reader or (lambda: read_kernel_error_log(environment.kernel_log_since))
    lines, source_description = reader()
    if lines is None:
        return CategoryResult('宿主盘·内核日志', 'unchecked', f'没查（{source_description}）', [],
                              '把当前用户加进 adm 或 systemd-journal 组，或手工 sudo journalctl -k -b -p err 看一遍，'
                              '结论写进里程碑记录；读不到不算通过')
    device_mapper_major = (environment.device_mapper_major if environment.device_mapper_major is not None
                           else read_device_mapper_major())
    groups = {'host': [], 'other': [], 'test': [], 'unrelated': []}
    for line in lines:
        groups[classify_kernel_log_line(line, host_device_names, host_major_minor_numbers,
                                        device_mapper_major)].append(line)
    information_lines = []
    if groups['test']:
        information_lines.append(f'不判红：测试设备（loop / dm，不在宿主挂载栈里）上的错误 {len(groups["test"])} 行，'
                                 f'第一行 {groups["test"][0][:120]}；最后一行 {groups["test"][-1][:120]}'
                                 '（e129-tear-injector.sh 有意注入 dm error 段）')
    if groups['unrelated']:
        information_lines.append(f'不判红：不是块设备的错误 {len(groups["unrelated"])} 行')
    red_lines = groups['host'] + groups['other']
    host_names = '、'.join(sorted(host_device_names)) or '（没认出）'
    if red_lines:
        shown = [f'宿主盘：{line[:200]}' for line in groups['host'][:15]]
        shown += [f'别的真盘或认不出设备：{line[:200]}' for line in groups['other'][:15]]
        return CategoryResult('宿主盘·内核日志', 'red',
                              f'{source_description} 里 {len(groups["host"])} 行落在宿主盘栈（{host_names}）上、'
                              f'{len(groups["other"])} 行是别的真盘或认不出设备的块设备错误',
                              shown + information_lines,
                              '逐行看列出的错误；是宿主盘的 I/O 错就停下交用户（带 --smart 再跑一次看盘自己的记录），'
                              '不在这块盘上继续跑测试；确认是测试设备而没认出来的，把设备名规则补进本脚本')
    return CategoryResult('宿主盘·内核日志', 'green',
                          f'{source_description} 共 {len(lines)} 行，没有落在宿主盘栈（{host_names}）上的块设备错误',
                          information_lines)


def judge_smart(environment, host_device_names):
    if not environment.smart_requested:
        return CategoryResult('宿主盘·SMART', 'skipped', '没给 --smart', [])
    status, headline, detail_lines = check_smart(host_device_names)
    return CategoryResult('宿主盘·SMART', status, headline, detail_lines,
                          '有异常就停下交用户；没查成的装 smartmontools 或 nvme-cli、'
                          '确认 .env 里有 SUDO_PASS_A / SUDO_PASS_B 再带 --smart 跑一次')


def collect_host_results(environment):
    mount_lines = read_mount_lines(environment)
    covering_pairs = [(path, covering_mount(path, mount_lines)) for path in host_target_paths(environment)]
    resolver = environment.host_device_resolver or resolve_host_devices
    host_device_names, host_major_minor_numbers = resolver(
        [mount_line for path, mount_line in covering_pairs if mount_line is not None])
    return [judge_readonly(covering_pairs), judge_free_space(environment, covering_pairs),
            judge_ext4_errors(environment, host_device_names),
            judge_kernel_log(environment, host_device_names, host_major_minor_numbers),
            judge_smart(environment, host_device_names)]


def print_category_result(result):
    if result.status == 'green':
        print(f'  ✓ {result.title}：{result.headline}')
    elif result.status == 'skipped':
        print(f'  - {result.title}：没跑（{result.headline}）')
    elif result.status == 'unchecked':
        print(f'  ! {result.title}：没查成（{result.headline}）')
    else:
        print(f'  ✗ {result.title}：{result.headline}')
    for detail_line in result.detail_lines:
        print(f'       {detail_line}')
    if result.status in ('red', 'unchecked'):
        print(f'     → 怎么办：{result.next_step}')


def summarize(results):
    red = [result.title for result in results if result.status == 'red']
    unchecked = [f'{result.title}（{result.headline}）' for result in results if result.status == 'unchecked']
    skipped = [f'{result.title}（{result.headline}）' for result in results if result.status == 'skipped']
    green_count = sum(1 for result in results if result.status == 'green')
    summary = (f'汇总：查了 {len(results)} 类；✓ {green_count} 类，判红 {len(red)} 类'
               f'{"（" + "、".join(red) + "）" if red else ""}，没查成 {len(unchecked)} 类'
               f'{"（" + "；".join(unchecked) + "）" if unchecked else ""}，没跑 {len(skipped)} 类'
               f'{"（" + "；".join(skipped) + "）" if skipped else ""}')
    exit_code = 1 if red else (3 if unchecked else 0)
    return summary, exit_code


def write_report(report_path, environment, results, entries, summary, exit_code):
    """另写一份 Markdown 给里程碑记录附上。排他新建：不盖掉已有的文件。"""
    now_epoch = time.time()
    status_words = {'green': '✓ 通过', 'red': '✗ 判红', 'unchecked': '! 没查成', 'skipped': '- 没跑'}
    lines = [f'# 测试环境检查 {format_time(now_epoch)}', '',
             f'- 命令：`python3 {SCRIPT_RELATIVE_PATH} check`，临时目录 {"、".join(environment.temporary_directories)}',
             f'- 退出码：{exit_code}', f'- {summary}', '',
             '| 类别 | 判定 | 一句话 |', '|---|---|---|']
    for result in results:
        lines.append(f'| {result.title} | {status_words[result.status]} | {result.headline.replace("|", "／")} |')
    lines += ['', '## 临时目录下的 singlefs-*', '',
              '| 路径 | 类型 | 实际占用 | 表观大小 | 最后修改 | 判定 | 为什么 |', '|---|---|---|---|---|---|---|']
    verdict_words = {'residual': '残留', 'in_use': '在用', 'allowlisted': '白名单',
                     'produced_by_this_run': '这一次跑自己产生的'}
    for entry in entries:
        lines.append(f'| `{entry.path}` | {entry.kind} | {format_bytes(entry.actual_bytes)} | '
                     f'{format_bytes(entry.apparent_bytes)} | {format_time(entry.latest_modification_epoch)} | '
                     f'{verdict_words[entry.verdict]} | {entry.reason.replace("|", "／")} |')
    if not entries:
        lines.append('| （没有） | | | | | | |')
    for result in results:
        if result.detail_lines or result.status in ('red', 'unchecked'):
            lines += ['', f'## {result.title}', '']
            lines += [f'- {detail_line}' for detail_line in result.detail_lines]
            if result.status in ('red', 'unchecked'):
                lines.append(f'- 怎么办：{result.next_step}')
    with open(report_path, 'x', encoding='utf-8') as handle:
        handle.write('\n'.join(lines) + '\n')


def run_check(environment, report_path=''):
    print(f'test-environment-check：check（临时目录 {"、".join(environment.temporary_directories)}）')
    device_scan = scan_devices(environment)
    entries, open_path_scan, unreadable_directories = scan_temporary_directories(
        environment.temporary_directories, environment.recent_seconds, environment.produced_after)
    results = judge_devices(device_scan)
    results.append(judge_temporary_entries(entries, open_path_scan, unreadable_directories,
                                           environment.temporary_directories))
    results += collect_host_results(environment)
    for result in results:
        print_category_result(result)
    summary, exit_code = summarize(results)
    print(f'  {summary}')
    if report_path:
        try:
            write_report(report_path, environment, results, entries, summary, exit_code)
            print(f'  报告写到 {report_path}')
        except FileExistsError:
            print(f'  ✗ 报告没写：{report_path} 已经存在，不盖掉它')
            print('     → 怎么办：换一个没用过的路径再跑一次 --report')
            return 2
        except OSError as error:
            print(f'  ✗ 报告没写：{report_path}（{error.strerror}）')
            print('     → 怎么办：确认目录存在、当前用户写得了，再跑一次')
            return 2
    return exit_code


# ── clean：只清前两类里判红的 ───────────────────────────────────────────

class RefusedDeletion(Exception):
    pass


def delete_temporary_entry(path, temporary_directories):
    """删一个临时目录下的 singlefs-* 条目，删完回读确认没了。不跟符号链接。"""
    parent = os.path.realpath(os.path.dirname(path))
    allowed_parents = {os.path.realpath(directory) for directory in temporary_directories}
    if not break_switch_is('deleteguard'):
        if parent not in allowed_parents or not os.path.basename(path).startswith(ENTRY_PREFIX):
            raise RefusedDeletion(f'{path} 不在临时目录 {"、".join(sorted(allowed_parents))} 下或不以 {ENTRY_PREFIX} 开头')
    if os.path.islink(path) or not os.path.isdir(path):
        os.unlink(path)
    else:
        shutil.rmtree(path)
    return not os.path.lexists(path)


@dataclasses.dataclass
class CleanAction:
    description: str
    device_command: list        # 要用 sudo 跑的命令；删临时条目时为空
    readback: object            # () -> bool，True 表示真的没了
    temporary_entry: object = None


def plan_clean(environment, device_scan, entries):
    actions = []
    for mount_line in sorted(device_scan.residual_mounts, key=lambda line: -len(line.mount_point)):
        mount_point = mount_line.mount_point
        actions.append(CleanAction(
            f'卸载 {mount_point}（{mount_line.source}）', ['umount', mount_point],
            lambda mount_point=mount_point: all(line.mount_point != mount_point
                                                for line in read_mount_lines(environment))))
    for device_name, target_name, why in device_scan.residual_device_mappers:
        name_path = os.path.join(environment.system_block_root, device_name, 'dm', 'name')
        actions.append(CleanAction(
            f'拆 device-mapper 目标 {target_name}（{device_name}，{why}）', ['dmsetup', 'remove', target_name],
            lambda name_path=name_path, target_name=target_name:
                (read_text_or_none(name_path) or '').strip() != target_name))
    for device_name, backing in device_scan.residual_loops:
        backing_path = os.path.join(environment.system_block_root, device_name, 'loop', 'backing_file')
        actions.append(CleanAction(
            f'摘 loop 设备 /dev/{device_name}（← {backing}）', ['losetup', '-d', f'/dev/{device_name}'],
            lambda backing_path=backing_path: not os.path.exists(backing_path)))
    for entry in entries:
        if entry.verdict != 'residual':
            continue
        actions.append(CleanAction(
            f'删{entry.kind} {entry.path}（实际 {format_bytes(entry.actual_bytes)}；{entry.reason}）', [],
            lambda path=entry.path: not os.path.lexists(path), temporary_entry=entry))
    return actions


def execute_clean(environment, actions):
    """逐步做、逐步回读；返回没做成的步数。"""
    failures = 0
    runner = environment.command_runner
    if any(action.device_command for action in actions) and runner is None:
        password = find_sudo_password()
        runner = make_sudo_runner(password) if password is not None else None
    fresh_open_path_scan = scan_open_paths(environment.temporary_directories)
    for action in actions:
        if action.device_command:
            if runner is None:
                print(f'  ✗ 没做：{action.description}——sudo 不通')
                print('     → 怎么办：确认仓根 .env 里有能用的 SUDO_PASS_A / SUDO_PASS_B，再跑一次 clean --yes')
                failures += 1
                continue
            return_code, output = runner(action.device_command)
            if return_code != 0 or not action.readback():
                print(f'  ✗ 没做成：{action.description}（退出码 {return_code}，回读仍在）{output[-200:]}')
                print('     → 怎么办：设备还被占着就先停掉占着它的进程或卸载它上面的挂载，再跑一次 clean --yes')
                failures += 1
                continue
            print(f'  ✓ 做了：{action.description}，回读确认没了')
            continue
        entry = action.temporary_entry
        try:
            recheck = classify_temporary_entry(entry.path, fresh_open_path_scan, time.time(),
                                               environment.recent_seconds)
        except FileNotFoundError:
            print(f'  - 跳过：{entry.path} 已经不在了')
            continue
        if recheck.verdict != 'residual':
            print(f'  - 跳过：{entry.path} 现在判成不是残留（{recheck.reason}）')
            continue
        try:
            is_gone = delete_temporary_entry(entry.path, environment.temporary_directories)
        except (RefusedDeletion, OSError) as error:
            print(f'  ✗ 没删：{entry.path}（{error}）')
            print('     → 怎么办：看上面的原因；路径不在临时目录下的是脚本的错，别手工绕过去删')
            failures += 1
            continue
        if not is_gone:
            print(f'  ✗ 删完回读还在：{entry.path}')
            print('     → 怎么办：看是不是别的进程又建了同名条目，查清再跑一次 clean --yes')
            failures += 1
            continue
        print(f'  ✓ 做了：{action.description}，回读确认没了')
    return failures


def run_clean(environment, confirmed):
    print(f'test-environment-check：clean（临时目录 {"、".join(environment.temporary_directories)}，'
          f'{"动手" if confirmed else "只打印计划"}）')
    device_scan = scan_devices(environment)
    entries, open_path_scan, unreadable_directories = scan_temporary_directories(
        environment.temporary_directories, environment.recent_seconds)
    actions = plan_clean(environment, device_scan, entries)
    freed_bytes = sum(action.temporary_entry.actual_bytes for action in actions if action.temporary_entry)
    print(f'  打算做 {len(actions)} 步，删临时条目能腾出实际 {format_bytes(freed_bytes)}：')
    for action in actions:
        command_text = f'  [sudo {" ".join(action.device_command)}]' if action.device_command else ''
        print(f'    · {action.description}{command_text}')
    for process_identifier, parent_identifier, command_text in device_scan.residual_emulators:
        print(f'    · 不自动处理：qemu 进程 pid {process_identifier}（父 pid {parent_identifier}）{command_text[:120]}；'
              f'确认不是正在跑的门禁或实验之后另起 kill {process_identifier}')
    kept = [entry for entry in entries if entry.verdict != 'residual']
    for entry in kept:
        print(f'    · 不碰：{entry.path}（{entry.reason}）')
    for directory_text in unreadable_directories:
        print(f'    · 没扫到：{directory_text}')
    # 不碰的那几项要在最后一行也报出来。逐条列在上面了，而读的人常常只看末尾那一行——
    # 「太新」这一档尤其要点名：它不是白名单，是这一次清不掉、下一次门禁开跑前又已经存在，
    # 于是 77 号必然判它红（清完紧接着跑门禁时撞过一次，白跑一轮全量）。
    too_recent = [entry for entry in kept if '可能正在跑' in entry.reason]
    kept_summary = (f'；不碰 {len(kept)} 项'
                    + (f'，其中 {len(too_recent)} 项只是太新（不到 {environment.recent_seconds // 60} 分钟没改），'
                       f'这一次清不掉、下一次门禁会判它们红：{"、".join(os.path.basename(entry.path) for entry in too_recent)}'
                       if too_recent else ''))
    if not confirmed:
        print(f'  没动任何东西{kept_summary}；看过上面这份清单、确认没有正在跑的活在用它们之后，加 --yes 再跑一次')
        return 0
    failures = execute_clean(environment, actions)
    print('  清完复查：')
    recheck_scan = scan_devices(environment)
    recheck_entries, recheck_open_scan, recheck_unreadable = scan_temporary_directories(
        environment.temporary_directories, environment.recent_seconds)
    recheck_results = judge_devices(recheck_scan) + [judge_temporary_entries(
        recheck_entries, recheck_open_scan, recheck_unreadable, environment.temporary_directories)]
    for result in recheck_results:
        print_category_result(result)
    still_red = [result.title for result in recheck_results if result.status == 'red']
    print(f'  clean 汇总：做了 {len(actions) - failures} / {len(actions)} 步，没做成 {failures} 步；'
          f'复查仍判红 {len(still_red)} 类{"（" + "、".join(still_red) + "）" if still_red else ""}'
          f'{kept_summary}')
    return 1 if failures or still_red else 0


# ── --selftest：在临时工作区里造样本，断言分类、clean 与宿主盘各项判红判绿 ─────────

SLEEPER_CODE = 'import sys, time\nhandle = open(sys.argv[1], "a")\nprint("ready", flush=True)\ntime.sleep(120)\n'


def start_holder_process(file_path, working_directory=None, argument_zero=None):
    """起一个开着 file_path 的子进程，等它报 ready。argument_zero 给了就把 argv[0] 换掉。"""
    command = [argument_zero or sys.executable, '-c', SLEEPER_CODE, file_path]
    process = subprocess.Popen(command, executable=sys.executable, stdout=subprocess.PIPE, text=True,
                               cwd=working_directory)
    ready_line = process.stdout.readline()
    if ready_line.strip() != 'ready':
        raise RuntimeError(f'持有文件的子进程没起来：{ready_line!r}')
    return process


def find_dead_process_identifier():
    """起一个马上退出的子进程，等它退出、回收，拿它的 pid：这个号此刻一定没有进程。"""
    process = subprocess.Popen([sys.executable, '-c', 'pass'])
    process.wait()
    if os.path.exists(f'/proc/{process.pid}'):
        raise RuntimeError(f'pid {process.pid} 刚退出又被复用了，重跑一次自检')
    return process.pid


def make_sparse_file(path, apparent_bytes):
    with open(path, 'wb') as handle:
        handle.truncate(apparent_bytes)
        handle.write(b'x' * 4096)


def set_tree_modification_time(path, epoch_seconds):
    for directory_path, directory_names, file_names in os.walk(path):
        for child_name in file_names + directory_names:
            os.utime(os.path.join(directory_path, child_name), (epoch_seconds, epoch_seconds), follow_symlinks=False)
    os.utime(path, (epoch_seconds, epoch_seconds), follow_symlinks=False)


class SelftestRecorder:
    def __init__(self):
        self.assertion_count = 0
        self.failures = []

    def expect(self, condition, description):
        self.assertion_count += 1
        if not condition:
            self.failures.append(description)


def build_temporary_samples(fake_temporary, outside_directory, holders):
    """造临时目录样本，返回 {名字: 期望判定}。holders 收集起的子进程，收尾时停掉。"""
    old_epoch = time.time() - 3600
    dead_identifier = find_dead_process_identifier()
    expectations = {}

    replay_dead = f'singlefs-replay-{dead_identifier}'
    os.mkdir(os.path.join(fake_temporary, replay_dead))
    with open(os.path.join(fake_temporary, replay_dead, 'E1.out'), 'w') as handle:
        handle.write('stale\n')
    set_tree_modification_time(os.path.join(fake_temporary, replay_dead), old_epoch)
    expectations[replay_dead] = 'residual'

    image_dead = f'singlefs-step67-sample-{dead_identifier}-0-dev0.img'
    make_sparse_file(os.path.join(fake_temporary, image_dead), 64 << 20)
    expectations[image_dead] = 'residual'

    anchor_holder = start_holder_process(os.path.join(outside_directory, 'anchor.txt'))
    holders.append(anchor_holder)
    image_alive = f'singlefs-step67-sample-{anchor_holder.pid}-0-dev1.img'
    make_sparse_file(os.path.join(fake_temporary, image_alive), 64 << 20)
    expectations[image_alive] = 'in_use'

    replay_reused = f'singlefs-replay-{anchor_holder.pid}'
    os.mkdir(os.path.join(fake_temporary, replay_reused))
    with open(os.path.join(fake_temporary, replay_reused, 'E2.out'), 'w') as handle:
        handle.write('stale\n')
    set_tree_modification_time(os.path.join(fake_temporary, replay_reused), old_epoch)
    expectations[replay_reused] = 'residual'

    for cache_name in ('singlefs-crates-mutation-target', 'singlefs-pdftext-1000'):
        os.mkdir(os.path.join(fake_temporary, cache_name))
        with open(os.path.join(fake_temporary, cache_name, 'cached'), 'w') as handle:
            handle.write('cache\n')
        set_tree_modification_time(os.path.join(fake_temporary, cache_name), old_epoch)
        expectations[cache_name] = 'allowlisted'

    mktemp_dead = 'singlefs-e129t.AbC123'
    os.mkdir(os.path.join(fake_temporary, mktemp_dead))
    make_sparse_file(os.path.join(fake_temporary, mktemp_dead, 'data.img'), 16 << 20)
    set_tree_modification_time(os.path.join(fake_temporary, mktemp_dead), old_epoch)
    expectations[mktemp_dead] = 'residual'

    mktemp_open = 'singlefs-e72.XyZ789'
    os.mkdir(os.path.join(fake_temporary, mktemp_open))
    make_sparse_file(os.path.join(fake_temporary, mktemp_open, 'd0.img'), 16 << 20)
    holders.append(start_holder_process(os.path.join(fake_temporary, mktemp_open, 'd0.img')))
    set_tree_modification_time(os.path.join(fake_temporary, mktemp_open), old_epoch)
    expectations[mktemp_open] = 'in_use'

    mktemp_recent = 'singlefs-e34.Rec123'
    os.mkdir(os.path.join(fake_temporary, mktemp_recent))
    with open(os.path.join(fake_temporary, mktemp_recent, 'progress'), 'w') as handle:
        handle.write('running\n')
    expectations[mktemp_recent] = 'in_use'

    os.symlink(outside_directory, os.path.join(fake_temporary, 'singlefs-link'))
    os.utime(os.path.join(fake_temporary, 'singlefs-link'), (old_epoch, old_epoch), follow_symlinks=False)
    expectations['singlefs-link'] = 'residual'

    os.mkdir(os.path.join(fake_temporary, 'other-directory'))
    return expectations


def check_temporary_samples(recorder, fake_temporary, outside_directory, expectations):
    entries, open_path_scan, unreadable_directories = scan_temporary_directories([fake_temporary], 600)
    verdicts = {os.path.basename(entry.path): entry for entry in entries}
    for name, expected_verdict in expectations.items():
        actual = verdicts[name].verdict if name in verdicts else '没扫到'
        recorder.expect(actual == expected_verdict,
                        f'临时条目 {name} 应判 {expected_verdict}，实际 {actual}'
                        f'（{verdicts[name].reason if name in verdicts else ""}）')
    recorder.expect('other-directory' not in verdicts, '不以 singlefs- 开头的 other-directory 不该被扫进来')
    recorder.expect(not unreadable_directories, f'样本临时目录应读得了：{unreadable_directories}')
    sparse_names = [name for name in verdicts if name.endswith('-dev0.img')]
    if sparse_names:
        sparse_entry = verdicts[sparse_names[0]]
        recorder.expect(sparse_entry.apparent_bytes == 64 << 20 and sparse_entry.actual_bytes < 1 << 20,
                        f'稀疏镜像应报实际占用而不是表观大小：实际 {sparse_entry.actual_bytes}，'
                        f'表观 {sparse_entry.apparent_bytes}')
    guard_sample = os.path.join(outside_directory, 'singlefs-guard-sample.txt')
    with open(guard_sample, 'w') as handle:
        handle.write('must survive\n')
    try:
        delete_temporary_entry(guard_sample, [fake_temporary])
        refused = False
    except RefusedDeletion:
        refused = True
    recorder.expect(refused and os.path.exists(guard_sample),
                    '临时目录之外的 singlefs-* 路径，删除函数必须拒绝且一个字节不动')
    return entries


def write_text(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w', encoding='utf-8') as handle:
        handle.write(text)


def build_device_samples(workspace, fake_temporary):
    """造假 sysfs 与假 mounts，返回 (sysfs 根, mounts 文件, ext4 根)。"""
    system_block_root = os.path.join(workspace, 'sys-block')
    resolved_temporary = os.path.realpath(fake_temporary)
    write_text(os.path.join(system_block_root, 'loop0', 'loop', 'backing_file'),
               f'{resolved_temporary}/singlefs-e72.XyZ789/d0.img\n')
    write_text(os.path.join(system_block_root, 'loop1', 'loop', 'backing_file'), '/var/lib/snapd/snaps/core_1.snap\n')
    os.makedirs(os.path.join(system_block_root, 'loop2'))
    write_text(os.path.join(system_block_root, 'dm-0', 'dm', 'name'), 'sfs_e129_tear\n')
    write_text(os.path.join(system_block_root, 'dm-1', 'dm', 'name'), 'vg0-root\n')
    write_text(os.path.join(system_block_root, 'dm-2', 'dm', 'name'), 'plain\n')
    os.makedirs(os.path.join(system_block_root, 'dm-2', 'slaves', 'loop0'))
    os.makedirs(os.path.join(system_block_root, 'nvme0n1'))
    mounts_path = os.path.join(workspace, 'mounts')
    write_text(mounts_path, '\n'.join([
        '/dev/nvme0n1p2 / ext4 rw,relatime 0 0',
        f'/dev/loop0 {workspace}/mnt-a ext4 rw 0 0',
        f'/dev/mapper/sfs_e129_tear {workspace}/mnt-b xfs rw 0 0',
        f'{resolved_temporary}/singlefs-e129t.AbC123/data.img {workspace}/mnt-c ext4 rw 0 0',
        f'tmpfs {resolved_temporary}/singlefs-e34.Rec123/mnt tmpfs rw 0 0',
        '/dev/loop1 /snap/core/1 squashfs ro 0 0']) + '\n')
    ext4_system_root = os.path.join(workspace, 'fs-ext4')
    write_text(os.path.join(ext4_system_root, 'nvme0n1p2', 'errors_count'), '0\n')
    return system_block_root, mounts_path, ext4_system_root


def make_fake_command_runner(environment, recorded_commands):
    def run(command):
        recorded_commands.append(list(command))
        if command[0] == 'umount':
            kept = [line for line in (read_text_or_none(environment.mounts_path) or '').splitlines()
                    if line.split()[1] != command[1]]
            write_text(environment.mounts_path, '\n'.join(kept) + '\n')
            return 0, ''
        if command[:2] == ['dmsetup', 'remove']:
            for device_name in os.listdir(environment.system_block_root):
                name_path = os.path.join(environment.system_block_root, device_name, 'dm', 'name')
                if (read_text_or_none(name_path) or '').strip() == command[2]:
                    shutil.rmtree(os.path.join(environment.system_block_root, device_name))
                    return 0, ''
            return 1, f'没有 {command[2]}'
        if command[:2] == ['losetup', '-d']:
            shutil.rmtree(os.path.join(environment.system_block_root, os.path.basename(command[2]), 'loop'))
            return 0, ''
        return 1, f'自检的假命令不认得 {command}'
    return run


def check_devices_and_clean(recorder, environment, holders, fake_temporary, outside_directory, expectations):
    emulator_with_image = start_holder_process(
        os.path.join(fake_temporary, 'singlefs-e72.XyZ789', 'd0.img'), argument_zero='qemu-system-x86_64')
    emulator_plain = start_holder_process(os.path.join(outside_directory, 'plain.img'),
                                          argument_zero='qemu-system-x86_64')
    holders += [emulator_with_image, emulator_plain]
    environment.process_identifier_filter = {emulator_with_image.pid, emulator_plain.pid}
    device_scan = scan_devices(environment)
    recorder.expect([name for name, backing in device_scan.residual_loops] == ['loop0'],
                    f'残留 loop 应只有 loop0，实际 {device_scan.residual_loops}')
    recorder.expect((device_scan.loop_devices_checked, device_scan.loop_devices_attached) == (3, 2),
                    f'应查 3 个 loop、2 个挂着文件，实际 {device_scan.loop_devices_checked}、'
                    f'{device_scan.loop_devices_attached}')
    recorder.expect([device for device, target, why in device_scan.residual_device_mappers] == ['dm-0', 'dm-2'],
                    f'残留 dm 应是 dm-0（前缀）与 dm-2（压在 loop0 上），实际 {device_scan.residual_device_mappers}')
    recorder.expect(len(device_scan.residual_mounts) == 4,
                    f'残留挂载应是 4 条，实际 {[line.mount_point for line in device_scan.residual_mounts]}')
    recorder.expect([emulator_entry[0] for emulator_entry in device_scan.residual_emulators] == [emulator_with_image.pid],
                    f'带 singlefs 的 qemu 应只抓到 pid {emulator_with_image.pid}，实际 {device_scan.residual_emulators}')
    for emulator in (emulator_with_image, emulator_plain):
        emulator.terminate()
        emulator.wait()

    recorded_commands = []
    environment.command_runner = make_fake_command_runner(environment, recorded_commands)
    dry_run_output = io.StringIO()
    with contextlib.redirect_stdout(dry_run_output):
        dry_run_exit = run_clean(environment, confirmed=False)
    all_present = all(os.path.lexists(os.path.join(fake_temporary, name)) for name in expectations)
    recorder.expect(dry_run_exit == 0 and not recorded_commands and all_present,
                    f'不带 --yes 的 clean 不许动任何东西：退出码 {dry_run_exit}，命令 {recorded_commands}')
    # 汇总那一行自己要报出「不碰几项、其中几项只是太新」：读的人常常只看末尾一行，而「太新」这一档
    # 这一次清不掉、下一次门禁开跑前又已经存在，77 号必然判它红（实测因此白跑一轮全量门禁）。
    dry_run_summary = next((line for line in dry_run_output.getvalue().split('\n') if '没动任何东西' in line), '')
    kept_count = sum(1 for verdict in expectations.values() if verdict != 'residual')
    recorder.expect(f'不碰 {kept_count} 项' in dry_run_summary,
                    f'clean 预演的汇总行要报不碰几项，实际：{dry_run_summary}')
    recorder.expect('只是太新' in dry_run_summary and 'singlefs-e34.Rec123' in dry_run_summary,
                    f'clean 预演的汇总行要把太新的那几项逐个点名，实际：{dry_run_summary}')
    clean_output = io.StringIO()
    with contextlib.redirect_stdout(clean_output):
        clean_exit = run_clean(environment, confirmed=True)
    recorder.expect(clean_exit == 0, f'clean --yes 应全做成、复查全绿，退出码 {clean_exit}：{clean_output.getvalue()[-600:]}')
    for name, expected_verdict in expectations.items():
        exists = os.path.lexists(os.path.join(fake_temporary, name))
        recorder.expect(exists == (expected_verdict != 'residual'),
                        f'clean --yes 之后 {name}（期望 {expected_verdict}）{"还在" if exists else "没了"}')
    recorder.expect(os.path.exists(os.path.join(outside_directory, 'anchor.txt')),
                    '删符号链接不许跟进去删它指向的目录')
    expected_commands = sorted([['umount', path] for path in (
        f'{os.path.dirname(environment.mounts_path)}/mnt-a', f'{os.path.dirname(environment.mounts_path)}/mnt-b',
        f'{os.path.dirname(environment.mounts_path)}/mnt-c',
        f'{os.path.realpath(fake_temporary)}/singlefs-e34.Rec123/mnt')] +
        [['dmsetup', 'remove', 'sfs_e129_tear'], ['dmsetup', 'remove', 'plain'], ['losetup', '-d', '/dev/loop0']])
    recorder.expect(sorted(recorded_commands) == expected_commands,
                    f'clean --yes 应发的设备命令对不上：{recorded_commands}')
    recorder.expect(recorded_commands[:4] == sorted(recorded_commands[:4], key=lambda command: -len(command[1]))
                    and all(command[0] == 'umount' for command in recorded_commands[:4])
                    and recorded_commands[-1][0] == 'losetup',
                    f'次序应是先卸载（深的先）、再拆 dm、最后摘 loop：{recorded_commands}')


KERNEL_LOG_PREFIX = '2026-09-10T06:05:39+0000 selftest-host kernel: '
TEST_DEVICE_LOG_LINES = [
    KERNEL_LOG_PREFIX + 'Buffer I/O error on dev dm-0, logical block 1, async page read',
    KERNEL_LOG_PREFIX + 'device-mapper: table: 252:0: thin-pool: Invalid block size (-EINVAL)',
    KERNEL_LOG_PREFIX + 'device-mapper: ioctl: error adding target to table',
    KERNEL_LOG_PREFIX + 'ACPI Error: AE_NOT_FOUND, While resolving a named reference package element',
]
HOST_DEVICE_LOG_LINES = [
    KERNEL_LOG_PREFIX + 'blk_update_request: I/O error, dev nvme0n1, sector 2048 op 0x1:(WRITE) flags 0x0',
    KERNEL_LOG_PREFIX + 'EXT4-fs error (device nvme0n1p2): ext4_find_entry:1455: inode #2: reading directory lblock 0',
    KERNEL_LOG_PREFIX + 'nvme nvme0: I/O 12 QID 3 timeout, aborting',
    KERNEL_LOG_PREFIX + 'sd 2:0:0:0: [sdz] tag#0 FAILED Result: hostbyte=DID_OK driverbyte=DRIVER_OK',
]


def host_environment(workspace, mounts_text, kernel_lines, space, ext4_errors='0'):
    """每次造一套独立的宿主盘输入；空临时目录、空 sysfs，只让宿主盘那几项有东西可判。"""
    case_directory = tempfile.mkdtemp(prefix='host-case-', dir=workspace)
    write_text(os.path.join(case_directory, 'mounts'), mounts_text)
    os.makedirs(os.path.join(case_directory, 'sys-block'))
    os.makedirs(os.path.join(case_directory, 'tmp'))
    write_text(os.path.join(case_directory, 'fs-ext4', 'nvme0n1p2', 'errors_count'), f'{ext4_errors}\n')
    return Environment(
        temporary_directories=[os.path.join(case_directory, 'tmp')],
        system_block_root=os.path.join(case_directory, 'sys-block'),
        mounts_path=os.path.join(case_directory, 'mounts'),
        ext4_system_root=os.path.join(case_directory, 'fs-ext4'),
        kernel_log_reader=lambda: (kernel_lines, '假内核日志') if kernel_lines is not None else (None, '假：权限'),
        host_device_resolver=lambda covering_mounts: ({'nvme0n1p2', 'nvme0n1', 'nvme0'}, {'259:2', '259:0'}),
        filesystem_space_reader=lambda path: space, device_mapper_major=252,
        process_identifier_filter=set()), case_directory


def check_host_disk(recorder, workspace):
    writable = '/dev/nvme0n1p2 / ext4 rw,relatime 0 0\n'
    readonly = '/dev/nvme0n1p2 / ext4 ro,relatime 0 0\n'
    roomy = (1000 * GIBIBYTE, 500 * GIBIBYTE)
    cases = [
        ('全部正常', writable, TEST_DEVICE_LOG_LINES, roomy, '0', {}, 0),
        ('根被改成只读', readonly, [], roomy, '0', {'宿主盘·只读挂载': 'red'}, 1),
        ('剩余低于 10%', writable, [], (1000 * GIBIBYTE, 60 * GIBIBYTE), '0', {'宿主盘·剩余空间': 'red'}, 1),
        ('剩余高于 10% 但低于 50 GiB（取严）', writable, [], (300 * GIBIBYTE, 40 * GIBIBYTE), '0',
         {'宿主盘·剩余空间': 'red'}, 1),
        ('ext4 记过错', writable, [], roomy, '3', {'宿主盘·ext4 错误计数': 'red'}, 1),
        ('内核日志读不到', writable, None, roomy, '0', {'宿主盘·内核日志': 'unchecked'}, 3),
        ('只读且日志读不到', readonly, None, roomy, '0',
         {'宿主盘·只读挂载': 'red', '宿主盘·内核日志': 'unchecked'}, 1),
    ]
    for host_line in HOST_DEVICE_LOG_LINES:
        cases.append((f'宿主盘栈上的错误：{host_line[len(KERNEL_LOG_PREFIX):][:40]}', writable,
                      TEST_DEVICE_LOG_LINES + [host_line], roomy, '0', {'宿主盘·内核日志': 'red'}, 1))
    for case_name, mounts_text, kernel_lines, space, ext4_errors, expected_statuses, expected_exit in cases:
        environment, case_directory = host_environment(workspace, mounts_text, kernel_lines, space, ext4_errors)
        results = collect_host_results(environment)
        statuses = {result.title: result.status for result in results}
        for title, status in statuses.items():
            wanted = expected_statuses.get(title, 'skipped' if title == '宿主盘·SMART' else 'green')
            recorder.expect(status == wanted, f'宿主盘样本「{case_name}」：{title} 应判 {wanted}，实际 {status}')
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            exit_code = run_check(environment)
        printed = output.getvalue().splitlines()
        recorder.expect(exit_code == expected_exit, f'宿主盘样本「{case_name}」：退出码应是 {expected_exit}，实际 {exit_code}')
        for line_number, line in enumerate(printed):
            if line.lstrip().startswith(('✗', '!')):
                following = printed[line_number + 1:]
                next_rejection = next((offset for offset, later in enumerate(following)
                                       if later.lstrip().startswith(('✗', '!', '✓', '-'))), len(following))
                recorder.expect(any('→ 怎么办' in later for later in following[:next_rejection]),
                                f'宿主盘样本「{case_name}」：拒绝行后面没有「→ 怎么办」：{line}')
    environment, case_directory = host_environment(workspace, writable, [], roomy)
    report_path = os.path.join(case_directory, 'report.md')
    with contextlib.redirect_stdout(io.StringIO()):
        first_exit = run_check(environment, report_path)
        second_exit = run_check(environment, report_path)
    report_text = read_text_or_none(report_path) or ''
    recorder.expect(first_exit == 0 and '| 宿主盘·只读挂载 | ✓ 通过 |' in report_text,
                    f'--report 应写出每类一行的表：退出码 {first_exit}')
    recorder.expect(second_exit == 2, f'--report 指向已有文件时应拒绝、不盖掉，退出码应是 2，实际 {second_exit}')


def run_selftest():
    break_switch = os.environ.get(BREAK_SWITCH_VARIABLE, '')
    if break_switch and break_switch not in BREAK_SWITCH_NAMES:
        print(f'  ✗ 认不出破坏开关 {BREAK_SWITCH_VARIABLE}={break_switch}')
        print(f'     → 怎么办：取 {" / ".join(BREAK_SWITCH_NAMES)} 之一，或不设它')
        return 2
    recorder = SelftestRecorder()
    holders = []
    # 工作区名字不以 singlefs- 开头：自检跑到一半被杀，也不会被当成测试残留
    workspace = tempfile.mkdtemp(prefix='test-environment-check-selftest-')
    try:
        fake_temporary = os.path.join(workspace, 'tmp')
        outside_directory = os.path.join(workspace, 'outside')
        os.mkdir(fake_temporary)
        os.mkdir(outside_directory)
        expectations = build_temporary_samples(fake_temporary, outside_directory, holders)
        check_temporary_samples(recorder, fake_temporary, outside_directory, expectations)
        system_block_root, mounts_path, ext4_system_root = build_device_samples(workspace, fake_temporary)
        environment = Environment(temporary_directories=[fake_temporary], system_block_root=system_block_root,
                                  mounts_path=mounts_path, ext4_system_root=ext4_system_root)
        check_devices_and_clean(recorder, environment, holders, fake_temporary, outside_directory, expectations)
        check_host_disk(recorder, workspace)
    finally:
        for holder in holders:
            if holder.poll() is None:
                holder.terminate()
                holder.wait()
            if holder.stdout is not None:
                holder.stdout.close()
        shutil.rmtree(workspace, ignore_errors=True)
    switch_note = f'（破坏开关 {break_switch}）' if break_switch else ''
    if recorder.failures:
        print(f'  ✗ --selftest 没通过{switch_note}：{len(recorder.failures)} / {recorder.assertion_count} 条断言不中')
        for failure in recorder.failures:
            print(f'       {failure}')
        print('     → 怎么办：按上面不中的那几条修本脚本的判据；设了破坏开关时判红是对的，没设时每一条都要中')
        return 1
    print(f'  ✓ --selftest 通过{switch_note}：{recorder.assertion_count} 条断言都中（临时条目分类、'
          '删除护栏、稀疏文件按实际占用、loop / dm / 挂载 / qemu、clean 预演与动手及次序、'
          '宿主盘只读 / 剩余空间 / ext4 计数 / 内核日志 / 退出码 / 报告不覆盖）')
    return 0


def build_argument_parser():
    parser = argparse.ArgumentParser(
        prog='test-environment-check.py',
        description='里程碑开工时清一次、完结时查一次：测试环境残留与宿主盘异常。')
    parser.add_argument('action', nargs='?', default='check', choices=('check', 'clean'),
                        help='check（默认）逐类判；clean 只清前两类判红的，不给 --yes 只打印计划')
    parser.add_argument('--selftest', action='store_true', help='在临时工作区里造样本自证会红')
    parser.add_argument('--temporary-directory', action='append', default=None,
                        help='要查的临时目录，可给多次；默认 /tmp，TMPDIR 设了且不同就再加上它')
    parser.add_argument('--report', default='', help='另写一份 Markdown 报告（排他新建，不盖掉已有文件）')
    parser.add_argument('--minimum-free-percent', type=float, default=10.0, help='剩余空间下限（百分比），默认 10')
    parser.add_argument('--produced-after', type=float, default=0.0,
                        help='这个时刻（epoch 秒）之后才出现的临时条目，单列成「这一次跑自己产生的」、不判红；'
                             '门禁 77 号把本次门禁的开跑时刻传进来（C448）')
    parser.add_argument('--minimum-free-gibibytes', type=float, default=50.0, help='剩余空间下限（GiB），默认 50；两条取严')
    parser.add_argument('--recent-minutes', type=float, default=10.0,
                        help='名字里没有 pid 的条目这么多分钟内还在改就当可能正在跑，默认 10')
    parser.add_argument('--kernel-log-since', default='',
                        help='只看这个时间之后的内核日志（传给 journalctl --since，按本机时钟 UTC）；默认本次开机以来')
    parser.add_argument('--smart', action='store_true', help='再用 sudo 跑 smartctl -H 或 nvme smart-log')
    parser.add_argument('--yes', action='store_true', help='clean 真的动手')
    return parser


def main(argument_list):
    parser = build_argument_parser()
    try:
        arguments = parser.parse_args(argument_list)
    except SystemExit as exit_signal:
        return 0 if exit_signal.code == 0 else 2
    if arguments.selftest:
        return run_selftest()
    if os.environ.get(BREAK_SWITCH_VARIABLE):
        print(f'  ✗ 设了 {BREAK_SWITCH_VARIABLE}：破坏开关只给 --selftest 用，check / clean 不许在它下面跑')
        print(f'     → 怎么办：unset {BREAK_SWITCH_VARIABLE} 再跑')
        return 2
    if arguments.yes and arguments.action != 'clean':
        print('  ✗ --yes 只对 clean 有意义')
        print(f'     → 怎么办：要清就跑 python3 {SCRIPT_RELATIVE_PATH} clean --yes')
        return 2
    temporary_directories = arguments.temporary_directory or default_temporary_directories()
    environment = Environment(temporary_directories=temporary_directories,
                              recent_seconds=int(arguments.recent_minutes * 60),
                              minimum_free_percent=arguments.minimum_free_percent,
                              minimum_free_gibibytes=arguments.minimum_free_gibibytes,
                              kernel_log_since=arguments.kernel_log_since, smart_requested=arguments.smart,
                              produced_after=arguments.produced_after)
    if arguments.action == 'clean':
        return run_clean(environment, arguments.yes)
    return run_check(environment, arguments.report)


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
