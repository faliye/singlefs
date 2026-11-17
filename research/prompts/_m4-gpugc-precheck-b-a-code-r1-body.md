# m4-gpugc-precheck-b-a-code-r1：里程碑四七节 GPU 垃圾回收，抬 F 先算改法乙、甲，代码轮第一轮正文（2026-11-12）

<!-- doc-lint:not-numbers T1 T2 T3 -->

## 一、题面

用户定这一轮开改法乙、甲（记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 1002 行起的第 125 批，A/B 读数在第 1019 行，任务 13802）：

- **改法乙**：抬 F 之前先算的槽对那一判，上界不再按「每个放得开的槽各补一对」算，改成段外两槽都不罩在未释放分配记录下的槽对数（defer、隔离、抬 F 扣住三样全放开之后成的对，现有的在里面），由分配器随每次改未释放记录按段增量维护。
- **改法甲**：这次调用还一串没推、被拒是空间不够、被拒的是合取 7 判的前台写入时，写之前那一刻全空聚簇段已少于 N₀，也一串不推。先判乙与式子，再判甲。

两份补丁已打进主工作区，合入后验证 13896 测试全绿（现判 217、复用 227），clippy 两种特性组合重跑 13911、13912 退 0。

改动全文：`research/prompts/_m4-gpugc-precheck-b-a-code-r1-diff.md`。被判的产品代码两份：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/mounted_session.rs`；真池用例 `crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs` 是测试代码，给腿读、不被判。

## 二、实现今天的样子（主 agent 的观测，2026-11-12）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1/repo/`；被判文件 sha256 在 `research/prompts/m4-gpugc-precheck-b-a-code-r1-snapshot/crates-sha256.txt`，冻结副本 `sha256sum -c` 两份 OK。
- 乙（冻结副本行号）：`allocator.rs:783` `slot_pairs_without_an_unreleased_record_outside_the_cluster_segments` 是先算读的数；`allocator.rs:873` `note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts` 在每改一槽的未释放计数时同步改逐槽位与按段计数；`allocator.rs:500` `pairs_inside_the_cluster_segments` 是两个段外计数共用的「减去聚簇段里的槽对」；`mounted_session.rs:6074` `most_a_floor_raise_could_make_available` 槽对那一臂的上界改读它，不再另加此刻可用。
- 甲：`mounted_session.rs:6153` `of_the_user_change` 分前台写入（写文件内容、建 inode、并进一次发布的数据写类）与免或发之前不知道（unlink、删快照、改 size、建快照）；`mounted_session.rs:6175` `empty_cluster_segments_below_the_floor_before_any_series` 四样都成立才判（还一串没推、空间不够、前台写入、准入开着），调合取 7 自己用的 `admit_the_empty_cluster_segments_floor_on_every_device`；`mounted_session.rs:6222` `judge_before_pushing_a_floor_raise_for_a_refusal` 先判乙再判甲；`mounted_session.rs:6292` `push_one_floor_raise_for_a_refusal_for_space` 开头调它。调用处三处：`publish_user_change`、`accept`（传被拒那一批的改动）、走空（传免）。
- 拒绝成员 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 加了公开字段 `empty_cluster_segments_below_the_floor`（`mounted_session.rs:695`），与 `devices_short` 二选一有值，类型上没挡住两样同时空或同时有。
- 新用例：`singlefs-core` 库单测 13 条（`mounted_session::everything_a_floor_raise_could_release_tests` 11 条，`allocator::tests` 2 条用定种子的 600 步伪随机历史比增量计数与按记录扫、与真放开之后的数）；验证包两条真池用例，只经 `accept` 一处。变异行追加 15、删 8，证红 13840「跑了 13 条，抓到 13 条」、13891「跑了 2 条，抓到 2 条」，主工作区复判 13882「抓到 13 条、没红 0 条、无效 0 条、锚点失效 0 行」。
- A/B（主 agent 复跑，记录第 1019 行，任务 13802，372 个配置）：现行 `c_white=19444`；乙 `c_white=3151 precheck_saved_a_vain_series=17813 published_eaten_vs_as_coded=0 outlet_fewer_than_as_coded=4801`；甲 `fired_by_conjunct7=315 outlet_fewer_than_as_coded=115`；出路跟法乙 `stuck_followed=253`、`lost_first_followed=295`、`lost_after_followed=45` 写进 0。新造「推了写得进」那一族一律判放不下那一臂误拒 5465，全是式子那一判先拒，乙在槽对那一判上误不误拒没量到。
- 崩溃点重放带 `--item` 三项：不缩域 13907 退 79（两项态数算不出，不放行），缩到展开深度 4 重交 13913（写正文时在跑，结局见判决）。
- 压着的条款：`.claude/kb/decisions/16-发布语义.md` D16（发布语义） 已定项 10「射程」那一段（第 276 行）只写「短在段外槽对那一判同样先算，上界恰好等于需求照推」，没写上界怎么算、没写甲；欠账 `.claude/kb/checks-owed.md` C1002（抬 F 先算对槽对那一判挡得少）、C1003（被宽护卡住那条出路没量过）。实现员建议的射程新文在它报告「要改的条款」一节（`~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-writer/result.md`）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　乙的上界对不对：不许算小、该算多少
增量维护的那几个数在每一种改未释放记录的路上（分配、释放、复活、回收、隔离与清隔离、抬 F 扣住与放开、开段与 bump 提交内生块、管理员回退）是不是都跟着改，有没有一条路绕开 `note_slots_under_an_unreleased_record`；「分配位置着而不罩在未释放记录下就是 defer 里的」这条读法在分配器里成不成立；扣住的槽按已释放记录算放得开对不对；上界与探针 `bound_by_real_slot_pairs` 的口径差（实现员报告「改法乙」末段「两边差的只有……」那一句）在真分配器上造不造得出。乙少报出路 4801 格（推一串后报被宽护卡住、乙先算就拒）：读码判这 4801 格丢的是不是一条走得通的出路——A/B 里现行与乙跟着出路卸载重挂都写进 0。乙误拒：槽对那一判上有没有「推了写得进、乙判放不下」的形，推串时抬 F 换下、同一次准入里后一串又回收得到的固定点单元（实现员报告「停下交主 agent 的」第 5 条）算不算进上界。

### T2　甲的射程与次序
甲的射程比 A/B 探针宽一形：先走空发成了几步、没推串、然后用户那一次写被拒，也判（实现员报告「改法甲」末段）；「推串之前」读成「这次调用还一串没推」对不对得上 D16 已定项 10 射程原文。改 size 与建快照不先判、取不会误拒的那一边，有没有该判而漏的前台写入。先判乙后判甲的次序会不会让甲该报的被报成乙（报里的 `devices_short` 与 `empty_cluster_segments_below_the_floor` 二选一），调用方与测试按哪一个读。判的数是不是合取 7 在写那一刻读的同一个数：甲判时与真提交时之间有没有别的东西改全空段数。

### T3　覆盖与报
`publish_user_change` 与走空两处传哪一种没有真池用例（只有 `accept` 一处）；甲那条真池用例用 `EmptyClusterSegmentsFloor::ForcedForTests` 把 N₀ 设成每块盘全空段数最大值加一，不是产品的 N₀（主 agent 认了），会不会让用例罩不到产品 N₀ 下的形；`DeviceFreeMap` 每槽一位的新位图（内存随单元区槽数线性涨）与准入 O(聚簇段数) 读法有没有违 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。

### 检查单：走不到的理由二次查实
实现员报告「走不到的理由」原样：

> crates/singlefs-core/src/allocator.rs:512 聚簇段起点在段边界上：这条断言是从 `unblocked_slot_pairs_outside_the_cluster_segments` 原样搬进共用函数 `pairs_inside_the_cluster_segments` 的。`cluster_segments` 只在 `PoolAllocator` 开段那一处插入（`allocator.rs` 里 `self.cluster_segments.insert(segment_start)`），插的是各盘一致答出来的全空段起点 `lowest_empty_segment…`，按段号乘 64 加单元区起点算出、单元区起点在段边界上（`UnitAreaStart` 的不变量）；两个调用者 `unblocked_slot_pairs_outside_the_cluster_segments`（`PlacementRoomOnOneDevice::of_every_device_of_the_allocator`）与 `slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`（`SlotsAFloorRaiseCouldRelease::of_the_device`）都只传 `allocator.cluster_segments()`。

冻结副本里那条断言在 `allocator.rs:514`。条款没定的错误成员：这次没加。腿照调用点逐个追，另核乙的增量计数里有没有新的减法会下溢（实现员报告提到「总数不加」那条变异在 debug 下先红的是减法下溢）。

### 检查单：类型与名字
新加：字段 `is_under_an_unreleased_record`、`slot_pairs_without_an_unreleased_record_per_segment`、`slot_pairs_without_an_unreleased_record`；`SlotsAFloorRaiseCouldRelease` 的 `slot_pairs_once_everything_is_released`；枚举 `RefusedPublishUnderTheEmptyClusterSegmentsConjunct`（成员 `ForegroundWriteTheConjunctJudges`、`ExemptOrNotKnownBeforeThePublish`）；拒绝成员上的 `empty_cluster_segments_below_the_floor: Option<…>` 与 `devices_short` 二选一靠构造路守、类型没挡。腿核槽数与槽对数有没有共用裸整数、`of_the_user_change` 有没有 `_ =>`、新 `expect` 写没写依赖的不变量。

### 检查单：不兼容旧代码与旧格式
本轮改动没有这一类：没碰盘上字节、没加版本分支（查过 `_-diff.md` 两份补丁全文）。腿现查。

### 检查单：不新建判定库
本轮改动没有这一类（查过同一份全文：没有新目录、TSV、sqlite、RocksDB）。腿现查。

### 检查单：所有任务多机可用
新用例输入都在用例里现造（内存分配器、内存稀疏盘），不读本机状态、不写主机名（实现员两份报告「拒算的测试」一节）。腿现查。

### 检查单：测试优先多线程
`allocator::tests` 两条伪随机历史是单线程 600 步；真池两条是单场景。腿判该不该并行、不并行的写没写理由。

### 检查单：带 --item 的命令与项
实现员报告里与准入相关的崩溃点重放三项 `floor-raise-pushed-by-the-session`、`last-fitting-unit-then-refusal-and-unmount-on-a-nearly-full-pool`、`truncate-to-zero-on-a-nearly-full-pool`（列项任务 13885 列得出）；这一批没有新项。腿核命令写全没有。

### 检查单：新崩溃流多罩了什么
本轮改动没有新崩溃流。腿判乙、甲新拒的那几形（写之前就拒、盘上不变）要不要一条崩溃流钉「一串不推」。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T3 与检查单八格，T1 主攻 | 核「代码做的是不是用户定的乙、甲」：乙的增量计数每条路都跟上、上界与 A/B 口径一致；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1–T3 与检查单八格，T2、T3 主攻 | 找乙算小（误拒推了写得进的写）、乙丢出路、甲误拒或该判没判、报的成员读错的形；能造就在冻结副本的拷贝上造探针；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改；复跑脚本要能安全重跑（服务撞内存上限会重排同一条命令，草稿目录非空时先清再跑），探针的内存与时间写进报告；长跑的探针每个配置往标准错误打一行进度（服务看日志 10 分钟不涨判卡住）；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。cargo 命令都点名包。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
