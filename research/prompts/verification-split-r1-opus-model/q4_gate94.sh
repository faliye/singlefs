#!/usr/bin/env bash
# Q4：快照树的 94 号（与 HEAD 那一份）对几种「dev-dependency 进 checker 库」形态的判定。用法：bash q4_gate94.sh [快照目录] [草稿目录]
set -uo pipefail
REPOSITORY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
S="${1:-/tmp/claude-1000/three-way-attack/verification-split-r1/snap}"
W="${2:-/tmp/claude-1000/three-way-attack/verification-split-r1/q4-rerun}"
mkdir -p "$W"
make_variant() { rm -rf "${W:?}/$1"; mkdir -p "$W/$1/crates"; cp "$S/Cargo.toml" "$W/$1/"
  for crate in singlefs-checker singlefs-core singlefs-format singlefs-harness; do mkdir -p "$W/$1/crates/$crate"; cp "$S/crates/$crate/Cargo.toml" "$W/$1/crates/$crate/"; cp -r "$S/crates/$crate/src" "$W/$1/crates/$crate/"; done; }
for variant in V0 V1 V2 V3 V4 V5 V6; do make_variant $variant; done
L=crates/singlefs-checker/src/lib.rs; MANIFEST=crates/singlefs-checker/Cargo.toml
printf '\n#[cfg(test)]\nmod core_probe {\n    #[allow(unused_imports)]\n    use singlefs_core as _;\n}\n' >> "$W/V1/$L"
printf 'implementation = { package = "singlefs-core", path = "../singlefs-core" }\n' >> "$W/V2/$MANIFEST"
printf '\n#[cfg(test)]\nmod core_probe {\n    #[test]\n    fn builds_an_image_with_the_implementation() {\n        let _ = implementation::transaction::TransactionGroup::default;\n    }\n}\n' >> "$W/V2/$L"
printf '\n/// ```\n/// use singlefs_core as _;\n/// ```\npub fn doctest_probe() {}\n' >> "$W/V3/$L"
printf '\n#[path = "../../singlefs-core/src/transaction.rs"]\n#[allow(dead_code)]\nmod copied_from_the_implementation;\n' >> "$W/V4/$L"
printf '\ninclude!("../../singlefs-core/src/transaction.rs");\n' >> "$W/V5/$L"
printf '\n#[cfg(test)]\nmod harness_probe {\n    #[allow(unused_imports)]\n    use singlefs_harness::crash as _;\n}\n' >> "$W/V6/$L"
git -C "$REPOSITORY" show HEAD:.claude/gate.d/94-checker-implementation-disjoint.sh > "$S/.claude/gate.d/94-head-version.sh"
for variant in V0 V1 V2 V3 V4 V5 V6; do
  out="$(bash "$S/.claude/gate.d/94-checker-implementation-disjoint.sh" "$W/$variant" 2>&1)"; rc=$?
  echo "== $variant snapshot-94 rc=$rc: $(grep -E '✓|✗' <<< "$out" | cut -c1-90 | tr '\n' ' ')"
done
for variant in V0 V2; do
  out="$(bash "$S/.claude/gate.d/94-head-version.sh" "$W/$variant" 2>&1)"; rc=$?
  echo "== $variant HEAD-94 rc=$rc: $(grep -E '✓|✗' <<< "$out" | cut -c1-90 | tr '\n' ' ')"
done
