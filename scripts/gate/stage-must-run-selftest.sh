#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command git
# `stage-must-run.sh` 的自证：在一个临时小仓里造几种局面，逐格核它判「要跑」还是「可跳过」；格数由成功行现算。
# 判别力靠的是**同一个仓、同一道阶段，只换一样东西**，两次判定必须不同；只证明会红不够，
# 还要证明它分得出差别（`.claude/singlefs-ai-sop/rules/test-discipline.md`「检查本身也可能是错的」）。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREDICATE="$HERE/stage-must-run.sh"
work="$(mktemp -d)"
trap 'rm -rf "${work:?}"' EXIT
failures=0
checked=0

note() { printf '  %s %s\n' "$1" "$2"; }
expect() { # <期望退出码> <说明> <环境赋值…>
  local want="$1" what="$2"; shift 2
  local out rc
  checked=$((checked + 1))
  out="$(env "$@" bash "$PREDICATE" "$work" demo-stage.sh 2>&1)"; rc=$?
  if [[ "$rc" == "$want" ]]; then
    note "✓" "$what（退出码 $rc）"
  else
    note "✗" "$what：要 $want，实际 $rc —— $out"
    failures=$((failures + 1))
  fi
}

export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master "$work"
mkdir -p "$work/.claude/gate.d" "$work/crates/demo/src" "$work/records"
printf 'pub fn one() -> u32 { 1 }\n' > "$work/crates/demo/src/lib.rs"
printf '[workspace]\n' > "$work/Cargo.toml"
printf '# lock\n' > "$work/Cargo.lock"
printf '# 清单\ndemo-stage.sh\tcrates/ Cargo.toml Cargo.lock\t# 跑 crates 里的东西\n' > "$work/.claude/gate.d/stage-inputs.tsv"
git -C "$work" add -A && git -C "$work" commit -qm base

tree_now() { git -C "$work" write-tree; }
set_green() { git -C "$work" update-ref refs/sop/staged-green "$(git -C "$work" commit-tree "$1" -m green)"; }

base_tree="$(tree_now)"

# ① 还没有过全绿的暂存树 ⇒ 要跑
expect 0 "没有 staged-green 时判要跑" SINGLEFS_STAGED_TREE="$base_tree"
# ② 没给 SINGLEFS_STAGED_TREE（不是 gate-staged.sh 起的）⇒ 要跑，哪怕 ref 已经有了
set_green "$base_tree"
expect 0 "不是 gate-staged.sh 起的时候不许复用" SINGLEFS_UNUSED=1
# ③ 输入逐字相同 ⇒ 可跳过
expect 1 "输入逐字相同时判可跳过" SINGLEFS_STAGED_TREE="$base_tree"
# ④ 动一个 crates 里的文件 ⇒ 要跑
printf 'pub fn one() -> u32 { 2 }\n' > "$work/crates/demo/src/lib.rs"
git -C "$work" add -A
expect 0 "crates 里改了一行就判要跑" SINGLEFS_STAGED_TREE="$(tree_now)"
# ⑤ 只改一份不在清单里的文件 ⇒ 可跳过
# 把内容写回去，不用 git 的撤销命令（`command-safety.md`：脚本里一律不许出现它们，
# 脚本跑起来的时候没人在旁边看 git status，而那些命令没有 undo）。
# 这里要退回的就是上一格改的那一行，直接写回原样最直白。
printf "pub fn one() -> u32 { 1 }\\n" > "$work/crates/demo/src/lib.rs"
printf '改了一行文档\n' > "$work/records/note.md"
git -C "$work" add -A
expect 1 "只改清单外的文件仍判可跳过" SINGLEFS_STAGED_TREE="$(tree_now)"
# ⑥ 清单自己少写一条 ⇒ 要跑（不然那条输入永远不会让它重跑）
printf '# 清单\ndemo-stage.sh\tcrates/\t# 少写了 Cargo.toml 与 Cargo.lock\n' > "$work/.claude/gate.d/stage-inputs.tsv"
git -C "$work" add -A
expect 0 "清单自己变了就判要跑" SINGLEFS_STAGED_TREE="$(tree_now)"
printf "# 清单\\ndemo-stage.sh\\tcrates/ Cargo.toml Cargo.lock\\t# 跑 crates 里的东西\\n" > "$work/.claude/gate.d/stage-inputs.tsv" && git -C "$work" add -A
# ⑦ 不看判绿那一次是多久以前：绿树是 30 天前那一次也照样可跳过（同一棵树，只把绿提交的时刻往前拨）
set_green_dated() { git -C "$work" update-ref refs/sop/staged-green "$(GIT_COMMITTER_DATE="$2" GIT_AUTHOR_DATE="$2" git -C "$work" commit-tree "$1" -m green)"; }
set_green_dated "$base_tree" "$(( $(date +%s) - 30 * 86400 )) +0000"
expect 1 "判绿那一次是 30 天前也照样可跳过（复用不看时刻）" SINGLEFS_STAGED_TREE="$base_tree"
set_green "$base_tree"
# ⑧ 强制全跑
expect 0 "SINGLEFS_GATE_FULL=1 强制跑" SINGLEFS_STAGED_TREE="$base_tree" SINGLEFS_GATE_FULL=1
# ⑨ 只改阶段脚本自己 ⇒ 要跑（判据本身变了，上一次的判定不再作数；清单里没登记它也一样）
printf '#!/usr/bin/env bash\nexit 0\n' > "$work/.claude/gate.d/demo-stage.sh"
git -C "$work" add -A
expect 0 "只改阶段脚本自己也判要跑" SINGLEFS_STAGED_TREE="$(tree_now)"
rm -f "${work:?}/.claude/gate.d/demo-stage.sh" && git -C "$work" add -A
# ⑩ 同一道阶段在清单里写了两行 ⇒ 两行的路径都要进比对；只动第二行登记的路径也判要跑
mkdir -p "$work/docs" && printf 'v1\n' > "$work/docs/d.md"
printf "# 清单\ndemo-stage.sh\tcrates/ Cargo.toml Cargo.lock\t# 第一行\ndemo-stage.sh\tdocs/\t# 第二行\n" > "$work/.claude/gate.d/stage-inputs.tsv"
git -C "$work" add -A && set_green "$(tree_now)"
expect 1 "两行登记的路径都没变时可跳过" SINGLEFS_STAGED_TREE="$(tree_now)"
printf 'v2\n' > "$work/docs/d.md" && git -C "$work" add -A
expect 0 "只动第二行登记的路径也判要跑" SINGLEFS_STAGED_TREE="$(tree_now)"
# 只改注释 ⇒ 可跳过：两棵树逐文件按源码摘要比，注释与空白不算（C800；scripts/gate/lib_source_digest.py）
#    .rs 加一行 // 注释、改文档注释措辞、加行尾注释与换行；阶段脚本只改 # 行、加空行与行尾注释
printf "# 清单\ndemo-stage.sh\tcrates/ Cargo.toml Cargo.lock\t# 跑 crates 里的东西\n" > "$work/.claude/gate.d/stage-inputs.tsv"
rm -rf "${work:?}/docs"
printf '/// 返回一\npub fn one() -> u32 { 1 }\n' > "$work/crates/demo/src/lib.rs"
printf '#!/usr/bin/env bash\n# 阶段说明\nexit 0\n' > "$work/.claude/gate.d/demo-stage.sh"
git -C "$work" add -A && set_green "$(tree_now)"
printf '// 只加了一行注释\n/// 返回常数一（措辞换了）\npub fn one() -> u32 {\n    1 // 行尾注释\n}\n' > "$work/crates/demo/src/lib.rs"
printf '#!/usr/bin/env bash\n# 阶段说明改了措辞\n\nexit 0   # 行尾注释\n' > "$work/.claude/gate.d/demo-stage.sh"
git -C "$work" add -A
comments_only_tree="$(tree_now)"
expect 1 "只改注释（.rs 加一行 // 注释、改文档注释措辞、阶段脚本只改 # 行）判可跳过" SINGLEFS_STAGED_TREE="$comments_only_tree"
# 同一棵只差注释的树，弄坏开关 gate-reuse-by-blob（退回逐字比）下判要跑：「只改注释判可跳过」那一格分得出逐字比与按摘要比
expect 0 "弄坏开关 gate-reuse-by-blob 下「只改注释判可跳过」那一格转红（判要跑）" SINGLEFS_STAGED_TREE="$comments_only_tree" ADMISSION_BREAK=gate-reuse-by-blob
# crates 下加一份没人读的 .md ⇒ 可跳过（不进指纹）
printf '没人读的说明\n' > "$work/crates/demo/notes.md" && git -C "$work" add -A
expect 1 "crates 下加一份没人读的 .md 判可跳过" SINGLEFS_STAGED_TREE="$(tree_now)"
# 在只差注释的基础上改一个字面量 ⇒ 要跑
printf '// 只加了一行注释\n/// 返回常数一（措辞换了）\npub fn one() -> u32 {\n    2 // 行尾注释\n}\n' > "$work/crates/demo/src/lib.rs"
git -C "$work" add -A
expect 0 "只差注释之外再改一个字面量（1 → 2）就判要跑" SINGLEFS_STAGED_TREE="$(tree_now)"

if ((failures > 0)); then
  echo "  ✗ stage-must-run.sh 自证没过：$failures 格判错"
  echo "     → 怎么办：照上面每一格的「要 X 实际 Y」改被测脚本；判据与为什么这么定见它的文件头。"
  exit 1
fi
echo "  ✓ stage-must-run.sh 自证通过：$checked 格都对（没有绿树、不是 gate-staged.sh 起的、输入相同、crates 改了、清单外改了、清单自己变了、判绿那一次很久以前、强制全跑、阶段脚本自己变了、清单里同一道阶段写两行、只改注释、弄坏开关 gate-reuse-by-blob、crates 下没人读的 .md、改字面量）"
