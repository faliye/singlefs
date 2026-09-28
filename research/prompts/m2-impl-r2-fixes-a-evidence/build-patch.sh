#!/usr/bin/env bash
# 草稿：把「主工作区现版本 + 我在副本里的改动」拼成 crates.patch
set -euo pipefail
S=/tmp/claude-1000/impl-r2-fixes-a; M=<仓根>
cd "$S"; rm -rf "${S:?}/patchwork"; mkdir -p patchwork/a patchwork/b
for f in crates/singlefs-checker/src/image.rs crates/singlefs-checker/src/walk.rs crates/singlefs-checker/src/lib.rs crates/singlefs-harness/tests/checker_known_bad_images.rs crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs; do
  mkdir -p patchwork/a/$(dirname $f) patchwork/b/$(dirname $f)
  cp $M/$f patchwork/a/$f; cp $M/$f patchwork/b/$f
  diff -u baseline/$f copy/$f > patchwork/mine.diff || true
  patch -s --no-backup-if-mismatch patchwork/b/$f < patchwork/mine.diff
done
for f in litmus/publish-returns-after-the-rotation-barrier.litmus litmus/publish-returns-after-the-rotation-barrier-nofence.litmus crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs; do
  test ! -e $M/$f
  mkdir -p patchwork/b/$(dirname $f); cp copy/$f patchwork/b/$f
done
rm patchwork/mine.diff
(cd patchwork && git diff --no-index a b > ../patch/crates.patch.raw || true)
python3 - <<'PY'
import re
p='/tmp/claude-1000/impl-r2-fixes-a/patch/'
t=open(p+'crates.patch.raw',encoding='utf-8').read()
t=re.sub(r'^diff --git a/[ab]/(\S+) b/[ab]/(\S+)$', r'diff --git a/\1 b/\2', t, flags=re.M)
t=re.sub(r'^--- a/a/', '--- a/', t, flags=re.M); t=re.sub(r'^\+\+\+ b/b/', '+++ b/', t, flags=re.M)
t=re.sub(r'^index .*\n', '', t, flags=re.M)
open(p+'crates.patch','w',encoding='utf-8').write(t)
PY
rm patch/crates.patch.raw
cd $M && git apply --check -v $S/patch/crates.patch
