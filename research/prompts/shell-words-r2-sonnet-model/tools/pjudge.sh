#!/usr/bin/env bash
# pjudge.sh <SW_ROOT> <cases.json> <分几块> <输出前缀> : 把命令清单切成几块，每块一个 judge.py 进程并行跑，最后把各块输出按块号拼起来写进 <输出前缀>.txt，末行「合计」
set -u
root="$1"; cases="$2"; n="$3"; prefix="$4"; here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 -I - "$cases" "$n" "$prefix" <<'PY'
import json, sys
cases, n, prefix = sys.argv[1], int(sys.argv[2]), sys.argv[3]
items = json.load(open(cases))
for i in range(n):
    json.dump(items[i::n], open(f"{prefix}.chunk{i}.json", "w"), ensure_ascii=False)
PY
for ((i = 0; i < n; i++)); do rm -f "${prefix}.c${i}.rc"; done
for ((i = 0; i < n; i++)); do
  { nice -n 19 python3 -I "$here/judge.py" "$root" "${prefix}.chunk${i}.json" "${prefix}.c${i}" > "${prefix}.c${i}.txt" 2>&1; echo "$?" > "${prefix}.c${i}.rc"; } &
done
wait
[[ "$(ls "${prefix}".c*.rc | wc -l)" -eq "$n" ]] || { echo "pjudge_rc_files_missing" >&2; exit 1; }
cat "${prefix}".c*.rc | sort | uniq -c | sed 's/^/judge_exit_codes /'
: > "${prefix}.txt"
for ((i = 0; i < n; i++)); do cat "${prefix}.c${i}.txt" >> "${prefix}.txt"; done
grep -h '^统计' "${prefix}".c*.txt | python3 -I -c "
import sys, re, ast, collections
t = collections.Counter(); syn = 0
for l in sys.stdin:
    m = re.match(r'统计 \(修补前判定,修补后判定\): (\{.*\}) 语法错 (\d+)', l)
    for k, v in ast.literal_eval(m.group(1)).items(): t[k] += v
    syn += int(m.group(2))
print('合计', dict(sorted(t.items())), '语法错', syn, '共', sum(t.values()) + syn)" | tee -a "${prefix}.txt"
