#!/usr/bin/env python3
"""攻方自造的坏实现 V242u（不在 crates/mutations.tsv 里，只在攻方副本上用过）：E171 第 242 行那一类（系统配置轮换先于根槽 FUA），
只在带单元的发布上错（零单元发布照旧先根后轮换）。用法：apply-variant-v242u.py <副本根>"""
import os, sys
path = f"{sys.argv[1]}/crates/singlefs-core/src/transaction.rs"
source = open(path, encoding="utf-8").read()
original = """    writer.perform_commit_step(CommitStep::Barrier)?;
    persist_the_root_then_rotate_the_system_configuration(
        writer,
        writes.checkpoint_txg,
        &writes.root_slot,
        writes.journal_tail,
        writes.journal_instance,
    )
}"""
replacement = """    writer.perform_commit_step(CommitStep::Barrier)?;
    if writes.units.is_empty() {
        return persist_the_root_then_rotate_the_system_configuration(
            writer,
            writes.checkpoint_txg,
            &writes.root_slot,
            writes.journal_tail,
            writes.journal_instance,
        );
    }
    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {
        journal_tail: writes.journal_tail,
        journal_instance: writes.journal_instance,
    })?;
    writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {
        checkpoint_txg: writes.checkpoint_txg,
        root_slot: &writes.root_slot,
    })?;
    writer.perform_commit_step(CommitStep::Barrier)
}"""
hits = source.count(original)
if hits != 1:
    sys.exit(f"原文命中 {hits} 次")
mutated = source.replace(original, replacement)
open(path + ".mutating", "w", encoding="utf-8").write(mutated)
os.replace(path + ".mutating", path)
assert open(path, encoding="utf-8").read() == mutated
print("applied V242u")
