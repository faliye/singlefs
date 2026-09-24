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
# 工作进程数取 GATE_MUTATION_WORKERS，没设就按核数算（每个给 4 核，最多 8 个）；设成 1 就是原来的串行跑法。
# 活是**动态领的**（谁先跑完谁再领下一条），不按条数预先切片：`singlefs-harness` 那几条点名的是几十秒的重测试，
# 等分之下拿到它们的那一片会独自拖住整道（实测：7 片跑完、最后 1 片又跑了 4 分钟）。
# ⚠️ **输出与进程数无关**：判定行按变异表的行号排序再打印到 stdout，GATE_MUTATION_WORKERS=1 与 =8 的 stdout 要逐字相同，
# 这一条是并发化之后重新证明它还红得出来的那一半（command-safety.md「改成并行之后要重新证明它红得出来」）。
# 进度实时打到 stderr，不进 stdout，不参与那条比对。
#
# 编译产物放 ${GATE_MUTATION_TARGET_DIR:-${TMPDIR:-/tmp}/singlefs-crates-mutation-target}（跨轮复用，第一次要整编一遍）；
# 分片各自的编译目录是它加 -w<片号>，第一次从它拷一份种子，省掉每片各冷编译一遍。
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

root, table = sys.argv[1], sys.argv[2]


def unescape(text):
    return text.replace("\\n", "\n")


rows = []
with open(os.path.join(root, table), encoding="utf-8") as handle:
    for line_number, line in enumerate(handle, 1):
        line = line.rstrip("\n")
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


def crate_of(path):
    """`crates/<crate 名>/…` 里的 crate 名：分片按它切，同一片里连着改同一个 crate，增量编译才稳。"""
    parts = path.split("/")
    return parts[1] if len(parts) > 2 and parts[0] == "crates" else path


def available_memory_in_gibibytes():
    """还能用多少内存；读不到就当 0，交给调用方退回按核数算。"""
    try:
        with open("/proc/meminfo", encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("MemAvailable:"):
                    return int(line.split()[1]) / (1024 * 1024)
    except (OSError, ValueError, IndexError):
        pass
    return 0.0


def worker_count_for(row_count):
    """开几个工作进程：核数与可用内存两道护栏，`GATE_MUTATION_WORKERS` 压过它们。

    一条变异的挂钟 = 重编译那几个 crate + 跑点名的那个测试，而**测试那一段是单线程的**。
    实测 8 个进程、不限 cargo 的编译并行度时，峰值才 16 个 rustc——编译的并行度本来就用不满
    （依赖链是串的），多出来的核闲着。所以开得比「核数 ÷ 4」多，再用 GATE_MUTATION_CARGO_JOBS
    把每个 cargo 的编译并行度收住，让总并行度落回核数。
    每个进程按 1.5 GiB 估（cargo 加它起的 rustc），内存不够时宁可少开。
    """
    requested = os.environ.get("GATE_MUTATION_WORKERS", "").strip()
    if requested:
        return max(1, min(int(requested), row_count))
    processor_count = os.cpu_count() or 4
    by_processor = max(1, min(16, processor_count // 2))
    available = available_memory_in_gibibytes()
    by_memory = max(1, int(available / 1.5)) if available else by_processor
    return max(1, min(by_processor, by_memory, row_count))


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
    try:
        run = subprocess.run(["cargo", "test", "--offline"] + args.split(),
                             cwd=work, env=environment, capture_output=True, text=True)
    finally:
        open(full, "w", encoding="utf-8").write(pristine)
    output = run.stdout + run.stderr
    red_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. FAILED$", re.M)
    ran_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. ", re.M)
    if run.returncode != 0 and red_pattern.search(output):
        return line_number, "caught", f"  ✓ {name}：{expected} 红了"
    tail = "\n".join(output.splitlines()[-8:])
    if ran_pattern.search(output):
        return line_number, "failure", f"{table}:{line_number} {name}：{expected} 没红（退出码 {run.returncode}）\n{tail}"
    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:
        # 替换文写进源码之后编不过：既不算被抓也不算没红，是一条无效变异，
        # 它要证明的那个行为今天零变异覆盖（.claude/rules/mutation-sampling.md 第八类）。
        # 报成「没红」会把排查指向「去补一条用例」，而要改的是这一行替换文。
        return line_number, "invalid", f"{table}:{line_number} {name}：替换文写进源码之后编不过（退出码 {run.returncode}）\n{tail}"
    return line_number, "failure", f"{table}:{line_number} {name}：点名的测试 {expected} 没跑到（退出码 {run.returncode}）\n{tail}"


ordered_rows = rows_grouped_by_crate(rows)
worker_count = worker_count_for(len(rows))
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
for _, verdict, text in judgements:
    if verdict == "caught":
        print(text)
if invalid:
    print("  ✗ 有变异无效（替换文写进源码之后编不过，那条行为今天零变异覆盖）：")
    for item in invalid:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：改这一行替换文，不是去补用例。先看反斜杠——表里只有 \\n 会被还原成换行，\\& \\\" 这类原样写进源码就编不过；")
    print("       别的编译错就在副本里把替换后的那一行 cargo check 一遍，改成编得过、而且真会改行为的写法（.claude/rules/mutation-sampling.md 第八类）。")
if failures:
    print("  ✗ 有变异没红：")
    for item in failures:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：没红 = 那处改法没有任何测试守着：先造一条会红的用例再改代码（show-me-test.md）；「没跑到」多半是 cargo test 参数选错了范围。")
if invalid or failures:
    sys.exit(1)
# ⚠️ 进程数与 cargo 编译并行度**不写进 stdout**：它们是两次跑唯一不同的输入，写进成功行就让
# 「GATE_MUTATION_WORKERS=1 与 =16 的 stdout 逐字相同」这句话当场为假（独立复核实测：191 行判定行逐字相同，
# 只有这一行收尾不同）。要看用了几个进程，看 stderr 上的进度行。
print(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次；"
      f"工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、与进程数无关）")
print(f"  … 这一轮 {worker_count} 个工作进程，每个 cargo 编译并行度 {cargo_jobs}", file=sys.stderr)
PY
