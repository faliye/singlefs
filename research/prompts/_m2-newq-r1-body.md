# 用户 2026-09-24 勾进这个里程碑的三道「另起一题」：第一轮正文（2026-09-24）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 -->

## 一、这一轮要判什么

用户 2026-09-24 定：下面三道条款空白在这个里程碑做，走三方再交用户定（`records/2026-09-24-里程碑二收尾调度.md` 第二节）。这一轮每格要答「今天的已定分项推得出一个答案吗」；推不出的，写成岔路（候选各自的定义、翻面观测、每条路的代价要量什么），交用户之前要有代价数（`.claude/rules/three-way-inference.md`「岔路交用户定之前，每条路要有代价数」）。

| 格 | 题 | 出处 |
|---|---|---|
| N1 | 释放之前按映射条目位置项里的校验和读盘核，**读盘本身失败**（设备报错，不是校验和对不上）怎么办：归哪个错误成员、发布照不照成、那个槽隔不隔离 | `.claude/kb/checks-owed.md` C394 前置列；`.claude/kb/decisions/19-块指针的结构与宽度预算.md`「#### 已定项 5」硬规则 1 |
| N2 | 从盘上重建上一版的路径（`crates/singlefs-core/src/recovery.rs` 的 `rebuild_version`）与释放判定（`crates/singlefs-core/src/transaction.rs` 的 `placements_to_release_via_mapping`）要不要用同一条判据核位置项 | 同上 |
| N3 | 核出对不上而隔离的槽，跨不跨重挂、进不进准入式子 | 同上；`.claude/kb/decisions/28-挂载期承诺量.md`「#### 已定项 1」第九项；C318 |
| N4 | 「用户数据的落点与政策函数不一致的次数」`policy_mismatches` 在第一版结构上恒 0：删掉、改口径、还是留着 | `.claude/kb/checks-owed.md` C515；`.claude/kb/verification-build.md` 事务层第一版那张表第 76 行 |
| N5 | 实例切换改成重新读盘择根之后：读盘选出的根比内存里记的新时，被重发的在飞 checkpoint 还重不重发 | `.claude/kb/checks-owed.md` C458；`.claude/kb/decisions/23-journal的角色与格式.md`「#### 已定项 14」实例切换那一句 |
| N6 | 同一情形下，切换写的实例表行与 W 按哪个根写（读盘选出的，还是内存里记的） | 同上；`.claude/kb/decisions/18-块里携带什么信息.md`「#### 已定项 11」 |
| N7 | 回退那次发布里，读盘选出的根与管理员选的 R_old 怎么对上 | 同上；D23 已定项 14「显式例外：管理员回退」 |

## 二、实现今天的样子（主 agent 的观测，2026-09-24 现查）

- **N1–N3**：`placements_to_release_via_mapping` 今天只核「映射里查得到 key、分配记录在册、没释放过、跨度对得上」，不读盘核校验和；`rebuild_version` 读映射条目只取 key、丢掉位置项与校验和。「核 + 隔离」的实现另有一个实现员正在副本里写（收口表第 13 行），读盘失败那一支它会停在一个点名「条款没定」的错误成员上。隔离位今天只有一个来源：挂载时 `crates/singlefs-core/src/mount.rs` 的 `isolate_slots_referenced_only_by_abandoned_roots` 按根环现算（影子账），经 `crates/singlefs-core/src/allocator.rs` 的 `PoolAllocator::isolate_abandoned` → `DeviceFreeMap::isolate` 置位，内存里、不落盘；分配记录条目只有「已分配」与「已释放 + 释放代」两个态（D3（空间分配） 已定项 7）。
- **N4**：字段 `crates/singlefs-core/src/allocator.rs` 第 629 行 `policy_mismatches`，构造处置 0（第 655 行），没有任何地方加它；读它的：`allocator.rs` 第 1157 行的断言、`crates/singlefs-harness/src/scenario.rs` 第 61、133 行、真设备二进制 `first_transaction_on_device.rs` 第 780 行打进结果行、三份用例断言它为 0。用户 2026-09-23 定盘不等大落点第一版维持拒绝（D3（空间分配） 已定项 8）。
- **N5–N7**：`crates/` 里没有挂载内的实例切换（`grep -rn 'instance_switch\|switch_instance\|InstanceSwitch' crates/*/src/` 只命中真设备二进制里的 `switch_instance_and_publish_third_version`，那是关闭再挂载，不是挂载内切换）。实现那一半已跟收口表第 16、40 行一起挪到后面的里程碑，这一轮只定条款。

方案按 `crates/` 今天的实现来谈。

## 三、分工（两条攻方腿的攻击面分开写）

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端攻方（Opus）** | 给每格造出候选各自会错的历史 | **N1、N2、N3** | 每格先列候选（至少：N1 当成对不上处置 / 发布失败交回 / 走 D23 已定项 14 的失败表；N2 同判据 / 各自判；N3 只在内存、落盘、重挂时现算），再对每个候选在 `crates/` 副本上造一条让它出错或不出错的历史（写序列 + 故障点 + 期望读数）并真跑；给不出的写卡在哪。另报每个候选的代价：多写几个字节、多读几次、改不改盘上格式 |
| **云端正推（Sonnet）** | 从已定分项推 | **N4、N5、N6、N7**，外加 N1–N3 各一句「已定分项里有没有推得出的」 | 每格去 kb 里找能推出答案的已定分项，整段抄原文（`research/scripts/quote-kb.py`），写从原文到答案的那一步；推不出的写缺哪一句、候选有哪几个 |
| **本地攻方** | 逐格填表 | **N5、N6、N7** | 事实表：行是「读盘选出的根与内存里记的根」的关系（相等 / 读盘的更新 / 读盘的更旧 / 内存里没有根）× 在飞 checkpoint 的状态（没有 / 有、未提交 / 有、已提交），列是 D23 已定项 14 与 D18 已定项 11 里每一句条文在那一格给出什么动作；给不出的填 undefined 并写是哪一句没罩到。不许只答 yes / no |

两条攻方腿不重叠：云端攻方拿 N1–N3（释放核验与隔离），本地攻方拿 N5–N7（切换与回退的条文覆盖）。

## 四、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「推得出」⇒ 照推出的写成条款，交用户确认 | 正推抄得出整段原文、推导不引用没有条款的取法，攻方在那一格造不出与推出的答案冲突的可达历史 |
| 一格判「岔路」⇒ 写进岔路单 `research/prompts/m2-newq-r1-forks.md`，缺代价数的登记计数实验 | 正推推不出，或攻方给出两个候选在同一段历史上分得开 |
| N4 判「删」⇒ 同时列出删掉要同步改的每一处（能被一条命令核） | 正推与攻方都说不出一段它能大于 0 的第一版历史 |

## 五、各条腿交什么

- 报告先写进 `research/prompts/m2-newq-r1-<腿名>-output.md`，**每次工具调用写进文件的内容不超过 150 行**，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `/tmp/claude-1000/<腿名>/`；攻方的模型与驱动脚本随报告拷进 `research/prompts/m2-newq-r1-opus-model/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数。
- 引产物整行抄，带文件名。
