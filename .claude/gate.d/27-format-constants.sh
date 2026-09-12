#!/usr/bin/env bash
# gate-stage: 格式常量在 kb 与实验源码之间同步
#
# **实测出来的，不是想出来的**（2026-08-31）：D23（journal 的角色与格式）已定项 9 在
# 2026-08-30 把记录头从 84 抬到 86，kb 的两个下游数跟着改成 426 / 4010 并写明「产物不改」。
# 而 `research/e7-index-bench/src/bin/e43_ext_budget.rs` 里
# `const JOURNAL_HDR: u64 = 84`、单测 `assert_eq!(..., 428)`、产物 `hdr=84 room=428`
# **三处都停在旧值**，`grep -rn JOURNAL_HDR .claude/gate.d/ .claude/scripts/` 零命中
# ⇒ 没有任何东西把实验源码里的格式常量绑到 kb 的现行值上。
#
# 为什么这条静默：`87-replay.sh` 是**逐字节比对产物**，源码与产物一起停在旧值时它永远绿；
# `10-kb-rot.sh` 只看 kb 内部。两条都在跑，而这一类失同步从两条中间漏过去。
#
# 权威在 kb，形态是一行机器可读标记，紧挨着定这个值的那句话：
#
#     <!-- format-const: JOURNAL_HEADER_BYTES = 86 stale=hdr=84|room=428 -->
#
# `stale=` 列的是**旧值的字面串**（`|` 分隔，可省）。它们不许再出现在 kb 正文与
# 实验源码/产物里——但**允许出现在「## 历史版本」之后与 *-history.md 里**，
# 那正是 `.claude/singlefs-ai-sop/rules/writing-discipline.md`「正文只写现状，历史进文末」
# 给旧值留的位置。
#
# ⚠️ **它管得了哪一半**：只检查**已经登记了标记**的常量。一个新加的、没登记标记的
# 格式常量仍然可以静默漂移——那一半靠人，落点见 kb/checks-owed.md。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d .claude/kb ]] || { echo "  ! 找不到 kb，本阶段跳过"; exit 0; }

python3 - <<'PY'
import re, glob, sys, os

MARK = re.compile(r'<!--\s*format-const:\s*(\w+)\s*=\s*(-?\d+)\s*(?:stale=([^>]*?))?\s*-->')

def body_of(text):
    """正文 = 「## 历史版本」之前那一段。历史里留旧值是文档纪律要求的，不判红。"""
    i = text.find('\n## 历史版本')
    return text if i < 0 else text[:i]

# ⚠️ `*-history.md` **不是登记位**：变更史里会原样引用标记（记「本轮加了哪个标记」），
# 那是历史陈述，不是第二处权威记录。与旧值字面串的豁免同一条理由
# （`writing-discipline.md`「正文只写现状，历史进文末」）。
kb_all = sorted(glob.glob('.claude/kb/**/*.md', recursive=True))
kb_files = [f for f in kb_all if not f.endswith('-history.md') and '/decisions-history/' not in f]

# ---- 1. 收标记，且同一个常量只许登记一处（kb-discipline 第 4 条）----
marks, dup = {}, []
for f in kb_files:
    text = open(f, encoding='utf-8').read()
    for name, val, stale in MARK.findall(text):
        stale = [s for s in (stale or '').split('|') if s.strip()]
        if name in marks and marks[name][0] != f:
            dup.append((name, marks[name][0], f))
        marks.setdefault(name, (f, int(val), stale))

bad = []
for name, a, b in dup:
    bad.append(f'{name}：登记了两处（{a} 与 {b}）—— 同一个事实只许一处权威记录')

if not marks:
    print('  ! kb 里一个 format-const 标记都没有，本阶段**什么也没验**')
    print('     → 在定住格式常量的那句话旁边加 <!-- format-const: 名字 = 值 stale=旧字面串 -->')
    sys.exit(1)

# ---- 2. 源码里的 const 定义必须等于 kb 的现行值 ----
srcs = sorted(glob.glob('research/**/*.rs', recursive=True))
seen_in_src = set()
for f in srcs:
    for i, line in enumerate(open(f, encoding='utf-8', errors='ignore'), 1):
        # 值要读到分号为止：只取开头那段数字时，`16384 * 2` 读成 16384 判通过（静默放行），
        # `16 * 1024` 与 `16_384` 读成 16 判红（2026-09-11 改名回扫时实测）。
        m = re.match(r'\s*(?:pub\s+)?const\s+(\w+)\s*:\s*\w+\s*=\s*([^;]+);', line)
        if not m or m.group(1) not in marks:
            continue
        name, rhs = m.group(1), m.group(2).strip()
        literal = re.fullmatch(r'(-?[0-9][0-9_]*)(?:[iu](?:8|16|32|64|128|size))?', rhs)
        seen_in_src.add(name)
        if not literal:
            bad.append(f'{f}:{i}  const {name} 的值写成了「{rhs}」，门禁读不出它等于几 → 写成整数字面量（可带 _ 分隔）')
            continue
        val = int(literal.group(1).replace('_', ''))
        kbf, want, _ = marks[name]
        if val != want:
            bad.append(f'{f}:{i}  const {name} = {val}，而 {kbf} 定的现行值是 {want}')

# ---- 3. 旧值的字面串不许留在正文与源码里 ----
# ⚠️ `research/results/` **不在扫描范围**，理由与阶段 26 排除 `research/prompts/` 同一条：
# 产物是**那一轮的原始输出**，改它等于产物不再对应它的输入，证据链当场断掉。
# 「源码改了而产物没重跑」由 `87-replay.sh` 逐字节比对抓——改了源码它就会红，直到重跑。
scan = [(f, MARK.sub('', body_of(open(f, encoding='utf-8').read()))) for f in kb_files
        if not f.endswith('-history.md') and '/decisions-history/' not in f]
scan += [(f, open(f, encoding='utf-8', errors='ignore').read()) for f in srcs]

for name, (kbf, want, stale) in sorted(marks.items()):
    for s in stale:
        for f, text in scan:
            if s in text:
                ln = text[:text.index(s)].count('\n') + 1
                bad.append(f'{f}:{ln}  还留着 {name} 的旧值字面串「{s}」（现行值 {want}）')

if bad:
    print('  ✗ 格式常量在 kb 与实验源码之间对不上：')
    for b in bad[:20]:
        print(f'     {b}')
    if len(bad) > 20:
        print(f'     …… 另有 {len(bad)-20} 处')
    print('     → 权威是 kb 里的 format-const 标记。改常量要三处一起动：')
    print('       ① kb 标记与正文 ② 实验源码的 const 与钉死它的单测 ③ 重跑实验并更新 research/results/ 的产物')
    print('     → 旧值只许留在「## 历史版本」之后与 *-history.md 里。')
    sys.exit(1)

only_kb = sorted(set(marks) - seen_in_src)
print(f'  ✓ 格式常量同步（{len(marks)} 个已登记，{len(seen_in_src)} 个在源码里被钉住）')
if only_kb:
    print(f'     ! 这些还没有任何实验源码用到，本阶段对它们只验了唯一性：{", ".join(only_kb)}')
PY
