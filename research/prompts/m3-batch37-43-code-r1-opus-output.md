# m3-batch37-43-code-r1 攻方（Opus）报告：K2 哪一格会错（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 Y1 Y2 Y3 Y4 Z1 A1 B1 C1 H1 H2 H3 H4 F1 F2 R P Pm M1 M2 -->

## 复跑与模型文件

复跑（串行，每个阶段前台 4 分钟内；本腿在空目录 `~/.local/share/singlefs/work/m3-batch37-43-code-r1-opus/rerun1/` 里按 `prep build` → `swow swsw3` → `sc` → `l0` → `f2 exist summary` 分五次从头跑通，合计约 10 分钟）：

```
SINGLEFS_REPO=~/.local/share/singlefs/work/m3-batch37-43-code-r1/tree SINGLEFS_TOOLS=<仓根> THREADS=15 \
  bash research/prompts/m3-batch37-43-code-r1-opus-model/rerun.sh <空工作目录> [阶段…]
bash research/prompts/m3-batch37-43-code-r1-opus-model/compare.sh <工作目录>/summary.txt
```

本腿那一次的比对原样：`COMPARE same lines=2324`。

| 文件（`research/prompts/m3-batch37-43-code-r1-opus-model/` 下） | sha256 |
|---|---|
| rerun.sh | 1f377fb5443e23db1b8347f5021647f681a0b44c706bbb29b20c707a329afadd |
| patch_arms.py | 043dd869be6f881855f64d9a3a771ddab40ce87b728db245a7ec421d9307bb27 |
| summarize.py | 4e3b1594116d6be746dc835cb9595edeecc1a09bf2f84ac865ac08f5ce2c1b05 |
| compare.sh | 60ede438a81056f39ab613a8b8b308fcdbe58374b045fe0f0bc7a124d8476542 |
| expected-summary.txt | 8dea33447e40cec02be2e42cbff7d90411201eb41d23f0ade04ee57e68f2721d |
| probe/Cargo.toml | 1c70b9f4c62e1d2e8fb2119609b4b4d8fbf3a33a7c0094beac1f4139d6ea64f2 |
| probe/src/lib.rs | 36464da36c0ca2fffae8ab60dfa014f6032c1d0823f3b8f189ae483fee17ae80 |
| probe/tests/opus_b3743_probe.rs | 5394b696f4d815482ca08fb4ce3da360fcd16ae82bdda8ad1266a592e37a02f9 |

装置：冻结副本原样拷进工作目录，`patch_arms.py` 只加四条靠环境变量打开的臂（不设 = 冻结副本原样），探针是另起的 crate `opus-probe`（依赖 harness 与 checker 档库，只用它的层 0 枚举函数，不跑 checker 档包的测试）。盘面照 `crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs`：384 槽小盘、第一个文件之后覆盖写 k 次、录一次发布 R（ow 覆盖写一次、一条记录；sw3 顺序写三个数据单元、三条记录），崩在 R 的根槽落盘之后、轮换之前；P 是 R 之前那一版（已确认），Pm 是 P 之前那一版。读故障是「读回全零」那一层（写到哪一段那一段就不再藏）。副本上的数都不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y2「映射算不出不拒」那一支 | **打中**（三故障，条款字面许可） | R 的根读得出，R 的记录、中央映射树根、账（树表或分配记录树的一个节点）同时读回全零：M1 落到 P、C 隔离 0 槽，写行那次根落盘之前写掉 R 的 14 份单元；ow k=1/3/5 各 4 格、sw3 k=5 4 格；层 0 全量 20516 态里池级 checker 红 20504 态（对照格 0 态） |
| Y4 F2_NEWEST 再加一样故障 | **打中**（三故障，条款字面许可） | H4 那一形（R 的单元 + P 那次轮换写的较新系统配置槽）再加 P 的根槽读回全零：三处都落到 Pm，跳过已确认的 P；只有 R 的单元 + P 的根槽时三处照报读不出（4 / 4 组） |
| Y4 实例表读不出时只认恰好相等 | **打中**（三故障，条款字面许可，只读挂载一处） | R 的单元 + 较新系统配置槽 + R 指着的实例表那一片：只读挂载落到 Pm、跳过已确认且根读得出的 P；恢复报读不出、可写挂载拒（4 / 4 组） |
| C1 判官复算 | 没打中新形 | 置零镜像上复算 C_MAPPING 拒 40 次：20 次合法，20 次不合法全是「账根以下的节点读不出」那一形（已知）；F1 拒 590 次全合法；C_MAPPING 与 C393 没有报串 |
| Y1 映射现算漏 / 多 | 没打中 | 账读得出那一支与映射现算那一支隔离槽数逐格相等（ow 12 / 12、sw3 17 / 17） |
| Y3 F1_AFTER_F2 水位 | 没打中 | sw3 扫描里 R 的单元读不出且记录不齐的格全被 F1 拒（295 格），层 0 二次崩溃 4 态 F1 拒全合法 |
| Y4 / A1 c_见证 = 0 | 没打中（读码） | CPU、GPU 两边守卫等价 |
| Z1 / A1 / B1 GPU 与 CPU | 没打中（读码，GPU 不跑） | 逐支对过 F2_NEWEST 的 GPU 实现，没找到判得不同的一形 |
| 三处触发条件不同 | 观察，交主 agent 定归不归这一轮 | 恢复按走读失败退、只读挂载按打开失败退、可写挂载按建上一版失败退：R 只有一个数据单元读回全零时恢复落 P、只读与可写挂载落 R；本轮没改触发条件 |

## 一、Y2：「映射算不出不拒」那一支放过的一形（打中）

### 历史（每一步指到许可它的那一句）

1. mkfs、第一个文件、会话里覆盖写 k 次（P = 最后一次），再发一次 R；断电在 R 的根槽落盘之后、它那次系统配置轮换之前（R 没见证）。
2. 下一次可写挂载 M1 那一刻，R 的根槽读得出；R 的全部记录（两盘各一份）、R 的中央映射树根（两份）、R 的账的一个节点（分配记录树根 K3，或树表 T，或分配记录树里另一个节点 N，各两份）读回全零。R 建不出来、系统配置没见证它，M1 退到 P：`.claude/kb/decisions/23-journal的角色与格式.md:396`「恢复、可写挂载读阶段与只读挂载择根三处用同一套逻辑，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根」。
3. C 读 R（`C_NEWER_ROOTS` 那一份）：账读不出，R 那次发布一条记录都读不出，映射现算不成，不隔离、不拒：`.claude/kb/decisions/23-journal的角色与格式.md:392`「较新可读根那次发布的记录一条都读不出、账也读不出时，映射现算得出就隔离，现算不成不隔离、不拒（主 agent 2026-10-08 定）」；实现在 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:1942`「if readable_records_of_its_publish.is_empty() {」，现算不成走 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:1952`「None => continue,」。
4. M1 取号、写行；写行那次发布从 P 的账里的空闲槽分配，正是 R 新分配的那几槽，根落盘之前就写了下去。用户动作不需要：写掉发生在 M1 自己的写行发布里；放开用户动作（覆盖写、建 inode、顺序写两个数据单元，深度 2）只会写掉更多。

### 量到的（副本，`expected-summary.txt` 原样）

```
sw-ow-k135-sub2-base	TALLY kind=ow k=5 root_hidden=false M1=Ok r_units=hit cells=4
sw-ow-k5-sub3-base	TALLY kind=ow k=5 root_hidden=false M1=Ok r_units=hit cells=30
sw-sw3-k5-sub2-base	TALLY kind=sw3 k=5 root_hidden=false M1=Ok r_units=hit cells=4
l0-hit-ow-k5	L0 kind=ow k=5 unit=u6K3+u8M rec_hidden=1 root_hidden=false L0 states=20516 closed=20496 chk_red=20504 chk={"I-2.1", "I-3.1", "I-3.10", "I-3.11", "I-4.8", "I-7.2", "I-7.4", "I-9.1"} rec_R_bad=0 r_overwritten_in_ring=20504 landed={"FileRead((2,10) c=P)": 4107, "FileRead((2,11) c=P)": 9, "FileRead(P c=P)": 13476, "FileRead(R c=R)": 2924}
l0-ctrl-ow-k5	L0 kind=ow k=5 unit=u8M rec_hidden=1 root_hidden=false L0 states=20516 closed=20496 chk_red=0 chk={} rec_R_bad=0 r_overwritten_in_ring=0 landed={"FileRead((2,10) c=P)": 4107, "FileRead((2,11) c=P)": 9, "FileRead(R c=R)": 16400}
```

- 两两组合那一档（单元组大小 ≤ 2、记录全部子集、R 的根读得出）：ow k=1、3、5 各 112 格里 4 格打中，sw3 k=5 共 736 格里 4 格打中。打中的格都是「R 的全部记录 + 中央映射树根 M + 账上一个节点」：ow 是 `u4N+u8M`、`u5N+u8M`、`u6K3+u8M`、`u8M+u9T`，sw3 是 `u7N+u11M`、`u8N+u11M`、`u9K3+u11M`、`u11M+u12T`（`grep -P '\tHIT' expected-summary.txt`）。每格写行那次根落盘之前写掉 R 的 14 份单元，那一刻池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4（sw3 另加 I-8.6、I-8.9），与上一轮 H1 同一组红。
- 层 0：对 M1 的录制流全量枚举（每一段都展开，`enumerate_layer0_selecting_versions_observing_each_state`，闭式 20496 态加 20 个撕裂态），打中格 20504 态池级 checker 红、R 的根在环里而它的单元被写掉；对照格（只藏映射树根，账读得出、C 隔离 12 / 12 槽）0 态红。两格的恢复都没有一态落到 R 而读错（`rec_R_bad=0`）：R 没见证，恢复照 F2 读回 P。所以用户读到的内容不错，红的是池级不变量，与上一轮 H1 坐实的那一类相同。
- 根槽也读回全零的那一半（R 在 M1 那一刻整个看不见）是已知的 C747，不算这一格。

### 四句

1. 分不分辨臂：分。冻结副本原样（base）4 格写掉；臂 `OPUS_ARM_REFUSE_UNMAPPED=2` 这 4 格都改成拒（`sw-ow-k135-sub2-arm2	TALLY kind=ow k=5 root_hidden=false M1=Refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords r_units=- cells=4`，sw3 那一档同样 16 格拒、0 格打中）。
2. 系统当时看不看得到判别子：看得到。M1 那一刻 R 的根读得出（R 在环里），映射与账读不出，系统配置里取过号的实例代号等于 R 的实例代号（R 之后没有别的挂载取过号）。实现员担心的入库 `urJ` 那一形里，取过号的实例代号比 R 新（之前那次挂载看不见 R、已取号、已写掉 R 的单元）。这一位两形不同，臂 2 用的就是它。
3. 满足的是判据字面的哪一分句：D23 已定项 14 第 392 行那一句的「现算不成不隔离、不拒」，条款字面许可；后果与判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K2 W1 / W2 H1 那一行「M1 写行根落盘之前写掉 R 的 14 份单元」同形。主 agent 定这一支时的依据是 `records/2026-10-01-删除与并发事务.md` 第 1266 行①「映射算得出就护住、算不出不拒（照拒 `urJ` 入库用例红、不算 H1 那几格写掉），认」；这一形里映射树根自己也读不出，「算不出」落在 H1 同形的格上，不只是 `urJ`。
4. 跑前条款给的改法：正文第五节只写「打中：主 agent 现查坐实，改法进下一批」，没给具体改法。下表是我提的，**只在我的模型上量过、被攻过零轮**。

| 改法 | 这 4 × 4 格（ow k=1/3/5、sw3 k=5） | 8 份入库用例（38 条） | 代价 |
|---|---|---|---|
| 臂 1：记录一条都读不出、映射现算不成时，照「记录不齐」那一支办（重读、凭据、仍不成拒 C_MAPPING） | 量过：全拒 | 量过：`recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed` 红 1 条（`a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts`，即 `urJ`），其余 37 条过 | `urJ` 那一形永远拒可写（量过） |
| 臂 2：同臂 1，只在系统配置里取过号的实例代号不比 R 新时拒 | 量过：全拒 | 量过：38 条全过 | 故障一直在时这几形只剩只读挂载（推的）；R 之后已有一次挂载取过号、这一次又碰上这三样故障的那一形照旧不拒（推的，没造盘面） |
| 维持今天（不拒） | 量过：全写掉 | 量过：38 条全过 | 上表那几格池级 checker 红 |

臂 2 的判别量取 `SystemRuntimeQuantities::journal_instance`（每份系统配置里记着的实例代号）。两条臂的改动原文在 `patch_arms.py` 里。

## 二、Y4：F2_NEWEST 再加一样故障就退过已确认的 P（打中）

### 历史

1. 同第一节第 1 步：R 的根槽落盘、轮换没落，P 已确认（P 那次轮换写的那一槽在每块盘上都是较新的那一槽）。
2. 恢复、只读挂载或可写挂载读的那一刻，读回全零的是：R 的单元（Ru）、每块盘较新的那个系统配置槽（cfg）、P 的根槽（Proot）。
3. c_见证 落回 P 之前那一版 Pm 的计数器（较旧的那一槽的 tail），R 没见证、走读失败，退到 [Pm, R) 里读得出的最新一条根：P 的根读不出，不在这一段里，落到 Pm。许可它的是 `.claude/kb/decisions/23-journal的角色与格式.md:396`「恢复、可写挂载读阶段与只读挂载择根三处用同一套逻辑，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根」，实现 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5527`「witnessed_publish <= order」那一段取的就是读得出的根。
4. 可写挂载在 Pm 上取号，P 被新实例抛弃。

另一形（只读挂载一处）：第 2 步的 Proot 换成 R 的根指着的实例表那一片（Itab；ow 与 sw3 的 R 都没重写实例表，这一片也是 P、Pm 的）。所选根的实例表读不出，只认与见证到的那一版恰好相等的 Pm：`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5525`「None => order == witnessed_publish,」，条款 `.claude/kb/decisions/23-journal的角色与格式.md:396`「所选根的实例表读不出时只认与见证到的那一版恰好相等、读得出的那条根」。只读挂载打开 Pm（打开那一步不读实例表），P 的根读得出、已确认，被跳过；恢复与可写挂载在 Pm 上走读 / 建上一版也读实例表，照报失败。

### 量到的（`f2-base`，ow 与 sw3 各 k=3、5，四组同形；这里抄 k=5 两组）

```
OPUS F2 kind=ow k=5 hidden=Ru+Proot rec=Failed(R UnitUnreadable) ro=Err(Open) rw=Err:Recovery
OPUS F2 kind=ow k=5 hidden=Ru+cfg rec=FileRead(P) ro=Ok(P) rw=Ok(P)
OPUS F2 kind=ow k=5 hidden=Ru+cfg+Proot rec=FileRead(Pm) ro=Ok(Pm) rw=Ok(Pm)
OPUS F2 kind=ow k=5 hidden=Ru+cfg+Itab rec=Failed(R UnitUnreadable) ro=Ok(Pm) rw=Err:Recovery
OPUS F2 kind=sw3 k=5 hidden=Ru+Proot rec=Failed(R UnitUnreadable) ro=Err(Open) rw=Err:Recovery
OPUS F2 kind=sw3 k=5 hidden=Ru+cfg rec=FileRead(P) ro=Ok(P) rw=Ok(P)
OPUS F2 kind=sw3 k=5 hidden=Ru+cfg+Proot rec=FileRead(Pm) ro=Ok(Pm) rw=Ok(Pm)
OPUS F2 kind=sw3 k=5 hidden=Ru+cfg+Itab rec=Failed(R UnitUnreadable) ro=Ok(Pm) rw=Err:Recovery
```

`Ru+cfg` 是上一轮 H4，F2_NEWEST 修好了（四组都落 P）。`Ru+Proot` 是第 33 批 T 那一轮的 A 形，`F2_EXACT` 把它改成照报读不出，F2_NEWEST 保住了。两样合起来（三故障）四组都退过已确认的 P：A 形那一保护靠「见证到的那一版根读不出就不退」，cfg 一藏，见证到的那一版换成了 Pm，P 落在区间里面，读不出就被跳过。

### 四句

1. 分不分辨臂：分。臂 `OPUS_ARM_F2_NO_SKIP=1` 下 `Ru+cfg+Proot` 四组都改成三处报读不出，`=2` 下 `Ru+cfg+Itab` 的只读挂载也改成报失败（`f2-noskip1`、`f2-noskip2`）。
2. 系统看不看得到判别子：看得到。P 那次发布带末条标志的记录读得出（P 的记录没藏），它的 (txg, 实例) 落在退到的 Pm 与所选根 R 之间、又不被 R 的实例表抛弃：系统知道 Pm 之后还有一次写齐了记录的发布。
3. 字面分句：「读得出、不被所选根的实例表抛弃的最新一条根」里的「读得出」（Proot 那一形），与「所选根的实例表读不出时只认与见证到的那一版恰好相等」（Itab 那一形）。条款字面许可。用户认下的代价（`.claude/kb/decisions/23-journal的角色与格式.md:421`「已确认的版本在较新的系统配置槽与它的单元同时读回全 0 时，恢复、可写挂载与只读挂载悄悄读回上一版」）说的是较新系统配置槽加**已确认那一版的单元**；这里坏的是 P 的根槽或实例表，P 的单元完好，不在那一句里。
4. 改法（我提的，**只在我的模型上量过、被攻过零轮**）：

| 改法 | Ru+cfg+Proot（4 组） | Ru+cfg+Itab（4 组） | 8 份入库用例（38 条） |
|---|---|---|---|
| 臂 F2_NO_SKIP=1：退到的那一版与所选根之间有一次末条读得出、根读不出、不被所选根实例表抛弃的发布，就不退 | 量过：三处都报读不出 | 量过：不变（只读挂载仍落 Pm） | 量过：38 条全过 |
| 臂 F2_NO_SKIP=2：同上，那次发布的根读得出也算 | 量过：三处都报读不出 | 量过：只读挂载也报打开失败 | 量过：红 1 条 `a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version`（它钉的正是「实例表读不出时只认恰好相等」那一退，与这一臂对撞） |
| GPU 恢复内核 | 推的：两条臂都要在 `newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to` 的 wgsl 那一份同改，否则 GPU 照今天落 Pm、CPU 报读不出，对拍红 | 同左 | — |

两条臂都只读内存里的记录、不加读盘；代价是这几形从「静默读回 Pm」变成「报读不出 / 拒」，等故障撤掉之后再挂（推的）。

## 三、C1：C_MAPPING 那一拒的复算与报串（没打中新形）

做法：扫描里每一次拒可写（630 次），把这一次读回全零的那几段在盘上真的写成零，当一份不带读故障的崩溃镜像，交崩溃注入第三截用的同一个判官入口 `judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount`（崩溃态注入过读故障、被拒的挂载读的就是这份镜像；`selected_version` 取这份镜像上恢复落到的那一版；只读读出按「在允许集合里」交，这一条没核）。原样：

```
     20 REFUSED M1=Refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords judge_on_zeroed=Illegal[CriterionFalseWhenRecomputedOnTheCrashImage] judge_selected=P
     20 REFUSED M1=Refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords judge_on_zeroed=Legal judge_selected=P
    590 REFUSED M1=Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify judge_on_zeroed=Legal judge_selected=P
```

（`grep -P '\tREFUSED' expected-summary.txt`，去掉格名再 `uniq -c`。）判不合法的 20 次全是「映射树根 + 分配记录树根以下的一个节点（ow 的 u4、u5，sw3 的 u7、u8）」：实现读账读整棵分配记录树、拒，复算只看账根 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2791`「只看账根那一个节点：账根以下的节点读不出时实现照拒、这里判读得出。」，正文「已知」第 5 条那一形；那几个节点是账根以下的节点是推的（藏它们 C 走了账读不出那一支，复算又说账根读得出）。判合法的 20 次是账根（K3）或树表（T）加映射树根。没有一次拒报成 C393 那一成员（`AccountOfAnAbandonedRoot` 在汇总里命中 0 次），C_MAPPING 那一拒点名的都是 R。

二次崩溃那一族（入库判官在崩溃注入里真正碰到的形状）：M1 在 R 的根槽与全部记录读回全零下可写挂载（看不见 R，C747 那一形），对 M1 的录制流层 0 全量枚举，每个崩溃态上不带读故障再挂 M2：

```
sc-ow-k5-rootrec	OPUS SC kind=ow k=5 m1_hide=root+rec STATES enumerated=20516 closed=20496
sc-sw3-k5-rootrec	OPUS SC kind=sw3 k=5 m1_hide=root+rec STATES enumerated=20521 closed=20496
sc-sw3-k5-rootrec	OPUS SC kind=sw3 k=5 m1_hide=root+rec CLASS count=4 M2=Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify rec=FileRead(P c=P) verdict=Legal r_in_ring=true r_overwritten_already=14 sample=111111111111111100000000000000000000000011000000
```

41037 态里 M2 只拒了这 4 态（F1，判合法），没有一态出 C_MAPPING；做成的 M2 落到的版本与崩溃镜像上恢复落到的逐态相同（`div=true` 0 类），也没有一态在写行根落盘之前新写掉 R 的单元。M2 落到 R 而 R 的单元已被 M1 写掉一部分的那几类（`M2=Ok eff=R … r_overwritten_already=1`…`7`）是 M1 看不见 R 造成的，属 C747。这一族里 C_MAPPING 走不到：M1 的那条写行记录只盖得到 R 的第一条记录（计数器接在 P 后面），R 的末条要被盖掉，得先让 M1 的根落盘，而那一槽正是 R 的根槽。

## 四、Y1 映射现算漏 / 多、Y3 F1_AFTER_F2 水位（没打中）

Y1：同一组「R 的全部记录读回全零」下，只藏映射树根（账读得出，C 读账）与只藏账（K3 或 T，C 从映射现算）隔离的槽数逐格相等（`rerun1/out/sw-ow-k135-sub2-base.log`、`sw-sw3-k5-sub2-base.log` 原样）：

```
OPUS CB kind=ow k=5 unit=u8M rec_hidden=1 root_hidden=false recF=FileRead(P c=P) roF=Ok(P) M1=Ok effective=P isolated=[12, 12]
OPUS CB kind=ow k=5 unit=u6K3 rec_hidden=1 root_hidden=false recF=FileRead(P c=P) roF=Ok(R) M1=Ok effective=P isolated=[12, 12]
OPUS CB kind=sw3 k=5 unit=u11M rec_hidden=111 root_hidden=false recF=FileRead(P c=P) roF=Ok(P) M1=Ok effective=P isolated=[17, 17]
OPUS CB kind=sw3 k=5 unit=u9K3 rec_hidden=111 root_hidden=false recF=FileRead(P c=P) roF=Ok(R) M1=Ok effective=P isolated=[17, 17]
```

这几格放开用户动作（深度 2，13 个结点）之后 R 的单元一份没被写掉。只比了槽数，没逐槽比两份集合。

Y3：sw3 两两组合那一档里 R 的单元读不出、记录不齐的 295 格全被 F1 拒（`sw-sw3-k5-sub2-base	TALLY kind=sw3 k=5 root_hidden=false M1=Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify r_units=- cells=295`），置零镜像上复算全合法（上一节 590 次里各一半是 base 与臂 2 两份）。第二节那一形退到 Pm 时 F1 按 Pm 当水位再判：P 的记录齐、R 的记录齐，不拒，与条款同。

## 五、三处触发条件不同（观察，不算本轮打中）

D23 已定项 14 写三处「用同一套逻辑」（第 396 行）。判法函数三处是同一个，但触发它的失败不同：恢复在走读失败时退（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5615`「// 走读失败才再扫一遍环拿记录给退回的判法」），只读挂载在打开失败时退（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mounted_read.rs:624`「Err(failure_of_the_selected_version) => {」），可写挂载在建上一版失败时退（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:6778`「match rebuild_previous_version(devices, unit_area_start, &effective_root, own_record) {」）。三样读的单元不同，同一份盘面落到的版本就不同。单个单元两份都读回全零、R 的记录读得出时：

```
sw-ow-k135-sub2-base	DIVERGE kind=ow k=5 unit=u0D rec_hidden=0 root_hidden=false recF=P roF=R M1=R
sw-ow-k135-sub2-base	DIVERGE kind=ow k=5 unit=u6K3 rec_hidden=0 root_hidden=false recF=P roF=R M1=P
sw-sw3-k5-sub2-base	DIVERGE kind=sw3 k=5 unit=u0D rec_hidden=000 root_hidden=false recF=P roF=R M1=R
sw-sw3-k5-sub2-base	DIVERGE kind=sw3 k=5 unit=u9K3 rec_hidden=000 root_hidden=false recF=P roF=R M1=P
```

数据单元读不出：恢复读回 P，只读挂载开在 R（之后读那个文件报错），可写挂载在 R 上取号。分配记录树根读不出：恢复与可写挂载落 P，只读挂载开在 R。R 都没见证，三种落法都在 F2 许可的结局里，用户读不到错的内容；分叉的是「三处同一套逻辑」那一句。触发条件是 F2 合入时（第 33 批 T）就有的，本轮只改了退到哪一条，所以不记本轮打中；层 0 oracle 与 GPU 判官都按恢复那一路判，挂载透明只判不带读故障的态（推的：这一分叉今天没有判官会报）。交主 agent 定要不要另立一题。

## 六、Z1 / A1 / B1：GPU 与 CPU（读码，没打中）

GPU 不跑，读 `crash_judge_recovery.wgsl` 的 F2 那一份，按 CPU 判法对：

- c_见证 = 0：CPU 先判 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5474`「if witnessed_journal_counter == 0 {」，GPU 在见证过之后一起判 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:208`「if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {」；三个条件任一为真都不退，次序不影响结局。
- 实例表读不出只认恰好相等：GPU `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:220`「var is_admissible = is_the_witnessed_version;」，与 CPU 第 5525 行同。表从所选根读（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:211`「let chosen_instance_table_is_readable = load_rows_of_the_instance_table_chain(chosen);」），超过 512 行或 256 片记判不了、回落 CPU。
- 同序几条取环序上最后那条：CPU `max_by_key` 与 GPU「不早于就替换」同取最后一条。
- `R_ROWS` 共用：F2 那一次把所选根的表读进 `R_ROWS`，之后 `check_records` 重读落到的那一版的表再判，读不串。
- B1（第 42 批那条对拍流的造法）没逐字核，只看到它只改对拍文件、不改 wgsl 判法。

推的：第一、二节的臂要是采纳，GPU 恢复内核必须同改（第二节两条臂改的是恢复那一路的判法，GPU 照它判）；不同改，对拍在那几形上红。

## 没打中的形状与取样范围

- Y2：R 的根也读回全零（C747，已知）；记录只缺一部分（走「记录不齐」那一支，映射算不成就拒，12 格）；映射树根读得出（C 现算得出、隔离住，放开用户动作深度 2 没写掉）；账读得出而只藏映射树根（C 读账、隔离住）。
- Y4：R 的单元 + 较新系统配置槽 + P 的末条记录（四组都落 P）；只藏 P 的根槽（照报读不出）；c_见证 落在 R 那次发布自己的记录上没造出来（这几段历史里轮换不会落在一次发布中间，推的）。
- C1：630 次拒的复算，见第三节；二次崩溃 41037 态。
- 取样（缩过的是历史，不是崩溃态）：R 两种（ow、sw3）；ow 取 k = 1、3、5，sw3 只取 k = 5（`f2` 那一段两种都取 k = 3、5）；R 的单元组子集大小 ≤ 2（ow k = 5 另到 ≤ 3），R 的记录读不出取全部子集，R 的根读得出；用户动作放开到深度 2（覆盖写、建 inode、顺序写两个数据单元，13 个结点）；二次崩溃只取「M1 看不见 R」那一种读故障。一格（含深度 2 扫描）release 约 0.075 秒，层 0 一段约 2.05 万态、15 线程约 95 秒，都按先跑一小段量的时长估。
- 层 0 状态数：每段跑之前按 `closed_form_state_count` 算全量（20496，各段都远低于 10⁶）；本腿合计跑了草稿 4 段加复跑 4 段，另有一段在草稿里中途停掉（那一段被自动转到后台，我照 `proc.py stop` 停了测试进程），合计不到 18.5 万态，低于 10⁷。

## 这条腿自己的限度

- 读故障只建模「读回全零」、只在一次挂载里有；设备报读错、跨两次挂载的故障没扫。
- 盘只有 384 槽小盘一档、R 只有两种发布；分配记录树都只有一两层。
- 判官复算的「只读读出在允许集合里」那一条按真交，没算 oracle。
- 置零镜像是「挂载读到的字节」的代用品，不是崩溃注入真走到的盘面；二次崩溃那一族真走到了，但只造出 F1 拒。
- 第一、二节的四条臂都是我提的，只在我的模型上量过、被攻过零轮；臂的代价（只剩只读挂载、报读不出）是推的，没量频率。

## 要主 agent 跑的命令

无。全部阶段在前台跑完，复跑见开头。

## 没做什么

- K1、K3 不归这条腿，没判；变异行没核。
- GPU 与 checker 档测试、重型测试一概没跑；GPU 一侧只读了 wgsl。
- 第五节的触发条件分叉没给改法、没量代价。
- 开工时 `ps` 看到别的会话在跑 `cargo test`（主工作区 `singlefs-harness` 一条、四包一条），没有性能测量；我的 cargo 都经包装排队、没等锁。
