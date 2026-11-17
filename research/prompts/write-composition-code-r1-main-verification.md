# write-composition-code-r1：判决（2026-11-17）

正文 `research/prompts/_write-composition-code-r1-body.md`，开工快照 `research/prompts/write-composition-code-r1-snapshot/`（被判文件的副本与 `changes.patch`；开轮时没记 sha256 清单，`crates-sha256.txt` 是写这份判决时照快照目录里的两份副本现算补上的，两份副本没改）。开这一轮的会话已不在、没写判决；两个在跑的会话都回了不是自己的，`python3 scripts/agents/who-touched.py research/prompts/_write-composition-code-r1-body.md` 列出的在跑会话 Edit/Write 都是 0。照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 15 条由这一会话补判决。

## 一、腿与核查

| 腿 | 报告 | 结论 |
|---|---|---|
| 正推 | `research/prompts/write-composition-code-r1-forward-output.md` | 没打中：拆写定理的前提推得出两段合法、长度相同、每个字节相同，端点切分与空请求在范围内 |
| 攻方 | `research/prompts/write-composition-code-r1-attack-output.md` | 没打中：七种重叠、包含、相邻、空写形状手算都不成反例；五条负例的锚点各命中一次、替换不靠语法错 |

核查：

- 冻结副本 `research/prompts/write-composition-code-r1-snapshot/crates/singlefs-replay-prefix/src/partial_write_proofs.rs` 的 sha256 是 `ffdb2a130b76e80c3a7f08f79968458ec82e6a40d8a4470ecdbaa1405ddd3c73`，与正推报告写的摘要相同；腿用的副本 `/home/fy5090/.local/share/singlefs/work/formal-write-composition/copy` 里那一份同一摘要。
- `task-server/task submit -- python3 scripts/agents/cite-check.py research/prompts/write-composition-code-r1-forward-output.md research/prompts/write-composition-code-r1-attack-output.md --root /home/fy5090/.local/share/singlefs/work/formal-write-composition/copy --background research/prompts/_write-composition-code-r1-body.md`（任务 18656，退 0）：「核了 0 处引文、4 处裸行号，对上 0 处，没判 4 处」。四处裸行号（`partial_write_proofs.rs` 第 108、109、250 行，`verus-must-red-worlds.tsv` 第 394 行）与腿引的其余行（第 113、207、211、212、228、236、254、259、271、287、288、289 行，表第 395–398 行）在冻结副本上逐行人工核过，内容与腿的描述相符。
- 这一版的机器验证是任务 17012（门禁 formal-verification 三格）：「formal-verification.sh：3 格里 3 格绿（verus-proof verus-vacuity verus-must-red-worlds），0 格本次未跑（无）」，五条必红世界落点与表里 landing= 相同；普通构建任务 17013 退 0。

## 二、判决

两条腿都没打中，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」第一条，这一题停在第一轮，不开第二轮。

被判的两份文件在快照之后又改过：此刻 `crates/singlefs-replay-prefix/src/partial_write_proofs.rs` 的 sha256 是 `77713175d6db54fe8da0936728ec6cd2296e96a3cc007039aa1f478afa593710`、`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 是 `c87f9beed710cf89f604484249f46ed7328b15648cb2a896ea390ba9b9fe6055`，与 `research/prompts/replay-prefix-backfill-code-r1-snapshot/crates-sha256.txt` 记的相同。现行这一版由 `research/prompts/replay-prefix-backfill-code-r1-main-verification.md` 判，它的打中与改法以那份判决为准；这份判决的结论只管 2026-11-14 冻结那一版。

## 三、改法

无。

## 交用户的

无。

## 这一轮被判过的文件

- `research/prompts/write-composition-code-r1-snapshot/crates/singlefs-replay-prefix/src/partial_write_proofs.rs`：判了 2026-11-14 那一版新增的拆写定理、后写优先定理与具体字节见证，两条腿都没打中。
- `research/prompts/write-composition-code-r1-snapshot/crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`：判了那一版新增的五条负例（第 394–398 行）的锚点与落点，两条腿都没打中。

## 回看决策

不涉及决策：这一轮只判固定长度覆盖写字节模型上的证明与负例，没有拿它确立或推翻哪条决策。
