#!/usr/bin/env bash
# 攻方腿模型（m3-broad-gate-gov-r1，G2）：广谱变异门禁判的是索引，判绿后写的全绿标记却按工作区算指纹；之后把工作区那份暂存，复用判定认它。
# 用法：bash marker-model.sh <冻结副本的拷贝（不带 .git）> <草稿目录>
# 在草稿目录里建一个临时 git 仓（拷贝 + git init + 一次提交），只在那里写；不碰真仓与真库目录，不起驱动、不起 cargo 的编译与测试
# （admission.py 算指纹时会起 cargo -V、rustc -V 与本阶段的 --tool-versions，只问版本）。
# 判绿那一步用的是阶段第 377 行同一条调用 admission.py stage-marker-write <根> checker-tier-broad-mutation.sh；格本身判绿的前提（索引就是库目录那一版）这里不重做，
# 只核「格判的那份」与「标记算指纹的那份」不是同一份：索引里 lib.rs 的 blob 与工作区 lib.rs 的 sha 各打一行。
set -uo pipefail
source_tree="$1"; scratch="$2"
stage=checker-tier-broad-mutation.sh
run_case() {   # $1 情形名；$2 = dirty 时判绿那一刻工作区比索引多一处没暂存的改动，clean 时没有
  local name="$1" mode="$2" repo="$scratch/marker-$1"
  rm -rf "$repo"; cp -a "$source_tree" "$repo"
  git -C "$repo" init -q && git -C "$repo" -c user.name=m -c user.email=m@m add -A && git -C "$repo" -c user.name=m -c user.email=m@m commit -q -m base
  local file="$repo/crates/singlefs-core/src/lib.rs"
  local extra="$repo/crates/singlefs-core/src/model_new_file.rs"
  case "$mode" in
    dirty|commit-first|partial) printf '\n// 别的会话在工作区里改了、还没暂存（U）\n' >> "$file"
      if [[ "$mode" == partial ]]; then printf '\n// U 的第二处\n' >> "$repo/crates/singlefs-core/Cargo.toml"; fi ;;
    untracked) printf 'pub fn model_only() {}\n' > "$extra" ;;
  esac
  echo "[$name] 判绿那一刻：git diff --quiet 退 $(git -C "$repo" diff --quiet -- crates; echo $?)；索引 lib.rs blob $(git -C "$repo" ls-files -s -- crates/singlefs-core/src/lib.rs | awk '{print substr($2,1,12)}')，工作区 lib.rs blob $(git -C "$repo" hash-object "$file" | cut -c1-12)"
  echo "[$name] 阶段判绿之后写标记（第 377 行那一条）：$(python3 "$repo/research/scripts/admission.py" stage-marker-write "$repo" "$stage" >/dev/null 2>&1; echo "退 $?")"
  if [[ "$mode" == clean ]]; then printf '\n// 判绿之后才改、再暂存的（U）\n' >> "$file"; fi
  if [[ "$mode" == commit-first ]]; then git -C "$repo" -c user.name=m -c user.email=m@m commit -q --allow-empty -m "提交判绿的那一版（S1）" >/dev/null; echo "[$name] 先把判绿的 S1 提交了（索引就是 S1，空提交也照记一次提交）"; fi
  case "$mode" in
    untracked) git -C "$repo" add crates/singlefs-core/src/model_new_file.rs ;;
    *) git -C "$repo" add crates/singlefs-core/src/lib.rs ;;
  esac
  echo "[$name] 之后暂存 U：索引 lib.rs blob $(git -C "$repo" ls-files -s -- crates/singlefs-core/src/lib.rs | awk '{print substr($2,1,12)}')（判绿时格判的不是这一份）"
  local check; check="$(python3 "$repo/research/scripts/admission.py" stage-marker-check "$repo" "$stage" 2>&1)"; local code=$?
  echo "[$name] 下一次提交第 1 步 stage-marker-check 退 $code：$(head -n 1 <<<"$check" | sed -E "s|$repo|<repo>|g; s|[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9:]{8}Z|<跑完的时刻>|g")"
  local tree; tree="$(git -C "$repo" write-tree)"
  local worktree="$scratch/marker-$name-staged"
  rm -rf "$worktree"; git -C "$repo" worktree add -q --detach "$worktree" HEAD && git -C "$worktree" read-tree "$tree" && git -C "$worktree" checkout-index -a -f
  local reuse; reuse="$(SINGLEFS_STAGED_TREE="$tree" python3 "$repo/research/scripts/admission.py" gate-reuse "$worktree" "$stage" 2>&1)"; code=$?
  echo "[$name] 整轮 --staged 的临时树上 gate-reuse 退 $code（10 = 可跳过、0 = 要跑）：$(head -n 1 <<<"$reuse" | sed -E "s|$worktree|<staged>|g; s|$repo|<repo>|g; s|[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9:]{8}Z|<跑完的时刻>|g")"
}
run_case dirty dirty
run_case commit-first commit-first
run_case untracked untracked
run_case partial partial
run_case clean clean
