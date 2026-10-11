# 删除（新事务）的设计：第二轮判决（2026-10-01）

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 V6 V7 S1 R3 L4 X1 F1 T1 M1 M2 M3 Z1 Z2 Z3 Z4 Z5 Z6 E-b E-c E-d -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 | `research/prompts/_m3-delete-r2-body.md`、`research/prompts/_m3-delete-r2-background.md`（2648 行） | — |
| 事实表 | `research/prompts/m3-delete-r2-facts.md`（sha256 d30c6837…d775） | — |
| 开工快照 | `research/prompts/m3-delete-r2-snapshot/`：`crates-sha256.txt`（210 份，冻结副本 `/tmp/claude-1000/m3-delete-r2/tree/`）、`kb-sha256.txt` | 腿交齐之后冻结副本 `sha256sum -c` 全 OK；kb 清单在主仓全 OK |
| 云端辩方（Sonnet） | `research/prompts/m3-delete-r2-sonnet-output.md`（sha256 293e8e1c…7b7e）；模型 `research/prompts/m3-delete-r2-sonnet-model/` | 第四节复跑 |
| 云端攻方（Opus） | `research/prompts/m3-delete-r2-opus-output.md`（sha256 3efaa983…c15c）；模型 `research/prompts/m3-delete-r2-opus-model/` | 第四节复跑 |
| 引文 | 两份腿报告 | `python3 scripts/agents/cite-check.py <两份报告> --root /tmp/claude-1000/m3-delete-r2/tree --root . --background research/prompts/_m3-delete-r2-background.md`：「✓ 核了 108 处引文，对上 108 处，没判 0 处」 |

两条腿的数都是副本原型上量的（原型没有意图记录、`deleted_inodes`、墓碑、nlink 字段、续做挂点）；进 kb 之前在入库装置上重做。攻方的 Z1–Z6 与辩方的收严都被攻过零轮。

## 二、跑前写死的判据逐条

| # | 判据（正文第五节） | 打中了什么 | 判 |
|---|---|---|---|
| 1 | S1 的某个中间态不合法 ⇒ 那一格的改法不收 | 攻方量到删除中（还剩 4 单元）跑 `publish_new_inodes` 会重写 inode 1 所在的叶，不带 nlink 的写者把它写回 1（`run-final-e-lines.txt` 的 `E name=v6 op=publish_new_inodes_1 rewrites_inode_leaf=true raw_nlink_after=1 units_after=4`）；删除中顺序写照收（`E name=v6 op=sequential_write_mid_delete result=Ok(units=2) raw_nlink_after=1`）。辩方、攻方都推「有树无文件」若借今天的 `WithoutFile` 会撞会话、回退入口、卸载、挂载前判四处 | S1 第 4 条只写「加 nlink」不够：`parse` 要读它（`crates/singlefs-core/src/records.rs:85` 今天跳过）、`to_bytes` 写结构里的值，现造记录的入口（`inode_record_of_file_version`）之前拒「nlink = 0」与「没有这条记录」两种。S1 第 5 条改成「带树、文件集合为空」的第三种版本，不借 `WithoutFile` |
| 2 | V1：非幂等的续做三条判据都过 ⇒ 判据收严 | 攻方量到「待删占用减两次」「不减」两种非幂等续做三条都过，中间版本标签 4 与镜像现算不符 3 行，checker 0 红（`E name=v1_criteria rule=SubtractedTwice … rows_not_equal_to_the_recompute=3`）；辩方独立推出同一种（报告 4.2 节） | **收严**：加第 ④ 条「每一个被判的状态（不止终态），记账标签 4、6 等于从这一版镜像重算的值」，由 checker 判；前提是 4、6 的值是镜像上算得出的函数（第三节 V2）。另加辩方的 ①b「同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同」（辩方量了一个样本：`run-final-d-lines.txt` 的 `D name=redo_from_the_same_image images_byte_equal=true … instance_first=2 instance_second=2 txg_first=8 txg_second=8`）：第一轮判决第二节第 2 行把字节判据整个判成写错，只对「原进程那一次 vs 重挂载那一次」成立，同镜像重挂载两次这一读法判据没写错 |
| 3 | V2：已预付的池上删除发布被准入拒，或同一批字节扣两次 ⇒ 式子不收；I-3.4 不能只从镜像重算 ⇒ 记一格 | 辩方算：「按发布前读数判」的朴素实现在写后可用为 0 的池上四次删除发布全拒；按「发布之后的标签」抵扣全放行（报告 3.4 节紧池表，算的）。攻方量到 W 不是写入那一刻算得出的上界（同一个 145 单元文件，写完做不同的事再删，合计 43 → 49 槽；`E name=w_state` 八行），K 是测试旋钮时按产品 K 预付不够（145 单元预付 43、实走 2116；`E name=batch_knob`），漏了 `publish_new_inodes`（满池上空 inode 的意图被拒；`E name=empty_inode_delete_on_a_full_pool … SpaceAdmissionRefused`） | R3 按文件逐个预付那一读法：朴素实现不收；带抵扣的式子收得进，但 P 要取「单元数 + 几何常量」的纯函数上界、K 要么进盘要么只按产品 K、建 inode 也要预付——代价见第五节，交用户 |
| 4 | V3：含未完成删除的崩溃态，可写挂载之后不续做或因续做失败 ⇒ 挂点不收 | 攻方：4 GiB 盘、满池、账里一字节不给删除时 100 / 200 单元每一批都做成（`E name=ring_window … result=Ok` 三行）；M3（挂载里不走、会话第一件事走）在崩溃重放的挂载交回那一刻一批都没走，C1（意图续做做成常规路径） 字面「正常挂载后没跑续做即判红」红（推的） | M3 不收。M1（挂载里走到空）在 4 GiB 盘上没打中；小盘长环那一格没量（攻方推 1 GiB 盘保留量约 400 槽、每区 16 槽时一圈约 720 槽，那时会撞落点那一判） |
| 5 | V4：三种回退之后 I-4.7、I-9.5、I-3.4 不成立，或续做把带回来的文件再截一次 ⇒ 改法不收 | 三种情形按 S1 第 8 条推都成立（攻方 3.2 节表）。攻方量到 R3 下回退到更大的文件版本时已承诺预留按 R_old 抬上去、回退那次发布不判准入，之后每盘可用 −1 212 416 字节（`E name=rollback_raise … available_after=[-1212416, -1212416]`）——不满足 I-3.4 字面（它只要求扣了），破的是 R3 自己的承诺 | 改法收；R3 的这一格随 K-U4 重定 |

## 三、逐格判决

| 格 | 判 | 依据 | 轮数 |
|---|---|---|---|
| V1 | 判据定成五条：① 计划相等、①b 同镜像重挂载续做逐字节相同、② 删完再续做一个写都不发、③ 每个写点截断重开续做到底终态逻辑内容相同且 checker 全绿、④ 每个被判状态标签 4、6 等于镜像现算。「逻辑内容」照辩方 4.4 节表：含 inode 每条（号、对象出生代、size、nlink）、extent 单元序号集与内容校验和、`deleted_inodes` key 集、墓碑记录多重集、标签 12 / 4 / 6、读者的答案；不含实例号、txg、改动计数、落点、单元头的诞生代号与写序、逐盘标签 1 / 2 / 5 / 10 / 11 | 第二节第 2 行 | ④ 一轮（两腿独立提出、攻方量过）；①b 零轮（一个样本） |
| V2 | **交用户重定 K-U4**：第一轮判决给用户的 R3 代价「约为数据的 4.5%」够不着（两腿一致：13 槽只是 K = 2、6 单元文件一个点）。重量的数见第五节 | 辩方 3.1、3.2 节；攻方 2.1–2.6 节 | — |
| V2 不论选哪支都要的 | 标签 4、6 做成版本的函数（每次发布从这一版重算，不当计数器增减）；删除类发布按「发布之后的标签」抵扣，不按发布前读数判；W 连固定点一起算（每批固定点 8–15 槽，攻方 `E name=w_row` 行）；写明式子那一判与落点那一判各关不关（落点那一判要「普通 + ckpt_cost」约 541 槽，攻方 2.6 节）；墓碑单元算进同一份 | 辩方 3.3 节；攻方 2.6、2.7 节 | 一轮 |
| V3 | 挂点取 M1：`mount_writable` 在写行与暖机之后、交回之前把队列走到空；不进取号前预演（预演之后那道「预演落点 = 真发落点」的断言只比写行与暖机一串）；会话里的队列每次从现行那一版的 `deleted_inodes` 读，不缓存。崩溃注入加 `WritableMountPhase::Continuation`。小盘长环那一格登记实验 | 攻方第四节；辩方没攻 | M1 一轮（攻方没打中）；其余零轮 |
| V4 | `deleted_inodes` 随 R_old；标签 4、6 从 R_old 那一版重算（做成版本的函数之后免费，辩方 3.6 节）；回退那次发布要写墓碑退役时它有了普通分配、要判准入（攻方 3.3 节，归 m3-tomb-r1 的 K-W5） | 攻方 3.2、3.3 节 | 一轮 |
| V5 | 「有树无文件」是 `RebuiltVersion` / `PreviousVersion` / `PoolVersion` 的第三个成员，不借 `WithoutFile`（两腿一致）。inode 树删空取零条目根（照 D8（核心索引结构） 已定项 6），根的 key 区间取常量 `[0, u64::MAX]`；extent 树删空取零条目根，key 区间按位置规定（`[(0,0,0),(0,142,u64::MAX)]`，`crates/singlefs-core/src/extent_tree.rs:120` 的位置规则）；`deleted_inodes` 空队列取根指针置零（E-b：空队列在盘上只有一种样子、最后一批少 1 个单元） | 辩方 5.1–5.3 节；攻方第七节 | 第三个成员一轮；三棵树的空形态零轮 |
| V6 | 见第二节第 1 行判 | 攻方第五节（量过） | 一轮 |
| V7 | `deleted_inodes` 派生态、落 D15（格式冻结政策） 已定项 7 第 3 / 4 层，条件是 D8 新分项写一句「队列 = inode 树里 nlink = 0 的号集合」（辩方 6.1 节）；144 是自由选取，正文不写成「由 W 定」；「只有一条路」与入口名带批号站得住 | 辩方第 6 节 | 一轮 |
| V7 F1 状态数 | 第一轮判决与正推腿的「约 2 936 万」没有墓碑单元。辩方按「墓碑 1 个单元」重算为 41 943 084（E-c）/ 38 797 356（E-b）；m3-tomb-r1 判的墓碑树让意图那次多 2 个单元（容器 + 根），还要再算。数在流上量过才钉（辩方的单元数是单文件池上量的） | 辩方 6.4 节 | 数随 K-W5 与墓碑定案重算 |
| 第一轮出局的 | S-b「size 不跟着截」、待删占用口径 A / A2、惰性建树：辩方复核三项都站得住 | 辩方 1.1、1.4、1.6 节 | 两轮 |
| 第一轮行号 | 第一轮判决与腿报告引的 `recovery.rs` 行号是第一轮副本的，第二轮副本里漂了（辩方 1.5 节：3365 → 3298、2452 → 2453、2505 → 2506）；实现按第二轮副本 | — | — |

## 四、复跑

- 攻方：`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 … run-with-memory-cap.sh 4G … capped.sh 8 bash research/prompts/m3-delete-r2-opus-model/rerun.sh /tmp/claude-1000/m3-delete-r2/tree <草稿>`：「test result: ok. 8 passed; 0 failed」；`grep -o 'E name=.*' | sort` 254 行与 `run-final-e-lines.txt` 排序后逐字相同。第二、三节引的 `E name=` 行都在里面。
- 辩方：`… run-with-memory-cap.sh 2G … capped.sh 8 bash research/prompts/m3-delete-r2-sonnet-model/rerun.sh <仓根> /tmp/claude-1000/m3-delete-r2/tree <草稿>`：「test result: ok. 4 passed; 0 failed」；`D name=` 17 行与 `run-final-d-lines.txt` 排序后逐字相同；`python3 cells.py` 输出与 `cells.out` 逐字相同。
- 两份副本（722M、704M）已删。

## 五、交用户的与下一步

K-U4 带新数重新交用户（岔路单 `research/prompts/m3-delete-r1-forks.md` 那一行改回「开着」，补一条候选）：

| 候选 | 预付的量 | 满池删得动吗 | 量过还是推的 |
|---|---|---|---|
| **R3 逐文件**（用户 2026-10-01 选的）：写文件时把这个文件将来删除各批的最坏新写量记进已承诺预留 | 3 单元文件 50 槽 / 盘对数据 6 槽（8.3 倍）；300 单元连续落点约 10%；落点散落时约 98%；要 checker 从镜像重算就得取纯函数上界，再大 5 到 20 倍；size 0 的 inode 每个约 22 槽 / 盘（4 GiB 盘只够 9 634 个） | 能，靠预付 | 前三个数量过（副本），上界与空 inode 是算的 / 代用 |
| **全池一份环窗口**（攻方 Z2）：已承诺预留拆成「每个对象一份墓碑」+「全池一份（根环里的根数 + 1）× 每批上界」，不随文件个数与大小长 | 4 GiB 盘、根环 24 条：删 100 / 200 单元时实际借保留量的顶点 258 / 304 槽 / 盘（约 4–5 MiB），根环 48 条时 712 槽（约 11 MiB）；同样的文件 R3 要 1515 / 3817 槽 | 4 GiB 盘上能（量过：账里一字节不给删除也删完）；小盘长环没量 | 环窗口那一半量过，拆法零轮 |

下一步：K-U4、K-W5、K-W5b 三条交用户；定了之后开第三轮（m3-delete-r3 与 m3-tomb-r2 可以合成一轮），只攻前两轮站住的形态加用户新选的两支，派辩方复核这两份判决。第二节第 4 行小盘长环那一格登记一个计数实验（借保留量的顶点随盘大小与环长怎么走）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 5 | 备料 | 2026-10-01 不受影响：幂等判据收严成五条，第三轮之前不写回 |
| D8（核心索引结构） 已定项 6 | 备料 | 2026-10-01 不受影响：零条目根照它；`deleted_inodes` 取根指针置零是零轮 |
| D3（空间分配） 已定项 2 | 备料 | 2026-10-01 不受影响：R3 的代价数重量，交用户 |
| D3（空间分配） 已定项 15 | 备料 | 2026-10-01 不受影响：「量要小」被辩方用来判纯函数上界的代价 |
| D3（空间分配） 已定项 17 | 备料 | 2026-10-01 不受影响：两个候选都满足「删除不申请空间」，差在预付量 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-01 不受影响：抵扣式子与标签 4、6 的取法零轮 |
| D15（格式冻结政策） 已定项 7 | 备料 | 2026-10-01 不受影响：`deleted_inodes` 派生态带一句条件 |
| D21（权威态与派生态的分界） | 备料 | 2026-10-01 不受影响：同上 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-01 不受影响：回退取 R_old 的树加 `deleted_inodes`，标签 4、6 改成从 R_old 重算 |
| D13（验证路线） 已定项 4 | 备料 | 2026-10-01 不受影响：F1 状态数随墓碑定案重算 |
