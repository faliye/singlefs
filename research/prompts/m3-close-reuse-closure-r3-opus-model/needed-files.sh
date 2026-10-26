#!/usr/bin/env bash
# 甲要按值打包的文件，另一条路数（不同于第二轮攻方的 audit hook）：strace 记 host-resources.py --json -- true 整棵进程树真打开过的仓内文件
# （python 的 import、exec_module、bash 的 source 与当程序起的脚本都算；只要 openat 成功的、是普通文件的），
# 再按后缀与位置分开报。多机开关关着（假的空配置），等于「由参数选这一台」那一种跑法。
# 用法：needed-files.sh <树> <输出目录>
set -u
root="$(cd "$1" && pwd)" out="$2"
mkdir -p "$out"
config="$out/empty-multi-host.env"; : > "$config"
(cd "$root" && env SINGLEFS_MULTI_HOST_CONFIG="$config" strace -f -qq -e trace=openat,execve -o "$out/strace.txt" \
   python3 -B scripts/hosts/host-resources.py --json -- true > "$out/stdout.json" 2> "$out/stderr.txt"); echo "name=needed_files_run rc=$?"
grep -oE '(openat\(AT_FDCWD, "[^"]+", [^)]*\) = [0-9]+|execve\("[^"]+")' "$out/strace.txt" | grep -oE '"[^"]+"' | tr -d '"' \
  | while read -r path; do real="$(realpath -m "$path" 2>/dev/null)"; case "$real" in "$root"/*) [[ -f $real ]] && echo "${real#$root/}";; esac; done | sort -u > "$out/repo-files.txt"
echo "name=needed_files repo_files=$(wc -l < "$out/repo-files.txt") python=$(grep -c '\.py$' "$out/repo-files.txt") shell=$(grep -c '\.sh$' "$out/repo-files.txt") other=$(grep -vcE '\.(py|sh)$' "$out/repo-files.txt") outside_scripts_hosts=$(grep -vc '^scripts/hosts/' "$out/repo-files.txt")"
sed 's/^/  file=/' "$out/repo-files.txt"
