# 根环实现第 10z 批第二块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 R -->

## 一、题面

这一轮攻第 10z 批第二块：抬 F 照记着的落点。它照用户 2026-10-07 定案实现，定案出自第 10y 批代码轮判决 `research/prompts/m3-batch10y-code-r1-main-verification.md` 第二节 K4-2 那一格与第三节。

用户定案原话：抬 F 那一遍，被抛弃根一律照根环表里记着的落点算影子账，不再从盘上重读账，读不出也不拒。「崩了再可写挂载」那一道照 B 拒，不变。

条款：
- `.claude/kb/decisions/16-发布语义.md` 已定项 1「根槽这一次读坏」那一行，末句「**抬 F 那一串（准入与卸载）里的被抛弃根**」；
- `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 回退段（B 凭据放行那一句之后「抬 F 那一串（准入与卸载）不走这一道」）与「可写挂载时的预隔离」一段；
- `.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第九项。

改动全文：`research/prompts/_m3-batch10z-b-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-b/patch/report.md`。

| # | 改动 | 落点 |
|---|---|---|
| A | 新枚举 `PlacementsOfAbandonedRoots`：抬 F 那一串取 `RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts`，按现行那一版实例表判成被抛弃的根，根环表里按 (实例代号, txg) 记着同一条根的照记着的落点、不读账、读不出不拒；表里没记着的照今天读盘、照 B。可写挂载那一道传 `ReadFromTheirAccounts` | `crates/singlefs-core/src/mount.rs` |
| B | `RootRingOccupant::AbandonedRoot` 加实例代号与 txg，新方法 `placements_recorded_for_the_abandoned_root`；`RaisedFloor::abandoned_roots_unreadable` 不再数记着落点的根 | `crates/singlefs-core/src/allocator.rs`、`mount.rs` |
| C | 新用例三条（卸载做成且 checker 不红 I-7.4、I-4.8；会话写准入抬 F 不拒；崩了再可写挂载没有凭据照拒）；改旧用例 LR3（再可写挂载那一刻把后一条被抛弃根的根槽抹零，落进「表里没记着」那一形）；变异追加 4、替换 3 | `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`、`crates/mutations.tsv` |

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/`，取在 10z 第二块合入之后（底含删除线 W、V 与根环线 10z 第一块）；被判文件 sha256 在 `research/prompts/m3-batch10z-b-code-r1-snapshot/crates-sha256.txt`（330 份）。
- 合后验证（`~/.local/share/singlefs/work/m3-r1/merge-10z-b/verify-summary.txt`）：core lib `200 passed; 0 failed`；harness 档四个包 179 个测试二进制全 ok；fmt、clippy 退 0。实现员证红：追加 4 行、替换 3 行。
- 实现员报告「停下交主 agent」第 1 条：根环表里**没记着**落点、照今天读盘照 B 的有四形：
  1. 可写挂载自己写行之后才判成被抛弃的根（第 10y 批 K4-1 的 R：挂载那一刻按最新那条根的表判有效，记成 `ValidRoot`，`mount.rs` 第 2405、2575 行；写行之后表里的类不跟着换，第 5280 行）；
  2. 挂载那一刻读坏或读回全零的槽；
  3. E2 重读后从表里拿掉的槽；
  4. 那一槽记着的是另一条根。
  派发举的「挂着时管理员回退」一例不成立：回退不写实例表行（`mount.rs` 第 7160 行），目标之上那几条根仍有效。
- 第 1 形的另一个候选：根环表里 C 那一圈按槽记的落点（`placements_named_by_unapplied_journal_records`，删除线第 35 批 W 加了 C_ACCOUNT：读得出的记录的新根段读那次发布的账）。删除线下一批（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md`，没合）要加 C_NEWER_ROOTS（C 并上环里比所选那一版新、按实例表有效、读得出的根的账）与 C_MAPPING。
- 删除线第 37 批 X 的发现（没合）：`allocator.rs` 的 `protected.extend(...)` 那一句在 4 GiB 池上造得出、删掉证红；384 槽小盘上同一形删掉不红，因为回收走抬 F 那一路、那一路自己补隔离。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　定案做到没有
1. 抬 F 那一串的每个入口（`raise_rollback_floor`、`raise_rollback_floor_to_the_admission_ceiling`、`raise_the_floor_through`、准入推空间那一路）都走「记着的落点」吗？有没有一条抬 F 的路径还在读被抛弃根的账、读不出照 B 拒？
2. 按 (实例代号, txg) 认同一条根：有没有一形对上了却不是同一条根，或是同一条根却对不上？
3. 可写挂载那一道（含崩了再可写挂载）的 B、C、F1、F2 是不是一个字没动？

### K2　I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽）
1. 记着的落点是挂载那一刻算的。抬 F 时照它算影子账，有没有一形被抛弃根引用的单元掉出影子账、被分配器发出去（挂载时记得不全、挂载之后那条根的引用变了、T1 甲那一类）？
2. 表里没记着的四形照今天读盘照 B：第 1 形（K4-1 的 R）照今天，与改用 C 那一圈按槽记的落点相比，哪一臂会掉出影子账、哪一臂会白拒？两臂都量。
3. `RaisedFloor::abandoned_roots_unreadable` 不再数记着落点的根：有没有调用方靠这个数做决定？

### K3　测试判别力
新用例与改过的 LR3 能不能拦住这一批每一处被改回去？LR3 改完之后，它原来要钉的那件事（被抛弃根的账读不出时从中央映射现算）还钉着吗？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
