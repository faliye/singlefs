倒推快照时 kb 原样的改动清单（2026-10-04，主 agent）：快照 kb-sha256.txt 取在派腿那一刻，之后有四批改动落在 D16（发布语义）、checks-owed.md、invariants.md 上，按落盘的逆序反着做（`kb-reverse.py`），四份文件的 sha256 与 kb-sha256.txt 逐份相同。
- 删除与并发那条线第三份写回：主 agent 手改 C693 补 C159 一处；scribe-b 插 C691–C693 三行、定点替换 9 条里落在这三份的（`/tmp/claude-1000/m3-writeback3/scribe-b/replace-spec.json`）；scribe-a 规格（`peer-deletion-writeback3/scribe-a/spec.md`）落在 D16 的 2 处。
- 本线 C342 回填一处（replace-once）。
- 本线写回乙 11 处（`/tmp/claude-1000/m3-r1/batch9-writeback-b/spec.json`，落在 D16 与 invariants）。
- 本线写回甲落在 checks-owed 的 7 处（`/tmp/claude-1000/m3-r1/batch9-writeback-a/spec.json` 第 9–15 条；C398 那一行从开着的表挪去还清的表，倒推时插回 C399 之前）。
D13（验证路线） 自快照以来没变。
