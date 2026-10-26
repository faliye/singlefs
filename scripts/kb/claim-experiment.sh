#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command git python3
# 取实验号并当场占住：查号与建第一个文件在同一步里做，建文件用排他方式（noclobber）。
#
#   bash scripts/kb/claim-experiment.sh --next              # 打印下一个没人用的实验号
#   bash scripts/kb/claim-experiment.sh E139 简称            # 占住 E139：建 research/prompts/e139-preregistration.md
#   bash scripts/kb/claim-experiment.sh --selftest
#
# 为什么要有它：2026-09-11 两个会话前后几分钟都取了 E137——一边查过号、隔几轮对话才写文件，
# 另一边在这中间写了同名的未提交跑前登记，被 Write 整份盖掉（没进过 git），靠对话记录逐字复原。
# 查号与写文件分两步，中间就有窗口；这里把两步并成一步，写文件用 O_EXCL（set -C），
# 两个会话同时占同一个号时只有一个成功。
#
# 查号的范围：目录登记 .claude/gate.d/experiment-number-directories.tsv 列的每一处（跑前登记、实验源码、入库装置、产物、变异表、kb 实验页与索引行）——
# 哪一处出现过这个号都算用过。登记的读法在 scripts/kb/lib_experiment_number_directories.py，门禁 doc-experiments 的 experiment-orphans 格读同一份登记、同一份读法；
# 实验号住进新地方就在登记里加一行，这里跟着认。登记取占号那一棵工作树里的那份；登记不在、写坏，或哪个目录读不了，拒绝给号（宁可不给，不能少查）。
# 这些处在 `git worktree list --porcelain` 列出的每一棵工作树里都查，不只查本棵：主工作区和开发分支的工作树各取各的号，
# E185 到 E187 撞过。--next 取全部工作树里最大的号加一；占号时别的工作树里用过的号一样被拒，占号的文件建在本棵工作树里。
# 不在 git 仓里（临时目录）就只查给定的那一棵；在 git 仓里而 git 读不出工作树清单时拒绝给号（宁可不给，不能少查一棵）。
# 自证会红：--selftest 在临时目录里摆几处已用的号，确认 --next、「已用的号被拒」「同一个号第二次占被拒」；
# 再起一个临时 git 仓和它的另一棵工作树，确认另一棵里用过的号在本棵被拒、--next 取两棵的最大号加一、两棵工作树互相占不到同一个号；
# 再摆一棵带目录登记的临时树，确认入库装置目录里的号被拒、登记里加一行 notes 之后 notes 下的号被拒且 --next 跟着涨、登记不在时拒绝给号。
# 弄坏开关（都让 --selftest 判红，门禁 code-tooling 的 break-switch-replay 格逐条复跑，登记在 .claude/gate.d/break-switch-replay-files.tsv）：
#   CLAIM_EXPERIMENT_BREAK=used-check-skipped  claim() 跳过「已经有人用了」那一判：已有源码的号被占成功
#     查号那一格（「✗ selftest: E6 已有源码却没被拒」）
#   CLAIM_EXPERIMENT_BREAK=worktrees-skipped  worktree_roots() 不读工作树清单、只给给定的那一棵：另一棵里用过的号被占成功、--next 少数
#     跨工作树那一格（「✗ selftest: 另一棵工作树用过 E12，本棵 --next 该是 E13」）
#   CLAIM_EXPERIMENT_BREAK=registry-ignored  查号不读目录登记，退回改前写死的那组目录（不看入库装置目录、不认登记里新加的一处、登记不在也照给号）
#     目录登记那一格（「✗ selftest: 目录登记：」）
#   CLAIM_EXPERIMENT_BREAK 只在 --selftest 里认：平常占号、--next 设了它退 2 并点名这个变量。
set -uo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

if [[ -n "${CLAIM_EXPERIMENT_BREAK:-}" && "${1:-}" != --selftest ]]; then
  echo "  ✗ CLAIM_EXPERIMENT_BREAK=$CLAIM_EXPERIMENT_BREAK 只在 --selftest（自证）里认：平常占号、--next 设了它会少查（不查号、少查工作树或目录），把用过的号再发出去"
  echo "    → unset CLAIM_EXPERIMENT_BREAK 再跑；要看开关打开后自证红在哪，跑 CLAIM_EXPERIMENT_BREAK=<used-check-skipped|worktrees-skipped|registry-ignored> bash scripts/kb/claim-experiment.sh --selftest"
  exit 2
fi

LIB_EXPERIMENT_NUMBER_DIRECTORIES="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib_experiment_number_directories.py"
REGISTRY_RELATIVE_PATH=.claude/gate.d/experiment-number-directories.tsv
# 改前写死的那组目录：只给弄坏开关 CLAIM_EXPERIMENT_BREAK=registry-ignored 用
PRE_REGISTRY_DIRECTORIES=$'research/prompts\te-number-names\t# 改前写死的那一组\nscripts/experiment-apparatus\te-number-names\t# 改前写死的那一组\nresearch/results\te-number-names\t# 改前写死的那一组\nscripts/mutation/tables\te-number-names\t# 改前写死的那一组\n.claude/kb/experiments\tnumber-names\t# 改前写死的那一组\n.claude/kb/experiments.md\tindex-rows\t# 改前写死的那一组'

# 给定的这一棵加上同一个仓的其余工作树，一行一个根；清单读不出（在 git 仓里而 git 失败）退 1，不在 git 仓里只给这一棵。
worktree_roots() {
  local root="$1" listing line path
  printf '%s\n' "$root"
  [[ "${CLAIM_EXPERIMENT_BREAK:-}" == worktrees-skipped ]] && return 0
  git -C "$root" rev-parse --is-inside-work-tree >/dev/null 2>&1 || return 0
  if ! listing="$(git -C "$root" worktree list --porcelain 2>/dev/null)"; then
    echo "  ✗ $root 在 git 仓里，但 git worktree list 读不出工作树清单，不知道别的工作树用了哪些号" >&2
    echo "    → 在仓里单跑 git worktree list --porcelain 看它报什么；修好之前不取号，免得和别的工作树撞号" >&2
    return 1
  fi
  while IFS= read -r line; do
    [[ "$line" == "worktree "* ]] || continue
    path="${line#worktree }"
    # 已经被删掉、还没 prune 的工作树目录不在了，没有号可查
    [[ -d "$path" ]] && printf '%s\n' "$path"
  done <<<"$listing"
  return 0
}

# 一棵树里用过的号，一行一个（照目录登记查）。登记写坏或读不了目录退 1，原因由读法那份库打在标准错误。
used_numbers_in_tree() {
  local tree="$1" registry="$2" listing
  if [[ "${CLAIM_EXPERIMENT_BREAK:-}" == registry-ignored ]]; then
    listing="$(python3 "$LIB_EXPERIMENT_NUMBER_DIRECTORIES" numbers - "$tree" <<<"$PRE_REGISTRY_DIRECTORIES")" || return 1
  else
    listing="$(python3 "$LIB_EXPERIMENT_NUMBER_DIRECTORIES" numbers "$registry" "$tree")" || return 1
  fi
  awk -F'\t' '$1 == "number" { print $2 }' <<<"$listing"
}

# 全部工作树里用过的号，升序去重。登记不在、写坏，工作树清单或哪个目录读不了，退 1、什么都不打。
used_numbers() {
  local root="$1" registry roots tree listing collected=""
  registry="$root/$REGISTRY_RELATIVE_PATH"
  if [[ "${CLAIM_EXPERIMENT_BREAK:-}" != registry-ignored && ! -f "$registry" ]]; then
    echo "  ✗ $registry 不在：不知道实验号住在哪几处，查不全" >&2
    echo "    → 从 git 找回这份登记（git log --diff-filter=D --name-only -- $REGISTRY_RELATIVE_PATH），写法在它的文件头；修好之前不取号" >&2
    return 1
  fi
  roots="$(worktree_roots "$root")" || return 1
  while IFS= read -r tree; do
    [[ -n "$tree" ]] || continue
    listing="$(used_numbers_in_tree "$tree" "$registry")" || return 1
    collected+="$listing"$'\n'
  done <<<"$roots"
  sort -un <<<"$collected" | sed '/^$/d'
}

next_number() {
  local used highest
  used="$(used_numbers "$1")" || return 1
  highest="$(tail -1 <<<"$used")"
  echo "E$(( ${highest:-0} + 1 ))"
}

claim() {
  local root="$1" tag="$2" short_name="$3" number file used used_count tree_count
  if [[ ! "$tag" =~ ^E([0-9]+)$ ]]; then
    echo "  ✗ 实验号要写成 E<数字>，拿到的是「$tag」"
    echo "    → 例：bash scripts/kb/claim-experiment.sh E139 某某的代价；不知道用几号先跑 --next"
    return 2
  fi
  number="${BASH_REMATCH[1]}"
  if [[ -z "$short_name" ]]; then
    echo "  ✗ 没给简称"
    echo "    → 简称写进跑前登记的标题；编号在别处引用时要带着它"
    return 2
  fi
  used="$(used_numbers "$root")" || return 3
  if [[ "${CLAIM_EXPERIMENT_BREAK:-}" != used-check-skipped ]] && grep -qx "$number" <<<"$used"; then
    echo "  ✗ E$number 已经有人用了（本棵或同一个仓的别的工作树）"
    echo "    → 取下一个：bash scripts/kb/claim-experiment.sh --next"
    return 1
  fi
  file="$root/research/prompts/e$number-preregistration.md"
  mkdir -p "$root/research/prompts"
  if ! ( set -C; printf '# E%s 跑前登记：%s\n\n写于 %s，装置写之前。\n' "$number" "$short_name" \
         "$(TZ=Asia/Tokyo date +%F)" > "$file" ) 2>/dev/null; then
    echo "  ✗ $file 已经存在（刚被别的会话占了）"
    echo "    → 取下一个：bash scripts/kb/claim-experiment.sh --next"
    return 1
  fi
  used_count="$(grep -c . <<<"$used" || true)"
  tree_count="$(worktree_roots "$root" | sort -u | wc -l)"
  echo "  ✓ 占住 E$number：$file（对过 $tree_count 棵工作树里用过的 $used_count 个号）"
}

# 自证的临时树用的目录登记：与仓里那份同样五处，自证不依赖仓里那份的内容。
SELFTEST_REGISTRY=$'research\te-number-names\t# 样本：跑前登记与产物\nscripts\te-number-names\t# 样本：实验源码与变异表\ncrates/singlefs-checker-tier/src/bin\te-number-names\t# 样本：入库装置\n.claude/kb/experiments\tnumber-names\t# 样本：实验页\n.claude/kb/experiments.md\tindex-rows\t# 样本：实验索引'

# 自证的一条失败：失败行带 ✗（门禁 break-switch-replay 格只在带 ✗ 的行里找点名的项），下一行写怎么办。
selftest_fail() {
  echo "  ✗ selftest: $1"
  echo "    → 读这一条对应的函数（claim、worktree_roots、used_numbers_in_tree）；设着 CLAIM_EXPERIMENT_BREAK 的，这一条红是开关该有的结局"
  return 1
}

write_selftest_registry() {
  mkdir -p "$1/.claude/gate.d" && printf '%s\n' "$SELFTEST_REGISTRY" > "$1/$REGISTRY_RELATIVE_PATH"
}

selftest() {
  local root next rc_used rc_first rc_again
  local repo other repo_next other_next rc_other rc_repo_first rc_other_again rc_other_second
  local registry_root bare_root next_before next_after rc_bin rc_notes bare_next rc_bare
  root="$(mktemp -d)"
  repo="$(mktemp -d)"
  other="$(mktemp -d)"
  registry_root="$(mktemp -d)"
  bare_root="$(mktemp -d)"
  # 中途报错、被打断也删（成功、失败都删；local 变量到退出时已不在，路径在这里就展开）
  trap "rm -rf -- '$root' '$repo' '$other' '$registry_root' '$bare_root'" EXIT
  mkdir -p "$root/research/prompts" "$root/scripts/experiment-apparatus" "$root/.claude/kb/experiments"
  write_selftest_registry "$root"
  : > "$root/research/prompts/e5-preregistration.md"
  : > "$root/scripts/experiment-apparatus/e6_sample.rs"
  : > "$root/.claude/kb/experiments/7-样本.md"
  printf '| E%s（%s） | 已跑 |\n' 8 样本 > "$root/.claude/kb/experiments.md"   # 拼出来写：源码里不留编号加简称的字面，免得被当成对 E8 的引用
  next="$(next_number "$root")"
  claim "$root" E6 样本 >/dev/null; rc_used=$?
  claim "$root" E9 样本 >/dev/null; rc_first=$?
  claim "$root" E9 样本 >/dev/null; rc_again=$?
  rm -rf "${root:?}"
  [[ "$next" == E9 ]] || { selftest_fail "--next 该是 E9，拿到 $next"; return 1; }
  [[ $rc_used -eq 1 ]] || { selftest_fail "E6 已有源码却没被拒"; return 1; }
  [[ $rc_first -eq 0 ]] || { selftest_fail "E9 没人用却没占成"; return 1; }
  [[ $rc_again -eq 1 ]] || { selftest_fail "E9 第二次占也成功了"; return 1; }

  # 两棵工作树：临时 git 仓（本棵，用过 E5）加它的另一棵（用过 E12，只在另一棵里）。目录登记随提交进两棵。
  mkdir -p "$repo/research/prompts"
  write_selftest_registry "$repo"
  : > "$repo/research/prompts/e5-preregistration.md"
  git -C "$repo" init -q || { selftest_fail "临时 git 仓建不起来"; return 1; }
  git -C "$repo" add -A
  git -C "$repo" -c user.email=selftest@example.invalid -c user.name=selftest -c commit.gpgsign=false commit -q -m 样本 \
    || { selftest_fail "临时 git 仓提交不成"; return 1; }
  git -C "$repo" worktree add -q -b other-tree "$other" >/dev/null 2>&1 || { selftest_fail "临时 git 仓加不出另一棵工作树"; return 1; }
  mkdir -p "$other/research/prompts"
  : > "$other/research/prompts/e12-preregistration.md"
  repo_next="$(next_number "$repo")"
  other_next="$(next_number "$other")"
  claim "$repo" E12 样本 >/dev/null; rc_other=$?
  claim "$repo" E13 样本 >/dev/null; rc_repo_first=$?
  claim "$other" E13 样本 >/dev/null; rc_other_again=$?
  claim "$other" E14 样本 >/dev/null; rc_other_second=$?
  [[ "$repo_next" == E13 ]] || { selftest_fail "另一棵工作树用过 E12，本棵 --next 该是 E13，拿到 $repo_next"; return 1; }
  [[ "$other_next" == E13 ]] || { selftest_fail "本棵用过 E5、另一棵用过 E12，另一棵 --next 该是 E13，拿到 $other_next"; return 1; }
  [[ $rc_other -eq 1 ]] || { selftest_fail "E12 只在另一棵工作树里用过，本棵占它却没被拒（退 $rc_other）"; return 1; }
  [[ $rc_repo_first -eq 0 ]] || { selftest_fail "E13 两棵里都没人用却没占成（退 $rc_repo_first）"; return 1; }
  [[ $rc_other_again -eq 1 ]] || { selftest_fail "E13 刚被本棵占了，另一棵又占成功了（退 $rc_other_again）"; return 1; }
  [[ $rc_other_second -eq 0 ]] || { selftest_fail "E14 两棵里都没人用，另一棵没占成（退 $rc_other_second）"; return 1; }

  # 目录登记：入库装置目录里的 E40 被拒；登记里加一行 notes 之后 notes 下的 E50 被拒、--next 从 E41 涨到 E51；登记不在时拒绝给号。
  mkdir -p "$registry_root/crates/singlefs-checker-tier/src/bin" "$registry_root/notes"
  write_selftest_registry "$registry_root"
  : > "$registry_root/crates/singlefs-checker-tier/src/bin/e40_sample.rs"
  : > "$registry_root/notes/e50-sample.md"
  next_before="$(next_number "$registry_root" 2>/dev/null)"
  claim "$registry_root" E40 样本 >/dev/null 2>&1; rc_bin=$?
  printf 'notes\te-number-names\t# 样本：登记里新加的一处目录\n' >> "$registry_root/$REGISTRY_RELATIVE_PATH"
  next_after="$(next_number "$registry_root" 2>/dev/null)"
  claim "$registry_root" E50 样本 >/dev/null 2>&1; rc_notes=$?
  bare_next="$(next_number "$bare_root" 2>/dev/null)"; rc_bare=$?
  [[ "$next_before" == E41 ]] || { selftest_fail "目录登记：入库装置目录里有 E40、登记里还没有 notes，--next 该是 E41，拿到 ${next_before:-（空）}"; return 1; }
  [[ $rc_bin -eq 1 ]] || { selftest_fail "目录登记：E40 只在入库装置目录里用过，占它却没被拒（退 $rc_bin）"; return 1; }
  [[ "$next_after" == E51 ]] || { selftest_fail "目录登记：登记里加了 notes 那一行，--next 该涨到 E51，拿到 ${next_after:-（空）}"; return 1; }
  [[ $rc_notes -eq 1 ]] || { selftest_fail "目录登记：登记里加了 notes 那一行，notes 下用过的 E50 占它却没被拒（退 $rc_notes）"; return 1; }
  [[ $rc_bare -ne 0 && -z "$bare_next" ]] || { selftest_fail "目录登记：树里没有目录登记，该拒绝给号，却给了 ${bare_next:-（空）}（退 $rc_bare）"; return 1; }
  echo "selftest: 通过（--next 取到 E9；已有源码的 E6 被拒；E9 第一次占成、第二次被拒；两棵工作树：另一棵的 E12 在本棵被拒、--next 取两棵最大号加一、E13 只有一棵占得成；" \
    "目录登记：入库装置目录里的 E40 被拒，登记里加 notes 一行之后 --next 从 E41 涨到 E51、E50 被拒，没有登记时拒绝给号）"
}

case "${1:-}" in
  --selftest) selftest; exit $? ;;
  --next) next_number "${CLAIM_ROOT:-$(git rev-parse --show-toplevel)}"; exit $? ;;
  "") echo "  ✗ 缺参数"; echo "    → bash scripts/kb/claim-experiment.sh --next，或 … E<数字> 简称"; exit 2 ;;
  *) claim "${CLAIM_ROOT:-$(git rev-parse --show-toplevel)}" "$1" "${*:2}"; exit $? ;;
esac
