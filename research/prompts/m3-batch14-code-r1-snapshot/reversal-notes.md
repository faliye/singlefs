# m3-batch14-code-r1 快照倒推记录（2026-10-04）

`bash scripts/snapshot-check.sh m3-batch14-code-r1`：23 个文件里 6 个对不上。

| 文件 | 怎么倒推 | 结果 |
|---|---|---|
| `.claude/kb/decisions/16-发布语义.md` | 现行文件上反做 `own-writeback3-scribe-a-spec.json` 里 D16 的 2 处（第 169、224 行，K 代回扫），再反做 `peer-root-ring-batch9-writeback-b-spec.json` 里 D16 的 4 处（已定项 1 第 35、37、38 行一带） | sha256 `0e43f3447863…` 与快照相同 |
| `.claude/kb/decisions/08-核心索引结构.md` | 反做 `own-writeback3-scribe-a-spec.json` 里 D8 的 1 处 | sha256 `92d97707a0b7…` 与快照相同 |
| `crates/mutations.tsv`、`allocator.rs`、`mount.rs`、`writable_mount_of_a_formatted_pool.rs` | 腿引代码行一律指冻结副本 `/tmp/claude-1000/m3-batch14-code-r1/tree/`，cite-check 以它为根；工作区的改动出自根环第 9 批（`peer-root-ring-batch9-*-crates.patch`）与 2026-10-03 19:11 的提交 | 不影响引文 |

腿引的 kb 行：D16 第 34、39、42 行在快照原样与现行文件里逐字相同；第 37、38 行被根环写回乙改过，攻方引第 37 行的准入式子「min(每块盘上最新的持久有效根, …)」、正推引第 38 行的前半句，都落在没被改的那一段。D8 第 195、470 行与快照相同。
`python3 scripts/cite-check.py`（两份腿报告，根：冻结副本与仓根）：「✓ 核了 81 处引文，对上 81 处，没判 0 处」。
