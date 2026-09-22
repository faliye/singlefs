#!/usr/bin/env bash
# gate-stage: crates 变异表复跑（每条改坏一处、点名的测试必须红）
#
# 判据：`crates/mutations.tsv` 里每一条变异（文件、原文、替换文、cargo test 参数、必须红的测试名），
# 原文在文件里恰好命中一次；把仓拷到临时目录、改坏那一处、跑点名的测试，那条测试必须判红；跑完还原再下一条。
# 一条锚点腐化、一条没红，整道红。
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
# 分片数取 GATE_MUTATION_WORKERS，没设就按核数算（每片给 4 核，最多 8 片）；设成 1 就是原来的串行跑法。
# ⚠️ **输出与分片数无关**：结果按变异表的行号排序再打印，GATE_MUTATION_WORKERS=1 与 =8 的输出要逐字相同，
# 这一条是并发化之后重新证明它还红得出来的那一半（command-safety.md「改成并行之后要重新证明它红得出来」）。
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
import concurrent.futures
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
shared_target_directory = os.environ["GATE_MUTATION_TARGET_DIR"]


def crate_of(path):
    """`crates/<crate 名>/…` 里的 crate 名：分片按它切，同一片里连着改同一个 crate，增量编译才稳。"""
    parts = path.split("/")
    return parts[1] if len(parts) > 2 and parts[0] == "crates" else path


def shard_count_for(row_count):
    requested = os.environ.get("GATE_MUTATION_WORKERS", "").strip()
    if requested:
        return max(1, min(int(requested), row_count))
    return max(1, min(8, (os.cpu_count() or 4) // 4, row_count))


def rows_grouped_by_crate(all_rows):
    """按 crate 归堆、堆内保持表序，再把几堆首尾相接：切片时同一个 crate 才会落在同一片里。"""
    groups = {}
    for row in all_rows:
        groups.setdefault(crate_of(row[2]), []).append(row)
    ordered = []
    for crate_name in sorted(groups, key=lambda name: (-len(groups[name]), name)):
        ordered.extend(groups[crate_name])
    return ordered


def split_into_shards(ordered_rows, shard_count):
    """切成 shard_count 片，长度差不超过 1；空片不产生。"""
    shards = []
    start = 0
    for shard_index in range(shard_count):
        length = len(ordered_rows) // shard_count + (1 if shard_index < len(ordered_rows) % shard_count else 0)
        if length == 0:
            continue
        shards.append(ordered_rows[start:start + length])
        start += length
    return shards


def prepare_shard(shard_index):
    """一份源码副本加一个编译目录；编译目录从共用的那个拷一份种子，省掉每片各冷编译一遍。"""
    work = tempfile.mkdtemp(prefix=f"singlefs-mutation-replay-{shard_index}-")
    for item in COPIED_INTO_EACH_SHARD:
        source = os.path.join(root, item)
        if not os.path.exists(source):
            continue
        target = os.path.join(work, item)
        if os.path.isdir(source):
            shutil.copytree(source, target, ignore=shutil.ignore_patterns("target"))
        else:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy2(source, target)
    if shard_index == 0:
        target_directory = shared_target_directory
    else:
        target_directory = f"{shared_target_directory}-w{shard_index}"
        if not os.path.exists(target_directory) and os.path.isdir(shared_target_directory):
            shutil.copytree(shared_target_directory, target_directory, symlinks=True)
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


def run_shard(shard_index, shard_rows):
    """一片顺序跑完它分到的那些条目；副本与编译目录是这一片独有的，跑完删副本、留编译目录跨轮复用。"""
    work, target_directory = prepare_shard(shard_index)
    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = target_directory
    try:
        return [judge_one(work, environment, row) for row in shard_rows]
    finally:
        shutil.rmtree(work, ignore_errors=True)


shards = split_into_shards(rows_grouped_by_crate(rows), shard_count_for(len(rows)))
judgements = []
with concurrent.futures.ThreadPoolExecutor(max_workers=len(shards)) as pool:
    pending = {pool.submit(run_shard, index, shard): index for index, shard in enumerate(shards)}
    for future in concurrent.futures.as_completed(pending):
        # 分片里抛出来的异常在这里原样炸出去：并行不许把失败吃掉（command-safety.md）
        judgements.extend(future.result())

# 派出去多少条就要收回来多少条：对不上整道红，不许少跑一条还报绿（command-safety.md）
if len(judgements) != len(rows):
    print(f"  ✗ 派出去 {len(rows)} 条变异，只收回 {len(judgements)} 条判定")
    print("     → 怎么办：这是分片并发自己的完整性闸红了，不是变异的问题；把 GATE_MUTATION_WORKERS=1 再跑一遍看串行下是不是全的，")
    print("       是就去查分片切分与收束那一段（split_into_shards 与 ThreadPoolExecutor 那几行）。")
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
shard_sizes = "、".join(str(len(shard)) for shard in shards)
print(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次；"
      f"{len(shards)} 个分片并发，各分到 {shard_sizes} 条，输出按表的行号排序、与分片数无关）")
PY
