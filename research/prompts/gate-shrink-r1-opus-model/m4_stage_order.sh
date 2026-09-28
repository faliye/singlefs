#!/usr/bin/env bash
# 模型 M4（gate-shrink-r1 云端攻方）：去编号、改成 <类>-<内容>.sh 之后，gate.sh 的 `find … | sort` 排出来的次序。
# 名字取方案 A 表里写出的九个，checker-tier 那几道与 77 号的名字方案没写，按「checker-tier-<原文件名去编号>」取；
# 77 号另试几个「改名让它排最后」的候选。lib.sh 先试 C.UTF-8、C.utf8，都没有才 en_US.UTF-8，两种 locale 都排一遍。
# 用法：bash m4_stage_order.sh
set -uo pipefail
names=(doc-kb.sh doc-experiments.sh doc-process-records.sh doc-text.sh
       code-source-discipline.sh code-tooling.sh code-research-build.sh
       harness-tests.sh checker-independence-and-sync.sh
       checker-tier-layer0-replay.sh checker-tier-qemu-device-streams.sh checker-tier-lkmm.sh
       checker-tier-crates-mutation-replay.sh checker-tier-replay.sh)
for candidate_for_77 in checker-tier-test-environment.sh harness-test-environment.sh zz-test-environment.sh test-environment.sh; do
  for locale_name in C.UTF-8 en_US.UTF-8; do
    order="$(printf '%s\n' "${names[@]}" "$candidate_for_77" | LC_ALL="$locale_name" sort)"
    position="$(grep -nxF "$candidate_for_77" <<<"$order" | cut -d: -f1)"
    after="$(awk -v p="$position" 'NR > p' <<<"$order" | tr '\n' ' ')"
    printf '%s\t%s\t第 %s / %s 个\t排在它后面的：%s\n' "$candidate_for_77" "$locale_name" "$position" "$(wc -l <<<"$order")" "${after:-无}"
  done
done
echo "-- 两种 locale 下整张表的次序是否相同（77 号取 checker-tier-test-environment.sh）："
diff <(printf '%s\n' "${names[@]}" checker-tier-test-environment.sh | LC_ALL=C.UTF-8 sort) \
     <(printf '%s\n' "${names[@]}" checker-tier-test-environment.sh | LC_ALL=en_US.UTF-8 sort) && echo "相同"
