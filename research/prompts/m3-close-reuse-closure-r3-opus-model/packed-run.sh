#!/usr/bin/env bash
# 甲的按值打包，在本机上模拟「第二台」：只把 needed-files.sh 量出的那几份仓内文件按原相对路径拷进一个没有 git、没有 multi-host.env 的空目录，
# HOME 换成一个空目录（第二台没有本机的 ~/.config/singlefs 与包装、派发器的账），在那里跑 host-resources.py --json -- true。
# 两种 python：照原样；PYTHONPATH 前面放一份 import 就抛 ModuleNotFoundError 的 tomllib.py（模拟第二台的 python 低于 3.11，没有 tomllib）。
# 对照：今天送到第二台的那一段（remote_probe_program：memory_peaks_lib.py 原文 + probe_this_host + PROBE_DRIVER）在同一个模拟下跑不跑得起来。
# 用法：packed-run.sh <树> <needed-files 的清单> <输出目录>
set -u
root="$(cd "$1" && pwd)" list="$2" out="$3"
rm -rf -- "$out"; mkdir -p "$out/packed" "$out/home" "$out/old-python"
while read -r file; do mkdir -p "$out/packed/$(dirname "$file")"; cp -p "$root/$file" "$out/packed/$file"; done < "$list"
printf 'raise ModuleNotFoundError("No module named %s（模拟第二台的 python 低于 3.11）")\n' "'tomllib'" > "$out/old-python/tomllib.py"
for arm in same-python old-python; do
  extra=(); [[ $arm == old-python ]] && extra=(PYTHONPATH="$out/old-python")
  (cd "$out/home" && env -i PATH="$PATH" HOME="$out/home" "${extra[@]}" python3 -B "$out/packed/scripts/hosts/host-resources.py" --json -- true \
     > "$out/$arm.stdout" 2> "$out/$arm.stderr"); rc=$?
  echo "name=packed_run arm=$arm rc=$rc stdout_is_json=$(python3 -c 'import json,sys; json.load(open(sys.argv[1])); print("yes")' "$out/$arm.stdout" 2>/dev/null || echo no) last_stderr_line=$(tail -n 1 "$out/$arm.stderr" | cut -c1-160)"
done
python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import importlib.util as u; s = u.spec_from_file_location("hr", sys.argv[2]); m = u.module_from_spec(s); s.loader.exec_module(m); open(sys.argv[3], "w").write(m.remote_probe_program())' \
  "$root/scripts/hosts" "$root/scripts/hosts/host-resources.py" "$out/today-remote-program.py" 2> "$out/today-build.stderr"
for arm in same-python old-python; do
  extra=(); [[ $arm == old-python ]] && extra=(PYTHONPATH="$out/old-python")
  (cd "$out/home" && env -i PATH="$PATH" HOME="$out/home" "${extra[@]}" python3 -B -c "$(cat "$out/today-remote-program.py")" 1 > "$out/today-$arm.stdout" 2> "$out/today-$arm.stderr"); rc=$?
  echo "name=today_remote_program arm=$arm rc=$rc probe_line=$(grep -c '^HOST_RESOURCES_PROBE ' "$out/today-$arm.stdout") last_stderr_line=$(tail -n 1 "$out/today-$arm.stderr" | cut -c1-160)"
done
python3 - "$out/same-python.stdout" <<'PY'
import json, sys
data = json.load(open(sys.argv[1]))
host = data["hosts"][0]
prediction = (data.get("command") or {}).get("prediction") or {}
print(f"name=packed_same_python_view config_read={'no' if data.get('local_only_because') else 'yes'} memory_history={prediction.get('memory_history')} predicted_memory={prediction.get('memory')}")
PY
