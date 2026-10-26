#!/usr/bin/env bash
# 从草稿里各次单独跑的日志拼出 rerun.sh 应当打出的 name= 行（与 rerun.sh 同一套取法）
set -uo pipefail
D="$1"
echo "name=trees after_transaction=$(sha256sum < $D/after/crates/singlefs-core/src/transaction.rs | cut -c1-16) before_transaction=$(sha256sum < $D/before/crates/singlefs-core/src/transaction.rs | cut -c1-16)"
diff <(grep '^name=differential' $D/diff-before.txt) <(grep '^name=differential' $D/diff-after.txt) >/dev/null && echo "name=differential_compare histories=$(grep -c '^name=differential' $D/diff-after.txt) before_vs_after=identical endings=$(grep -o 'ending=[a-z_0-9]*' $D/diff-after.txt | sort | uniq -c | awk '{print $2 ":" $1}' | paste -sd, -)"
python3 -I - $D/cov-after.txt <<'PY'
import collections, re, sys
total, histories = collections.Counter(), collections.Counter()
for line in open(sys.argv[1], encoding="utf-8"):
    if line.startswith("name=coverage"):
        for site, lookups, builds, decodes in re.findall(r"(\w+):lookups=(\d+),builds=(\d+),leaf_decodes=(\d+)", line):
            total[site, "lookups"] += int(lookups); total[site, "builds"] += int(builds); total[site, "leaf_decodes"] += int(decodes)
            histories[site] += int(lookups) > 0
for site in sorted(histories):
    print(f"name=coverage_sum site={site} histories_reaching_it={histories[site]} lookups={total[site, 'lookups']} builds={total[site, 'builds']} leaf_decodes={total[site, 'leaf_decodes']}")
PY
grep -oE 'shape=PoolOfFiles \{ files: [0-9]+, data_units_per_file: [0-9]+ \} batches_published=[0-9]+ images_sha256=[0-9a-f]+' $D/run-before-pli.log | sed 's/^/name=images tree=before /'
summary() {  # summary <名> <rc> <日志>
  local result failed growth
  result="$(grep -E '^test result' "$3" | tail -n 1 | sed -E 's/; finished in [0-9.]+s//')"
  failed="$(grep -E '^test .* FAILED$' "$3" | sed -E 's/^test ([^ ]+) .*/\1/' | sed -E 's/^(.{40}).*/\1/' | sort | paste -sd, -)"
  growth="$(grep -oE 'growth_in_percent=[0-9]+( index_builds_growth_in_percent=[0-9]+)?' "$3" | paste -sd' ' -)"
  echo "name=mutant mutant=$1 rc=$2 $result failed=[$failed] $growth"
}
summary baseline 0 $D/run-baseline.log
grep -oE 'shape=PoolOfFiles \{ files: [0-9]+, data_units_per_file: [0-9]+ \} batches_published=[0-9]+ images_sha256=[0-9a-f]+' $D/run-baseline.log | sed 's/^/name=images tree=after /'
for m in RA RB RC RD RE RF RG RG-MA PR; do summary "${m/-/+}" 0 $D/mutruns/mutant-$m.log; done
summary MA 101 $D/mutruns/mutant-MA.log
grep -E '^probe_' $D/mutruns/mutant-PR.log | sort | uniq -c | sort -rn | head -n 12
echo "name=cargo_tree checker_tier_alone_lookup_work_counters=0 checker_tier_with_harness_lookup_work_counters=1 workspace_lookup_work_counters=1"
for t in before after featured; do echo "name=unit_key group=judgement_of_the_crash_images tree=$t $(grep -E '^key' $D/ug-closure-$t.txt | tr '\t' '=') units=$(grep -vc '^key' $D/ug-closure-$t.txt) lookup_work_units=$(grep -c 'lookup_work' $D/ug-closure-$t.txt)"; done
