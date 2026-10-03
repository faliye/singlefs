#!/usr/bin/env bash
# 复现 U3（真起单元，主 agent 带前缀跑）：起一个 detached-run 单元，单元里跑 run-with-memory-cap.sh 包一条 sleep；
#   内存包装经 systemd-run --user --scope 把 sleep 放进 singlefs-heavy.slice 下一个独立的 scope（不是这个 service 单元的 cgroup）。
#   detached-run.sh stop <名> 停这个 service 单元：systemd 只停 service 单元自己 cgroup 里的进程（runner bash 与包装 bash），
#   内存包装的 trap（run-with-memory-cap.sh:800「exit 143」）只删账与标记、不停 scope，scope 在别的 slice 里活下来，sleep 成孤儿。
#   单元名前缀 singlefs-detached-opusr3-，登记目录临时；收尾停 service、按名停残留的 scope、核零残留。
# 用法：bash repro-u3-scope-survives-unit-kill.sh <仓根>
set -u
repo="$1"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr3-u3repro.XXXXXX")"
export DETACHED_RUN_STATE_DIRECTORY="$work/state"
label="opusr3-scope-$$"; unit="singlefs-detached-$label.service"
inner="$work/inner.sh"
cat > "$inner" <<INNER
#!/usr/bin/env bash
echo \$\$ > "$work/sleep.pid.wrap"
exec nice -n 19 bash "$repo/research/scripts/run-with-memory-cap.sh" 64M bash -c 'echo \$\$ > "$work/sleep.pid"; exec sleep 600'
INNER
chmod +x "$inner"
echo "== start 单元"
bash "$repo/research/scripts/detached-run.sh" start --label "$label" -- bash "$inner" > "$work/start.out" 2>&1
echo "start 退 $?"; grep -E '✓ 起了|✗' "$work/start.out" | head -2
for i in $(seq 100); do [[ -s "$work/sleep.pid" ]] && break; sleep 0.1; done
sp="$(cat "$work/sleep.pid" 2>/dev/null)"
echo "sleep 进程 $sp 的 cgroup：$(sed 's#.*/##' /proc/$sp/cgroup 2>/dev/null)"
echo "service 单元 ActiveState=$(systemctl --user show -p ActiveState --value "$unit")"
echo "== detached-run.sh stop $label（停 service 单元）"
timeout 60 bash "$repo/research/scripts/detached-run.sh" stop "$label" > "$work/stop.out" 2>&1
echo "stop 退 $?"; grep -E '✓|✗' "$work/stop.out" | head -2
sleep 1
echo "停完：service ActiveState=$(systemctl --user show -p ActiveState --value "$unit")"
scope="$(sed 's#.*/##' /proc/$sp/cgroup 2>/dev/null)"
echo "sleep $sp 还在吗：$(cat /proc/$sp/comm 2>/dev/null || echo 不在)；它的 scope=$scope ActiveState=$([[ -n "$scope" ]] && systemctl --user show -p ActiveState --value "$scope" || echo '（读不到）')"
echo "== 期望：sleep 还在、scope 仍 active ⇒ 内存包装的 scope 在 service 被停之后活下来（U3 造出）"
echo "== 收尾"
[[ -n "$scope" ]] && systemctl --user stop "$scope" 2>/dev/null
systemctl --user reset-failed "$unit" "$scope" 2>/dev/null
left="$(systemctl --user list-units --all --plain --no-legend 'singlefs-detached-opusr3-*' 2>&1)"
echo "残留 singlefs-detached-opusr3-* 单元 $(printf '%s' "$left" | grep -c .) 个${left:+：$left}"
rm -rf -- "$work"
