#!/usr/bin/env python3
"""gen_alt2.py：构建脚本那一链的两条自备变异，写 alt2-mutations.tsv。"""
H = "-p singlefs-harness --test publish_lookup_by_identity -- --exact "
T = "the_core_build_script_reports_only_the_features_cargo_enabled_even_with_the_feature_variable_brought_in_from_outside"
rows = [
 ("T4a 核心层构建脚本：特性表里把特性名写错一个字母", "crates/singlefs-core/build.rs",
  '    "lookup-work-counters",\n    cfg!(feature = "lookup-work-counters"),',
  '    "lookup-work-counter",\n    cfg!(feature = "lookup-work-counters"),', H + T, T),
 ("T4b 核心层构建脚本：特性的 cfg 写死成假（开着也报没开）", "crates/singlefs-core/build.rs",
  '    "lookup-work-counters",\n    cfg!(feature = "lookup-work-counters"),',
  '    "lookup-work-counters",\n    false,', H + T, T),
 ("T4c 核心层构建脚本：特性的 cfg 写死成真（没开也报开着）", "crates/singlefs-core/build.rs",
  '    "lookup-work-counters",\n    cfg!(feature = "lookup-work-counters"),',
  '    "lookup-work-counters",\n    true,', H + T, T),
]
with open("alt2-mutations.tsv", "w", encoding="utf-8") as out:
    for name, f, old, new, args, test in rows:
        out.write("\t".join([name, f, old.replace("\n", "\\n"), new.replace("\n", "\\n"), args, test]) + "\n")
print(len(rows))
