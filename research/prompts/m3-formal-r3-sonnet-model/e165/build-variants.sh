#!/usr/bin/env bash
set -u
cd /tmp/claude-1000/m3-formal-r3-sonnet/work/e165
for row in 513 518 1003 1004 24 512 36; do
  python3 ../../scripts/apply_mutation.py . restore > /dev/null
  python3 ../../scripts/apply_mutation.py . "$row" || { echo "apply failed $row" >> ../build-variants.log; continue; }
  nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/capped.sh 4 cargo build --release --locked --offline -p singlefs-checker-tier --bin e165_replay_bounded_enumeration > ../build-v$row.log 2>&1
  echo "row $row build exit=$?" >> ../build-variants.log
  cp target/release/e165_replay_bounded_enumeration ../bin-v$row
done
python3 ../../scripts/apply_mutation.py . restore >> ../build-variants.log
echo done >> ../build-variants.log
