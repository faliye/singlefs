#!/usr/bin/env python3
"""攻方腿（m3-batch10z-tu-code-r1）自己的变异：mutants.py <仓副本根> <变异名>。原文在文件里必须恰好命中一次，否则退 3。"""
import sys
from pathlib import Path

F = "crates/singlefs-core/src/mounted_session.rs"
MUTANTS = {
    # K1-2 / K5：只进内存的盘那一层
    "A-overlay-check-always-ok": (F,
        "        let block_bytes = self.block_bytes();\n        if !offset.0.is_multiple_of(block_bytes) || !length.is_multiple_of(block_bytes) {",
        "        let block_bytes = self.block_bytes();\n        if offset.0 < u64::MAX {\n            return Ok(());\n        }\n        if !offset.0.is_multiple_of(block_bytes) || !length.is_multiple_of(block_bytes) {"),
    "B-overlay-reads-never-see-own-writes": (F,
        "            match self.written_blocks.get(&block_index) {",
        "            match self.written_blocks.get(&(block_index | (1 << 63))) {"),
    "C-overlay-trailing-unwritten-run-not-read": (F,
        "        if let Some(run_start) = unwritten_run_start {\n            self.read_unwritten_blocks_from_the_disk(run_start, end_block, first_block, buffer)?;\n        }",
        "        let _ = unwritten_run_start;"),
    "D-overlay-keeps-each-block-one-block-later": (F,
        "                .insert(first_block + block_offset, block.to_vec());",
        "                .insert(first_block + block_offset + 1, block.to_vec());"),
    "E-overlay-write-zeroes-not-kept": (F,
        "        self.keep_in_memory(offset, &zeros);",
        "        let _ = &zeros;"),
    # K1-1：这次改动还剩什么（三处调用点）与预演怎么走
    "F-remainder-of-the-walk-before-is-only-the-walk": (F,
        "                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),\n                published_in_this_change,",
        "                RemainderOfTheUserChange::WalkTheQueue,\n                published_in_this_change,"),
    "G-remainder-of-the-change-is-only-the-walk": (F,
        "                    RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),\n                    publishes_pushed,",
        "                    RemainderOfTheUserChange::WalkTheQueue,\n                    publishes_pushed,"),
    "H-remainder-after-the-unlink-intent-republishes-the-change": (F,
        "                        RemainderOfTheUserChange::WalkTheQueue,\n                        published_in_this_change,",
        "                        RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),\n                        published_in_this_change,"),
    "I-rehearsal-with-the-shadow-ledger-off": (F,
        "                shadow_ledger,\n                change,\n            )\n            .map(|_published| ())",
        "                ShadowLedger::Off,\n                change,\n            )\n            .map(|_published| ())"),
    "K-push-refused-in-the-rehearsal-counts-as-admitted": (F,
        "        return RootRingPushRehearsal::PushRefused(Box::new(refusal));",
        "        let _ = refusal;\n        return RootRingPushRehearsal::ChangeAdmittedAfterThePush {\n            isolated_slots_released_by_the_push_on_each_device: Vec::new(),\n        };"),
    "N-rehearsal-replays-only-the-walk": (F,
        "        shadow_ledger,\n        remainder,\n    );\n    WideProtectionAtTheCeiling::StuckBehindTheWideProtection",
        "        shadow_ledger,\n        RemainderOfTheUserChange::WalkTheQueue,\n    );\n    let _ = remainder;\n    WideProtectionAtTheCeiling::StuckBehindTheWideProtection"),
}

def main() -> int:
    root, name = Path(sys.argv[1]), sys.argv[2]
    if name == "--list":
        print("\n".join(MUTANTS))
        return 0
    path, original, replacement = MUTANTS[name]
    target = root / path
    text = target.read_text()
    hits = text.count(original)
    if hits != 1:
        print(f"{name}: 原文命中 {hits} 次（要恰好 1 次）", file=sys.stderr)
        return 3
    target.write_text(text.replace(original, replacement))
    print(f"{name}: 改了 {path}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
