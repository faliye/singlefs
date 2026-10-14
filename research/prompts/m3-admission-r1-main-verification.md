# 准入条款簇：第一轮判决（2026-10-12）

<!-- doc-lint:not-numbers A1 A2 A3 A4 A5 A6 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 W1 W2 W3 W4 R1 R2 R3 K1 K2 K3 K4 S1 S2 S3 P1 P2 Q1 Q2 G1 G2 G3 L1 L2 L3 -->

## 一、这一轮的材料与证据

- 正文 `research/prompts/_m3-admission-r1-body.md`（格 A1–A6），背景材料 `research/prompts/_m3-admission-r1-background.md`（2026-10-12 按当天 kb 重拼：2026-10-11 那一版的快照 7 份 kb 全部对不上，腿还没派，旧材料挪进会话草稿，`three-way-materials.py` 重拼，清单 107 抄、11 不抄）。
- 冻结副本 `~/.local/share/singlefs/work/m3-admission-r1/repo/`：2026-10-12 UTC 11:12 被别的会话通删，主 agent 照 `research/prompts/m3-admission-r1-snapshot/frozen-copy-sha256.txt` 重建（admission、allocator、make_filesystem 取 HEAD，transaction、mount 取暂存区，unequal_devices 三处相同，`crates/mutations.tsv` 取悬空对象 b28a4c26），点名 7 份 sha256 全过；点名之外的文件是 2026-10-12 暂存区那一版。删除之前派出的两腿已停、草稿已删，现两份报告出自重派的两腿。
- 正推腿（Sonnet）`research/prompts/m3-admission-r1-sonnet-output.md`（sha256 dfad6ed8c57f478e9968d7d8f42f1d7270ec28346ea94671cc80544a646bf714），模型 `research/prompts/m3-admission-r1-sonnet-model/`。
- 攻方腿（Opus）`research/prompts/m3-admission-r1-opus-output.md`（sha256 778db65265cc472026173a788b43b682dbd96a122902812f495b426c04abf870），模型 `research/prompts/m3-admission-r1-opus-model/`。
- 开工快照核对：腿跑着时 `.claude/kb/milestone/03-third-txn.md`（工作区多别的会话插的待优化第 37 行，暂存区那一版即快照）与 `.claude/kb/checks-owed.md`（UTC 12:27 被别的会话改过并暂存；快照那一版取悬空对象 a2b556ac553b230e4fe5e4a1bc82fc710a908142，sha256 e201956f…dde7 与快照相同）对不上。主 agent 拼「快照时的 kb + 冻结副本 crates」核对树（会话草稿 `adm-snap/`，7 份 kb sha256 全过）。
- 引文核对：`python3 scripts/agents/cite-check.py <两份报告> --root <核对树> --background research/prompts/_m3-admission-r1-background.md` 末行「✓ 核了 61 处引文，对上 61 处，没判 0 处」。
- 腿自报的偏差：攻方一次 `bash -x` 起 rerun 没经内存包装（输出作废、没进 `out/`）；一次误写会话共用暂存目录（当场删）；正推一次前台 timeout 设 480000。都不影响入库的 `out/`。

## 二、共用前提：三道闸与 `df` 两份定义

两腿读码一致：发布路径上是「落得下」（`admit_the_units_landing_on_every_device`）→ 走固定点（拷贝上，取不到落点报 `PlacementRefused`）→ D28 式子（`admit_on_every_device`）三道，都在任何写之前；会话把 `SpaceAdmissionRefused` 与 `PlacementRefused` 一律当空间不够推抬 F（`crates/singlefs-core/src/mounted_session.rs` 那一处 `=> true`）。第一道（用户 2026-09-27 定「准入先拒」）在 D28 / D3 条款正文里没有，只在代码文档与已还清的 C545 行里。

`df` 在 kb 里有两份不相等的定义：D28 已定项 3「df 报可用」（D28 式子），D16 已定项 1 的 `df` 行（不扣环窗口、切换预留、已承诺预留，加活元数据）。`crates/` 里两份都没实现（两腿各自 grep `statfs|fn df_|df_available` 零命中）。

## 三、逐格判决

| 格 | 判定 | 依据（核过的） | 去向 |
|---|---|---|---|
| A1 需求的单位（C370） | 实现已取、条款还缺一句 | 两腿一致：`admission.rs` 需求按盘字节（模块文档自陈「条款没写」）；正推乘盘数变异使 admission 21 条单测 1 条转红（`out/demand-times-devices.txt`） | 第一版全镜像两种读法每盘相等，交用户定写哪一句（F1）；那条变异做成会红用例并进下一个代码批 |
| A1 `df` 的写法（C370、C371） | 还缺，打中 | 攻方在 73 个状态上二分「写得进的最大字节」对 7 种写法：D16 字面 73/73 多报；各盘可用取小 52/73、再按单元换载荷 46/73 多报；不等大池上之和 ÷ 副本数 31/31 多报；攻方自提 H 0/73 多报（被攻过零轮）。正推的「逐设备取小不多报」只按几何算了容量那一项，没算一次写自己的非数据需求，被攻方的数推翻（多出的那一截就是 `truth_write` 的普通分配非数据槽与固定点） | 交用户：`df` 跟哪一条式子（W1–W4），活元数据随它（F2–F4 合并成这一条）；H 标「零轮」 |
| A2 第九项（C356） | 符号与口径已被吃掉；C356 给的检查写不成等式 | 两腿一致：`isolated_slots()` 是 u64 计数，D28 已定项 1 第九项取实际隔离集合；攻方在扣住位还在的状态上量到「可用 − 分配器可发」差 10117 槽，正推量到隔离已分配槽差 −2（C762 双扣）。差出来的都不是第九项的口径，按「打中不分辨臂」不拿来判第九项 | 交用户 C356 行去向（F5：还清 / 改写检查把扣住与重叠各列一项 / 留着） |
| A3 扣住的槽（C570、C546） | 还缺，打中 | 攻方造出整条历史（宽池，抬 F 第 2 或第 3 次空发布写错、原样重发）：四格里三格 F 已在两块盘生效、扣住位 10117 槽没放，同一个写式子可用 13807 ≥ 需求、落得下按段外槽对拒，推抬 F 停在 `FloorAlreadyAtTheCeiling`、交回空间不够；只放开扣住位同一个写做成；先做一次小写可脱出（不是吸收态）。正推量到机制两段（扣住期读数多算 4 槽；写错后残留 15 槽），整条没造，与攻方相容 | 交用户（F6+F7 合并）：K2 生效即放 / K3 推满时先把现行 F 带到缺它的盘 / K1 式子加扣住项——K1 在打中的四格上不起作用（攻方推的，写明）；K2、K3 都是攻方自提、零轮 |
| A4 段内关着的空槽（C571） | 条款还缺；C571 前置已满足 | 两腿一致：读数今天算得出（段外槽对 `unblocked_slot_pairs_outside_the_cluster_segments`）；攻方在 900 轮取样里关着的空槽最多 928 槽，没超过常驻扣项 2491 槽，造不出假性 ENOSPC；正推在分配器层把段外槽全标已分配造出「式子放行、落得下拒」，那是合成盘面，可达性没核 | C571 前置列改「无」；交用户 F8（报 / 不报 / 另报一项）；「没打中」只抽一次样，不支撑「不会出」 |
| A5 C573 固定点上界 | 第一半已被吃掉；第二半还缺，打中 | 攻方量到一次写 4000–5400 单元时真写固定点比 ckpt_cost 多 1–16 槽，n = 5385–5393 九格落得下放行、走固定点 `PlacementRefused(MappingTree…)`；三种拒都在写之前 | 交用户 F9（P1 先拒用真上界 / P2 条款写明留给落点） |
| A5 C574 两槽对齐、各盘同槽 | 两槽对齐还缺（数据单元那一半已判）；各盘同槽没打中 | 正推在分配器与整条发布路径上量到：落得下放行两槽容器、`PlacementRefused` 落在 inode 叶容器（`out/a3-a2-a4-a5-probe.txt`、`out/c576-switch-off-probe.txt`）；攻方指出 C574 行「落得下不判它们」对数据单元那一半不成立（`admission.rs` 数偶数起点槽对）；等大池上读盘核坏一份，两盘记录一起留在已分配，60 次写没分叉 | C574 行改写两半；交用户 F10、F11 |
| A5 C576 | 部分 | 两腿一致：第一条用例已改钉 `SpaceAdmissionRefused`，发布路径走不到「各盘去处不同」那个成员；主 agent 现查冻结副本 `crates/mutations.tsv`：第 76、77、78、80、145 行与准入相关（正推施变异后五行都红），第 149 行是「判崩溃点时取择根…」与准入无关（攻方说的行号漂移属实）；第 12 行「失败的发布不退回分配器」三种史下不红（正推），靶子失效或等价没判完 | C576 行改成按变异名登记；第 12 行交下一个代码批的变异分诊（F12） |
| A6 C568 io_min 1 MiB | 登记行「接受」那一臂按写的两步做不到 | 两腿各自量到：io_min 1 MiB 时槽距 1 MiB，系统配置槽 1 与根环区域 0 都从 1 MiB 起，S = 4/6/8/16 盘 0 上真重叠；「只比同盘 + 改清零」只解开 256–512 KiB 里 S 大的格 | 交用户 F13（G1 不接受写进射程 / G2 按盘比 + 改清零 / G3 再挪系统配置槽 1，动格式） |
| A6 C582 挂载一侧下界 | 部分 | 正推（只 mkfs 的池）：[U, U+3) 被记录几何检查或单元读不出拒，[U+3, U+24) 被准入式拒，≥ U+24 可写；攻方（带文件的池）：起作用的是「现行账里每条记录都在盘内」（第一个文件之后 U+75），40 次可写挂载都在写之前拒、没有 panic，错因五花八门 | 交用户 F14（L1 写今天那一判 / L2 取 mkfs 下界 / L3 择根后判「盘装得下账」、专门错因） |

## 四、复跑

两腿的模型目录拷进会话草稿（`rr-sonnet/`、`rr-opus/`），各经 `bash scripts/hosts/run-with-memory-cap.sh 16G bash <拷贝>/tools/rerun.sh --repo <仓根> --frozen <重建的冻结副本> --work <新目录> --model <拷贝>` 并行跑，都退 0：

- 攻方 17 步全部 `name=rerun_step … same_as_model_out=yes`（`held_small`、`held_wide`、`df_small_equal`、`df_small_unequal`、`df_wide_big`、`df_wide_long`、`fp_small`、`fp_wide`、`fp_wide_single` 五档、`fp_which` 两份、`geometry`、`qd`）；`tools/summarize_df.py` 对复跑输出与对 `out/` 的汇总 sha256 都是 7b1f6f48197a7e82e88626a031ece01497a5346f49b44d72eef5079721fdc087。
- 正推 21 份 `name=rerun_compare … same=yes`（探针、量表、乘盘数变异、`geom.txt`、C576 七行变异的基线与施变异后）。

这些数是冻结副本拷贝上的探针数（内存稀疏盘），没进入库装置；判决引它们只用来判打没打中。要写进 kb 的数（`df` 各写法的多报、扣住位的假性 ENOSPC、固定点超出），定案之后照 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」做成入库装置的一次跑。

## 五、交用户的与下一步

岔路单 `research/prompts/m3-admission-r1-forks.md`。这一轮不写回决策正文；定案之后照判决写回 D28、D16、D3、D2 与 `checks-owed.md` 那十一行，改 `crates/` 的进代码批。攻方自提的 H、K2、K3 被攻过零轮，交用户时写明。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-12 不受影响：需求单位一句与扣住项交用户定（F1、F6），定案之后写回 |
| D28（挂载期承诺量） 已定项 3 | 备料 | 2026-10-12 不受影响：「df 报可用」那一读被量到多报，交用户定 `df` 跟哪条式子（F2） |
| D28（挂载期承诺量） 已定项 4 | 备料 | 2026-10-12 不受影响：C573、C574 两格交用户（F9–F11） |
| D16（发布语义） 已定项 1 | 备料 | 2026-10-12 不受影响：`df` 行、活元数据、扣住位怎么放交用户（F2、F4、F7） |
| D3（空间分配） 已定项 8 | 备料 | 2026-10-12 不受影响：段内关着的空槽报不报交用户（F8） |
| D3（空间分配） 已定项 9 | 备料 | 2026-10-12 不受影响：第 1 条是这一轮的判据，条款本身不动 |
| D2（RAID 条带策略） 已定项 19 | 备料 | 2026-10-12 不受影响：io_min 1 MiB 接不接受交用户（F13） |

## 事后补记（2026-10-13）

- 开工快照 `research/prompts/m3-admission-r1-snapshot/frozen-copy-sha256.txt` 原来没有 `crates/singlefs-core/src/mounted_session.rs` 一行，而判决点名了它；2026-10-13 由走提交的主 agent 事后补上，哈希取腿读的冻结副本那一版（`~/.local/share/singlefs/work/m3-admission-r1/repo/`，与 2026-10-12 暂存区那一版逐字节相同；与提交 f8a9ddaa 那一版只差 6 行注释、行号不变，两条腿报告都写明读的是暂存区那一版）的 `sha256sum`，由 singlefs-13 2026-10-13 核出纠正（门禁 doc-process-records 的 code-round-snapshot 格 2026-10-13 报红，`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」）。
