#!/usr/bin/env bash
# 核一份 qemu-io 的 blklogwrites 过滤节点能不能把多段 writev 完整记进日志。QEMU 8.2.2 原版写日志那一笔是「条目头 2 段 + 数据全部段」，
# 数据段数 ≥ 1023 时超过 IOV_MAX=1024，日志文件的 pwritev 回 EINVAL，来宾拿到 EIO；补丁 scripts/devices/qemu/blklogwrites-linearize-over-iov-max.patch，
# 装补好的版本用 scripts/devices/build-patched-qemu.sh（装完前拿它核一次），scripts/devices/vm-bench.sh 的 blklogwrites 一支开跑前也调它。
#
#   qemu-blklogwrites-check.sh <qemu-io 路径>   逐条跑 1022、1023、1024、2048 段，全过退 0；任一条不成立退 1，点名哪一条、哪一项不成立
#   qemu-blklogwrites-check.sh --selftest       不起真 qemu-io：用手造的日志与数据盘核每一项判法，再用假 qemu-io（原版与补好两种形状）走一遍整条路
#
# 每一条：临时目录里建 16 MiB 数据盘与日志盘，blklogwrites 配置与 vm-bench.sh 同一套（数据节点 cache.direct=on,aio=native，日志节点 driver=file，
# log-sector-size=512,log-append=off），一次 writev 交 N 个 4 KiB 段，每条用一个不同的填充字节。判五项：
#   qemu-io 退 0，输出里有 wrote N*4096/N*4096 那一行、没有 failed；
#   日志系统配置魔数、版本、扇区大小对，条数恰好 2（那一笔 WRITE 加 qemu-io 关盘时刷的 FLUSH）；
#   第 1 扇区是 WRITE 条目头：盘扇区 0、N*8 扇区（全零就是空洞：那一笔日志没写进去），它之后第 N*8+2 扇区是 FLUSH 条目头；
#   日志里那一条的数据与写进去的逐字节相同；
#   数据盘前 N*4096 字节与写进去的逐字节相同。
# 够不着的：qemu-io 一次 writev 的所有段填同一个字节，段之间次序错了看不出来。
# 临时目录在 ${TMPDIR:-/tmp} 下，数据节点要 O_DIRECT，TMPDIR 在 tmpfs 上时 qemu-io 打不开盘；跑完删掉，QEMU_BLKLOGWRITES_CHECK_KEEP=1 时留着并打印路径。
#
# 弄坏开关 QEMU_BLKLOGWRITES_CHECK_BREAK=<项>（只给证红用；--selftest 里点名的那一格判错）：
#   skip-qemu-io-status     不看 qemu-io 的退出码与 wrote 那一行 → 「qemu-io 报 writev failed，日志与数据盘都对」那一格
#   skip-system_configuration-count   不核系统配置条数                     → 「系统配置条数多一条」那一格
#   skip-entry-shape        不核两个条目头（含空洞）           → 「第一条条目头的盘扇区写成 8」「原版 1023 段的空洞」两格
#   skip-payload-compare    不比日志里那一条的数据             → 「日志数据错一个字节」那一格
#   skip-disk-compare       不比数据盘                         → 「数据盘错一个字节」那一格
#
# admission: always 判的是这一刻给的那个 qemu-io 二进制的行为，每次调都要现跑
# run-condition: command python3
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

die() {
  printf '  ✗ %s\n' "$1" >&2
  printf '     → 怎么办：%s\n' "$2" >&2
  exit 1
}

usage_howto="用法：qemu-blklogwrites-check.sh <qemu-io 路径>，或 qemu-blklogwrites-check.sh --selftest"
[[ $# -eq 1 ]] || die "要恰好一个参数，给了 $# 个" "$usage_howto"
case "$1" in
  --selftest) check_mode=selftest; qemu_io_path="" ;;
  -*) die "认不出的选项 $1" "$usage_howto" ;;
  *)
    check_mode=check; qemu_io_path="$1"
    [[ -x "$qemu_io_path" && ! -d "$qemu_io_path" ]] || die "$qemu_io_path 不是可执行文件" "给 qemu-io 的路径：系统的是 /usr/bin/qemu-io，补好的在 scripts/devices/build-patched-qemu.sh 装进的私有目录的 bin/qemu-io"
    ;;
esac

exec python3 - "$check_mode" "$qemu_io_path" <<'PYTHON'
import os
import shutil
import subprocess
import sys
import tempfile

IMAGE_WRITER_SOURCE = r'''
import struct

SECTOR_BYTES = 512
SEGMENT_BYTES = 4096
IMAGE_BYTES = 16 * 1024 * 1024
IOV_MAX = 1024
LOG_MAGIC = 0x6A736677736872
LOG_VERSION = 1
FLUSH_FLAG = 1
SYSTEM_CONFIGURATION_FORMAT = "<QQQI"
ENTRY_FORMAT = "<QQQQ"


def pad_to_sector(header_bytes):
    return header_bytes + bytes(SECTOR_BYTES - len(header_bytes))


def write_images(disk_path, log_path, segment_count, pattern_byte, log_shape, mutation=""):
    """Write a data disk and a dm-log-writes log the way blklogwrites leaves them after one writev and a close.

    log_shape "complete": system_configuration, WRITE entry with the data, FLUSH entry.
    log_shape "hole": what unpatched QEMU 8.2.2 leaves after EINVAL on the log write: the WRITE entry
    region stays zero, the FLUSH entry sits where it would have followed it, the system_configuration counts both.
    """
    data_bytes = segment_count * SEGMENT_BYTES
    data_sectors = data_bytes // SECTOR_BYTES
    payload = bytes([pattern_byte]) * data_bytes
    for path in (disk_path, log_path):
        with open(path, "wb") as image_file:
            image_file.truncate(IMAGE_BYTES)
    disk_payload = bytearray(payload)
    if mutation == "disk-byte-flip":
        disk_payload[data_bytes // 2] ^= 0xFF
    with open(disk_path, "r+b") as disk_file:
        disk_file.write(disk_payload)
    entry_count = 3 if mutation == "system_configuration-count-3" else 2
    with open(log_path, "r+b") as log_file:
        log_file.write(pad_to_sector(struct.pack(SYSTEM_CONFIGURATION_FORMAT, LOG_MAGIC, LOG_VERSION, entry_count, SECTOR_BYTES)))
        if log_shape == "complete":
            entry_disk_sector = 8 if mutation == "entry-sector-8" else 0
            log_file.seek(SECTOR_BYTES)
            log_file.write(pad_to_sector(struct.pack(ENTRY_FORMAT, entry_disk_sector, data_sectors, 0, 0)))
            log_payload = bytearray(payload)
            if mutation == "log-byte-flip":
                log_payload[data_bytes // 3] ^= 0xFF
            log_file.write(log_payload)
        log_file.seek((2 + data_sectors) * SECTOR_BYTES)
        log_file.write(pad_to_sector(struct.pack(ENTRY_FORMAT, 0, 0, FLUSH_FLAG, 0)))
'''
exec(IMAGE_WRITER_SOURCE)

FAKE_QEMU_IO_SOURCE = "#!/usr/bin/env python3\n" + IMAGE_WRITER_SOURCE + r'''
import os
import sys


def fake_main(arguments):
    shape = os.environ["FAKE_QEMU_IO_SHAPE"]
    if arguments == ["--version"]:
        print("qemu-io version 0.0.0 (selftest fake, %s)" % shape)
        return 0
    options = dict(item.split("=", 1) for item in arguments[arguments.index("--image-opts") + 1].split(","))
    command = arguments[arguments.index("-c") + 1].split()
    pattern_byte = int(command[2], 16)
    segment_count = len(command[4:])
    hole = shape == "unpatched" and segment_count + 2 > IOV_MAX
    write_images(options["file.filename"], options["log.filename"], segment_count, pattern_byte, "hole" if hole else "complete")
    if hole:
        print("writev failed: Invalid argument")
        return 1
    data_bytes = segment_count * SEGMENT_BYTES
    print("wrote %d/%d bytes at offset 0" % (data_bytes, data_bytes))
    return 0


sys.exit(fake_main(sys.argv[1:]))
'''

CASE_SEGMENT_COUNTS = (1022, 1023, 1024, 2048)
BREAK_ITEMS = ("skip-qemu-io-status", "skip-system_configuration-count", "skip-entry-shape", "skip-payload-compare", "skip-disk-compare")
QEMU_IO_TIMEOUT_SECONDS = 120
FIX_HOWTO = ("这份 qemu-io 的 blklogwrites 没打 scripts/devices/qemu/blklogwrites-linearize-over-iov-max.patch，或打了没生效；"
             "跑 scripts/devices/build-patched-qemu.sh 装补好的版本，再拿它装进的私有目录的 bin/qemu-io 重核")

break_item = os.environ.get("QEMU_BLKLOGWRITES_CHECK_BREAK", "")
if break_item and break_item not in BREAK_ITEMS:
    print("  ✗ QEMU_BLKLOGWRITES_CHECK_BREAK=%s 认不出" % break_item, file=sys.stderr)
    print("     → 怎么办：只认 %s；不证红就别设它" % "、".join(BREAK_ITEMS), file=sys.stderr)
    sys.exit(1)


def case_pattern_byte(case_index):
    return 0x41 + case_index


def first_difference(actual, expected):
    for offset, (actual_byte, expected_byte) in enumerate(zip(actual, expected)):
        if actual_byte != expected_byte:
            return "第 %d 字节是 0x%02x、该是 0x%02x" % (offset, actual_byte, expected_byte)
    return "长度 %d、该是 %d" % (len(actual), len(expected))


def judge_case(segment_count, pattern_byte, qemu_io_status, qemu_io_output, disk_path, log_path):
    """Return the list of problems for one case; empty means the case holds."""
    problems = []
    data_bytes = segment_count * SEGMENT_BYTES
    data_sectors = data_bytes // SECTOR_BYTES
    expected_payload = bytes([pattern_byte]) * data_bytes
    if break_item != "skip-qemu-io-status":
        wrote_line = "wrote %d/%d bytes at offset 0" % (data_bytes, data_bytes)
        if qemu_io_status != 0 or "failed" in qemu_io_output or wrote_line not in qemu_io_output:
            problems.append("qemu-io 退出码 %d、输出「%s」，要退 0 且有「%s」" % (qemu_io_status, " / ".join(qemu_io_output.split("\n")[:2]).strip(), wrote_line))
    with open(log_path, "rb") as log_file:
        system_configuration = log_file.read(SECTOR_BYTES)
        if len(system_configuration) < SECTOR_BYTES:
            return problems + ["日志只有 %d 字节，读不出系统配置" % len(system_configuration)]
        magic, version, entry_count, sector_size = struct.unpack_from(SYSTEM_CONFIGURATION_FORMAT, system_configuration)
        if (magic, version, sector_size) != (LOG_MAGIC, LOG_VERSION, SECTOR_BYTES):
            return problems + ["日志系统配置不对：魔数 0x%x、版本 %d、扇区 %d 字节，该是 0x%x、%d、%d" % (magic, version, sector_size, LOG_MAGIC, LOG_VERSION, SECTOR_BYTES)]
        if break_item != "skip-system_configuration-count" and entry_count != 2:
            problems.append("日志系统配置记了 %d 条，该是 2 条（WRITE 加关盘时的 FLUSH）" % entry_count)
        log_file.seek(SECTOR_BYTES)
        write_header = log_file.read(SECTOR_BYTES)
        logged_payload = log_file.read(data_bytes)
        flush_header = log_file.read(SECTOR_BYTES)
    if break_item != "skip-entry-shape":
        if write_header == bytes(SECTOR_BYTES):
            problems.append("日志第 1 扇区（第一条条目头）全零：空洞，那一笔日志没写进去")
        else:
            disk_sector, sector_count, flags, _ = struct.unpack_from(ENTRY_FORMAT, write_header)
            if (disk_sector, sector_count, flags) != (0, data_sectors, 0):
                problems.append("第一条条目头是盘扇区 %d、%d 扇区、标志 %d，该是 WRITE 盘扇区 0、%d 扇区" % (disk_sector, sector_count, flags, data_sectors))
        if len(flush_header) < SECTOR_BYTES:
            problems.append("日志在第 %d 扇区之前就完了，没有 FLUSH 条目头" % (2 + data_sectors))
        else:
            _, flush_sector_count, flush_flags, _ = struct.unpack_from(ENTRY_FORMAT, flush_header)
            if (flush_sector_count, flush_flags) != (0, FLUSH_FLAG):
                problems.append("第 %d 扇区不是 FLUSH 条目头（%d 扇区、标志 %d）" % (2 + data_sectors, flush_sector_count, flush_flags))
    if break_item != "skip-payload-compare" and logged_payload != expected_payload:
        problems.append("日志里那一条的数据与写进去的不同：%s" % first_difference(logged_payload, expected_payload))
    if break_item != "skip-disk-compare":
        with open(disk_path, "rb") as disk_file:
            disk_payload = disk_file.read(data_bytes)
        if disk_payload != expected_payload:
            problems.append("数据盘前 %d 字节与写进去的不同：%s" % (data_bytes, first_difference(disk_payload, expected_payload)))
    return problems


def image_options(disk_path, log_path):
    return ("driver=blklogwrites,file.driver=file,file.filename=%s,file.cache.direct=on,file.aio=native,"
            "log.driver=file,log.filename=%s,log-sector-size=512,log-append=off") % (disk_path, log_path)


def run_cases(qemu_io_path, work_directory, extra_environment):
    """Run every case against qemu_io_path; return [(segment_count, problems)] in case order."""
    environment = dict(os.environ, **extra_environment)
    results = []
    for case_index, segment_count in enumerate(CASE_SEGMENT_COUNTS):
        pattern_byte = case_pattern_byte(case_index)
        case_directory = os.path.join(work_directory, "segments-%d" % segment_count)
        os.mkdir(case_directory)
        disk_path = os.path.join(case_directory, "disk.img")
        log_path = os.path.join(case_directory, "log.img")
        for path in (disk_path, log_path):
            with open(path, "wb") as image_file:
                image_file.truncate(IMAGE_BYTES)
        command = "writev -P 0x%02x 0 %s" % (pattern_byte, " ".join(["4k"] * segment_count))
        try:
            completed = subprocess.run([qemu_io_path, "--image-opts", image_options(disk_path, log_path), "-c", command],
                                       stdin=subprocess.DEVNULL, capture_output=True, text=True,
                                       timeout=QEMU_IO_TIMEOUT_SECONDS, env=environment)
            qemu_io_status, qemu_io_output = completed.returncode, completed.stdout + completed.stderr
        except subprocess.TimeoutExpired:
            qemu_io_status, qemu_io_output = -1, "%d 秒没退出" % QEMU_IO_TIMEOUT_SECONDS
        results.append((segment_count, judge_case(segment_count, pattern_byte, qemu_io_status, qemu_io_output, disk_path, log_path)))
    return results


def print_results(results):
    for segment_count, problems in results:
        if problems:
            print("  ✗ %d 段（%d 字节）：%s" % (segment_count, segment_count * SEGMENT_BYTES, "；".join(problems)))  # gate-lint:detail
        else:
            print("  ✓ %d 段（%d 字节）：qemu-io 写成功，日志恰好 WRITE 与 FLUSH 两条，日志数据与数据盘都与写进去的逐字节相同" % (segment_count, segment_count * SEGMENT_BYTES))


def qemu_io_version(qemu_io_path, extra_environment):
    try:
        completed = subprocess.run([qemu_io_path, "--version"], stdin=subprocess.DEVNULL, capture_output=True, text=True,
                                   timeout=QEMU_IO_TIMEOUT_SECONDS, env=dict(os.environ, **extra_environment))
    except subprocess.TimeoutExpired:
        return "（--version 没退出）"
    return (completed.stdout.split("\n") + [""])[0].strip() or "（--version 没有输出）"


def make_work_directory():
    return tempfile.mkdtemp(prefix="singlefs-blklogwrites-check.", dir=os.environ.get("TMPDIR") or "/tmp")


def check(qemu_io_path):
    work_directory = make_work_directory()
    if "," in work_directory:
        print("  ✗ 临时目录 %s 里有逗号，--image-opts 写不进去" % work_directory)
        print("     → 怎么办：把 TMPDIR 换成路径里没有逗号的目录再跑")
        shutil.rmtree(work_directory)
        return 1
    print("  qemu-io：%s（%s）" % (qemu_io_path, qemu_io_version(qemu_io_path, {})))
    try:
        results = run_cases(qemu_io_path, work_directory, {})
    finally:
        if os.environ.get("QEMU_BLKLOGWRITES_CHECK_KEEP") == "1":
            print("  ! 现场留着：%s" % work_directory)
        else:
            shutil.rmtree(work_directory)
    print_results(results)
    red_counts = [str(segment_count) for segment_count, problems in results if problems]
    if red_counts:
        print("  ✗ %d 条里 %d 条判红（%s 段）" % (len(results), len(red_counts), "、".join(red_counts)))
        print("     → 怎么办：%s" % FIX_HOWTO)
        return 1
    print("  ✓ %d 条全过（%s 段）：这份 qemu-io 的 blklogwrites 把超过 IOV_MAX 的多段写完整记进了日志" % (len(results), "、".join(str(count) for count in CASE_SEGMENT_COUNTS)))
    return 0


def selftest():
    work_directory = make_work_directory()
    judged_right = 0
    judged_wrong = []
    try:
        direct_cases = (
            # (格名, 段数, 日志形状, 手造的坏处, qemu-io 退出码, qemu-io 输出, 该红, 红的说明里要有的字, 弄坏哪个开关这一格判错)
            ("补好的形状、1023 段", 1023, "complete", "", 0, "wrote 4190208/4190208 bytes at offset 0", False, "", ""),
            ("qemu-io 报 writev failed，日志与数据盘都对", 1022, "complete", "", 1, "writev failed: Invalid argument", True, "qemu-io", "skip-qemu-io-status"),
            ("系统配置条数多一条", 1022, "complete", "system_configuration-count-3", 0, "wrote 4186112/4186112 bytes at offset 0", True, "系统配置", "skip-system_configuration-count"),
            ("第一条条目头的盘扇区写成 8", 1022, "complete", "entry-sector-8", 0, "wrote 4186112/4186112 bytes at offset 0", True, "第一条条目头", "skip-entry-shape"),
            ("原版 1023 段的空洞", 1023, "hole", "", 0, "wrote 4190208/4190208 bytes at offset 0", True, "空洞", "skip-entry-shape"),
            ("日志数据错一个字节", 1022, "complete", "log-byte-flip", 0, "wrote 4186112/4186112 bytes at offset 0", True, "日志里那一条的数据", "skip-payload-compare"),
            ("数据盘错一个字节", 1022, "complete", "disk-byte-flip", 0, "wrote 4186112/4186112 bytes at offset 0", True, "数据盘", "skip-disk-compare"),
        )
        for case_number, (name, segment_count, log_shape, mutation, qemu_io_status, qemu_io_output, should_be_red, keyword, guarding_break) in enumerate(direct_cases):
            disk_path = os.path.join(work_directory, "direct-%d-disk.img" % case_number)
            log_path = os.path.join(work_directory, "direct-%d-log.img" % case_number)
            write_images(disk_path, log_path, segment_count, 0x5A, log_shape, mutation)
            problems = judge_case(segment_count, 0x5A, qemu_io_status, qemu_io_output, disk_path, log_path)
            joined = "；".join(problems)
            right = (bool(problems) and keyword in joined) if should_be_red else not problems
            if right:
                judged_right += 1
                print("  ✓ %s：%s" % (name, ("判红（%s）" % joined) if problems else "判绿"))
            else:
                judged_wrong.append(name)
                print("  ✗ %s：该%s，判成「%s」" % (name, "红且说明里有「%s」" % keyword if should_be_red else "绿", joined or "绿"))  # gate-lint:detail
                if guarding_break:
                    print("     → QEMU_BLKLOGWRITES_CHECK_BREAK=%s 设着的话这一格本来就该判错；没设就看 judge_case 里那一项判法" % guarding_break)
        fake_qemu_io_path = os.path.join(work_directory, "fake-qemu-io")
        with open(fake_qemu_io_path, "w") as fake_file:
            fake_file.write(FAKE_QEMU_IO_SOURCE)
        os.chmod(fake_qemu_io_path, 0o755)
        for shape, expected_red in (("unpatched", ["1023", "1024", "2048"]), ("patched", [])):
            run_directory = os.path.join(work_directory, "fake-" + shape)
            os.mkdir(run_directory)
            results = run_cases(fake_qemu_io_path, run_directory, {"FAKE_QEMU_IO_SHAPE": shape})
            red = [str(segment_count) for segment_count, problems in results if problems]
            name = "假 qemu-io（%s）走整条路" % ("原版形状" if shape == "unpatched" else "补好的形状")
            if red == expected_red and len(results) == len(CASE_SEGMENT_COUNTS):
                judged_right += 1
                print("  ✓ %s：%d 条，判红的是 %s" % (name, len(results), "、".join(red) + " 段" if red else "无"))
            else:
                judged_wrong.append(name)
                print("  ✗ %s：%d 条，判红的是 %s，该是 %s" % (name, len(results), red or "无", expected_red or "无"))  # gate-lint:detail
                print("     → 怎么办：先看上面直接核判法的几格哪一格判错，修 judge_case；几格都对就看 run_cases 起 qemu-io 的那几行")
    finally:
        shutil.rmtree(work_directory)
    total = judged_right + len(judged_wrong)
    if judged_wrong:
        print("  ✗ 自证 %d 格里 %d 格判错：%s" % (total, len(judged_wrong), "、".join(judged_wrong)))
        print("     → 怎么办：看上面每一格判错的说明；设了 QEMU_BLKLOGWRITES_CHECK_BREAK 的话，判错的应当正是它点名的那一格")
        return 1
    print(f"  ✓ 自证 {judged_right + len(judged_wrong)} 格全部判对（手造日志 {len(direct_cases)} 格、假 qemu-io 两种形状各走一遍）")
    return 0


check_mode, qemu_io_path = sys.argv[1], sys.argv[2]
sys.exit(selftest() if check_mode == "selftest" else check(qemu_io_path))
PYTHON
