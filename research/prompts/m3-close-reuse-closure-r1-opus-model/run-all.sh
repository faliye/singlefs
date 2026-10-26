#!/usr/bin/env bash
# 攻方腿全部装置一趟跑完：<树> 是被判脚本在里面的那棵树（sha256 先核 judged-sha256.txt），<输出目录> 放每一步的原样输出，
# 末尾把可比的结论行（name=…，不带秒数、进程号这类随跑变的字）写进 <输出目录>/summary.txt。
# 用法：run-all.sh <树> <输出目录>
set -u
tree="$1" out="$2"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$out"
summary="$out/summary.txt"
: > "$summary"
(cd "$tree" && sha256sum -c "$here/judged-sha256.txt") > "$out/judged-sha256.check" 2>&1 || { cat "$out/judged-sha256.check"; echo "被判脚本的 sha256 对不上" >&2; exit 3; }
echo "name=judged_sha256 status=ok" >> "$summary"
[[ -e "$tree/.git" ]] || git -C "$tree" init -q .
python3 -B "$here/make-variants.py" "$tree" "$out/variants" > "$out/variants.txt"
units_of() { awk -F'\t' '$1!="via" && $1!="key" && $1!="refused"{print $1"\t"$2"\t"substr($3,1,index($3,":")-1)}' "$1" | sort -u; }
for entry in scripts/gate/verus-proof-check.py::check_proof scripts/gate/verus-proof-check.py::check_vacuity scripts/gate/verus-proof-check.py::check_must_red_worlds scripts/mutation/broad-mutation.py::Runner.escalate_step; do
  label="$(printf '%s' "$entry" | tr '/:.' '___')"
  for arm in full edge-cut; do
    root="$tree"; [[ $arm == edge-cut ]] && root="$out/variants/edge-cut"
    (cd "$tree" && python3 scripts/gate/lib_function_units.py closure "$root" "$entry" --explain) > "$out/closure-$arm-$label.txt" 2> "$out/closure-$arm-$label.err"
    rc=$?
    units_of "$out/closure-$arm-$label.txt" > "$out/units-$arm-$label.txt"
    refusals="$(grep '^refused' "$out/closure-$arm-$label.txt" | cut -f2,3 | tr '\t\n' ' ,')"
    echo "name=closure arm=$arm entry=$entry rc=$rc units=$(wc -l < "$out/units-$arm-$label.txt") lib_long_run_units=$(grep -c 'lib_long_run.py' "$out/units-$arm-$label.txt") host_dispatch_predict=$(grep -cP '^def\tpredict\tscripts/hosts/host-dispatch.py$' "$out/units-$arm-$label.txt") refusals=${refusals:-none}" >> "$summary"
  done
  comm -23 "$out/units-full-$label.txt" "$out/units-edge-cut-$label.txt" > "$out/dropped-by-edge-cut-$label.txt"
  echo "name=dropped_by_edge_cut entry=$entry total=$(wc -l < "$out/dropped-by-edge-cut-$label.txt") by_file=$(cut -f3 "$out/dropped-by-edge-cut-$label.txt" | sort | uniq -c | sort -k2 | awk '{printf "%s:%s,", $2, $1}')" >> "$summary"
done
for arm in full hostlookup-text; do
  root="$tree"; [[ $arm == hostlookup-text ]] && root="$out/variants/hostlookup-text"
  (cd "$tree" && python3 scripts/gate/lib_function_units.py dump "$root" --language shell --under scripts/hosts) > "$out/dump-shell-$arm.txt" 2>&1
  echo "name=wrapper_shell_units arm=$arm $(awk -F'\t' '$3 ~ /^scripts\/hosts\/run-with-memory-cap.sh:/ && $2 == "admission" {print "admission="$4}' "$out/dump-shell-$arm.txt") hostlookup_units=$(awk -F'\t' '$3 ~ /^scripts\/hosts\/run-with-memory-cap.sh:/' "$out/dump-shell-$arm.txt" | grep -c HostLookup)" >> "$summary"
done
for fixture in through-function ssh-only local-cd-literal local-dir-prefix; do
  (cd "$tree" && python3 scripts/gate/lib_function_units.py closure "$here/fixture-root" "scripts/hosts/$fixture.sh") > "$out/fixture-$fixture.txt" 2>&1
  rc=$?
  echo "name=fixture case=$fixture rc=$rc probe_units=$(grep -c 'scripts/hosts/probe.py' "$out/fixture-$fixture.txt") refusals=$(grep '^refused' "$out/fixture-$fixture.txt" | cut -f2,3 | tr '\t\n' ' ,')" >> "$summary"
done
export ATTACK_OUT="$out/worlds"
bash "$here/v1-tmpdir-world.sh" "$tree" tmp-kept 6 none > "$out/v1-tmp-kept.txt" 2>&1
bash "$here/v1-tmpdir-world.sh" "$tree" tmp-removed 6 3 > "$out/v1-tmp-removed.txt" 2>&1
bash "$here/v1-tmpdir-world.sh" "$tree" tmp-removed-lookup-off 6 3 RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off > "$out/v1-tmp-removed-lookup-off.txt" 2>&1
bash "$here/v1-tmpdir-world.sh" "$tree" tmp-removed-nohostlookup 6 3 RUN_WITH_MEMORY_CAP_BREAK=nohostlookup > "$out/v1-tmp-removed-nohostlookup.txt" 2>&1
bash "$here/v1-admitted-then-judged.sh" "$tree" admitted-peer-fits 3 none > "$out/v1-admitted-peer-fits.txt" 2>&1
bash "$here/v1-admitted-then-judged.sh" "$tree" admitted-peer-tail-rlimit 3 'Resource limit (rlimit) exceeded' > "$out/v1-admitted-peer-tail-rlimit.txt" 2>&1
bash "$here/v2-import-side-effect.sh" "$tree" import-intact > "$out/v2-import-intact.txt" 2>&1
bash "$here/v2-import-side-effect.sh" "$out/variants/broken-import" import-broken > "$out/v2-import-broken.txt" 2>&1
for file in "$out"/v1-tmp-*.txt "$out"/v2-import-*.txt; do
  sed -n 's/^\(name=[^ ]*\).* status=\([0-9]*\).* ran=\([a-z]*\).*/\1 status=\2 ran=\3/p' "$file" >> "$summary"
  grep -q 'FileNotFoundError' "$ATTACK_OUT/world-$(sed -n 's/^name=\([^ ]*\).*/\1/p' "$file")/stderr" 2>/dev/null && echo "$(sed -n 's/^\(name=[^ ]*\).*/\1/p' "$file") traceback=FileNotFoundError" >> "$summary"
done
for file in "$out"/v1-admitted-*.txt; do
  echo "$(sed -n 's/^\(name=[^ ]*\).*/\1/p' "$file") $(grep -E '^(wrapper_status|host_lookup_lines_in_diagnostics|problems|summary)=' "$file" | tr '\n' ' ' ; sed -n 's/^name=[^ ]* \(wrapper_status=[0-9]*\)/\1/p' "$file")" >> "$summary"
done
cat "$summary"
