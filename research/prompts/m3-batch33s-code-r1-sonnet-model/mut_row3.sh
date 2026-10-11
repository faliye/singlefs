#!/usr/bin/env bash
# compact variant of mut_row2.sh: one summary line per row
set -u
REPO=${SINGLEFS_REPO}; PRISTINE=${PRISTINE_TREE}; TABLE=${PRISTINE}/crates/mutations.tsv
export CARGO_TARGET_DIR=${SCRATCH}/target
for row in "$@"; do
  OUT=${SCRATCH}/logs/row-${row}.log
  python3 -I ${SCRATCH}/bin/apply_row.py "$TABLE" "$row" "$REPO" "$PRISTINE" restore >/dev/null
  python3 -I ${SCRATCH}/bin/apply_row.py "$TABLE" "$row" "$REPO" "$PRISTINE" apply > "$OUT.head" 2>&1 || { echo "row $row"; cat "$OUT.head"; continue; }
  cargo_args=$(grep '^CARGO ' "$OUT.head" | sed 's/^CARGO //')
  cd "$REPO"
  # shellcheck disable=SC2086
  bash ${SINGLEFS_MAIN_REPO}/scripts/capped.sh 7 nice -n 19 cargo test --release --offline $cargo_args > "$OUT" 2>&1
  code=$?
  python3 -I ${SCRATCH}/bin/apply_row.py "$TABLE" "$row" "$REPO" "$PRISTINE" restore >/dev/null
  must=$(grep '^MUST ' "$OUT.head" | sed 's/^MUST //')
  nfail=$(grep -cE '^test .* FAILED$' "$OUT")
  if grep -qE "^test (.*::)?${must} \.\.\. FAILED$" "$OUT"; then hit=MUST-RED; else hit=MUST-NOT-RED; fi
  echo "row $row exit $code $hit failed=$nfail $(grep -E 'test result' "$OUT" | head -1 | cut -c1-90) | $must" | cut -c1-300
done
