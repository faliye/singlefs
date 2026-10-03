# m3-formal-code-r1 云端正推腿的小脚本（只读、不编译）

三个脚本都是只读检查，冻结副本路径写死为 `/tmp/claude-1000/m3-formal-code-r1/tree`。

- `anchors.py`：`cd /tmp/claude-1000/m3-formal-code-r1/tree && python3 <本目录>/anchors.py crates/mutations.tsv .`，
  对每行变异表数「原文（`\n` 换成换行）」在文件里的命中数，不是 1 的列出；`crates/singlefs-replay-prefix/verus-mutations.tsv` 同法。
- `hits.py`：`python3 <本目录>/hits.py <仓根或冻结副本>`，`crates/mutations.tsv` 第 173、174、177 行的命中数（换行换算后与按字面反斜杠 n 数各一个）。
- `callgraph.py`：对 `e165_replay_bounded_enumeration.rs` 用正则切函数体，列出能走到 `replay_journal` / `scan_journal` / `choose_root` / `recover` 的 `check_…` 函数（下限，不含 trait 方法与闭包间接调用）。
