import sys, os, shutil
FROZEN = os.path.expanduser("~/.local/share/singlefs/work/m3-batch58-code-r1/repo")
MINE = os.path.expanduser("~/.local/share/singlefs/work/m3-batch58-code-r1-sonnet/repo")
path = "crates/singlefs-core/src/recovery.rs"
mine = f"{MINE}/{path}"
if sys.argv[1] == "restore":
    shutil.copyfile(f"{FROZEN}/{path}", mine); print("restored"); sys.exit(0)
s = open(mine, encoding="utf-8").read()
# 读者改法 A：逐条判里拒统计量标签 > 12
oldA = """    let entry = accounting_entry_parsed_at_its_field_table_width(entry_bytes)?;
    if entry.generation > root.checkpoint_txg || entry.sequence == 0 {"""
newA = """    let entry = accounting_entry_parsed_at_its_field_table_width(entry_bytes)?;
    if entry.statistic > STATISTIC_INODE_WATERMARK || entry.generation > root.checkpoint_txg || entry.sequence == 0 {"""
# 读者改法 B：数条目时不算统计量标签 > 12 的行
oldB = """    let entries_other_than_the_inode_number_watermark_steps = accounting_entry_bytes
        .iter()
        .filter(|bytes| {"""
newB = """    let entries_other_than_the_inode_number_watermark_steps = accounting_entry_bytes
        .iter()
        .filter(|bytes| !AccountingEntry::parse(bytes).is_some_and(|entry| entry.statistic > STATISTIC_INODE_WATERMARK))
        .filter(|bytes| {"""
for old, new in ((oldA, newA), (oldB, newB)):
    assert s.count(old) == 1, old
    s = s.replace(old, new)
open(mine, "w", encoding="utf-8").write(s); print("applied A+B")
