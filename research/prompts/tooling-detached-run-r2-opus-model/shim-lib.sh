# 模型共用：替身 systemctl（不碰真的用户级 systemd、不起单元）。source 这一份，再调 opusr2_shim <目录>。
#   show-environment 转给真的（preflight 要它）；show -p ActiveState / MainPID 按 $SHIM_STATE/active-<单元> 文件答；
#   list-units 列出 $SHIM_STATE 里每个 active-<单元>；$SHIM_STATE/bus-down 在时 show / list-units 退 1。
opusr2_shim() {
  local dir="$1" real
  real="$(command -v systemctl)"
  mkdir -p "$dir/bin" "$dir/units"
  cat > "$dir/bin/systemctl" <<SHIM
#!/usr/bin/env bash
for a in "\$@"; do [[ "\$a" == show-environment ]] && exec $(printf '%q' "$real") "\$@"; done
S=$(printf '%q' "$dir/units")
if [[ -e "\$S/bus-down" ]]; then echo 'Failed to connect to bus（替身）' >&2; exit 1; fi
args=" \$* "
if [[ "\$args" == *" show "* ]]; then
  unit="\${@: -1}"
  if [[ "\$args" == *" ActiveState "* ]]; then
    if [[ -e "\$S/active-\$unit" ]]; then echo active; else echo inactive; fi
  elif [[ "\$args" == *" MainPID "* ]]; then
    if [[ -e "\$S/active-\$unit" ]]; then echo 4242; else echo 0; fi
  fi
  exit 0
fi
if [[ "\$args" == *" list-units "* ]]; then
  for f in "\$S"/active-*; do [[ -e "\$f" ]] || continue; echo "\${f##*/active-} loaded active running 替身"; done
  exit 0
fi
echo "替身 systemctl 不认：\$*" >&2; exit 1
SHIM
  chmod +x "$dir/bin/systemctl"
}
# opusr2_record <登记目录> <名> [started]：照 detached-run.sh start 的写法造一条在跑的登记（没有 exit）
opusr2_record() {
  local r="$1/$2"
  mkdir -p "$r"
  printf '%s\n' "singlefs-detached-$2.service" > "$r/unit"
  printf '%s\n' "${3:-$(date +%s)}" > "$r/started"
  printf '%s\n' "2026-10-01" > "$r/started-date"
  printf '%s\n' "sleep 300" > "$r/command"
  printf '%s\n' "$r/log" > "$r/log"
  : > "$r/log.txt"
}
