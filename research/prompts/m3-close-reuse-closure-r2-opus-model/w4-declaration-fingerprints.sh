#!/usr/bin/env bash
# W4：候选条文要的「排除写在调用方源码的注释里（# closure-excluded-edge: …），声明进键，改了或删了键就变」，在今天的切单元规则下进不进指纹。
# 四处各造两份只差这一行注释的夹具（python 的 def 里、shell 函数体里、shell 顶层、shell 函数里喂给 python 的 here-doc 里），
# 用树里那一份 lib_function_units.py dump 比两份的单元行（种类、限定名、指纹）。
# 用法：w4-declaration-fingerprints.sh <树> <草稿目录>
set -u
root="$1" scratch="$2"
declaration='# closure-excluded-edge: caller -> scripts/hosts/host-resources.py：只打提示'
for arm in without with; do
  tree="$scratch/fixture-$arm"; rm -rf -- "$tree"; mkdir -p "$tree/scripts/hosts"
  line=""; [[ $arm == with ]] && line="$declaration"
  printf '%s\n' 'import subprocess' '' '' 'def caller():' "    ${line}" '    return subprocess.run(["python3", "-c", "pass"]).returncode' > "$tree/scripts/hosts/caller.py"
  printf '%s\n' '#!/usr/bin/env bash' 'caller() {' "  ${line}" '  true' '}' 'caller' > "$tree/scripts/hosts/function-body.sh"
  printf '%s\n' '#!/usr/bin/env bash' "${line}" 'true' > "$tree/scripts/hosts/top-level.sh"
  printf '%s\n' '#!/usr/bin/env bash' 'caller() {' "  python3 /dev/fd/3 3<<'PY'" 'import subprocess' "${line}" 'subprocess.run(["true"])' 'PY' '}' 'caller' > "$tree/scripts/hosts/heredoc.sh"
  (cd "$root" && python3 -B scripts/gate/lib_function_units.py dump "$tree" --under scripts/hosts) | awk -F'\t' '{print $1"\t"$2"\t"substr($3,1,index($3,":")-1)"\t"$4}' | sort > "$scratch/dump-$arm.txt"
done
for file in caller.py function-body.sh top-level.sh heredoc.sh; do
  same=$(diff <(grep -F "scripts/hosts/$file" "$scratch/dump-without.txt") <(grep -F "scripts/hosts/$file" "$scratch/dump-with.txt") > /dev/null && echo yes || echo no)
  echo "name=declaration_fingerprint file=$file units_without=$(grep -cF "scripts/hosts/$file" "$scratch/dump-without.txt") units_with=$(grep -cF "scripts/hosts/$file" "$scratch/dump-with.txt") identical=$same"
done
