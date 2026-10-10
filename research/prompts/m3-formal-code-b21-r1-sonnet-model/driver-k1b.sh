#!/usr/bin/env bash
# 第 21 批 K1 第二段驱动：在副本上跑 checker_known_bad_images 的探针与原用例，六种丢臂各一次（B21_DROP=种类码）。
set -u
BIN=/tmp/claude-1000/m3-formal-code-b21-r1-sonnet/target/debug/deps/checker_known_bad_images-fa5092303c98133d
for drop in none 4 6 7 8 11 12; do
  B21_DROP=$drop "$BIN" b21_probe_bad_images --nocapture --test-threads 1 2>&1 | grep -E '^PROBE|test result'
  echo "== orig test under drop=$drop"
  B21_DROP=$drop "$BIN" each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant 2>&1 | grep -E '^test |test result|第 [0-9]+ 张' | cut -c1-200 | head -6
done
