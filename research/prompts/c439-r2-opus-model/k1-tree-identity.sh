#!/usr/bin/env bash
# K1：里层建出的临时树与外壳 write-tree 的那棵逐字节是不是同一棵。两种建法各比一次：
#   today：冻结 gate.sh 第 55–65、87–99 行原样抽出来跑（暂存区 diff 套到 HEAD 上，规范副本与前缀那两段照跑）；
#   geng2_a：庚二落法 A，从外壳那棵树建 worktree（git worktree add <包着它的提交>），再照第 90–99 行拷规范副本、查前缀。
# 比两样：临时树的索引树与外壳的树；临时树磁盘上的文件（去掉 .git）与外壳那棵树检出的文件。用法同 k1-binary.sh
source "$(dirname "$0")/lib.sh" "$1" "$2" "$3"
SNIPPET="$WORK/gate-staged-build-snippet.sh"
{ echo 'die() { printf "die: %s\n" "$1"; return 1; }'
  awk 'NR>=55 && NR<=65' "$FROZEN/.claude/singlefs-ai-sop/scripts/gate.sh"
  awk 'NR>=87 && NR<=99' "$FROZEN/.claude/singlefs-ai-sop/scripts/gate.sh"; } > "$SNIPPET"
CARGO_COMMAND_PREFIX_FILE=.claude/cargo-command-prefix; SCRIPTS_DIRECTORY="$WORK/no-such-scripts-directory"
checkout_of_tree() { local d; d="$(mktemp -d "$WORK/co.XXXXXX")"; git archive "$1" | tar -x -C "$d"; echo "$d"; }
compare() {   # compare <标签> <临时树目录> <外壳的树>
  local label="$1" dir="$2" tree="$3" index_tree co files
  index_tree="$(git -C "$dir" write-tree 2>/dev/null || echo none)"; co="$(checkout_of_tree "$tree")"
  files="$(diff -rq --no-dereference -x .git "$dir" "$co" 2>&1 | sed "s#$dir#<临时树>#; s#$co#<外壳的树>#" | tr '\n' ';')"
  echo "  $label index_equal=$([[ $index_tree == "$tree" ]] && echo yes || echo no) files_equal=$([[ -z $files ]] && echo yes || echo no) ${files:+diff=$files}"
  rm -rf "$co"
}
build_today() {   # 在源仓里跑抽出来的那两段，结果目录放 $staged_tree
  source_repository_root="$PWD"; source "$SNIPPET" || echo "  today: 中途 die"
}
build_geng2_a() {
  local tree="$1" commit; staged_scratch_directory="$(mktemp -d)"; staged_tree="$staged_scratch_directory/tree"
  commit="$(git commit-tree "$tree" -m judged)"; git worktree add -q --detach "$staged_tree" "$commit"
  source_repository_root="$PWD"; source <(awk 'NR>=90 && NR<=99' "$FROZEN/.claude/singlefs-ai-sop/scripts/gate.sh"; echo :) || echo "  geng2_a: 中途 die"
}
cleanup_tree() { git worktree remove --force "$staged_tree" >/dev/null 2>&1; rm -rf "$staged_scratch_directory"; }
run_case() {   # run_case <名>：调用前源仓已就位
  local tree; tree="$(git write-tree)"; echo "=== case=$1 outer_tree=${tree:0:12}"
  build_today; [[ -d $staged_tree/.git || -f $staged_tree/.git ]] && compare today "$staged_tree" "$tree"; cleanup_tree
  build_geng2_a "$tree"; compare geng2_a "$staged_tree" "$tree"; cleanup_tree
}
new_repo "$WORK/repo-tree"; base_commit="$(git rev-parse HEAD)"
printf 'E7RESULT name=alpha value=1\n' > .claude/kb/new.md; git add .claude/kb/new.md; run_case staged_new_page
printf 'x\n' > .claude/kb/ita.md; git add -N .claude/kb/ita.md; run_case intent_to_add
git rm -q --cached .claude/kb/ita.md; rm .claude/kb/ita.md
ln -s experiments/900-x.md .claude/kb/link.md; git add .claude/kb/link.md; run_case staged_symlink
mkdir -p .claude/singlefs-ai-sop/scripts; printf 'lib v1\n' > .claude/singlefs-ai-sop/scripts/lib.sh; run_case ignored_family_copy_in_working_tree
git commit -qm wip; printf '*.md text eol=crlf\n' > .gitattributes; git add .gitattributes; git commit -qm attrs
printf 'E7RESULT name=alpha value=1\nmore\n' > .claude/kb/new.md; git add .claude/kb/new.md; run_case eol_crlf_attribute
printf '*.md ident\n' > .gitattributes; printf '$Id$\n' > .claude/kb/ident.md; git add -A; git commit -qm ident; printf 'more2\n' >> .claude/kb/new.md; git add .claude/kb/new.md; run_case ident_attribute
git update-index --add --cacheinfo "160000,$base_commit,research/sub"; run_case staged_gitlink
git rm -q --cached research/sub
printf 'prefix\n' > .claude/cargo-command-prefix; run_case untracked_cargo_prefix_in_working_tree; rm .claude/cargo-command-prefix
git add -A; git commit -qm settle; printf 'E7RESULT name=beta value=2\n' > .claude/kb/bypass.md; git add .claude/kb/bypass.md; git commit -qm bypass-L
printf 'note\n' > .claude/kb/other.md; git add .claude/kb/other.md
echo "=== case=head_moved_between_diff_and_worktree_add（today 那一建法；最后一个提交带 L（.claude/kb/bypass.md），暂存区只动另一份；git reset --soft HEAD~1 插在第 56 行与第 59 行之间，建完再挪回）"
tree="$(git write-tree)"; head_before="$(git rev-parse HEAD)"; source_repository_root="$PWD"
{ echo 'die() { printf "die: %s\n" "$1"; return 1; }'; awk 'NR>=55 && NR<=58' "$FROZEN/.claude/singlefs-ai-sop/scripts/gate.sh"
  echo 'git reset -q --soft HEAD~1'; awk 'NR>=59 && NR<=65' "$FROZEN/.claude/singlefs-ai-sop/scripts/gate.sh"
  awk 'NR>=87 && NR<=99' "$FROZEN/.claude/singlefs-ai-sop/scripts/gate.sh"; echo "git reset -q --soft $head_before"; } > "$WORK/head-moved-snippet.sh"
source "$WORK/head-moved-snippet.sh" || echo "  today: 中途 die"
echo "  write_tree_after=$(git write-tree | cut -c1-12)（收尾比树：与开跑时 ${tree:0:12} 相同就前移）"
compare today "$staged_tree" "$tree"; cleanup_tree
cd "$WORK"; rm -rf "$WORK/repo-tree" "$SNIPPET" "$WORK/head-moved-snippet.sh"
