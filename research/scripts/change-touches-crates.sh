#!/usr/bin/env bash
# 这次改动碰没碰 `crates/`：碰了退出 0，没碰退出 1，判不出来退出 0。
#
#   change-touches-crates.sh [项目根] [输入前缀…]   判一次，把判据与依据打到 stdout（前缀不给按 `crates/`）
#   change-touches-crates.sh --selftest   自证：只改文档判「没碰」、改一行 crates 判「碰了」、判不出来时判「碰了」
#
# 谁在用：`.claude/gate.d/` 里那几道只判 `crates/` 里代码的重阶段（层 0 崩溃点重放、QEMU 真设备、
# 崩溃点重放、模型对拍）。没碰 `crates/` 时它们退 77（本次未跑），不退 0——
# `.claude/singlefs-ai-sop/rules/show-me-test.md`：`exit 0` 的跳过在汇总里与「判过了」一模一样。
#
# 为什么要有它：一次只改文档与产物的提交，会让那几道重阶段把上一次提交已经验过的同一份代码
# 重新跑一遍（实测 2026-09-20：44 个文件、零行 `crates/`，跑了 18 分钟还没跑完）。
# 这是 C8（范围判定）「要么全跑（挂钟炸）要么少跑（有布局没被验）」的一个落点。
#
# 射程：它只判「碰没碰 `crates/`」这一刀，**不是 C8 要的那个按布局算的受影响集合**。
# C8 的 howto 要的是「diff 只碰某条布局的专属路径 ⇒ 只跑那一条；碰到共享代码 ⇒ 强制全跑」，
# 这里给的是它的粗粒度前身：碰到 `crates/` 里任何东西都当成碰了共享代码，一律全跑。
# 所以这一刀只把「零行代码的改动」摘出去，摘不出别的，C8 照旧欠着。
#
# 保守方向：判不出来（不在 git 工作树里、git 出错、取不到基）一律当成碰了，宁可多跑。
# `SINGLEFS_GATE_FULL=1` 强制当成碰了，用来在干净工作树上重新验一遍 HEAD。
set -uo pipefail

# 基取哪一个：显式的 GATE_BASE > 与上游的 merge-base > HEAD。与 .claude/gate.d/75-decision-experiment-links.sh 同一套取法。
base_of() {
  local repository="$1"
  if [[ -n "${GATE_BASE:-}" ]]; then printf '%s' "$GATE_BASE"; return 0; fi
  local upstream_base
  if upstream_base="$(git -C "$repository" merge-base HEAD '@{upstream}' 2>/dev/null)" && [[ -n "$upstream_base" ]]; then
    printf '%s' "$upstream_base"; return 0
  fi
  printf 'HEAD'
}

# 判一个项目根。碰了打印依据并回 0，没碰回 1。第二个参数起是这道阶段自己的输入前缀，不给就按 `crates/`。
judge() {
  local root="$1"; shift
  local prefixes=("$@")
  ((${#prefixes[@]})) || prefixes=("crates/")
  if [[ "${SINGLEFS_GATE_FULL:-}" == 1 ]]; then
    echo "碰了：SINGLEFS_GATE_FULL=1 强制全跑"
    return 0
  fi
  if ! git -C "$root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "碰了（保守）：$root 不在 git 工作树里，算不出改动范围"
    return 0
  fi
  if ! git -C "$root" rev-parse --verify -q HEAD >/dev/null 2>&1; then
    echo "碰了（保守）：仓里还没有 HEAD，算不出改动范围"
    return 0
  fi
  # 判别力样本目录住在仓里面（`.claude/gate.d/fixtures/<阶段>/{green,red}/`），门禁 89 号拿它当项目根来跑。
  # 那时候算「这次改动碰了什么」没有意义，而跳过等于把样本判错——样本必须永远判。
  # 判据：给的根不是这个仓的顶层，就当成样本或子目录，照常跑。
  local top_level; top_level="$(git -C "$root" rev-parse --show-toplevel 2>/dev/null)"
  local absolute_root; absolute_root="$(cd "$root" 2>/dev/null && pwd -P)"
  if [[ -z "$top_level" || -z "$absolute_root" || "$(cd "$top_level" && pwd -P)" != "$absolute_root" ]]; then
    echo "碰了：$root 不是仓顶层（判别力样本或子目录），照常判"
    return 0
  fi
  local base; base="$(base_of "$root")"
  local committed uncommitted
  if ! committed="$(git -C "$root" diff --name-only "$base" 2>/dev/null)"; then
    echo "碰了（保守）：拿不到与基 $base 的 diff"
    return 0
  fi
  if ! uncommitted="$(git -C "$root" status --porcelain 2>/dev/null)"; then
    echo "碰了（保守）：拿不到工作区状态"
    return 0
  fi
  # 工作区那一份把前两格的状态码去掉，再把改名的「旧 -> 新」切成新路径。
  local worktree_paths
  worktree_paths="$(printf '%s\n' "$uncommitted" | sed -e 's/^...//' -e 's/^.* -> //' -e 's/^"\(.*\)"$/\1/')"
  local all_paths
  all_paths="$(printf '%s\n%s\n' "$committed" "$worktree_paths" | sed '/^$/d' | sort -u)"
  local wanted; wanted="$(printf '%s ' "${prefixes[@]}")"
  if [[ -z "$all_paths" ]]; then
    echo "没碰：与基 $base 之间一个改动都没有（干净工作树；要重新验 HEAD 用 SINGLEFS_GATE_FULL=1）"
    return 1
  fi
  local hit=""
  local prefix
  for prefix in "${prefixes[@]}"; do
    local matched
    matched="$(printf '%s\n' "$all_paths" | grep -F "$prefix" | grep "^$(printf '%s' "$prefix" | sed 's/[.[\*^$]/\\&/g')" || true)"
    [[ -n "$matched" ]] && hit="$(printf '%s\n%s' "$hit" "$matched")"
  done
  hit="$(printf '%s\n' "$hit" | sed '/^$/d')"
  if [[ -n "$hit" ]]; then
    echo "碰了：$(printf '%s\n' "$hit" | wc -l) 个路径落在 ${wanted}底下，第一个是 $(printf '%s\n' "$hit" | head -1)"
    return 0
  fi
  echo "没碰：与基 $base 之间共 $(printf '%s\n' "$all_paths" | wc -l) 个改动路径，没有一个落在 ${wanted}底下"
  return 1
}

selftest() {
  local work; work="$(mktemp -d)"
  local failures=0
  # 造一个真仓：先提交一份基线，再分别造两种改动。
  git -C "$work" init -q 2>/dev/null
  git -C "$work" config user.email selftest@example.com
  git -C "$work" config user.name selftest
  mkdir -p "$work/crates/sample/src" "$work/docs"
  echo "基线" > "$work/docs/note.md"
  echo "fn sample() {}" > "$work/crates/sample/src/lib.rs"
  git -C "$work" add -A >/dev/null 2>&1
  git -C "$work" commit -q -m 基线 >/dev/null 2>&1

  echo "改一行文档 ⇒ 应当判「没碰」"
  echo "改过" >> "$work/docs/note.md"
  if judge "$work"; then
    echo "  ✗ 只改文档却判成碰了 crates/"
    echo "     → 怎么办：看 judge 里取路径那一段，多半是 status 的状态码没剥干净。"
    failures=$((failures + 1))
  else
    echo "  ✓ 判「没碰」"
  fi

  echo "再改一行 crates/ ⇒ 应当判「碰了」"
  echo "// 改过" >> "$work/crates/sample/src/lib.rs"
  if judge "$work"; then
    echo "  ✓ 判「碰了」"
  else
    echo "  ✗ 改了 crates/ 却判成没碰——这一刀漏了，那几道重阶段会被整个跳过"
    echo "     → 怎么办：看 grep '^crates/' 那一句与前面 all_paths 的拼法。"
    failures=$((failures + 1))
  fi

  echo "不是 git 工作树 ⇒ 应当保守判「碰了」"
  local plain; plain="$(mktemp -d)"
  if judge "$plain"; then
    echo "  ✓ 判「碰了（保守）」"
  else
    echo "  ✗ 算不出范围时判成没碰——判不出来必须往多跑那一侧倒"
    echo "     → 怎么办：看 rev-parse --is-inside-work-tree 那一支的返回值。"
    failures=$((failures + 1))
  fi
  rm -rf "${plain:?}"

  echo "干净工作树 + SINGLEFS_GATE_FULL=1 ⇒ 应当判「碰了」"
  git -C "$work" add -A >/dev/null 2>&1
  git -C "$work" commit -q -m 第二次 >/dev/null 2>&1
  if SINGLEFS_GATE_FULL=1 judge "$work"; then
    echo "  ✓ 判「碰了」"
  else
    echo "  ✗ 设了 SINGLEFS_GATE_FULL=1 还判成没碰"
    echo "     → 怎么办：看 judge 开头那一支。"
    failures=$((failures + 1))
  fi

  rm -rf "${work:?}"
  if [[ "$failures" != 0 ]]; then
    echo "  ✗ 自证没过：$failures 项判错"
    echo "     → 怎么办：上面每一项各自写了看哪一段。"
    return 1
  fi
  echo "  ✓ 自证通过：只改文档判「没碰」、改一行 crates 判「碰了」、算不出范围与强制全跑都判「碰了」"
  return 0
}

if [[ "${1:-}" == "--selftest" ]]; then
  selftest
  exit $?
fi

ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
shift 2>/dev/null || true
judge "$ROOT" "$@"
exit $?
