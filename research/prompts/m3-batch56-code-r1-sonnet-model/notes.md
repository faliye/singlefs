# notes (draft)
- K1: both readers order verified by reading: rebuild recovery.rs 2890(allocgen)->2920 width->2925 per-entry->2931 count ; cold 5389-5420 same + mapping count after.
- K2: write path only append fns container_tree.rs 476/511; recovery refuses empty container (recovery.rs ~4245). CPU walk.rs 11982-12046 lineage/ret, 12060-12120 tomb. product mode single container not judged (known).
- K3: GPU tests only knob 1 (lineage also knob 2 for >knob). No GPU red tests in product mode for new 3 judgments; no GPU red test for tombstone knob.
- Cannot run checker-tier tests. 
rows0-7 red as designed (see logs/row*.log)
- rows 0-11,15-20,31-36 of mutations.tsv (第56批) all red (logs/rowN.log). Row 35/36 red also on older tests.
- K4 mutants ka(left-first) kb(floor116) kc(ceil borrow) ke(> floor) kg(< fit) kh(sep key) ki(borrow count) red; kd(drop-empty absorb start hp instead of hp-1) GREEN; kf(always borrow attempt) GREEN (equivalent on reachable)
- probe: wide grid n<=4 33 values adjacency>=234: 221256 shapes 1744830 removals diff=0; control no-filter diffs=20+ (logs/probe-k4*.log)
- probe K2 abandon: unlink chain prefixes 63/63 mount ok 0 red (logs/probe-k2-b.log); rollback prefixes see probe-k2-d.log
