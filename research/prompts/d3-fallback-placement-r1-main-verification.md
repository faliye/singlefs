# d3-fallback-placement-r1 判决：提交内生块回落时要不要优先已在重写集里的叶（2026-11-09）

<!-- doc-lint:not-numbers P2a R -->

## 一、核查

- 腿：正推 `research/prompts/d3-fallback-placement-r1-sonnet-output.md`，攻方 `research/prompts/d3-fallback-placement-r1-opus-output.md`。各抽一次，「没打中」不支撑结论。
- `python3 scripts/agents/cite-check.py` 两份：`✓ 核了 59 处引文、39 处裸行号，对上 59 处，没判 39 处（其中裸行号没有引文可比 39 处；逐处列在上面）`。开工快照 `sha256sum -c` 没有对不上的行。
- 复跑：攻方任务 11468 退 0，观测 `~/.local/share/singlefs/work/d3-fallback-placement-r1-opus/rerun-1/observations.txt`；正推任务 11481 退 1（只因第五行那条回归用例），观测在 `~/.local/share/singlefs/work/d3-fallback-placement-r1-sonnet/rerun-1.attempts/attempt.4TqBs8/`。下面的数都是这两趟的。模型是腿的副本装置，读数不进 kb，老化段各抽一次、一种历史。
- 正推模型 `rerun.sh` 第 111 行范围声明写 `2 个测试函数`，闸不收，主 agent 在副本 `model-copy` 里改成 `1 个测试目标` 再跑，只差这一处。

## 二、逐格判

| 格 | 判 |
|---|---|
| 优先重写集、不够退最低空槽，修不修得了撞 64 | 打中（两腿一致，量实）：`forged_summary arm=today probes=89 did_not_settle=8 most_rounds=64`，`arm=prefer_then_lowest probes=89 did_not_settle=8 most_rounds=64`，正推补丁树与今天树同一份扫描 `forged-sweep-today-vs-patched.diff` 0 字节。这条政策本身不改那些探的任何一个落点。用户定这条政策要关的格它关不上 |
| 改退路才关得上 | 量实（锻造盘）：`prefer_then_richest_leaf did_not_settle=0 most_rounds=5`、`prefer_then_lowest_leaf_with_more_than_two_free did_not_settle=0 most_rounds=4`；叶链模型 `rewrite_set_then_densest_leaf` 同向 |
| 只用用户操作的老化盘：删一到四段之后再写 200 个文件（打中 C） | 打中，量实。今天 6 格全部写完。见第三节表。两种先取重写集的臂 2 格被 `EmptyClusterSegmentsBelowTheFloor` 拒；取空槽最多的叶 4 格；空槽多于 2 的最低叶 4 格；绕开全空段的臂 3 格。**用户删了一段，后面的写被拒**，撞 `.claude/kb/decisions/03-空间分配.md` D3（空间分配） 已定项 9 第 2 条「删掉一个 s 字节的对象之后，同样大小的写必须在有界步数内成功」（字面要现核，见第四节）。攻方跑前写的推翻条件没满足 |
| 老化盘上今天的政策撞不撞 64 | 没打中（一种历史、一次抽样）：`today` 六格 `most_rounds` 6，`did_not_settle` 0 |
| 多盘每块盘各答（打中 B） | 打中，量实：`unequal arm=prefer_then_lowest_on_each_device extra_slots=1 … answer=Err(CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined`，1、33、63 三格都拒，今天不拒。正推的定义（交集、各盘先答）不走这一形。只在分配器级量过 |
| 定义 | 真要改，只能按叶、取每块盘都重写的叶的交集（两腿一致）。今天的回归用例有一条钉在旧落点上：`raising_the_floor_when_the_only_empty_segment_is_held_falls_back_to_slots_outside_the_hold` 在正推补丁树上红 |
| 固定点终止论证 | `crates/singlefs-core/src/transaction.rs:13811` 那段「按同一串槽位往后取」在偏好之后失效（两腿一致），新论证推的、没证 |
| checker 不变量 | 不用动（两腿读码、grep，一次抽样），崩溃状态没枚举 |

## 三、老化盘读数（`aged_delete_summary`，写 200 个文件写成几个 / 收尾全空段数）

| 臂 | 最低 1 段 | 最低 4 段 | 中间 1 段 | 中间 4 段 | 最高 1 段 | 最高 4 段 | 锻造盘撞 64（89 探） |
|---|---|---|---|---|---|---|---|
| `today` | 200 / 1 | 200 / 3 | 200 / 4 | 200 / 5 | 200 / 6 | 200 / 3 | 8 |
| `prefer_then_lowest` | **127** / 0 | 200 / 5 | 200 / 4 | 200 / 2 | **89** / 0 | 200 / 3 | 8 |
| `prefer_then_richest_leaf` | 200 / 1 | **41** / 0 | **173** / 0 | 200 / 4 | **89** / 0 | **141** / 0 | 0 |
| `prefer_then_lowest_leaf_with_more_than_two_free` | **85** / 0 | **87** / 0 | **186** / 0 | 200 / 2 | **89** / 0 | 200 / 3 | 0 |
| `prefer_then_lowest_sparing_record_empty_segments` | **127** / 0 | 200 / 3 | 200 / 3 | 200 / 5 | **89** / 0 | **140** / 0 | 8 |

## 四、结论

优先重写集不实现。它关不上用户要关的格，还在只用用户操作走到的盘上让写被拒。能关上 64 的两条退路在老化盘上更差。这个判断只有一种老化历史、各抽一次，已定项 9 第 2 条的字面没逐句核，所以写成交用户，不写成定案。D3（空间分配） 已定项 8 第 2 条不改。

撞 64 那一形只在造的盘上走到过（E206 第一段 P2a 至多 5 轮；老化盘今天至多 6 轮），候选「撞上界报成空间不够让会话推抬 F」被攻过零轮，没代价数。

## 五、交用户

| 项 | 量到的 | 状态 |
|---|---|---|
| 不改落点政策，撞 64 登欠账 | 第三节 `today` 一行 | 主 agent 推荐 |
| 仍改落点政策，取哪条臂 | 第三节其余各行 | 都零轮 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 8 | 不影响 | 2026-11-09 不受影响：判决不改条款，交用户 |
| D3（空间分配） 已定项 9 | 不影响 | 2026-11-09 不受影响：只拿来判臂，条款没改 |
| D26（后台整理与放置回收） 已定项 3 | 不影响 | 2026-11-09 不受影响：底额条款没改，行为差异只在臂上 |
