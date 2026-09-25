#!/usr/bin/env bash
# gate-stage: crates 变异表复跑（每条改坏一处、点名的测试必须红）
#
# 判据：`crates/mutations.tsv` 里每一条变异（文件、原文、替换文、cargo test 参数、必须红的测试名），
# 原文在文件里恰好命中一次，文件落在每片拷的范围之内（COPIED_INTO_EACH_SHARD，派活之前判）；
# 把仓拷到临时目录、改坏那一处、跑点名的测试，那条测试必须判红；跑完还原再下一条。
# 一条锚点腐化、一条指到拷贝范围之外、一条没红，整道红。
#
# 为什么：show-me-test.md 要「存进仓的变异清单，交给门禁反复复跑」——commit message 里的叙述只被读一次。
# 2026-09-16 发布 B 三方第二轮攻方腿把「空闲独立维护」那处改法整个撤回，全仓零判红零警告：
# 改法本身没有任何东西守着。这张表让每一处三方打中之后的改法都留一条「改回去它就红」的变异，撤回时这里先响。
#
# 分片并发跑（command-safety.md「一个脚本里的检测项，能并行就并行」）：每条变异都要起 cargo 编译 + 跑测试，
# 彼此不依赖，而串行时一条约 8 秒、191 条约 25 分钟。并发的障碍是同一篇里写着的「共用一份可写状态」——
# 所有变异改同一份源码副本、共用一个 CARGO_TARGET_DIR（cargo 的文件锁会把它们重新串行化）。
# 所以每个分片各给一份源码副本与一个编译目录。分片按 crate 切：改 singlefs-core 要重编译它自己加全部下游，
# 改 singlefs-harness 只重编译它自己，同一分片里连着改同一个 crate，增量编译的命中率才稳。
#
# 工作进程数取下面两项的最小值、至少 1（worker_count_for）；实际开几个、各项给几个、卡在哪一项，打在 stderr 的头一行（stdout 要与进程数无关，见下一段）：
#   ① 核数的一半、至多 16；设了 GATE_MUTATION_WORKERS 就用它顶掉 ①（设成 1 就是原来的串行跑法）；
#   ② 表里的条数。
#   内存不在这里收：每条经 research/scripts/run-with-memory-cap.sh 跑，它起跑之前先判整机放不放得下（slice 已占的加这一条要的，不超过 slice 的总上限才起），
#   放不下的在它那里排队，几条合起来撞顶也只在 slice 里杀（records/2026-09-16-subagent拆分提案.md 第四十节第 30 行）。按可用内存收进程数只看得见开跑那一刻，
#   看不见别的重活同时来抢，治标不治本（用户 2026-09-25 定）。这一道自己在包装之外的进程（Python 父进程与工作进程、包装的 bash）不在 slice 里，
#   从包装的余量里出：2026-09-25 量进程池工作进程每个约 12 MiB、父进程约 15 MiB。
# 活是**动态领的**（谁先跑完谁再领下一条），不按条数预先切片：`singlefs-harness` 那几条点名的是几十秒的重测试，
# 等分之下拿到它们的那一片会独自拖住整道（实测：7 片跑完、最后 1 片又跑了 4 分钟）。
# ⚠️ **输出与进程数无关**：判定行按变异表的行号排序再打印到 stdout，GATE_MUTATION_WORKERS=1 与 =8 的 stdout 要逐字相同，
# 这一条是并发化之后重新证明它还红得出来的那一半（command-safety.md「改成并行之后要重新证明它红得出来」）。
# 进度实时打到 stderr，不进 stdout，不参与那条比对。
#
# 编译产物放 ${GATE_MUTATION_TARGET_DIR:-${TMPDIR:-/tmp}/singlefs-crates-mutation-target}（跨轮复用，第一次要整编一遍）；
# 分片各自的编译目录是它加 -w<片号>，第一次从它拷一份种子，省掉每片各冷编译一遍。
#
# 每条变异的 cargo test 放进内存上限里跑（research/scripts/run-with-memory-cap.sh：systemd 的临时 scope，MemoryMax=<上限>、MemorySwapMax=0），
# 撞上限只杀这一条的进程，不把整机拖进 OOM（2026-09-25 一条无界分配的变异两次把整机拖进 OOM，records/2026-09-16-subagent拆分提案.md 第四十节第 28 行）。
#   上限从三处取，先到先用：变异表里单起一行「# 每条变异的内存上限：<上限>」（判别力样本用它把上限压到 512M）、GATE_MUTATION_MEMORY_MAX、默认 4G。
#   默认 4G 的依据：本机 60 GiB 内存，本地模型服务（vllm 与 ray）连同会话常驻约 12 GiB（2026-09-25 `free -g` 的 used 列是 12），
#   同时可能有 3 件重活 ⇒ 每件 16 GiB；这一道同时开好几个工作进程，4G 给正常的编译与测试留余量。2026-09-25 量过编译那一半：
#   HEAD 的四个 crate 全部测试目标在 4G 上限、2 个并行编译下从零编得过，匿名内存峰值 0.60 GiB（每 0.2 秒取一次样）；点名的测试跑起来要多少没量，
#   包装的峰值表（research/scripts/memory-peaks.tsv）跑过一遍之后按条记着。换机器要重算（这些数只在本机成立）。
#   几条同时跑合起来放不放得下由包装排队判、slice 的总上限兜底，这里不按内存收进程数。
#   撞了这一条自己的上限（包装退 250）记「内存撞顶」：这条破坏让被测代码无界分配，点名的测试没来得及红，不算抓到也不算没红，单列一栏、整道判红。
#   被总上限挤掉（包装退 254：整个 slice 满了、内核在 slice 里挑了这一条杀）记「被总上限挤掉」，排队等满包装的等待上限还放不下（包装退 252）记「排不上没跑」：
#   两栏都不是这条变异的结论，单列、整道判红，出路是重跑。
#   带上限的 scope 或 slice 的总上限起不来、设不上（没有用户级 systemd、D-Bus 连不上）就整道判红，一条都不跑，不退回无上限去跑。
#
# 每条变异的 cargo test 限时（交给包装：RUN_WITH_MEMORY_CAP_TIME_LIMIT，给 scope 设 RuntimeMaxSec，从起跑算、不算排队的时间；限时里含这一条要做的重编译）：
#   秒数从三处取，先到先用：变异表里单起一行「# 每条变异的超时秒数：<秒>」（判别力样本用它压到 20 秒）、GATE_MUTATION_TIMEOUT、默认 1800。
#   到点 systemd 给 scope 里每个进程（cargo 与测试进程）发 TERM，TIMEOUT_KILL_GRACE_SECONDS 秒还不退再发 KILL，包装按 scope 的 Result=timeout 退 253。
#   不在包装外面套 timeout：排队的时间会算进限时里，排得久的变异会被误判成超时。
#   超过限时记「超时」：这条破坏让被测代码不终止了，或者点名的测试在并发下就要跑这么久。不算抓到、不算没红，与「内存撞顶」「没红」「无效」分开单列一栏、各自计数，整道判红。
#   默认 1800 秒的依据：点名随机历史测试的变异在 16 个工作进程并发时，一条卡在那个测试文件上超过 6–8 分钟（没记到跑完的挂钟），串行单次 1.5–3 分钟
#   （.claude/kb/checks-owed.md 的 C457（被测试自己的线程池与外层进程池叠加超订阅），2026-09-21 在副本上量的）；四个 crate 的全部测试目标在 4G 上限、
#   2 个并行编译下冷编译 23–25 秒（2026-09-25 量）。1800 秒约是 8 分钟的 4 倍，给没量到的尾巴留余地；代价是一条真挂死的变异让一个工作进程多占 30 分钟。
#   这些数只在本机成立，换机器要重算。
#
# 判别力：fixtures/59-crates-mutation-replay.sh/red 的变异表把上限压到 512M、限时压到 20 秒：加一条把 1 MiB 的缓冲改成 1536 MiB 的变异，必须报「内存撞顶」；
#   加一条把步长 1 改成 0、循环永远走不到上界的变异，必须报「超时」；计数行里没红、无效、内存撞顶、超时各数各的。
# green 同样压到 512M，留着一行撤掉了的「# 给整机留的内存余量：1T」：头一行必须报开 2 个（两条、核数的一半多于 2）、卡在「表里的条数」那一项，
#   不许再按内存收进程数（按内存收的那一版在这份样本上只开 1 个）；两条正常变异必须照旧红在点名的测试上。
#
#   bash .claude/gate.d/59-crates-mutation-replay.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
# 先问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
# 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
# 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
# 这一道原先没有任何范围判定，每趟跑满：2026-09-22 实测，一批 27 个路径里 crates/ 零个，它照跑不误。
reuse_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "$ROOT" "$(basename "$0")")"
reuse_rc=$?
if [[ "$reuse_rc" != 0 ]]; then
  echo "  ! 本阶段跳过（复用上一次整轮全绿的判定）：$reuse_reason"
  echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。"
  exit 77
fi

TABLE="crates/mutations.tsv"
if [[ ! -f "$TABLE" ]]; then
  echo "  ✗ 没有 $TABLE"
  echo "     → 怎么办：建一张六段制表符分隔的表（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），每一处三方打中之后的改法留一条。"
  exit 1
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "  ✗ 没有 cargo，变异复跑不了"
  echo "     → 怎么办：装 Rust 工具链（scripts/env.sh 会报），再跑这一道。"
  exit 1
fi
export GATE_MUTATION_TARGET_DIR="${GATE_MUTATION_TARGET_DIR:-${TMPDIR:-/tmp}/singlefs-crates-mutation-target}"
# 带内存上限跑一条命令的包装在这份阶段所在的仓里（样本仓里没有 research/）
GATE_MUTATION_MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
export GATE_MUTATION_MEMORY_CAP_RUNNER
python3 - "$ROOT" "$TABLE" <<'PY'
import atexit
import concurrent.futures
import fnmatch
import multiprocessing
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

root, table = sys.argv[1], sys.argv[2]
memory_cap_runner = os.environ["GATE_MUTATION_MEMORY_CAP_RUNNER"]
MEMORY_CAP_HIT_EXIT = 250            # run-with-memory-cap.sh：撞了这一条自己的上限（scope 的 Result 是 oom-kill、自己的 oom 计数不为 0）
MEMORY_CAP_UNAVAILABLE_EXIT = 251    # run-with-memory-cap.sh：带上限的 scope 或 slice 的总上限起不来
MEMORY_ADMISSION_REFUSED_EXIT = 252  # run-with-memory-cap.sh：排队等满还放不下，命令没跑
TIME_LIMIT_HIT_EXIT = 253            # run-with-memory-cap.sh：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT（scope 的 Result 是 timeout）
SLICE_TOTAL_HIT_EXIT = 254           # run-with-memory-cap.sh：被 slice 的总上限挤掉（整个 slice 满了，内核挑了这一条杀）
DEFAULT_MEMORY_MAX_PER_MUTATION = "4G"   # 依据见文件头
MEMORY_MAX_DIRECTIVE = re.compile(r"^#\s*每条变异的内存上限[：:]\s*(\S+)\s*$")
DEFAULT_TIMEOUT_SECONDS_PER_MUTATION = "1800"   # 依据见文件头
TIMEOUT_DIRECTIVE = re.compile(r"^#\s*每条变异的超时秒数[：:]\s*(\S+)\s*$")
TIMEOUT_KILL_GRACE_SECONDS = 30          # 到点发 TERM 之后再等这么久，还不退就发 KILL（交给包装的 RUN_WITH_MEMORY_CAP_KILL_GRACE）


def unescape(text):
    return text.replace("\\n", "\n")


rows = []
memory_max_from_table = None
timeout_seconds_from_table = None
with open(os.path.join(root, table), encoding="utf-8") as handle:
    for line_number, line in enumerate(handle, 1):
        line = line.rstrip("\n")
        directive = MEMORY_MAX_DIRECTIVE.match(line)
        if directive:
            memory_max_from_table = directive.group(1)
        directive = TIMEOUT_DIRECTIVE.match(line)
        if directive:
            timeout_seconds_from_table = directive.group(1)
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 6 or any(field == "" for field in fields):
            print(f"  ✗ {table}:{line_number} 不是六段（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名）：{line[:80]}")
            print("     → 怎么办：六段用制表符分隔，一段都不能空；原文 / 替换文里的换行写成 \\n。")
            sys.exit(1)
        rows.append((line_number, fields[0], fields[1], unescape(fields[2]), unescape(fields[3]), fields[4], fields[5]))
if not rows:
    print(f"  ✗ {table} 里一条成形的变异都没有")
    print("     → 怎么办：至少一条：三方打中之后的每一处改法，留一条「改回去它就红」的变异。")
    sys.exit(1)
memory_max = memory_max_from_table or os.environ.get("GATE_MUTATION_MEMORY_MAX", "").strip() or DEFAULT_MEMORY_MAX_PER_MUTATION
if not re.fullmatch(r"[1-9][0-9]*[KMGT]?", memory_max):
    print(f"  ✗ 每条变异的内存上限写成了「{memory_max}」，不是 systemd 的写法")
    print("     → 怎么办：写成 正整数[K|M|G|T]，例 4G、512M（表头那一行「# 每条变异的内存上限：…」或 GATE_MUTATION_MEMORY_MAX）。")
    sys.exit(1)
timeout_seconds_text = timeout_seconds_from_table or os.environ.get("GATE_MUTATION_TIMEOUT", "").strip() or DEFAULT_TIMEOUT_SECONDS_PER_MUTATION
if not re.fullmatch(r"[1-9][0-9]*", timeout_seconds_text):
    print(f"  ✗ 每条变异的限时写成了「{timeout_seconds_text}」，不是正整数秒")
    print("     → 怎么办：写成正整数秒，例 1800（表头那一行「# 每条变异的超时秒数：…」或 GATE_MUTATION_TIMEOUT）。")
    sys.exit(1)
timeout_seconds = int(timeout_seconds_text)
stale = []
for line_number, name, path, old, new, _args, _expected in rows:
    full = os.path.join(root, path)
    if not os.path.isfile(full):
        stale.append(f"{table}:{line_number} {name}：文件 {path} 不存在")
        continue
    count = open(full, encoding="utf-8").read().count(old)
    if count != 1:
        stale.append(f"{table}:{line_number} {name}：原文在 {path} 里命中 {count} 次，要恰好 1 次")
if stale:
    print("  ✗ 变异表的锚点腐化：")
    for item in stale:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：改代码时把变异表里的原文一起改到今天的写法（锚点腐化的那条变异等于没跑过，mutation-sampling.md 第七类）。")
    sys.exit(1)

COPIED_INTO_EACH_SHARD = ["crates", "Cargo.toml", "Cargo.lock", "litmus", "research/results"]
# 拷目录时跳过的名字（各 crate 自己的编译目录）。拷贝范围由这两样一起定：prepare_shard 照它们拷，下面那道路径检查照它们判。
IGNORED_WHEN_COPYING_INTO_EACH_SHARD = ["target"]
shared_target_directory = os.environ["GATE_MUTATION_TARGET_DIR"]


def is_copied_into_each_shard(path):
    """表里「文件」那一段在不在每片的源码副本里：落在 COPIED_INTO_EACH_SHARD 的某一项之内，
    而且那一项底下的每一段路径都不叫 IGNORED_WHEN_COPYING_INTO_EACH_SHARD 里的名字。绝对路径、用 `..` 跳出去的都不在。"""
    normalized = os.path.normpath(path)
    if os.path.isabs(normalized):
        return False
    for item in COPIED_INTO_EACH_SHARD:
        if normalized == item:
            return True
        if normalized.startswith(item + os.sep):
            below_item = normalized[len(item) + 1:].split(os.sep)
            return not any(fnmatch.fnmatch(part, pattern) for part in below_item for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD)
    return False


# 锚点只在真仓上核过：文件落在拷贝范围之外时，工作进程在自己那份副本里打不开它，要到派活之后才炸成一个没有出路的 traceback。
# 所以派活之前先判，有一行在外面就一个工作进程都不起。
rows_outside_shard_copy = [f"{table}:{line_number} {name}：{path}"
                           for line_number, name, path, _old, _new, _args, _expected in rows if not is_copied_into_each_shard(path)]
if rows_outside_shard_copy:
    print("  ✗ 变异表有几行的文件不在每片拷的范围里（锚点在真仓上核得过，工作进程在自己的源码副本里却打不开它）：")
    for item in rows_outside_shard_copy:
        print(f"      {item}")   # gate-lint:detail
    print(f"     → 怎么办：把那个路径（或它所在的目录）加进本阶段的 COPIED_INTO_EACH_SHARD（现在是 {'、'.join(COPIED_INTO_EACH_SHARD)}），"
          "或者把这条变异改到拷贝范围之内的文件上；这一轮一个工作进程都没起。")
    sys.exit(1)

# 派活之前先试一次带上限的 scope 起不起得来：起不来就一条都不跑，不退回无上限
memory_cap_check = subprocess.run(["bash", memory_cap_runner, "--check", memory_max], capture_output=True, text=True)
if memory_cap_check.returncode != 0:
    print(f"  ✗ 带内存上限（每条 {memory_max}）的 systemd scope 起不来（{memory_cap_runner} --check 退 {memory_cap_check.returncode}），一条变异都没跑：")
    for detail_line in (memory_cap_check.stdout + memory_cap_check.stderr).splitlines():
        print(f"      {detail_line}")   # gate-lint:detail
    print("     → 怎么办：照上面那几行修用户级 systemd / D-Bus（systemctl --user status）再跑这一道；不许拿掉上限去跑，无界分配的变异会把整机拖进 OOM。")
    sys.exit(1)


def crate_of(path):
    """`crates/<crate 名>/…` 里的 crate 名：分片按它切，同一片里连着改同一个 crate，增量编译才稳。"""
    parts = path.split("/")
    return parts[1] if len(parts) > 2 and parts[0] == "crates" else path


def worker_count_for(row_count):
    """开几个工作进程，连同头一行要报的话（各项给几个、卡在哪一项）。两项的意思与依据见文件头「工作进程数取下面两项的最小值」那一段。

    一条变异的挂钟 = 重编译那几个 crate + 跑点名的那个测试，而**测试那一段是单线程的**。
    实测 8 个进程、不限 cargo 的编译并行度时，峰值才 16 个 rustc——编译的并行度本来就用不满
    （依赖链是串的），多出来的核闲着。所以开得比「核数 ÷ 4」多，再用 GATE_MUTATION_CARGO_JOBS
    把每个 cargo 的编译并行度收住，让总并行度落回核数。
    内存不在这里收（文件头那一段）：每条经 run-with-memory-cap.sh 排队，放不下的在它那里等。
    """
    bounds = []   # (这一项叫什么, 给几个, 头一行里怎么说)
    requested = os.environ.get("GATE_MUTATION_WORKERS", "").strip()
    if requested:
        bounds.append(("GATE_MUTATION_WORKERS", int(requested), f"GATE_MUTATION_WORKERS 给 {int(requested)} 个"))
    else:
        processor_count = os.cpu_count() or 4
        by_processor = max(1, min(16, processor_count // 2))
        bounds.append(("核数", by_processor, f"核数 {processor_count} 的一半、至多 16，给 {by_processor} 个"))
    bounds.append(("表里的条数", row_count, f"表里 {row_count} 条"))
    smallest = min(count for _name, count, _text in bounds)
    binding = "、".join(f"「{name}」" for name, count, _text in bounds if count == smallest)
    floor_note = "（那一项不足 1，按 1 个开）" if smallest < 1 else ""
    worker_count = max(1, smallest)
    head_line = (f"  … 这一轮开 {worker_count} 个工作进程（各项取最小、至少 1）：{'；'.join(text for _name, _count, text in bounds)}。"
                 f"卡在{binding}这一项{floor_note}；内存不按进程数收：每条经 run-with-memory-cap.sh 排队，放不下的等（slice 的总上限兜底）")
    return worker_count, head_line


def rows_grouped_by_crate(all_rows):
    """按 crate 归堆、堆内保持表序，再把几堆首尾相接：切片时同一个 crate 才会落在同一片里。"""
    groups = {}
    for row in all_rows:
        groups.setdefault(crate_of(row[2]), []).append(row)
    ordered = []
    for crate_name in sorted(groups, key=lambda name: (-len(groups[name]), name)):
        ordered.extend(groups[crate_name])
    return ordered


# 每个工作进程自己的那一份：副本目录与编译目录，进程起来时建一次、这一轮里一直用
WORKER_STATE = {}


def init_worker(sequence_counter, shard_root):
    """工作进程启动时跑一次：领一个稳定的片号、建自己的源码副本与编译目录。

    片号从共享计数器领，不是按任务分——任务是动态领的（谁先跑完谁再领下一条），
    按条数预先切片会让拿到重测试的那一片独自拖住整道（实测：7 片跑完、最后 1 片又跑了 4 分钟）。
    """
    with sequence_counter.get_lock():
        sequence_counter.value += 1
        worker_sequence = sequence_counter.value
    work, target_directory = prepare_shard(worker_sequence, shard_root)
    WORKER_STATE["work"] = work
    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = target_directory
    WORKER_STATE["environment"] = environment
    # 源码副本建在父进程的 shard_root 底下、由父进程收尾时整个删；编译目录留着跨轮复用。
    # 不在这里挂 atexit：进程池的工作进程走 os._exit 退出，atexit 在这里一次都不跑（实测 /tmp 里堆了 131 个没删的副本）。


def judge_row(row):
    """进程池里的一格活：一条变异。副本与编译目录是这个工作进程独有的。"""
    return judge_one(WORKER_STATE["work"], WORKER_STATE["environment"], row)


def prepare_shard(worker_sequence, shard_root):
    """一个工作进程自己的一份源码副本加一个编译目录，进程起来时建一次。"""
    work = tempfile.mkdtemp(prefix=f"shard-{worker_sequence}-", dir=shard_root)
    for item in COPIED_INTO_EACH_SHARD:
        source = os.path.join(root, item)
        if not os.path.exists(source):
            continue
        target = os.path.join(work, item)
        if os.path.isdir(source):
            shutil.copytree(source, target, ignore=shutil.ignore_patterns(*IGNORED_WHEN_COPYING_INTO_EACH_SHARD))
        else:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy2(source, target)
    # 各片一个编译目录，第 0 片用共用的那个（跨轮热着），其余片自己编译起来。
    # ⚠️ 不从共用目录拷种子：实测 7 片拷 16 GiB 要 7 分钟还没拷完，期间只有 1 个 rustc 在跑——
    # 那是拿瓶颈资源（磁盘 I/O）去省空闲资源（32 核 CPU）。各片自己冷编译一次，编译走 CPU、彼此不争。
    target_directory = (shared_target_directory if worker_sequence == 1
                        else f"{shared_target_directory}-w{worker_sequence}")
    return work, target_directory


def judge_one(work, environment, row):
    """一条变异的判定：改坏那一处、跑点名的测试、还原。回的是 (行号, 档, 那一条要打印的话)。"""
    line_number, name, path, old, new, args, expected = row
    full = os.path.join(work, path)
    pristine = open(full, encoding="utf-8").read()
    assert pristine.count(old) == 1, (path, name)
    open(full, "w", encoding="utf-8").write(pristine.replace(old, new))
    # 限时交给包装（RuntimeMaxSec，从起跑算，不算排队）：不在外面套 timeout，排队的时间不算进限时
    run_environment = dict(environment, RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(timeout_seconds), RUN_WITH_MEMORY_CAP_KILL_GRACE=str(TIMEOUT_KILL_GRACE_SECONDS))
    try:
        run = subprocess.run(["bash", memory_cap_runner, memory_max, "cargo", "test", "--offline"] + args.split(),
                             cwd=work, env=run_environment, capture_output=True, text=True)
    finally:
        open(full, "w", encoding="utf-8").write(pristine)
    output = run.stdout + run.stderr
    tail = "\n".join(output.splitlines()[-8:])
    # 包装报的几种结局先判：被停、被杀的测试二进制可能已经打出了半截输出，不许拿它判抓到或没红
    if run.returncode == TIME_LIMIT_HIT_EXIT:
        return line_number, "timeout", f"{table}:{line_number} {name}：{timeout_seconds} 秒没跑完（超时），点名的测试 {expected} 没来得及红\n{tail}"
    if run.returncode == MEMORY_CAP_HIT_EXIT:
        return line_number, "memory", f"{table}:{line_number} {name}：撞了内存上限 {memory_max}（内存撞顶），点名的测试 {expected} 没来得及红\n{tail}"
    if run.returncode == SLICE_TOTAL_HIT_EXIT:
        return line_number, "squeezed", f"{table}:{line_number} {name}：被 slice 的总上限挤掉（整个 slice 满了，内核挑了这一条杀），这一条的结果不算数\n{tail}"
    if run.returncode == MEMORY_ADMISSION_REFUSED_EXIT:
        return line_number, "refused", f"{table}:{line_number} {name}：内存不够排不上（包装等满了等待上限），这一条没跑\n{tail}"
    if run.returncode == MEMORY_CAP_UNAVAILABLE_EXIT:
        return line_number, "unavailable", f"{table}:{line_number} {name}：带内存上限的 scope 起不来，这一条没跑\n{tail}"
    red_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. FAILED$", re.M)
    ran_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. ", re.M)
    if run.returncode != 0 and red_pattern.search(output):
        return line_number, "caught", f"  ✓ {name}：{expected} 红了"
    if ran_pattern.search(output):
        return line_number, "failure", f"{table}:{line_number} {name}：{expected} 没红（退出码 {run.returncode}）\n{tail}"
    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:
        # 替换文写进源码之后编不过：既不算被抓也不算没红，是一条无效变异，
        # 它要证明的那个行为今天零变异覆盖（.claude/rules/mutation-sampling.md 第八类）。
        # 报成「没红」会把排查指向「去补一条用例」，而要改的是这一行替换文。
        return line_number, "invalid", f"{table}:{line_number} {name}：替换文写进源码之后编不过（退出码 {run.returncode}）\n{tail}"
    return line_number, "failure", f"{table}:{line_number} {name}：点名的测试 {expected} 没跑到（退出码 {run.returncode}）\n{tail}"


ordered_rows = rows_grouped_by_crate(rows)
worker_count, worker_count_head_line = worker_count_for(len(rows))
# 实际开几个、各项给几个、卡在哪一项打在头一行，走 stderr：stdout 要与进程数无关（文件头）
print(worker_count_head_line, file=sys.stderr, flush=True)
# 每个 cargo 的编译并行度：进程数 × 它 ≈ 核数，免得 N 个 cargo 各自按核数开 rustc、互相抢
cargo_jobs = os.environ.get("GATE_MUTATION_CARGO_JOBS", "").strip() \
    or str(max(1, (os.cpu_count() or 4) // worker_count))
os.environ["CARGO_BUILD_JOBS"] = cargo_jobs
judgements = []
# 用进程池不用线程池：工作进程起来时要把源码拷一份，`shutil.copytree` 是纯 Python 循环、一路持着 GIL，
# 线程池下实测 8 个分片只有 1 个真在跑、其余 7 个卡在 futex 上等 GIL，九分钟一条判定都没出。
# chunksize=1 是动态调度：谁先跑完谁再领下一条，不预先按条数切片。条目按 crate 归过堆再发，
# 于是同一个工作进程连着拿到的多半是同一个 crate，增量编译的命中率还在。
sequence_counter = multiprocessing.Value("i", 0)
# 各工作进程的源码副本都建在这个总目录底下，父进程退出时整个删掉（父进程正常退出与 sys.exit 都走 atexit）。
shard_root = tempfile.mkdtemp(prefix="singlefs-mutation-replay-")
atexit.register(shutil.rmtree, shard_root, ignore_errors=True)
with concurrent.futures.ProcessPoolExecutor(max_workers=worker_count,
                                            initializer=init_worker,
                                            initargs=(sequence_counter, shard_root)) as pool:
    # 进度实时打到 stderr：判定行走 stdout、最后按表的行号排序统一打印，两边不混。
    # 只有 stdout 参与「输出与分片数无关」那条比对，进度是给盯着跑的人看的。
    try:
        for judgement in pool.map(judge_row, ordered_rows, chunksize=1):
            # 工作进程里抛出来的异常在这里原样炸出去：并行不许把失败吃掉（command-safety.md）
            judgements.append(judgement)
            print(f"  … 已判 {len(judgements)}/{len(rows)} 条", file=sys.stderr, flush=True)
    except concurrent.futures.process.BrokenProcessPool:
        # 一个工作进程被硬杀（OOM、段错误）时 pool.map 当场抛这个，而不是安静地少产出几项——
        # 下面那道「派多少收多少」的闸因此永远轮不到，真实的失败路径是一条没有出路的裸 traceback。
        print(f"  ✗ 有工作进程中途死了（已判 {len(judgements)}/{len(rows)} 条），这一轮的变异没跑全")
        print("     → 怎么办：多半是内存不够被 OOM 杀的——GATE_MUTATION_WORKERS 调小再跑一遍；")
        print("       单跑 GATE_MUTATION_WORKERS=1 能跑完就是并发度的问题，还死就去看那一条变异本身。")
        sys.exit(1)

# 派出去多少条就要收回来多少条：对不上整道红，不许少跑一条还报绿（command-safety.md）
if len(judgements) != len(rows):
    print(f"  ✗ 派出去 {len(rows)} 条变异，只收回 {len(judgements)} 条判定")
    print("     → 怎么办：这是分片并发自己的完整性闸红了，不是变异的问题；把 GATE_MUTATION_WORKERS=1 再跑一遍看串行下是不是全的，")
    print("       是就去查领活与收束那一段（init_worker、judge_row 与 pool.map 那几行）。")
    sys.exit(1)

# 按变异表的行号排序再打印：输出与分片数无关，GATE_MUTATION_WORKERS=1 与 =8 逐字相同
judgements.sort(key=lambda judgement: judgement[0])
invalid = [text for _, verdict, text in judgements if verdict == "invalid"]
failures = [text for _, verdict, text in judgements if verdict == "failure"]
memory_hits = [text for _, verdict, text in judgements if verdict == "memory"]
timeouts = [text for _, verdict, text in judgements if verdict == "timeout"]
unavailable = [text for _, verdict, text in judgements if verdict == "unavailable"]
squeezed = [text for _, verdict, text in judgements if verdict == "squeezed"]
refused = [text for _, verdict, text in judgements if verdict == "refused"]
caught = [text for _, verdict, text in judgements if verdict == "caught"]
for text in caught:
    print(text)
# 各栏各数各的：超时、内存撞顶、被总上限挤掉、排不上既不算抓到也不算没红，与无效一样单列；这一行与进程数无关，进 stdout
print(f"  计数：抓到 {len(caught)} 条、没红 {len(failures)} 条、无效 {len(invalid)} 条、内存撞顶 {len(memory_hits)} 条、超时 {len(timeouts)} 条、"
      f"被总上限挤掉 {len(squeezed)} 条、排不上没跑 {len(refused)} 条、scope 起不来没跑 {len(unavailable)} 条（共 {len(rows)} 条）")
if invalid:
    print(f"  ✗ 有变异无效（无效 {len(invalid)} 条；替换文写进源码之后编不过，那条行为今天零变异覆盖）：")
    for item in invalid:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：改这一行替换文，不是去补用例。先看反斜杠——表里只有 \\n 会被还原成换行，\\& \\\" 这类原样写进源码就编不过；")
    print("       别的编译错就在副本里把替换后的那一行 cargo check 一遍，改成编得过、而且真会改行为的写法（.claude/rules/mutation-sampling.md 第八类）。")
if failures:
    print(f"  ✗ 有变异没红（没红 {len(failures)} 条）：")
    for item in failures:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：没红 = 那处改法没有任何测试守着：先造一条会红的用例再改代码（show-me-test.md）；「没跑到」多半是 cargo test 参数选错了范围。")
if memory_hits:
    print(f"  ✗ 有变异撞了内存上限（内存撞顶 {len(memory_hits)} 条，每条上限 {memory_max}）：")
    for item in memory_hits:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：撞顶 = 这条破坏让被测代码无界分配，点名的测试没来得及红——不算抓到，也不是没红。换一处改法让它红在点名的测试上，")
    print("       或者给被测代码加一道先红的界（循环步数上限、分配量的断言）；正常的测试确实要这么多内存，才调大上限（表头「# 每条变异的内存上限：…」或 GATE_MUTATION_MEMORY_MAX）。")
if timeouts:
    print(f"  ✗ 有变异超时（超时 {len(timeouts)} 条，每条限 {timeout_seconds} 秒）：")
    for item in timeouts:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：超时 = 这条破坏让被测代码不终止了，或者点名的测试在并发下就要跑这么久——不算抓到，也不是没红。先在副本里只改坏这一处、")
    print("       不限时单跑它那一行的 cargo test 参数，看它结束不结束：不结束就换一处改法让它红在点名的测试上，或者给被测代码加一道先红的界（循环步数上限）；")
    print("       结束得了就调大限时（表头「# 每条变异的超时秒数：…」或 GATE_MUTATION_TIMEOUT），别把它记成抓到。")
if squeezed:
    print(f"  ✗ 有变异被 slice 的总上限挤掉（被总上限挤掉 {len(squeezed)} 条；这几条没撞各自的上限 {memory_max}，是 slice 里合起来满了）：")
    for item in squeezed:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：这几条的结果不算数，整道重跑；常这样说明包装的峰值表记少了（同时跑的几条实际占的比表里记的多），")
    print("       跑的时候 bash research/scripts/run-with-memory-cap.sh --status 看 slice 里是谁在占，别拿掉包装去跑。")
if refused:
    print(f"  ✗ 有变异排不上没跑（排不上没跑 {len(refused)} 条；包装等满了等待上限，slice 还是放不下）：")
    for item in refused:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，等那几件重活跑完再整道重跑；")
    print("       别拿掉包装去跑——排不上说明此刻整机放不下。")
if unavailable:
    print(f"  ✗ 跑到一半带内存上限的 scope 起不来了（{len(unavailable)} 条没跑）：")
    for item in unavailable:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：修好用户级 systemd / D-Bus（systemctl --user status）整道重跑；不许拿掉上限去跑。")
if invalid or failures or memory_hits or timeouts or squeezed or refused or unavailable:
    sys.exit(1)
# ⚠️ 进程数与 cargo 编译并行度**不写进 stdout**：它们是两次跑唯一不同的输入，写进成功行就让
# 「GATE_MUTATION_WORKERS=1 与 =16 的 stdout 逐字相同」这句话当场为假（独立复核实测：191 行判定行逐字相同，
# 只有这一行收尾不同）。要看用了几个进程、为什么，看 stderr 的头一行。
print(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次；每条在内存上限 {memory_max} 里跑、内存撞顶 0 条，"
      f"每条经 run-with-memory-cap.sh 排队、被总上限挤掉与排不上 0 条，每条限时 {timeout_seconds} 秒、超时 0 条；"
      f"工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、与进程数无关）")
print(f"  … 这一轮 {worker_count} 个工作进程，每个 cargo 编译并行度 {cargo_jobs}，每条变异内存上限 {memory_max}、限时 {timeout_seconds} 秒", file=sys.stderr)
PY
