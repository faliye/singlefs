#!/usr/bin/env bash
# gate-stage: kb 正文里整行抄的产物行，产物里逐字找得到
#
# 判据：kb 正文（「## 历史版本」之前；*-history.md 与 decisions-history/ 不算）里去掉首尾空白后
# 以 `E7RESULT ` 开头的行，必须在 research/results/*.out 的某一行里逐字出现。
# 找不到 = 抄的时候改了数，或者产物重跑之后正文没跟。
#
# ⚠️ **这条是实测出来的，不是想出来的**（2026-09-16）：规则早就写着「引产物就整行抄」「重跑之后要回对正文」
# （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`），检查一直没有——87 号复跑只比产物与二进制、不看正文，
# 40 号只看产物有没有被点名。树表条目 148 → 200 那一轮撞见 E146 的判决块混着两版产物（config 行 145、
# 下面几行 148），E142 的正文有三行停在旧产物上；这道检查接上的当天又扫出 E139 两行、E144 一行，
# 正文里的数在它自己点名的留存产物里找不到。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   1. 只认整行。句中反引号里的片段、按字段转述成一句话的数，一概不判。
#   2. 在全部产物里找、不按实验号配对：正文允许引别的实验的产物（E152 引 E142 的 write_list）。
#      代价是两个实验恰好吐出同一行时，引错了实验认不出来。
#   3. 历史节里的旧行不判：旧产物被删掉之后，历史节里留着当时抄的那一行是对的。
#   4. **只判这次改动新增或改写的行**（用户 2026-09-21 定）：正文里早先抄下的行是历史，仅作参考——
#      那一轮跑过、当时对得上，之后产物按「每次提交删上一次的实验记录」删掉了，再判就是判一个不存在的对照。
#      要推翻早先的结论，按 `.claude/rules/three-way-inference.md` 重新跑三腿，那时产物是新的、本阶段照样判得到。
#      拿不到 diff 基准时**退回全量判**（保守：宁可多判，不可漏判）。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d .claude/kb ]] || { echo "  ! 找不到 .claude/kb，本阶段无对象可判"; exit 77; }

BASE="${GATE_BASE:-}"
if [[ -z "$BASE" ]]; then
  BASE="$(git rev-parse --verify --quiet refs/singlefs/gate-ok || git rev-parse --verify --quiet '@{upstream}' || true)"
fi
export GATE_BASE_RESOLVED="$BASE"

python3 - <<'PY'
import glob, os, subprocess, sys

kb = sorted(f for f in glob.glob('.claude/kb/**/*.md', recursive=True)
            if not f.endswith('-history.md') and '/decisions-history/' not in f)
quoted = []
for path in kb:
    text = open(path, encoding='utf-8').read()
    cut = text.find('\n## 历史版本')
    body = text if cut < 0 else text[:cut]
    for number, line in enumerate(body.split('\n'), 1):
        stripped = line.strip()
        if stripped.startswith('E7RESULT '):
            quoted.append((path, number, stripped))
# 只判这次改动新增或改写的行：拿 diff 基准算出每份 kb 文件新增的 E7RESULT 行，
# 基准取不到就退回全量判（保守，宁可多判不可漏判）。
base = os.environ.get('GATE_BASE_RESOLVED', '').strip()
scope = '全量'
if base:
    added = set()
    for path in kb:
        try:
            diff = subprocess.run(['git', 'diff', '--unified=0', base, '--', path],
                                  capture_output=True, text=True, check=True).stdout
        except subprocess.CalledProcessError:
            added = None
            break
        for line in diff.split('\n'):
            if line.startswith('+') and not line.startswith('+++'):
                stripped = line[1:].strip()
                if stripped.startswith('E7RESULT '):
                    added.add((path, stripped))
    if added is not None:
        quoted = [(path, number, stripped) for path, number, stripped in quoted
                  if (path, stripped) in added]
        scope = '这次改动新增或改写的'

if not quoted:
    print('  ! %s kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判' % scope)
    sys.exit(77)

products = sorted(glob.glob('research/results/*.out'))
product_lines = set()
for product in products:
    with open(product, encoding='utf-8', errors='ignore') as handle:
        for line in handle:
            product_lines.add(line.strip().replace('\r', ''))

missing = [(path, number, stripped) for path, number, stripped in quoted if stripped not in product_lines]
if missing:
    print(f'  ✗ {scope} kb 正文里有 {len(missing)} 行整行抄的产物行，在 research/results/ 的 {len(products)} 份产物里一份都找不到：')
    for path, number, stripped in missing:
        print(f'     {path}:{number}  {stripped[:160]}')  # gate-lint:detail
    print('     → 怎么办：打开这个实验复跑命令登记的那份产物，把对应的行原样抄回来，别手改数；')
    print('               正文里照这几行写的结论句一起回对。产物确实没留存，就删掉这几行并写明「原始输出未留存」。')
    sys.exit(1)
print(f'  ✓ {scope} kb 正文里整行抄的产物行都在产物里逐字找得到（判了 {len(quoted)} 行，对照 {len(products)} 份产物）')
print(f'     没判的：早先抄下的行（历史参考，产物按「每次提交删上一次的实验记录」删掉了）；要推翻它们按三方重跑')
PY
