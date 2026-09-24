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
#
# 标记与 Rust const 的解析在 `lib-format-const.py`，39、92 号用的是同一份，不另抄。
# 以 `<!-- format-const` 开头而按文法读不出来的（多写了一个键、值不是整数）判红，不跳过；
# 同一个名字登记两次判红，同一份文件里写两次也算——第二个值不许被静默丢掉。
# 源码一侧只认类型是单个标识符的 const（`u64`、`usize`），值要是整数字面量：
# 实验里同名的扫描表（`const NODE_BYTES: [u64; 2] = …`）不是那个格式常量。
#
# 判别力：fixtures/27-format-constants.sh/red 放一处源码落后于 kb 的常量、一处正文残留的旧值、
# 一条多写了键的标记、一个同一份文件里登记两次的名字，以及值跨行写、带 pub(crate) 的两处落后声明，必须判红；
# green 另放一张与格式常量同名的扫描表（数组类型），必须判绿。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
LIB="$(cd "$(dirname "$0")" && pwd)/lib-format-const.py"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d .claude/kb ]] || { echo "  ! 找不到 kb，本阶段跳过"; exit 77; }

python3 - "$LIB" <<'PY'
import glob, importlib.util, sys

library_spec = importlib.util.spec_from_file_location("format_const", sys.argv[1])
format_const = importlib.util.module_from_spec(library_spec)
library_spec.loader.exec_module(format_const)

def body_of(text):
    """正文 = 「## 历史版本」之前那一段。历史里留旧值是文档纪律要求的，不判红。"""
    i = text.find('\n## 历史版本')
    return text if i < 0 else text[:i]

# ⚠️ `*-history.md` **不是登记位**：变更史里会原样引用标记（记「本轮加了哪个标记」），
# 那是历史陈述，不是第二处权威记录。与旧值字面串的豁免同一条理由
# （`writing-discipline.md`「正文只写现状，历史进文末」）。
kb_all = sorted(glob.glob('.claude/kb/**/*.md', recursive=True))
kb_files = [f for f in kb_all if not f.endswith('-history.md') and '/decisions-history/' not in f]

# ---- 1. 收标记：读不出来的判红；同一个常量只许登记一处，同一份文件里写两次也算（kb-discipline 第 4 条）----
marks, registrations, bad = {}, {}, []
unparsable_count = 0
for f in kb_files:
    parsed = format_const.parse_marks(open(f, encoding='utf-8').read())
    for unparsable in parsed.unparsable:
        unparsable_count += 1
        bad.append(f'{f}:{unparsable.line_number}  format-const 标记按文法读不出来：「{unparsable.excerpt}」')
    for mark in parsed.marks:
        registrations.setdefault(mark.name, []).append(f'{f}:{mark.line_number}')
        marks.setdefault(mark.name, (f, mark.value, list(mark.stale_literals)))

duplicate_count = 0
for name, places in registrations.items():
    if len(places) > 1:
        duplicate_count += 1
        bad.append(f'{name}：登记了 {len(places)} 处（{" 与 ".join(places)}）—— 同一个事实只许一处权威记录')

if not marks and not bad:
    print('  ! kb 里一个 format-const 标记都没有，本阶段**什么也没验**')
    print('     → 在定住格式常量的那句话旁边加 <!-- format-const: 名字 = 值 stale=旧字面串 -->')
    sys.exit(1)

# ---- 2. 源码里的 const 定义必须等于 kb 的现行值 ----
srcs = sorted(glob.glob('research/**/*.rs', recursive=True) + glob.glob('crates/**/*.rs', recursive=True))  # 2026-09-14 起格式常量模块住 crates/singlefs-format，同一套标记绑住它
seen_in_src = set()
for f in srcs:
    source_text = open(f, encoding='utf-8', errors='ignore').read()
    for declaration in format_const.read_rust_consts(source_text, only_scalar_types=True):
        if declaration.name not in marks:
            continue
        name = declaration.name
        seen_in_src.add(name)
        # 值要读到分号为止：只取开头那段数字时，`16384 * 2` 读成 16384 判通过（静默放行），
        # `16 * 1024` 与 `16_384` 读成 16 判红（2026-09-11 改名回扫时实测）。
        if declaration.value is None:
            shown = format_const.normalized_value_text(declaration.value_text)
            bad.append(f'{f}:{declaration.line_number}  const {name} 的值写成了「{shown}」，门禁读不出它等于几 → 写成整数字面量（可带 _ 分隔）')
            continue
        kbf, want, _ = marks[name]
        if declaration.value != want:
            bad.append(f'{f}:{declaration.line_number}  const {name} = {declaration.value}，而 {kbf} 定的现行值是 {want}')

# ---- 3. 旧值的字面串不许留在正文与源码里 ----
# ⚠️ `research/results/` **不在扫描范围**，理由与阶段 26 排除 `research/prompts/` 同一条：
# 产物是**那一轮的原始输出**，改它等于产物不再对应它的输入，证据链当场断掉。
# 「源码改了而产物没重跑」由 `87-replay.sh` 逐字节比对抓——改了源码它就会红，直到重跑。
scan = [(f, format_const.strip_marks(body_of(open(f, encoding='utf-8').read()))) for f in kb_files]
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
    if unparsable_count:
        print('     → 读不出来的标记照这一条文法改写：<!-- format-const: 名字 = 整数 stale=旧串|旧串 -->，')
        print('       stale= 之外不许有别的键；它读不出来时，这个常量在本阶段眼里就没登记过。')
        print('       它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」。')
    if duplicate_count:
        print('     → 一个名字只留一处登记（定这个值的那一处）；别处要提它，写成不带 <!-- 的文字，例如「`format-const: 名字`」。')
    sys.exit(1)

only_kb = sorted(set(marks) - seen_in_src)
print(f'  ✓ 格式常量同步（{len(marks)} 个已登记，{len(seen_in_src)} 个在源码里被钉住）')
if only_kb:
    print(f'     ! 这些还没有任何实验源码用到，本阶段对它们只验了唯一性：{", ".join(only_kb)}')
PY
