# m3-batch16-code-r1 快照倒推记录（2026-10-04）

`bash research/scripts/snapshot-check.sh m3-batch16-code-r1`：9 个文件里 3 个对不上。

| 文件 | 怎么倒推 | 结果 |
|---|---|---|
| `.claude/kb/decisions/16-发布语义.md` | 快照 21:30 取，文件 21:33 被改（不是这一轮的腿）；`git diff HEAD` 三处里第 42、77 行是快照之前删除线第四次写回的，第 38 行是快照之后那一处。把现行文件第 38 行换回 HEAD 的第 38 行 | sha256 `f9aabb900e8d…` 与快照相同；攻方引的第 38 行半句「挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽」在快照那一行里就有；第 42 行没动 |
| `.claude/kb/checks-owed.md` | 快照之后只在 C695 之后插行（C696–C708） | 正推引的 C692 在第 548 行，插行在它之后，行号没动 |
| `crates/mutations.tsv` | 快照之后只在末尾追加 | 腿引的第 2386–2393 行没动 |

`python3 research/scripts/cite-check.py`（两份腿报告，根：冻结副本与仓根）：「✓ 核了 57 处引文，对上 57 处，没判 0 处」。
