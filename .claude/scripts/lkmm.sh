#!/usr/bin/env bash
# admission: always 判的是此刻 litmus/ 与代码里的锚点，上一次的结论不替这一次作保（门禁 checker-tier-lkmm 每份 litmus 按单元键复用，写在那一道文件头）
# run-condition: none herd7 与内核树由它自己找（PATH、opam、fetch-deps.sh），找不到就判红并给出路，checker-tier-lkmm 靠这一条判红，不交给调用方预判
# 用 LKMM 判内存序结论：herd7 给模型判定。
#
# 这是本工程自己的一份：上游 singlefs-ai-sop 2026-09-16 起不再管 herd7 / LKMM（移交那次的记录与删前原样在提交 fbae43e 里，`git show fbae43e:.claude/handover/qemu-herd7/README.md`），
# 原文取自那里的 `sop-0.0.50-snapshot/scripts/lkmm.sh`，只改了 lib.sh 的路径、`litmus/` 改指本仓 `litmus/`。
# 门禁阶段 `.claude/gate.d/checker-tier-lkmm.sh` 调它；提交前必跑（`.claude/rules/implementation-workflow.md`）。
#
#   lkmm.sh [项目根] [--verdict-library <目录> [--no-reuse]]   跑 <项目根>/litmus/*.litmus；给了判定库就按条复用（下面「判定库」一段）
#   lkmm.sh [项目根] --static-only    只跑不需要 herd7 的检查；全过也退 3——它不是通过
#   lkmm.sh --herd7-version           只找 herd7（与全量判定同一段找法，PATH 里没有就试 opam 的环境），stdout 打它版本输出的第一行、退 0；找不到退 1
#
# 每个 .litmus 必须在文件里声明期望判定：
#
#   (* singlefs-expect: Never *)      坏结果必须不可能发生
#   (* singlefs-expect: Sometimes *)  坏结果可能发生（对照组用）
#
# 每条 Never 还要声明它模拟的是哪段代码，一行一个锚点（写者、读者各一行也行），
# 锚点后面空一格可以写注释：
#
#   singlefs-models: crates/<crate>/src/<文件>.rs :: <单元限定名>
#   singlefs-models: none —— <为什么它不对应代码>
#
# 单元限定名照 scripts/gate/lib_function_units.py 的切法（文件头「种类与限定名」）：自由 fn 写名字，impl 的方法写 <impl 类型>::<方法>，
# 内联 mod 里的写 <mod>::<名>；`::` 两边各空一格。锚点要在那份文件里恰好命中一个单元：一个都没有、多于一个（同名的两处）都判红。
#
# 判定与声明不符 → 失败。没有声明 → 失败（不许「跑了但没人看结果」）。
#
# 会失败的检查（都是踩过的坑，做成拒绝执行而不是提醒句）：
#   1. 用了 rN 却没有 `int rN;` 声明（用法只在剥掉 (* … *) 注释的正文里认）—— herd7 不管，klitmus7 会在生成 C 之后
#      才报 undeclared，那时已经很难定位。这里提前拦。
#   2. init 块里给 atomic_t 形参赋初值不带类型 —— herd7 照跑且判定正确，
#      只有 klitmus7 会炸。也就是说这个错能一路混过模型判定，必须在这里拦。
#   3. 每条 Never 必须有**自己的**对照组：同名前缀 `<名>-*.litmus`、声明 Sometimes，
#      而且**内容就是它去掉屏障的形态**（判据在 is_fence_removal_of）。
#      全局数出一条 Sometimes 不算数——对抗测试实测：10 条互不相关的 Never
#      曾靠 1 条无关的 Sometimes 全部过闸。只按文件名认也不够：换一个 exists、换一个读者的
#      「对照组」照样过闸，名字前缀撞车时（a 与 a-b）a-b 的对照还会被算成 a 的。
#      「屏障挡住了」还是「本来就撞不上」，只有同一条测试去掉屏障才回答得了。
#   4. 每条 Never 都要绑到代码。herd7 只判 litmus 写下的那个形态：代码改了发布顺序而 litmus 没跟，
#      判定照样是 Never，门禁照样绿。所以锚点要在那份文件里恰好命中一个单元，而且 crates/ 下要有一个 .rs
#      写出这个 litmus 的项名（认法在 binding_mention_paths：前后不接别的文件名字符，前面可以带 litmus/）——
#      那是读它、拿它跟代码今天的形态比的测试。测得对不对门禁判不了，要人看。
#   5. 没有 herd7 直接失败，不静默跳过。
#
# 1–4 都排在 herd7 探测之前：它们不需要 herd7，
# 该红的先红出来（selftest 的样本也靠这一点才喂得进去）。
#
# 项名：litmus/ 下递归找 .litmus，每份的项名是它相对 litmus/ 的路径（顶层的就是文件名，子目录里的带目录，例 sub/publish.litmus）；
#   判定项、按条的临时文件（$WORK/<种类>/<项名>）与写出文件名那一查都用项名，两个子目录里同名的 litmus 不串。
#   弄坏开关 GATE_LKMM_BREAK=litmus-items-by-file-name（自证 scripts/gate/lkmm-selftest.sh 用）：项名只取文件名（改前），两个子目录里的同名 litmus 共用一个项名与一份临时文件，
#     两份都报成 binding:<文件名>、herd7:<文件名>（自证 litmus-same-name-subdirectories 那一格判错：找不到 binding:a/publish.litmus 那一行）。
# 判定库（--verdict-library <目录>，判定库 scripts/crash/lib_verdict_library.py；门禁给 git common-dir 下的 singlefs-lkmm-verdicts/）：每份 litmus 两条判定。
#   herd7:<项名>      herd7 的判定与声明相符。键是 litmus 去掉行首起的 (* … *) 注释之后的内容、singlefs-expect 声明的那个值（它在注释里却决定判据）、
#                     herd7（-version 第一行与可执行文件的 sha256）、内核树 tools/memory-model 里 herd7 读的模型文件（顶层的 .cfg、.cat、.bell、.def）
#                     逐份内容、内核树的版本（git 树的提交号，不是 git 树就读 Makefile 的 VERSION / PATCHLEVEL / SUBLEVEL / EXTRAVERSION），
#                     判法单元指纹是这份脚本里 judge_herd7_output 的单元闭包键。锚点指到的代码不进：herd7 不读代码。
#   binding:<项名>    每条 Never 的锚点都恰好命中一个单元、crates/ 下有 .rs 写出这份 litmus 的项名。键是 litmus 去注释的内容、锚点声明、
#                     锚点指到的那几个单元的（种类，去注释指纹）、项名（组成表那一行仍叫「litmus 文件名」）、写出文件名的测试（crates/ 下写出项名的 .rs 按路径排序，
#                     逐份相对路径与内容 sha256；一份都没有写固定的「无」），判法单元指纹是 check_binding_test_mentions 的单元闭包键。
#                     锚点与写出文件名的那批 .rs 每趟都现找（键要它们）：那批 .rs 删掉、改名或改掉那一处提及都让键变、这一条现判。
#   键相同、上次判绿就复用（herd7 那一条不起 herd7）；--no-reuse 只写不读。每条一行「· <项>：绿|红，现判（<原因>：<变了的单元>）|复用（…）」，
#   末了一行「· 按条：现判 N、复用 M」。不给 --verdict-library 不读不写，每条都现判。
#   弄坏开关 GATE_LKMM_BREAK=litmus-key-drops-every-comment（门禁 checker-tier-lkmm 的样本用）：herd7 那一条的键把注释整个去掉、连 singlefs-expect 的值也不留，
#     改了期望照样复用（样本 litmus-key-samples-expectation-changed-red 判绿退 77）。
#   弄坏开关 GATE_LKMM_BREAK=binding-key-drops-test-mentions（自证 scripts/gate/lkmm-selftest.sh 用）：绑定那一条的组成表不放「写出文件名的测试」那一行，
#     删掉唯一写出文件名的 .rs 照样复用判绿（自证 binding-test-mentions-in-key 那一格判错）。
#   弄坏开关 GATE_LKMM_BREAK=binding-mention-substring（自证 scripts/gate/lkmm-selftest.sh 用）：写出项名那一查退回按子串找（改前），
#     只写了 mp-publish.litmus 的 .rs 也算写出 publish.litmus（自证 binding-mention-boundary 那一格判错）。
#   复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：绑定、herd7 两种判定各一份，组成表都算好之后、判之前抽（候选按键文件查，verdict-library-candidate，
#     与 lookup 分开）；抽中强制现判的那一条那一行写「强制现判」、不读库；哨兵被现判、强制现判判定与库里不同，算一项红。种子与条数取 SINGLEFS_REUSE_SENTINEL_SEED、
#     SINGLEFS_REUSE_SENTINEL_COUNT；没给判定库、--no-reuse 不抽。弄坏开关 GATE_LKMM_BREAK=reuse-lookup-skipped：一律不读库，自证（scripts/gate/lkmm-selftest.sh）
#     reuse-three-questions 那一格判错、reuse-sentinel 那一格要的红照判。
#   复用三问与哨兵的自证在 scripts/gate/lkmm-selftest.sh（拿门禁 checker-tier-lkmm 的样本 litmus-key-samples-anchor-body-changed 的 setup.sh 造现场、假 herd7 跑这一份）：
#     这一份的 --selftest 会被重型测试闸按 herd7 拦，自证另放一份可以单独起的。
# 准入经 .claude/scripts/preflight.sh 判（规范副本先找这棵树的、再找主仓的，两处都没有就拒绝；坑写在它的文件头），不直接 source 副本里的 lib.sh：
#   worktree 与 gate.sh --staged 的临时树里没有副本，直接 source 会落空，preflight 成了找不到的命令、参数被清空、不带参数照跑。
#   lib.sh（ok、bad、howto、warn、say、die、head1、project_root）取 preflight.sh 找到的那一份副本里的：它与 preflight.py 同在副本的 scripts/ 下（PREFLIGHT_PROGRAM）。
source "$(dirname "${BASH_SOURCE[0]}")/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "${PREFLIGHT_PROGRAM%/*}/lib.sh" || { echo "  ✗ 规范副本里没有 lib.sh：${PREFLIGHT_PROGRAM%/*}/lib.sh（preflight.py 在而 lib.sh 不在，副本装得不全）"
  echo "     → 怎么办：重装规范副本（bash .claude/singlefs-ai-sop/install.sh）再跑。"; exit 1; }

LKMM_SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERDICT_LIBRARY_MODULE="$LKMM_SCRIPT_DIRECTORY/../../scripts/crash/lib_verdict_library.py"
STATIC_ONLY=0; HERD7_VERSION_ONLY=0; ROOT=""; VERDICT_LIBRARY=""; NO_REUSE=0; expect_library_directory=0
for argument in "$@"; do
  if ((expect_library_directory)); then VERDICT_LIBRARY="$argument"; expect_library_directory=0; continue; fi
  case "$argument" in
    --static-only) STATIC_ONLY=1 ;;
    --herd7-version) HERD7_VERSION_ONLY=1 ;;
    --verdict-library) expect_library_directory=1 ;;
    --no-reuse) NO_REUSE=1 ;;
    *) ROOT="$argument" ;;
  esac
done
((expect_library_directory == 0)) || die "--verdict-library 后面缺目录" "写成 --verdict-library <目录>（门禁 checker-tier-lkmm 给 git common-dir 下的 singlefs-lkmm-verdicts/）"

# herd7 不在 PATH 里就试一次 opam 的环境；找得到返回 0。全量判定与 --herd7-version 共用这一段找法。
find_herd7() {
  if ! command -v herd7 >/dev/null 2>&1 && command -v opam >/dev/null 2>&1; then
    export OPAMROOT="${OPAMROOT:-$HOME/.opam}"
    eval "$(opam env --root="$OPAMROOT" --set-root 2>/dev/null)" || true
  fi
  command -v herd7 >/dev/null 2>&1
}
report_herd7_missing() {
  bad "herd7 缺失"
  howto "opam install herdtools7" \
        "（装完若命令仍找不到，先 eval \"\$(opam env)\"）"
}
if [[ $HERD7_VERSION_ONLY -eq 1 ]]; then
  find_herd7 || { report_herd7_missing; exit 1; }
  if ! herd7_version_output="$(herd7 -version 2>&1)"; then
    bad "herd7 -version 退非 0：$herd7_version_output"
    howto "herd7 装坏了：opam reinstall herdtools7，再跑一次 bash .claude/scripts/lkmm.sh --herd7-version"
    exit 1
  fi
  printf '%s\n' "${herd7_version_output%%$'\n'*}"
  exit 0
fi
ROOT="${ROOT:-$(project_root)}"
# 对照组判据靠内嵌的 python3。缺了它，$(is_fence_removal_of …) 失败被当成「不匹配」，
# 于是每条 Never 都报「没有配对的对照组」——症状指着 litmus，病根在环境（审计实测）。
command -v python3 >/dev/null 2>&1 || die "缺 python3 —— 对照组「是不是去掉屏障的那一份」靠它判" \
  "装 python3（Debian/Ubuntu: sudo apt install python3）再跑；没有它时这一阶段报的原因是错的。"
# 绝对化：失败信息里的 rel 要靠 ${f#$ROOT/} 剥前缀，而 FILES 是 readlink -f 出来的绝对路径。
# ROOT 是相对路径时剥不掉，报出来的就是一长串绝对路径——看的人得自己找哪一段是项目内路径。
[[ -d "$ROOT" ]] && ROOT="$(cd "$ROOT" && pwd)"
LITMUS_DIR="$ROOT/litmus"
KTREE="${SINGLEFS_KERNEL_TREE:-}"

head1 "LKMM（herd7）"

[[ -d "$LITMUS_DIR" ]] || { bad "没有 $LITMUS_DIR 目录"
  howto "并发相关的改动要有 litmus：照 litmus/ 那对现成的抄，" \
        "放进 <项目根>/litmus/。没有并发改动就不该跑到这条检查。"; exit 1; }
# 路径必须绝对化：下面要 cd 进内核树跑 herd7（模型文件是相对路径引的），
# 相对路径 cd 之后就找不着了
# 每份的项名（LITMUS_KEY）是它相对 litmus/ 的路径：顶层的就是文件名（判定库里已有的判定照旧复用），子目录里的带目录，
# 两个子目录里同名的 litmus 各是各的项，判定项、临时文件与写出文件名那一查都不串（文件头「判定库」一段）。
# 弄坏开关 GATE_LKMM_BREAK=litmus-items-by-file-name 退回改前：项名只取文件名。
declare -A LITMUS_KEY=()
FILES=()
while IFS= read -r -d '' litmus_relative; do
  f="$(readlink -f "$LITMUS_DIR/$litmus_relative")"
  FILES+=("$f")
  if [[ ",${GATE_LKMM_BREAK:-}," == *,litmus-items-by-file-name,* ]]; then LITMUS_KEY["$f"]="${litmus_relative##*/}"; else LITMUS_KEY["$f"]="$litmus_relative"; fi
done < <(find "$LITMUS_DIR" -name '*.litmus' -printf '%P\0' | sort -z)
[[ ${#FILES[@]} -gt 0 ]] || { bad "$LITMUS_DIR 下没有 .litmus 文件"
  howto "照 litmus/ 那对现成的抄一对进来（Never + 去屏障的 Sometimes 对照），" \
        "或者删掉空的 litmus/ 目录。"; exit 1; }

# 按条判定的临时文件（组成表、单元清单）：跑完删
WORK="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-lkmm.XXXXXX")"
trap 'rm -rf "${WORK:?}"' EXIT
# work_file <种类> <项名>：这份 litmus 的临时文件路径 $WORK/<种类>/<项名>（项名里的子目录照建），调用方自己接后缀
work_file() {
  local path="$WORK/$1/$2"
  mkdir -p "${path%/*}" && printf '%s' "$path"
}

# ── 静态检查：期望声明 / 寄存器声明 / atomic_t 初值类型 ──
# 寄存器用法只在剥掉注释的正文里找：头部 (* … *) 注释里写 r1、r2 不是用了它，不能逼人往 litmus 里塞一个没人用的声明。
# 注释只认行首起的那种，与 is_fence_removal_of 同一条判据：代码里的 `WRITE_ONCE(*x, 1)` 也有「(*」。
litmus_code_without_comments() { # litmus_code_without_comments <litmus 文件>
  python3 - "$1" <<'PY'
import re
import sys

text = open(sys.argv[1], encoding='utf-8').read()
sys.stdout.write(re.sub(r'^[ \t]*\(\*.*?\*\)', '', text, flags=re.S | re.M))
PY
}
fails=0
declare -A EXPECT
for f in "${FILES[@]}"; do
  rel="${f#"$ROOT"/}"
  exp="$(sed -n 's/.*singlefs-expect:[[:space:]]*\([A-Za-z]*\).*/\1/p' "$f" | head -1)"
  case "$exp" in
    Never|Sometimes|Always) EXPECT["$f"]="$exp" ;;
    *) bad "$rel 缺 (* singlefs-expect: Never|Sometimes *) 声明"
       howto "在文件头注释块里写明期望判定。没有声明 = 跑了但没人看结果，不算验证。"
       fails=$((fails+1)); continue ;;
  esac
  if ! code_without_comments="$(litmus_code_without_comments "$f")"; then
    bad "$rel 剥不掉注释（python3 读它失败），寄存器声明没法查"
    howto "看上面 python3 的报错：多半是文件不是 UTF-8。照 litmus/ 现有文件的编码存一遍再跑。"
    fails=$((fails+1)); continue
  fi
  for r in $(printf '%s\n' "$code_without_comments" | grep -oE '\br[0-9]+\b' | sort -u); do
    grep -qE "^[[:space:]]*int[[:space:]]+$r[[:space:]]*;" <<<"$code_without_comments" \
      || { bad "$rel 用了 $r 但没有 'int $r;'（klitmus7 会在生成 C 之后才报）"
           howto "在用到 $r 的进程体开头补一行 'int $r;'。"
           fails=$((fails+1)); }
  done
  # 声明、init 块与 atomic_t 形参同样只在剥掉注释的正文里找：注释里单独一行 `int r2;` 不是声明过
  init="$(awk '/^\{/{f=1} f{print} f&&/\}/{exit}' <<<"$code_without_comments")"
  for v in $(grep -oE 'atomic_t[[:space:]]*\*[[:space:]]*[A-Za-z_][A-Za-z0-9_]*' <<<"$code_without_comments" \
             | sed -E 's/.*\*[[:space:]]*//' | sort -u); do
    printf '%s\n' "$init" | grep -qE "(^|[^[:alnum:]_])$v[[:space:]]*=" || continue
    printf '%s\n' "$init" | grep -qE "atomic_t[[:space:]]+$v[[:space:]]*=" \
      || { bad "$rel init 里 '$v = ...' 要写成 'atomic_t $v = ...'（只有 klitmus7 会炸）"
           howto "init 块里给 atomic_t 形参赋初值必须带类型，照 litmus/ 的写法。"
           fails=$((fails+1)); }
  done
done
[[ $fails -eq 0 ]] || { bad "静态检查未过：$fails 项"; exit 1; }   # gate-lint:summary

# ── 对照组是不是「这一条去掉屏障的形态」──
# 判据只写在这一处。去掉行首起的 (* … *) 注释、首行 `C <名>` 与空行之后逐行比，比的时候不看空白：
# 对照组只许比原测试少几行屏障，或者把 smp_store_release / smp_load_acquire 放宽成
# WRITE_ONCE / READ_ONCE，至少一处；exists、init、读者一个字都不许改。
# 注释只认行首起的那种：代码里的 `WRITE_ONCE(*x, 1)` 也有「(*」，当成注释开头会吞掉整段代码。
# 是 → 退 0；不是 → 退 1，stdout 打印第一处不同。
is_fence_removal_of() { # is_fence_removal_of <Never 那条> <候选对照组>
  python3 - "$1" "$2" <<'PY'
import re
import sys

FENCE_STATEMENT = re.compile(
    r'^(smp_mb|smp_wmb|smp_rmb|smp_mb__before_atomic|smp_mb__after_atomic'
    r'|smp_mb__after_spinlock|smp_mb__after_unlock_lock|synchronize_rcu)\(\);$')


def statements(path):
    text = open(path, encoding='utf-8').read()
    # 注释换成同样多的换行，报出来的行号才对得上原文件
    text = re.sub(r'^[ \t]*\(\*.*?\*\)', lambda comment: '\n' * comment.group(0).count('\n'),
                  text, flags=re.S | re.M)
    kept = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        shown = line.strip()
        if shown:
            kept.append((line_number, shown, re.sub(r'\s+', '', shown)))
    if kept and kept[0][1].startswith('C '):
        kept = kept[1:]
    return kept


def relaxed(compact):
    compact = re.sub(r'smp_store_release\((\w+),', r'WRITE_ONCE(*\1,', compact)
    return re.sub(r'smp_load_acquire\((\w+)\)', r'READ_ONCE(*\1)', compact)


never_statements = statements(sys.argv[1])
control_statements = statements(sys.argv[2])
never_position = control_position = weakened_count = 0
while never_position < len(never_statements):
    never_number, never_shown, never_compact = never_statements[never_position]
    control_statement = (control_statements[control_position]
                         if control_position < len(control_statements) else None)
    if control_statement and control_statement[2] == never_compact:
        never_position += 1
        control_position += 1
    elif FENCE_STATEMENT.match(never_compact):
        weakened_count += 1
        never_position += 1
    elif (control_statement and relaxed(never_compact) != never_compact
          and relaxed(never_compact) == control_statement[2]):
        weakened_count += 1
        never_position += 1
        control_position += 1
    else:
        where_in_control = (f'对照组第 {control_statement[0]} 行「{control_statement[1]}」'
                            if control_statement else '对照组已经到头')
        print(f'第一处不同：原测试第 {never_number} 行「{never_shown}」，{where_in_control}')
        sys.exit(1)
if control_position < len(control_statements):
    extra_number, extra_shown = control_statements[control_position][:2]
    print(f'对照组第 {extra_number} 行「{extra_shown}」在原测试里没有')
    sys.exit(1)
if weakened_count == 0:
    print('一处屏障都没去掉，内容与原测试相同')
    sys.exit(1)
PY
}

# ── 每条 Never 都要有自己的对照组 ──
n_never=0; n_some=0
for f in "${FILES[@]}"; do
  [[ "${EXPECT[$f]}" == Never ]] && n_never=$((n_never+1))
  [[ "${EXPECT[$f]}" == Sometimes ]] && n_some=$((n_some+1))
done
for f in "${FILES[@]}"; do
  [[ "${EXPECT[$f]}" == Never ]] || continue
  base="${f%.litmus}"; rel="${f#"$ROOT"/}"
  paired=0; rejected_candidates=()
  for g in "${FILES[@]}"; do
    [[ "$g" == "$base"-*.litmus && "${EXPECT[$g]}" == Sometimes ]] || continue
    if difference="$(is_fence_removal_of "$f" "$g")"; then paired=1; break; fi
    rejected_candidates+=("$(basename "$g")：$difference")
  done
  [[ $paired -eq 1 ]] && continue
  fails=$((fails+1))
  if [[ ${#rejected_candidates[@]} -eq 0 ]]; then
    bad "$rel  没有配对的对照组"
    say "        全局有几条 Sometimes 不算数：对照必须是这一条去掉屏障的形态，"
    say "        否则分不清「屏障挡住了」还是「这个模式本来就撞不上」。"
    howto "复制 $(basename "$f") 为 $(basename "$base")-nofence.litmus，删掉里面的屏障" \
          "（smp_wmb / smp_rmb 等），声明改成 (* singlefs-expect: Sometimes *)。" \
          "它判 Sometimes，原来那条的 Never 才有判别力。"
  else
    bad "$rel  没有配对的对照组：同名前缀的 Sometimes 都不是它去掉屏障的形态"
    for rejected_candidate in "${rejected_candidates[@]}"; do say "        $rejected_candidate"; done
    howto "对照组只许比原测试少几行屏障（smp_wmb / smp_rmb / smp_mb 等），或者把 smp_store_release /" \
          "smp_load_acquire 放宽成 WRITE_ONCE / READ_ONCE；exists、init、读者一个字都不许改——改了它回答的就是另一个问题。" \
          "从 $(basename "$f") 复制一份重新删屏障，别在旧的对照组上修。"
  fi
done
[[ $fails -eq 0 ]] || { bad "对照组检查未过：$fails 项"; exit 1; }   # gate-lint:summary

# ── 按条判定：组成表与判定库（文件头「判定库」一段）──
# litmus_facts <根> <litmus>：打这份 litmus 的去注释内容摘要与每行锚点的解析（scripts/gate/lib_function_units.py 切那份 .rs，限定名恰好命中一个单元）：
#   content<TAB><摘要>；declaration<TAB><原文>；none<TAB><理由>；anchor<TAB><文件><TAB><限定名><TAB><单元行>；bad<TAB><卡在哪><TAB><下一步>
litmus_facts() {
  python3 - "$1" "$2" "$LKMM_SCRIPT_DIRECTORY/../../scripts/gate/lib_function_units.py" <<'PY'
import hashlib
import importlib.util
import os
import re
import sys

sys.dont_write_bytecode = True
root, litmus, units_path = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(litmus, encoding="utf-8").read()
content = re.sub(r'^[ \t]*\(\*.*?\*\)', '', text, flags=re.S | re.M)
print("content\t" + hashlib.sha256(content.encode("utf-8")).hexdigest())
specification = importlib.util.spec_from_file_location("lib_function_units_for_lkmm", units_path)
units = importlib.util.module_from_spec(specification)
specification.loader.exec_module(units)
FORMAT = "格式： singlefs-models: <仓内路径> :: <单元限定名>（:: 两边各空一格；impl 的方法写 <impl 类型>::<方法>），例如 crates/singlefs-core/src/transaction.rs :: publish_first_file；不对应代码就写 singlefs-models: none —— <为什么>。"


def qualified_and_rest(rest):
    """限定名一直到深度 0 的第一个空白（<impl …> 里的空白不算），剩下的是注释。"""
    depth = 0
    for position, character in enumerate(rest):
        if character == "<":
            depth += 1
        elif character == ">":
            depth -= 1
        elif character.isspace() and depth == 0:
            return rest[:position], rest[position:].strip()
    return rest, ""


for raw in re.findall(r"singlefs-models:[ \t]*(.*)", text):
    declaration = re.sub(r"\s*\*\)\s*$", "", raw).rstrip()
    print("declaration\t" + declaration)
    if re.match(r"^none([^A-Za-z0-9_].*)?$", declaration):
        print("none\t" + re.sub(r"^none[\s—–:：-]*", "", declaration))
        continue
    found = re.match(r"^(\S+\.rs) :: (\S.*)$", declaration)
    if not found:
        print(f"bad\tsinglefs-models 写成了「{declaration}」：认不出「文件 :: 单元限定名」\t{FORMAT}")
        continue
    path, (qualified, _comment) = found.group(1), qualified_and_rest(found.group(2))
    full = os.path.join(root, path)
    if not os.path.isfile(full):
        print(f"bad\tsinglefs-models 指的 {path} 不存在\t代码搬了家或改了名，就跟着改这一行；那段同步逻辑整个删了，这条 litmus 也该跟着删或改写。")
        continue
    crate = os.path.dirname(path)
    while crate and not os.path.isfile(os.path.join(root, crate, "Cargo.toml")):
        crate = os.path.dirname(crate)
    rust_file = units.RustFile(root, path, open(full, encoding="utf-8", errors="surrogateescape").read(), crate or os.path.dirname(path))
    hits = [unit for unit in rust_file.units if unit.name == qualified and not unit.data]
    if len(hits) != 1:
        where = "、".join(f"第 {unit.first_line} 行" for unit in hits)
        print(f"bad\t{path} 里限定名 {qualified} 命中 {len(hits)} 个单元{'（' + where + '）' if hits else ''}，锚点要恰好命中一个\t"
              f"{'函数改了名就跟着改这一行（python3 scripts/gate/lib_function_units.py dump <根> 看限定名）；改名多半也动了同步的形态，回头对一遍 litmus 的写者和读者。' if not hits else '同一个限定名在这份文件里有几处（cfg 两支、重名的 impl 方法）：锚点判不准是哪一个，把要绑的那一处挪成名字唯一的单元，或锚点写得更细（<impl 类型>::<方法>、<mod>::<名>）。'}")
        continue
    print(f"anchor\t{path}\t{qualified}\t{hits[0].text()}")
PY
}
# unit_key_file <单元行文件> <输出>：单元行写成 lib_verdict_library.py 认的 unit-key 输出（单元行加一行 key：按（种类，指纹）排好取 sha256，不带名字、路径与行号）
unit_key_file() {
  python3 - "$1" "$2" <<'PY'
import hashlib
import sys

lines = [line for line in open(sys.argv[1], encoding="utf-8").read().split("\n") if line]
digest = hashlib.sha256("\n".join(sorted(line.split("\t")[0] + "\t" + line.split("\t")[3] for line in lines)).encode("utf-8")).hexdigest()
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    handle.write("".join(line + "\n" for line in lines) + f"key\t{digest}\n")
PY
}
digest_of() { printf '%s' "$1" | sha256sum | cut -d' ' -f1; }
# 判法单元指纹：这份脚本里判结局的那个函数的单元闭包键（lib_verdict_library.py judging-units；判定库与 lib_function_units.py 不进），写进 $WORK/judging-<函数>.txt
judging_units_file() { # judging_units_file <函数名>
  local output="$WORK/judging-$1.txt"
  [[ -s "$output" ]] && { printf '%s' "$output"; return 0; }
  python3 "$VERDICT_LIBRARY_MODULE" judging-units "$(cd "$LKMM_SCRIPT_DIRECTORY/../.." && pwd)" ".claude/scripts/lkmm.sh::$1" > "$output" || true
  printf '%s' "$output"
}
judged_count=0; reused_count=0
declare -A JUDGED_REASONS=()
# lookup_item <种类> <项> <组成表>：交 0 复用、1 现判；打那一行的前半，原因留在 $item_reason
lookup_item() {
  item_reason="没给判定库"
  if [[ -z "$VERDICT_LIBRARY" ]]; then return 1; fi
  local options=() answer=""
  ((NO_REUSE)) && options=(--no-reuse)
  if answer="$(python3 "$VERDICT_LIBRARY_MODULE" lookup "$VERDICT_LIBRARY" "$1" "$2" "$3" ${options[@]+"${options[@]}"})"; then
    item_reason="${answer#*$'\t'}"; return 0
  fi
  item_reason="$(printf '%s' "$answer" | cut -f2)"
  local detail; detail="$(printf '%s' "$answer" | cut -f3)"
  [[ -n "$detail" ]] && item_reason="$item_reason：$detail"
  return 1
}
record_item() { # record_item <种类> <项> <组成表> <一句判定> <跑法>
  [[ -n "$VERDICT_LIBRARY" ]] || return 0
  python3 "$VERDICT_LIBRARY_MODULE" record "$VERDICT_LIBRARY" "$1" "$2" "$3" --summary "$4" --run "$5" >/dev/null \
    || warn "$2 判绿了，没记进判定库 $VERDICT_LIBRARY：下一趟这一条照判"
}
count_item() { # count_item <复用 0|1> <原因>
  if [[ "$1" == 1 ]]; then reused_count=$((reused_count + 1)); else judged_count=$((judged_count + 1)); JUDGED_REASONS["${2%%：*}"]=$(( ${JUDGED_REASONS["${2%%：*}"]:-0} + 1 )); fi
}
# ── 复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：绑定、herd7 两种判定各一份，组成表都算好之后、判之前从判定库里抽 ──
REUSE_CHECKS_MODULE="$LKMM_SCRIPT_DIRECTORY/../../scripts/gate/lib_reuse_checks.py"
LKMM_SCRIPT_KEY=".claude/scripts/lkmm.sh"
# sentinel_plan <种类> <标签> <状态文件> <项数组名> <组成表数组名>：候选是这把键在库里有判绿记录的项（verdict-library-candidate 按键文件查，与 lookup 分开）
sentinel_plan() {
  local kind="$1" label="$2" state="$3" index candidates="$3.candidates" off=""
  local -n plan_items="$4" plan_tables="$5"
  if [[ -z "$VERDICT_LIBRARY" ]]; then off="没给判定库"; elif ((NO_REUSE)); then off="关了复用（--no-reuse）"; fi
  : > "$candidates"
  for index in "${!plan_items[@]}"; do
    [[ -n "$off" ]] && break
    if python3 "$REUSE_CHECKS_MODULE" verdict-library-candidate "$VERDICT_LIBRARY" "$kind" "${plan_tables[$index]}"; then
      printf '%s\t绿\n' "${plan_items[$index]}" >> "$candidates"
    fi
  done
  if [[ -n "$off" ]]; then
    python3 "$REUSE_CHECKS_MODULE" sentinel-plan "$state" --script "$LKMM_SCRIPT_KEY" --label "$label" --off "$off" || warn "复用哨兵（$label）没抽成"
  else
    python3 "$REUSE_CHECKS_MODULE" sentinel-plan "$state" --script "$LKMM_SCRIPT_KEY" --label "$label" --candidates "$candidates" || warn "复用哨兵（$label）没抽成"
  fi
}
# judge_lookup <种类> <项> <组成表> <状态文件>：交 0 复用、1 现判；judged_word 是那一行写「现判」还是「强制现判」（抽中的那一条不读库）。
# 弄坏开关 GATE_LKMM_BREAK=reuse-lookup-skipped 下一律不读库（复用哨兵该判红「复用失灵」）
judge_lookup() {
  judged_word=现判
  if python3 "$REUSE_CHECKS_MODULE" sentinel-forced "$4" "$2" 2>/dev/null; then
    judged_word=强制现判; item_reason="复用哨兵抽中：键相同，这一趟不读库"; return 1
  fi
  if [[ ",${GATE_LKMM_BREAK:-}," == *,reuse-lookup-skipped,* && -n "$VERDICT_LIBRARY" ]]; then
    item_reason="无记录（弄坏开关 reuse-lookup-skipped：没读库）"; return 1
  fi
  lookup_item "$1" "$2" "$3"
}
sentinel_observe() { # sentinel_observe <状态文件> <项> <结局> <判定>
  python3 "$REUSE_CHECKS_MODULE" sentinel-observe "$@" || warn "复用哨兵没记下 $2 的结局（这一条哨兵这一趟不判）"
}
sentinel_finish() { # sentinel_finish <状态文件>：哨兵被现判、强制现判判定与库里不同交 1（✗ 行带出路）
  python3 "$REUSE_CHECKS_MODULE" sentinel-finish "$1"
}
tally_text() {
  local text="现判 $judged_count、复用 $reused_count" reason columns=()
  for reason in "${!JUDGED_REASONS[@]}"; do columns+=("$reason ${JUDGED_REASONS[$reason]}"); done
  ((${#columns[@]})) && text="$text（现判的原因：$(printf '%s\n' "${columns[@]}" | sort | paste -sd'、')）"
  printf '%s' "$text"
}

# ── 每条 Never 都要绑到代码 ──
# binding_mention_paths <项名> <输出>：crates/ 下写出这份 litmus 的 .rs（相对项目根的路径，NUL 隔开）写进 <输出>，交 grep 的退出码（0 有、1 一份没有、2 及以上读不全）。
#   写出绑定查（check_binding_test_mentions）与组成表「写出文件名的测试」那一行（binding_mentions_value）共用这一份匹配。认的写法：
#   项名（顶层的是文件名，子目录里的是相对 litmus/ 的路径）前面不接文件名字符（字母、数字、_、-、.、/），或者前面紧挨着 litmus/ 这一级目录
#   （litmus 前面同样不接字母、数字、_、-、.，可以接 /，例 ../../litmus/<项名>）；后面不接文件名字符。
#   所以只写了 mp-bar.litmus 的不算写出 bar.litmus，写了 "litmus/bar.litmus" 的算；别的目录下的同名文件（other/bar.litmus、litmus/sub/bar.litmus）也不算写出顶层的 bar.litmus。
#   按字节比（LC_ALL=C）：汉字、引号这类非 ASCII 字符都算边界。
#   弄坏开关 GATE_LKMM_BREAK=binding-mention-substring 退回改前：按子串找（grep -F），不看边界。
binding_mention_paths() {
  local status=0 pattern
  : > "$2"
  [[ -d "$ROOT/crates" ]] || return 1
  if [[ ",${GATE_LKMM_BREAK:-}," == *,binding-mention-substring,* ]]; then
    (cd "$ROOT" && grep -rlZF --include='*.rs' -- "$1" crates) > "$2" 2>/dev/null || status=$?
    return "$status"
  fi
  pattern="$(printf '%s' "$1" | sed 's/[][\.*^$+?(){}|]/\\&/g')"
  (cd "$ROOT" && LC_ALL=C grep -rlZE --include='*.rs' -e "(^|[^A-Za-z0-9_./-]|(^|[^A-Za-z0-9_.-])litmus/)$pattern(\$|[^A-Za-z0-9_./-])" crates) > "$2" 2>/dev/null || status=$?
  return "$status"
}
# check_binding_test_mentions <项名>：crates/ 下有一个 .rs 写出这份 litmus（那是读它、拿它跟代码今天的形态比的测试，门禁按项名认，认法在 binding_mention_paths）
check_binding_test_mentions() {
  binding_mention_paths "$1" "$WORK/mention-check.paths" || true
  [[ -s "$WORK/mention-check.paths" ]]
}
# binding_mentions_value <项名>：组成表「写出文件名的测试」那一行的值（文件头「判定库」一段）。crates/ 下写出这份 litmus 的 .rs（binding_mention_paths）按路径排序，
#   逐份「相对路径<TAB>内容 sha256」取摘要；一份都没有交「无」；grep 读不全（退 2 及以上）交一个每趟都不同的值，这一条不复用
binding_mentions_value() {
  local listing grep_status=0
  listing="$(work_file mentions "$1").txt"
  binding_mention_paths "$1" "$listing.paths" || grep_status=$?
  if ((grep_status > 1)); then printf '读不全（grep 退 %s，%s）' "$grep_status" "$EPOCHREALTIME"; return 0; fi
  LC_ALL=C sort -z "$listing.paths" | while IFS= read -r -d '' mention_path; do
    printf '%s\t%s\n' "$mention_path" "$(sha256sum < "$ROOT/$mention_path" | cut -d' ' -f1)"
  done > "$listing"
  if [[ -s "$listing" ]]; then sha256sum < "$listing" | cut -d' ' -f1; else printf '无'; fi
}
n_bound=0; n_unbound=0
binding_items=(); binding_tables=(); binding_names=(); binding_rels=(); binding_anchor_counts=()
for f in "${FILES[@]}"; do
  [[ "${EXPECT[$f]}" == Never ]] || continue
  rel="${f#"$ROOT"/}"; litmus_key="${LITMUS_KEY[$f]}"
  facts="$(work_file facts "$litmus_key").txt"
  if ! litmus_facts "$ROOT" "$f" > "$facts"; then
    bad "$rel  解析不了锚点（scripts/gate/lib_function_units.py 切不出那份 .rs，输出在上面）"
    howto "先让 python3 scripts/gate/lib_function_units.py dump <根> --language rust 在这棵树上跑通。"
    fails=$((fails+1)); continue
  fi
  declaration_count="$(grep -c '^declaration' "$facts" || true)"
  if [[ "$declaration_count" == 0 ]]; then
    bad "$rel  缺 singlefs-models 声明：说不出它模拟的是哪段代码"
    howto "在文件头注释块里写一行 singlefs-models: crates/<crate>/src/<文件>.rs :: <单元限定名>，写者、读者各一行也行；" \
          "不对应任何代码就写 singlefs-models: none —— <为什么>。"
    fails=$((fails+1)); continue
  fi
  none_count="$(grep -c '^none' "$facts" || true)"; anchor_count="$(grep -c '^anchor' "$facts" || true)"
  if grep -q '^bad' "$facts"; then
    while IFS=$'\t' read -r _kind message next_step; do
      bad "$rel  $message"; howto "$next_step"
    done < <(grep '^bad' "$facts")
    fails=$((fails+1)); continue
  fi
  if [[ $none_count -gt 0 && $anchor_count -gt 0 ]]; then
    bad "$rel  singlefs-models 既写了 none 又写了锚点"
    howto "二选一：它对应代码，就删掉 none 那一行；不对应，就删掉锚点。"
    fails=$((fails+1)); continue
  fi
  if [[ $none_count -gt 0 ]]; then
    if [[ -z "$(grep '^none' "$facts" | head -1 | cut -f2)" ]]; then
      bad "$rel  singlefs-models: none 没写理由"
      howto "写成 singlefs-models: none —— <为什么它不对应代码>。理由不许省：" \
            "不对应代码的 Never 证明的只是一个抽象形态，读的人要知道它凭什么可以不对应。"
      fails=$((fails+1)); continue
    fi
    n_unbound=$((n_unbound+1)); continue
  fi
  # 锚点都恰好命中一个单元；这一条判定（binding:<项名>）按键复用写出文件名那一查，那一查读的 .rs 进键（「写出文件名的测试」那一行）
  anchor_units="$(work_file anchor-units "$litmus_key").txt"; anchor_key="$(work_file anchor-key "$litmus_key").txt"
  grep '^anchor' "$facts" | cut -f4- > "$anchor_units"
  unit_key_file "$anchor_units" "$anchor_key"
  table="$(work_file binding "$litmus_key").tsv"
  { printf 'litmus 去注释内容\t%s\n' "$(grep '^content' "$facts" | cut -f2)"
    printf '锚点声明\t%s\n' "$(digest_of "$(grep '^declaration' "$facts" | cut -f2-)")"
    printf '锚点单元\t@%s\n' "$anchor_key"
    printf 'litmus 文件名\t%s\n' "$(digest_of "$litmus_key")"
    [[ ",${GATE_LKMM_BREAK:-}," == *,binding-key-drops-test-mentions,* ]] || printf '写出文件名的测试\t%s\n' "$(binding_mentions_value "$litmus_key")"
    printf '判法单元指纹\t@%s\n' "$(judging_units_file check_binding_test_mentions)"; } > "$table"
  binding_items+=("binding:$litmus_key"); binding_tables+=("$table"); binding_names+=("$litmus_key"); binding_rels+=("$rel"); binding_anchor_counts+=("$anchor_count")
done
sentinel_plan binding-litmus 绑定 "$WORK/sentinel-binding.json" binding_items binding_tables
for index in "${!binding_items[@]}"; do
  item="${binding_items[$index]}"; table="${binding_tables[$index]}"; litmus_key="${binding_names[$index]}"; rel="${binding_rels[$index]}"
  anchor_count="${binding_anchor_counts[$index]}"
  if judge_lookup binding-litmus "$item" "$table" "$WORK/sentinel-binding.json"; then
    count_item 1 "$item_reason"
    say "  · $item：绿，复用（$item_reason）"   # gate-lint:detail
    sentinel_observe "$WORK/sentinel-binding.json" "$item" 复用 绿
    n_bound=$((n_bound+1)); continue
  fi
  count_item 0 "$item_reason"
  if ! check_binding_test_mentions "$litmus_key"; then
    say "  · $item：红，$judged_word（$item_reason）"   # gate-lint:detail
    sentinel_observe "$WORK/sentinel-binding.json" "$item" "$judged_word" 红
    bad "$rel  crates/ 下没有一个 .rs 写出 $litmus_key —— 没有测试把它和代码对上"
    howto "写一个测试读这个 litmus，拿它的写者 / 读者次序跟代码今天实际发出的次序比" \
          "（例：录制写请求流，按步骤分类后逐项比）。测试里要写出 $litmus_key（顶层的写文件名，子目录里的写相对 litmus/ 的路径，前面可以带 litmus/），" \
          "门禁按这个名字认，前后不接别的文件名字符（只写了 mp-${litmus_key##*/} 那种不算）。" \
          "herd7 只判 litmus 写下的形态：代码改了顺序而 litmus 没跟，判定照样是 Never。"
    fails=$((fails+1)); continue
  fi
  say "  · $item：绿，$judged_word（$item_reason）"   # gate-lint:detail
  sentinel_observe "$WORK/sentinel-binding.json" "$item" "$judged_word" 绿
  record_item binding-litmus "$item" "$table" "$anchor_count 个锚点各恰好命中一个单元，crates/ 下有 .rs 写出 $litmus_key" "lkmm.sh 绑定检查（锚点经 lib_function_units.py 解析）"
  n_bound=$((n_bound+1))
done
sentinel_finish "$WORK/sentinel-binding.json" || fails=$((fails+1))
[[ $fails -eq 0 ]] || { bad "代码绑定检查未过：$fails 项"; exit 1; }   # gate-lint:summary

if [[ $STATIC_ONLY -eq 1 ]]; then
  say ""
  warn "静态检查全过：$n_never 条 Never 每条都有内容对得上的对照组，$n_bound 条绑到代码、$n_unbound 条声明不对应代码；共 $n_some 条 Sometimes"
  warn "herd7 判定没跑（--static-only），退出码 3——这不是通过"
  exit 3
fi

# ── herd7 ──
find_herd7 || { report_herd7_missing; exit 1; }

# ── 内核树（herd7 要在 tools/memory-model 里跑，模型文件是相对路径引的）──
if [[ -z "$KTREE" ]]; then
  for c in "$ROOT/../linux" "$HOME/linux" "$HOME/linux-bug-fix/linux"; do
    [[ -d "$c/tools/memory-model" ]] && { KTREE="$c"; break; }
  done
fi
# 按需取：委托给 fetch-deps.sh，取树的逻辑只有那一份实现
if [[ -z "$KTREE" || ! -d "$KTREE/tools/memory-model" ]]; then
  CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/singlefs/linux-memory-model"
  if [[ -d "$CACHE/tools/memory-model" ]]; then
    KTREE="$CACHE"
  elif [[ -z "${SINGLEFS_NO_FETCH:-}" ]]; then
    bash "$(dirname "${BASH_SOURCE[0]}")/fetch-deps.sh" --kernel || exit 1
    [[ -d "$CACHE/tools/memory-model" ]] && KTREE="$CACHE"
  fi
fi

[[ -n "$KTREE" && -d "$KTREE/tools/memory-model" ]] || {
  bad "找不到带 tools/memory-model 的内核树"
  howto "herd7 要在内核树的 tools/memory-model 里跑（模型文件是相对路径引的）。" \
        "指一棵 Linux 源码树：" \
        "SINGLEFS_KERNEL_TREE=/path/to/linux bash .claude/scripts/lkmm.sh"
  exit 1
}
KTREE="$(readlink -f "$KTREE")"
HERD7_VERSION_LINE="$(herd7 -version 2>&1 | head -1)"
ok "内核树 $KTREE"
ok "herd7  $HERD7_VERSION_LINE"
# herd7 那一条判定共用的几样（文件头「判定库」一段）：herd7 本身、内存模型文件、内核树的版本
HERD7_IDENTITY="$(digest_of "$HERD7_VERSION_LINE"$'\t'"$(sha256sum "$(command -v herd7)" | cut -d' ' -f1)")"
MODEL_DIGEST="$(cd "$KTREE/tools/memory-model" && find . -maxdepth 1 -type f \( -name '*.cfg' -o -name '*.cat' -o -name '*.bell' -o -name '*.def' \) -printf '%f\n' \
                | LC_ALL=C sort | while IFS= read -r model_file; do printf '%s\t%s\n' "$model_file" "$(sha256sum "$model_file" | cut -d' ' -f1)"; done | sha256sum | cut -d' ' -f1)"
if KTREE_VERSION="$(git -C "$KTREE" rev-parse HEAD 2>/dev/null)"; then
  KTREE_VERSION="git $KTREE_VERSION"
elif [[ -f "$KTREE/Makefile" ]]; then
  KTREE_VERSION="$(grep -E '^(VERSION|PATCHLEVEL|SUBLEVEL|EXTRAVERSION) *=' "$KTREE/Makefile" | head -4 | paste -sd' ')"
else
  KTREE_VERSION="不是 git 树、没有 Makefile"
fi

# judge_herd7_output <herd7 的输出> <期望>：交 0 判定与声明相符；1 读不到 Observation；2 与声明不符。实际的判定写进 $herd7_observation
judge_herd7_output() {
  herd7_observation="$(printf '%s\n' "$1" | sed -n 's/^Observation[[:space:]]\+[^[:space:]]\+[[:space:]]\+\([A-Za-z]*\).*/\1/p' | head -1)"
  [[ -n "$herd7_observation" ]] || return 1
  [[ "$herd7_observation" == "$2" ]] || return 2
  return 0
}

# ── 跑 ──
say ""
cd "$KTREE/tools/memory-model" || die "进不去内核树的 tools/memory-model：$KTREE/tools/memory-model" "核一下 LKMM 内核树的路径（文件头写着从哪取）；树不全就重新取一份"
herd7_items=(); herd7_tables=()
for f in "${FILES[@]}"; do
  want="${EXPECT[$f]}"
  litmus_key="${LITMUS_KEY[$f]}"
  table="$(work_file herd7 "$litmus_key").tsv"
  { printf 'litmus 去注释内容\t%s\n' "$(digest_of "$(litmus_code_without_comments "$f")")"
    [[ ",${GATE_LKMM_BREAK:-}," == *,litmus-key-drops-every-comment,* ]] || printf '期望\t%s\n' "$(digest_of "$want")"
    printf 'herd7\t%s\n' "$HERD7_IDENTITY"
    printf '内存模型\t%s\n' "$MODEL_DIGEST"
    printf '内核树版本\t%s\n' "$(digest_of "$KTREE_VERSION")"
    printf '判法单元指纹\t@%s\n' "$(judging_units_file judge_herd7_output)"; } > "$table"
  herd7_items+=("herd7:$litmus_key"); herd7_tables+=("$table")
done
sentinel_plan herd7-litmus herd7 "$WORK/sentinel-herd7.json" herd7_items herd7_tables
for index in "${!FILES[@]}"; do
  f="${FILES[$index]}"
  rel="litmus/${LITMUS_KEY[$f]}"   # 此时已 cd 进内核树，不能再算相对路径
  want="${EXPECT[$f]}"
  litmus_key="${LITMUS_KEY[$f]}"
  item="${herd7_items[$index]}"; table="${herd7_tables[$index]}"
  if judge_lookup herd7-litmus "$item" "$table" "$WORK/sentinel-herd7.json"; then
    count_item 1 "$item_reason"
    ok "$rel  $want（复用：$item_reason）"
    say "  · $item：绿，复用（$item_reason）"   # gate-lint:detail
    sentinel_observe "$WORK/sentinel-herd7.json" "$item" 复用 绿
    continue
  fi
  count_item 0 "$item_reason"
  out="$(timeout 300 herd7 -conf linux-kernel.cfg "$f" 2>&1)" || {
    say "  · $item：红，$judged_word（$item_reason）"   # gate-lint:detail
    sentinel_observe "$WORK/sentinel-herd7.json" "$item" "$judged_word" 红
    bad "$rel  herd7 执行失败"; printf '%s\n' "$out" | tail -5 | sed 's/^/        /'
    howto "看上面的报错。多半是 litmus 语法问题，照 litmus/ 现有文件的格式改；" \
          "超时（300s）则是状态空间太大，把进程数或变量数减下来。"
    fails=$((fails+1)); continue
  }
  if judge_herd7_output "$out" "$want"; then herd7_verdict=0; else herd7_verdict=$?; fi
  if [[ $herd7_verdict == 1 ]]; then
    say "  · $item：红，$judged_word（$item_reason）"   # gate-lint:detail
    sentinel_observe "$WORK/sentinel-herd7.json" "$item" "$judged_word" 红
    bad "$rel  读不到 Observation 行 —— 判定不明，整条作废"
    printf '%s\n' "$out" | tail -5 | sed 's/^/        /'; fails=$((fails+1))
    howto "多半是 litmus 语法错。最常见的一处：进程签名后面不能跟行内注释，" \
          "注释只能写在文件头的 (* ... *) 块里。照 litmus/ 现有文件的格式改。"
  elif [[ $herd7_verdict == 0 ]]; then
    say "  · $item：绿，$judged_word（$item_reason）"   # gate-lint:detail
    sentinel_observe "$WORK/sentinel-herd7.json" "$item" "$judged_word" 绿
    ok "$rel  $herd7_observation（符合声明）"
    record_item herd7-litmus "$item" "$table" "herd7 判 $herd7_observation，与声明相符" "herd7 -conf linux-kernel.cfg litmus/$litmus_key（$HERD7_VERSION_LINE）"
  else
    say "  · $item：红，$judged_word（$item_reason）"   # gate-lint:detail
    sentinel_observe "$WORK/sentinel-herd7.json" "$item" "$judged_word" 红
    bad "$rel  期望 $want，实际 $herd7_observation"
    printf '%s\n' "$out" | grep -E '^(States|Condition|Observation)' | sed 's/^/        /' || true
    howto "两种可能，别急着改声明：" \
      "① 代码或模型真的少了屏障 → 这正是这条测试要抓的东西，去补屏障" \
      "② 这条声明本来就写错了   → 改 (* singlefs-expect: ... *)" \
      "先想清楚是哪一种。直接把声明改成实际值，等于把测试关掉。"
    fails=$((fails+1))
  fi
done

say ""
say "  · 按条：$(tally_text)"   # gate-lint:detail
sentinel_finish "$WORK/sentinel-herd7.json" || fails=$((fails+1))
[[ $fails -eq 0 ]] || { bad "LKMM 未通过：$fails 项"; exit 1; }   # gate-lint:summary
ok "LKMM 通过（$n_never 条 Never：每条都有内容对得上的对照组，$n_bound 条绑到代码、$n_unbound 条声明不对应代码；共 $n_some 条 Sometimes；按条$(tally_text)）"
