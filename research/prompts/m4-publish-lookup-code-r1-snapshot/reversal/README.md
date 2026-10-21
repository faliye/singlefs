# m4-publish-lookup-code-r1 写判决前的快照核对与倒推（2026-10-20）

`sha256sum -c research/prompts/m4-publish-lookup-code-r1-snapshot/sha256sums.txt`：`transaction.rs` 与 `crates/mutations.tsv` 对不上，其余四份 OK。

- `crates/mutations.tsv`：里程碑四预开发的会话 的 C860 丙补丁改过（追加 6 行、整行替换原第 4854、4855 行、删原第 4958 行，原文在 `b6-c860/`）；它打之前拷的原样 `mutations-before.tsv` 的 sha256 `99dc56e6387ada5b7c96ba36f0cd5d0f734eb0fc7e2b528f719e1241c5b0c63a` 与快照相同（主 agent 现核）。之后 里程碑三的会话 的三份补丁又追加、替换过若干行，不改快照那一刻已有的七·6 行。
- `crates/singlefs-core/src/transaction.rs`：里程碑三的会话 的系统配置落地补丁（`eb-sysconf/crates.patch`）改过；把这份补丁只对 `transaction.rs` 反向打回主工作区此刻那一份，得到的 sha256 前缀 `4f27be528003e1bf9abb` 与快照相同（主 agent 现核）。
- 两条腿引的行都在冻结副本 `~/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/` 里，不受上面两处改动影响。
