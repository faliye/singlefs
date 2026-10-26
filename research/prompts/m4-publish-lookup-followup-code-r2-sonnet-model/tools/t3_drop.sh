#!/usr/bin/env bash
# t3_drop.sh：不开特性 lookup-work-counters 的产品构建里，InsideALookupSite 实现 Drop（空体）有没有多出代价。
# 三份副本 after（第 11 批终稿）、dropless（after 里把 impl Drop 改回 cfg(feature) 之下，即第 11 批之前的写法）、before（第 11 批之前）各编 singlefs-core 的库
# （release、不开特性），用 --emit=llvm-ir 取优化后的 IR，数 InsideALookupSite 的 drop 调用、landingpad、cleanup 与 IR 总字节、函数定义数。
set -u
root=$(pwd); threads=${THREADS:-31}
for copy in ${COPIES:-after dropless before}; do
  ( cd "$root/$copy" && find crates -type f -exec touch {} + && CARGO_TARGET_DIR="$root/$copy/target-ir" bash "$root/scripts/hosts/capped.sh" "$threads" cargo rustc --offline --release -p singlefs-core --lib -- --emit=llvm-ir -C codegen-units=1 > "$root/t3drop-$copy.log" 2>&1 )
  ll=$(ls -t "$root/$copy"/target-ir/release/deps/singlefs_core-*.ll 2>/dev/null | head -1)
  echo "name=t3_drop copy=$copy ll=$(basename "$ll") ll_bytes=$(wc -c < "$ll") define=$(grep -c '^define ' "$ll") landingpad=$(grep -c 'landingpad' "$ll") drop_in_place_InsideALookupSite_refs=$(grep -c 'InsideALookupSite' "$ll") resume=$(grep -c 'resume ' "$ll")"
  grep -n 'InsideALookupSite' "$ll" | cut -c1-230 | sed "s/^/name=t3_drop_line copy=$copy /"
done
