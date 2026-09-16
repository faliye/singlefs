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
# 编译产物放 ${GATE_MUTATION_TARGET_DIR:-${TMPDIR:-/tmp}/singlefs-crates-mutation-target}（跨轮复用，第一次要整编一遍）。
#
#   bash .claude/gate.d/59-crates-mutation-replay.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
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
import os, re, shutil, subprocess, sys, tempfile
root, table = sys.argv[1], sys.argv[2]
def unescape(text): return text.replace("\\n", "\n")
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
work = tempfile.mkdtemp(prefix="singlefs-mutation-replay-")
try:
    for item in ["crates", "Cargo.toml", "Cargo.lock", "litmus", "research/results"]:
        source = os.path.join(root, item)
        if not os.path.exists(source):
            continue
        target = os.path.join(work, item)
        if os.path.isdir(source):
            shutil.copytree(source, target, ignore=shutil.ignore_patterns("target"))
        else:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy2(source, target)
    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = os.environ["GATE_MUTATION_TARGET_DIR"]
    failures = []
    for line_number, name, path, old, new, args, expected in rows:
        full = os.path.join(work, path)
        pristine = open(full, encoding="utf-8").read()
        assert pristine.count(old) == 1, (path, name)
        open(full, "w", encoding="utf-8").write(pristine.replace(old, new))
        try:
            run = subprocess.run(["cargo", "test", "--offline"] + args.split(), cwd=work, env=environment, capture_output=True, text=True)
        finally:
            open(full, "w", encoding="utf-8").write(pristine)
        output = run.stdout + run.stderr
        red_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. FAILED$", re.M)
        ran_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. ", re.M)
        if run.returncode != 0 and red_pattern.search(output):
            print(f"  ✓ {name}：{expected} 红了")
        elif ran_pattern.search(output):
            failures.append(f"{table}:{line_number} {name}：{expected} 没红（退出码 {run.returncode}）")
        else:
            tail = "\n".join(output.splitlines()[-8:])
            failures.append(f"{table}:{line_number} {name}：点名的测试 {expected} 没跑到（退出码 {run.returncode}）\n{tail}")
finally:
    shutil.rmtree(work, ignore_errors=True)
if failures:
    print("  ✗ 有变异没红：")
    for item in failures:
        print(f"      {item}")   # gate-lint:detail
    print("     → 怎么办：没红 = 那处改法没有任何测试守着：先造一条会红的用例再改代码（show-me-test.md）；「没跑到」多半是 cargo test 参数或测试名写错，或者副本里缺它要读的文件。")
    sys.exit(1)
print(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次）")
PY
