#!/usr/bin/env bash
# U1 模型：替身 systemctl 下的 watch（只读，不起任何单元）。ActiveState 的查询按计划文件逐次给 F（失败）或 A（active）。
# 用法：bash alarm_model.sh <detached-run.sh 路径> "<计划，空格分隔>"
set -u
script="$1"; plan="$2"
d="$(mktemp -d /tmp/claude-1000/tooling-detached-run-r3-sonnet/am.XXXXXX)"
mkdir -p "$d/state/u1" "$d/shim"
echo singlefs-detached-r3u1-am.service > "$d/state/u1/unit"; date +%s > "$d/state/u1/started"; echo "sleep 300" > "$d/state/u1/command"
echo 1 > "$d/state/u1/pid"
printf '%s\n' $plan > "$d/plan"; : > "$d/counter"
real="$(command -v systemctl)"
cat > "$d/shim/systemctl" <<SH
#!/usr/bin/env bash
for a in "\$@"; do [[ "\$a" == show-environment ]] && exec $real "\$@"; done
if [[ " \$* " == *" -p ActiveState "* ]]; then
  n=\$(( \$(wc -l < "$d/counter") + 1 )); echo x >> "$d/counter"
  v="\$(sed -n "\${n}p" "$d/plan")"; v="\${v:-A}"
  echo "\$n \$v" >> "$d/calls"
  if [[ "\$v" == F ]]; then echo 'Failed to connect to bus（替身）' >&2; exit 1; fi
  echo active; exit 0
fi
if [[ " \$* " == *" -p MainPID "* ]]; then echo 4242; exit 0; fi
if [[ " \$* " == *" list-units "* ]]; then exit 0; fi
exit 0
SH
chmod +x "$d/shim/systemctl"
PATH="$d/shim:$PATH" DETACHED_RUN_STATE_DIRECTORY="$d/state" timeout 15 bash "$script" watch --every 1 > "$d/watch.out" 2>&1; echo "watch 退 $?"
grep -E '^  == 第|报警|查不到 [0-9]+ 条不知道|还在跑  ' "$d/watch.out" | sed -E 's/（20[0-9-]+）//' | cut -c1-110
echo "--- 查询次数与计划：$(wc -l < "$d/counter") 次"
rm -rf -- "$d"
