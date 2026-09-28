#!/bin/bash
# A1 第②处（代码文件改名、挪目录）：这个仓 crates/ 下在 git 历史里改过几次名、改名那天一次改了几份、改名前的旧名后来有没有被别的文件占回去；
# 另数改名那两次提交前后 #[test] 函数名变了几个。只读 git，不改任何东西。用法：bash a1_rename_history.sh <仓根>
set -euo pipefail
root="${1:?仓根}"
cd "$root"
renames="$(git log --diff-filter=R -M --name-status --format='COMMIT %h %ad' --date=short -- crates | awk '/^COMMIT/{c=$2; d=$3} /^R/{print d, c, $2, $3}')"
total=$(printf '%s\n' "$renames" | grep -c . || true)
tests=$(printf '%s\n' "$renames" | awk '$3 ~ /\/tests\//' | grep -c . || true)
days=$(printf '%s\n' "$renames" | awk '{print $1}' | sort -u | tr '\n' ',' )
commits=$(printf '%s\n' "$renames" | awk '{print $2}' | sort -u | tr '\n' ',')
reused=0
while read -r _ _ old _; do
  [ -n "$old" ] && [ -e "$old" ] && reused=$((reused + 1))
done <<< "$renames"
first=$(git log --reverse --format=%ad --date=short -- crates | awk 'NR==1')
last=$(git log --format=%ad --date=short -- crates | awk 'NR==1')
current_tests=$(git ls-files 'crates/*/tests/*.rs' | wc -l)
before=$(git log --format=%h --diff-filter=R -M -- crates | tail -1)
after=$(git log --format=%h --diff-filter=R -M -- crates | awk 'NR==1')
names() { git grep -h -A2 -E '^\s*#\[test\]' "$1" -- 'crates/*.rs' | grep -oE 'fn [a-z0-9_]+' | sort -u; }
gone=$(comm -23 <(names "$before^") <(names "$after") | wc -l)
new=$(comm -13 <(names "$before^") <(names "$after") | wc -l)
echo "E7RESULT name=r3_a1_rename_history crates_history=${first}..${last} renamed_files=$total of_which_test_files=$tests current_test_files=$current_tests rename_days=$days rename_commits=$commits old_names_now_taken_again=$reused test_function_names_gone=$gone test_function_names_new=$new must_be_nonzero=$tests"
