#!/usr/bin/env python3
"""第二处副本改动：crates/mutations.tsv 第 2284、2285 行那两条第二截变异做成 B18_VARIANT 开关（s2_call_injected、s2_old_judge），给 t1 当会红的对照。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])
F = root / "crates/singlefs-checker-tier/src/crash_injection.rs"
t = F.read_text()
old = """            match judge_writable_mount_refusal_of_a_crash_state(
                refusal_on_the_crash_image.crash_image_materialized,
                refusal_on_the_crash_image.version_landed_on_after_the_crash,
                refusal_error,
                disk,
                refusal_on_the_crash_image.read_only_read_back,
                refusal_on_the_crash_image.read_faults,
            ) {"""
new = """            let b18_verdict = if b18_variant("s2_old_judge") {
                singlefs_harness::mount_transparency::judge_writable_mount_refusal(
                    refusal_on_the_crash_image.crash_image_materialized,
                    refusal_on_the_crash_image.version_landed_on_after_the_crash,
                    refusal_error,
                    disk,
                    refusal_on_the_crash_image.read_only_read_back,
                )
            } else {
                judge_writable_mount_refusal_of_a_crash_state(
                    refusal_on_the_crash_image.crash_image_materialized,
                    refusal_on_the_crash_image.version_landed_on_after_the_crash,
                    refusal_error,
                    disk,
                    refusal_on_the_crash_image.read_only_read_back,
                    if b18_variant("s2_call_injected") {
                        ReadFaultsInTheCrashState::Injected
                    } else {
                        refusal_on_the_crash_image.read_faults
                    },
                )
            };
            match b18_verdict {"""
assert t.count(old) == 1, t.count(old)
F.write_text(t.replace(old, new))
print("patched 2")
