#!/usr/bin/env bash
# cut_narrow_check.sh <变体目录（rerun.sh 的 $WORK/variants）> <输出目录>：切边加窄读法（只有按路径载入的模块、与它们 import 引来的模块，才收模块级 import；
# R3_NARROW_KINDS=import 只收 import，=import,assign 另收 assign）两种，对 check_proof 与 escalate_step 收闭包，报 memory_peaks_lib.py 的 import subprocess 在不在、lib_long_run.py 的单元数与 import 数。
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; variants="$1"; out="$2"; mkdir -p "$out"
for kinds in import import,assign; do
  tag="${kinds//,/-}"
  R3_NARROW_KINDS="$kinds" nice -n 19 python3 "$here/scan.py" "$variants/cut-narrow" "$here/entries-two.txt" "$out/scan-cut-narrow-$tag" > /dev/null
  python3 - "$out/scan-cut-narrow-$tag" "$tag" <<'PY'
import sys
prefix, tag = sys.argv[1:3]
for line in open(prefix + ".tsv", encoding="utf-8"):
    f = line.rstrip("\n").split("\t")
    members = open(f"{prefix}.sets/{int(f[0]):02d}.set", encoding="utf-8").read().splitlines()
    mp = sum(1 for m in members if m.startswith("import|subprocess|scripts/hosts/memory_peaks_lib.py|"))
    lr = [m for m in members if m.split("|")[2] == ".claude/hooks/lib_long_run.py"]
    print(f"name=cut_narrow kinds={tag} entry={f[1].split('::')[1]} units={f[3]} memory_peaks_lib_import_subprocess={mp} lib_long_run_units={len(lr)} lib_long_run_imports={sum(1 for m in lr if m.startswith('import|'))}")
PY
done
