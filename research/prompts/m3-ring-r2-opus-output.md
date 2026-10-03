# m3-ring-r2 攻方腿报告（Opus，2026-10-02）

接续：前一条同立场腿撞了用量上限，没写报告、没落模型（交接摘要里 0 个文件、0 条命令；草稿目录是空的），这一条从头做。被判代码取冻结副本 `/tmp/claude-1000/m3-batch4-freeze/tree/`，拷进草稿目录之前按 `research/prompts/m3-ring-r2-snapshot/tree-sha256.txt` 核过，`sha256sum -c` 不是 OK 的 0 行。副本上的数都是副本上的数，不算入库装置上的数。kb 的行号是冻结副本里那份的行号（主工作区的 D23 与 invariants.md 在冻结之后改过，行号已移位），所以 kb 引文没写成 `文件:行号` 的格式，写成「文件 冻结副本第 N 行：原文」。

## 复跑

```
bash research/prompts/m3-ring-r2-opus-model/scripts/rebuild-and-run.sh <草稿目录> [build|g1|direct|sweep|sweep2|selfred|rollback|g4|all] [树，默认冻结副本]
```

脚本把树拷进 `<草稿目录>/repo`、打 `core-and-checker.diff`（`patch -p2`）、放进原型 crate `ring-window-proto`、编译（`scripts/build.sh`：经 `run-with-memory-cap.sh 3G` 与 `capped.sh 4`），再经 `scripts/run.sh`（`run-with-memory-cap.sh 1G`、`capped.sh 4`）按批跑。`g1` 这一批这次从头验过：另一个草稿目录里重建、重跑，两份输出与 `results/g1/` 逐字节相同（`cmp` 无差异）。其余几批的脚本就是这次实际跑的那几份。

模型目录 `research/prompts/m3-ring-r2-opus-model/` 的文件 sha256（产物 28 个文件的 sha256 在 `results/SHA256SUMS`）：

```
f9e6679b5d9af98ec38964a765fe9a20d6837c88fe51ec9453a925c8d74219e9  ./core-and-checker.diff
43646da0aca9c9dc64bfb95cdc48b5844ea0cfbf30c29c2931dc7d543cbac465  ./results/SHA256SUMS
a1562dd579383c5ed407b1f0c33729075fbe581de8ff63e9e3660ef91c20b7e6  ./ring-window-proto/Cargo.toml
da9cb9bca06878b7bf71e657c1129da2e53dc64e5160e6994a6b7752efe59bd9  ./ring-window-proto/src/bin/fill_bytes_by_kind.rs
f2305c95c65e5a2ff4d1ee9b83ca5f5a426c01bccd48beef5c8dce47dba57060  ./ring-window-proto/src/bin/orphan_units_probe.rs
70a73a4273251e3ae3df29bee5b67ba38d6c88d7daecf832364bc97b0c93a660  ./ring-window-proto/src/bin/rollback_to_stale.rs
eded7e33b41bba489b4137edf885e1c8f1d5fff84a8c14f2ffc7245b52a29946  ./ring-window-proto/src/bin/window_k1_states.rs
c7808092fcd09f8b6bde58a327c7eb39e5629e0e9cba67a1f212f40ee692bede  ./ring-window-proto/src/bin/window_release_direct.rs
ab21a8e8c177a7b87cc287d7c8cdddfb3ff8291bc843dc20d364d944d1f3a0af  ./ring-window-proto/src/bin/window_static_check.rs
253aaf6fa4b9f5d4d44b98642f64d2a9c1e84c6895d92d968326d34f81a0cf27  ./ring-window-proto/src/lib.rs
7db3965fe64e36004d7255a9a7486f945d72b4f5f291317a0e5627088c47b316  ./scripts/batch2.sh
cab6945e2d21d26de64d50d4f84139e23dea057db921da64acd4409a276f3d45  ./scripts/batch3.sh
8751511b9d19de162cb0f0087671f538f0f02521361242a7da543325c35b0402  ./scripts/build.sh
05ac9aab8ad714d719c5322831f1fefb24e82ee446d225846db8124daf9fdc54  ./scripts/rebuild-and-run.sh
1f3669f593bba2947d16858ffb607ac1f8091054ae059e6209d0b34c16d65edd  ./scripts/run.sh
```

## 各格判定一览

| 格 | 判定 | 依据（量过 / 推的） |
|---|---|---|
| G2 字面：合法状态让候选集里的根引用已复用的单元 | **没打中**。900 段历史（K1、连崩两次、可补而不补的后洞；S ∈ {4, 8, 16}；挂载之后接 0 或 3S + 4 次覆盖写、接或不接回退），每一个单元写判全部崩溃状态，0 处。只抽了一次样，不支撑否定结论 | 量过（全域静态判法，判法会红已自证） |
| G2 门槛的锚 | 式子里的「最新根」必须写死成**环里读得出的最新根**；按 `latest_recorded_txg` 的挂载初值 first_txg − 1 取，K1 历史上挂载那一刻就把 R 的单元放出去，而崩在写行那次的状态里 R 回到窗口。今天的落点政策（提交内生块只进全空段）替它挡着，900 段里没中 | 挂载门槛量过（trace）；会中是推的 |
| G2 checker 一起改 | 必要：只改分配器时合法盘面上 I-3.1（或 I-2.1、I-3.1、I-3.10、I-4.8、I-7.4）红，S = 4 两段各 9/17 步、S = 8 两段各 21/29 步；checker 同改 0 步 | 量过 |
| G2 漏列的一处 | 影子账的候选过滤（`mount.rs:1298`）也要加窗口，正文第 2 条的「分配器、checker、回退候选集」没列它 | 推的（我的历史里没有被抛弃根） |
| G2 回退候选集 | 不加窗口时回退到 R 被今天的逐单元核拒（复用之后每一次），没读错；仍应加 | 量过（S = 4、8 各一段） |
| G2 放槽量（K1 后洞） | 同一步之差的最大值 S = 4：144 槽，S = 8：284 槽（盘 0）；挂载刚做完那一步 0 | 量过 |
| G1 写行 | **写行那一种补不了**：写行那次的记录从来不被施加（实例代号与所选根不同即停），它的洞就是 K1。三个字段对它不起作用；要补只能按别的实例的记录重建根，而那条根引用的单元已被下一次挂载盖掉 | 量过（`record_applied=false`；单元被盖量过；补进去会怎样是推的） |
| G1 首个文件 | 今天的施加法差在分配记录树根（池上不止一次挂载时），补错根 I-3.9、I-3.11 红；加字段后逐字节相同、绿 | 量过 |
| G1 卸载 | 差在卸载记号，补错根在那一串第 1 次发布上 I-7.9 红；加字段后逐字节相同、绿 | 量过 |
| G1 其余五种发布 | 覆盖写、空发布、准入抬 F、管理员回退、两种暖机：今天就逐字节相同 | 量过 |
| G1 三个字段的大小 | 实例表指针 86 字节在今天买不到任何一格；分配记录树根今天一位就够；连卸载记号共两位，放进记录标志字节的位 1、2，点名项保持 67（我提的，零轮） | 量过（差的字段只有两样）；方案零轮 |
| G3 丢写 | 不超过 120：S-辨 的「另一槽是上一世代」读者判不了，判别子与甲-读错相同 | 推的 + 上一轮旧数 |
| G3 两盘一起撕 | 「一槽撕裂」若按全池读，两块盘轮换一起撕裂的合法崩溃状态照旧永远只读；要按盘读 | 段序列量过；结论推的 |
| G3 持续设备读错 | S-辨 管不到，仍永远只读；按盘读（甲-读错-每盘，零轮）能放行 | 推的 |
| G4 (j, 0, 0) 删不掉 | 一半成立：下一次挂载 k 取号之前删不掉（j 的单元还在）；k 的写行把 j 的单元全盖掉，再下一次挂载就删得掉 | 量过（S = 4 两段） |
| G4 删错行 | 没找到 | 推的 |

## G2　一圈窗口门槛

### 式子写全（我按第一轮的形态写，副本 `core-and-checker.diff` 就是这个式子）

记 3S 为环长，N 为**根环里读得出的最新根**的 txg（按 (txg, 实例) 取最大，与择根同一读法），窗口 W(N) = {txg > N − 3S}。

- 有效：按 N 那条根的实例表判不是被抛弃的（今天的判法不变）。
- 候选集 C = 有效 ∧ txg ≥ F_生效 ∧ txg ∈ W(N)（今天是前两样）。
- 回收门槛 floor = max(F_生效, W(N) 里有效根的最小 txg)（今天是 max(F_生效, 全环有效根的最小 txg)，「reclaim_floor(effective_floor, oldest_valid_root),」（`crates/singlefs-core/src/mount.rs:1596`））。
- N 在三处各取什么：挂载重建分配器时取环里读得出的最新根（**不取** `first_txg − 1`，见下面「锚」一节）；进程内每记一条根时取这条根（`RootRingOccupancy` 的 `latest_recorded_txg`，记根的前一行它已经改成这条根，`crates/singlefs-core/src/allocator.rs:1641` 起）；checker 与回退候选集取镜像上读得出的最新根（回退时就是现行那一版）。
- 要一起改的五处（副本都改了）：挂载重建的门槛（`mount.rs:1590` 起）、进程内记根时的门槛（`RootRingOccupancy::oldest_valid_root`）、抬 F 那一串的门槛（`mount.rs:2509` 起）、影子账的候选过滤「.filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)」（`crates/singlefs-core/src/mount.rs:1298`）、checker 的候选过滤「let walked = *index == newest_index || (!abandoned && !below_floor);」（`crates/singlefs-checker/src/walk.rs:5977`）与 checker 里「由记录施加出来的版本」那一判用的最旧有效根。正文第 2 条列的是「分配器、checker、回退候选集」三样，**影子账的候选过滤不在里面**：不改它，一个同时被窗口外的旧根 R 与一条被抛弃根引用的槽，挂载时不隔离（R 算候选），又被窗口门槛回收（推的；我的历史里没有被抛弃根，没量）。

与别的条款的关系：
- F_生效：定义不动。窗口外的根不会是「某块盘上最新的有效根」（窗口里同一区域还有更新的根，除非那一区域在窗口里全是洞），所以不改 F_生效 的取值（推的）。
- 回退候选集：D23 已定项 14 写的「回退深度由候选集定：平时是整个根环」（`.claude/kb/decisions/23-journal的角色与格式.md` 冻结副本第 388 行）要改成「平时是最近一圈」，窗口外的旧根选不中。
- I-3.9：L 取不到窗口外的旧根，第一轮量到的「K1 后洞上单值判法红 10 步」那一类对象随之没了（推的，这一轮没量 I-3.9 单值）。
- I-3.1 / I-7.4：checker 不跟着开窗口，分配器开了窗口的合法盘面上就红（量过，下面「放槽量」一节）。

### 放槽量（`results/direct/`，`window_release_direct`）

K1 那一类后洞（h = 3S + 4，新实例写行那次崩）之后挂载 k，再覆盖写 3S + 4 次；每一步量最新根那一版账里「已释放、还扣着」的槽数（盘 0）。可补的后洞（同一实例覆盖写崩，今天不补）作对照。

| S | 历史 | 今天的峰值 | 窗口的峰值 | 同一步两者之差的最大值 | 挂载刚做完那一步之差 |
|---|---|---|---|---|---|
| 4 | K1 | 384 | 250 | 144 | 0 |
| 4 | 可补的后洞、不补 | 400 | 254 | 146 | 0 |
| 8 | K1 | 716 | 442 | 284 | 0 |
| 8 | 可补的后洞、不补 | 732 | 446 | 286 | 0 |

- 挂载刚做完那一步之差恒 0：窗口锚在环里最新根，挂载那一刻 N = h − 1，旧根 R = h − 3S 还在窗口里；写行那条根 h + 1 记下之后 R 才出窗口。
- 第一轮攻方的盘面度量是 366 / 174（S = 8 / 4，同一张盘面上两种门槛扣住的差），这一格是两种分配器各跑一遍、同一步比账，口径不同、数也不同；同一口径下的数以这张表为准（副本上的数）。
- checker 不开窗口（今天的 checker）配窗口分配器：S = 4 两段各 9 / 17 步红、S = 8 两段各 21 / 29 步红，红的是 `I-3.1` 或 `I-2.1、I-3.1、I-3.10、I-4.8、I-7.4`（`results/direct/d-*-ring-cw0.out`）；checker 开窗口：四段 0 步红。所以「checker 一起改」不是可选的。

### 最要紧的一问：有没有合法状态让候选集里的根引用已被复用的单元

判法（`window_static_check`，全域、不抽样）：一段历史录成一条流，对流里每一个单元写 w 判「有没有一个崩溃状态让 w 已落、而 w 盖住的槽正被那个状态的候选集（窗口按那个状态读得出的最新根算）里某条根引用着」。层 0 的枚举域里，w 在第 k 段时凡是 w 落了的状态都是「前 k 段全落 + 第 k 段的某个子集」；单元写是 COW、根槽 FUA 自成一段，所以这些状态读得出的根恒等于「前 k 段全落」那一个镜像上的根，候选集只取决于 k；原地覆写的撕裂只碰根槽、系统配置槽与 journal 槽，不碰单元区。于是对每一段算一次候选集就判完了这段历史的全部崩溃状态（含撕裂态）。打中的，另造「前 k 段全落 + 只有 w 落」那一个状态跑池级 checker（第二条路）。

扫描的范围与结果（`results/sweep/sweep1.txt` 612 行、`sweep2.txt` 288 行，每行一段历史）：

| 批 | S | 洞 txg | 历史 | 被判那次挂载之后的用户动作 | 锚 | 段数 | 单元写次数合计 | 打中 |
|---|---|---|---|---|---|---|---|---|
| 一 | 4、8、16 | 3S + 1 … 6S + 6（每个 S 取满一圈多） | K1、连崩两次写行（k1x2）、可补而今天不补的后洞（fillable） | 无（只录那一次挂载） | 环里最新根 / first_txg − 1 各 306 段 | 612 | 26 268 | 0 |
| 二 | 4、8 | 3S + 1 … 3S + 12 | 同上 | 覆盖写 3S + 4 次；再接或不接「回退到窗口里最旧的候选根 + 2 次覆盖写」（144 段回退成功） | 各 144 段 | 288 | 194 520 | 0 |

每行形如（第一批第一行原样）：

```
STATIC s=4 history=k1 hole=13 suffix=0 rollback=none anchor=Ring first_txg=14 writes=71 segments=13 unit_writes=54 valid_roots_outside_window_seen={1} hit_writes=0 hits={} first_hit=None  rc=0 
```

`valid_roots_outside_window_seen` 在 900 段里每段都非空：窗口确实把旧根 R 关在了外面，扫描不是空转。

判法会红（自证）：把判法里候选集的窗口拿掉（`STATIC_NO_WINDOW=1`）、分配器照样开窗口，K1、S = 4、h = 16、覆盖写 16 次那段历史上 32 次单元写盖住 R（txg 4）引用的槽，那一个状态上今天的 checker 红 `I-2.1、I-3.1、I-3.10、I-4.8、I-7.4`：

```
STATIC s=4 history=k1 hole=16 suffix=16 rollback=none anchor=Ring first_txg=17 writes=575 segments=77 unit_writes=478 valid_roots_outside_window_seen={} hit_writes=32 hits={"candidate_txg=4 newest=20": 4, "candidate_txg=4 newest=22": 16, "candidate_txg=4 newest=23": 8, "candidate_txg=4 newest=27": 4} first_hit=Some("segment17/write104/candidate_txg=4 newest=20") checker_on_hit_state=["I-2.1", "I-3.1", "I-3.10", "I-4.8", "I-7.4"] next_mount=ok chosen=20 effective=20 first_txg=21 checker_after_next_mount=["I-2.1", "I-3.1", "I-4.8", "I-5.1", "I-7.4"]
```

（被盖的那一槽在窗口里时就是 G2 判据的字面；窗口拿掉之后的 R 是「本来不该是候选」的那条，所以这一行证的是判法与 checker 抓得到「候选根的单元被复用」，不是打中。）

### 锚：挂载时取环里最新根，不取 first_txg − 1

分配器的根环表在挂载时把 `latest_recorded_txg` 装成 first_txg − 1（「新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1」那一行，`crates/singlefs-core/src/mount.rs:1648`），而 first_txg 按「环里根与环里全部自证过的记录」的最大值加一取。K1 那一类洞里 first_txg − 1 = h（那条没施加的写行记录），环里最新根是 h − 1。窗口若在挂载时按 first_txg − 1 算，R = h − 3S 在挂载那一刻就出窗口、它的单元当场回收（`results/k1-count/mount-floor-trace.txt`：`anchor=FirstTxgMinusOne first_txg=17 window_lower=Some(CheckpointTxg(4)) oldest_valid_in_window=Some(CheckpointTxg(5))`，锚在最新根时同一次挂载是 `window_lower=Some(CheckpointTxg(3)) oldest_valid_in_window=Some(CheckpointTxg(4))`）；崩在写行那次时环里最新根仍是 h − 1、R 回到窗口里成了候选。这一步推得出，扫描里没中：写行那次只写提交内生块，它们只进「最低的全空段」（`crates/singlefs-core/src/allocator.rs:794`），挂载时多放的那一代单元在 900 段里没有一次把一整段放空。**所以式子里 N 要写死成「环里读得出的最新根」**：这不是这一轮的打中，是落点政策替它挡着；落点政策一改（例如提交内生块回落到最低空槽），锚在 first_txg − 1 的实现就会在 K1 历史上中（推的）。

### 回退候选集不跟着改会怎样（`results/rollback/`）

窗口开在分配器与 checker 上、回退候选集那一道关掉（`SINGLEFS_ROLLBACK_NO_WINDOW=1`），挂载 k 之后覆盖写 0 … 3S + 4 次再回退到 R：

| S | 臂 | 放行（checker 绿） | `UserVisibleUnitStillAllocatedUnderAnotherGeneration` | `ResurrectedUnitCopyUnreadableOrMismatched` | `TargetVersionUnreadable` | `TargetNotACandidate` |
|---|---|---|---|---|---|---|
| 4 | 今天（不开窗口） | 9 | 0 | 0 | 0 | 8 |
| 4 | 窗口、回退不加窗口 | 2 | 1 | 1 | 5 | 8 |
| 8 | 今天 | 21 | 0 | 0 | 0 | 8 |
| 8 | 窗口、回退不加窗口 | 2 | 1 | 1 | 17 | 8 |

放行的两次是覆盖写 0、1 次（R 的单元还没被复用）；之后每一次都被今天回退那一道逐单元核拒（「仍分配而分配代与回退目标那一版账里的不同就拒」（`crates/singlefs-core/src/mount.rs:4953`） 那一段文档）。所以回退候选集不加窗口在这几段历史上读不错，代价是「候选集里有、回退却拒」的不一致与 D23 已定项 14 第一轮边角 ① 的同一个缺口（已释放的记录记的是释放代，看不出「复用过又释放了」）；回退候选集仍应加窗口（推的）。`TargetNotACandidate` 那 8 次是 R 出了环（slot(h) 被 h + 3S 盖掉）之后。

### 四句（G2）

1. 分不分辨臂：这一轮在 G2 字面上没有打中，锚的两种写法在 900 段里同为 0；分辨得出的是「checker 改不改」（不改 9/17、21/29 步红，改了 0）与放槽量。
2. 判别子看不看得到：环里最新根在挂载读根环那一遍就读到了（`rebuilt_allocator` 手里的 `roots`），锚取它不要新读。
3. 判据字面：G2「造出一个合法状态让回退候选集里的根引用已复用的单元」——没造出；跑前判据的出局条款不触发。
4. 改法在打中的格上还中不中：没有打中的格；「checker 一起改」那一格上，只改分配器不改 checker 的形态中、两样一起改的不中。

## G1　新根段加三个字段：施加出来的根与当初会写的根

### 量法

`ring-window-proto/src/bin/fill_bytes_by_kind.rs`：每一种发布录一条流，在第 n 次根槽 FUA 写之前切（之前的写全落、之后一个不落），在拷贝上做一次可写挂载取读阶段交出的 `chosen_root` 与 `effective_root`，与那次根槽写的字节逐字段比；再把施加出来的根照 P1 写进 slot(h)（同盘同偏移）跑池级 checker；再把三个字段（实例表指针、分配记录树根、卸载记号）换成当初的比字节。S = 4 与 S = 8 各跑一遍，两份逐行同形（`results/g1-s4.out`、`results/g1-s8.out`）。

### 量到的（S = 8，`results/g1-s8.out`，`differing_fields` 一列原样）

| 发布 | 记录施加了没有 | 施加出来的根与当初的差在哪 | 照 P1 补施加出来的根之后 checker | 三个字段换成当初的之后 |
|---|---|---|---|---|
| 覆盖写、带文件那一版上的空发布、准入抬 F 两次、管理员回退、两种暖机 | 施加了 | `[]`，逐字节相同 | `[]` | 逐字节相同 |
| 第一个文件，池上第一次挂载之后 | 施加了 | `[]` | `[]` | 相同 |
| 第一个文件，池上已有 2、3、4 次挂载 | 施加了 | `["allocation_record_tree_root"]` | `["I-3.9", "I-3.11"]` | 相同，`[]` |
| 卸载那一串第 1 次发布（现行 txg 7、8、9 三个起点） | 施加了 | `["unmount_marker"]` | `["I-7.9"]` | 相同，`[]` |
| 卸载那一串第 2、3 次发布 | 施加了 | `["unmount_marker"]` | `[]` | 相同 |
| 写行（带文件那一版上；树表 0 条那一版上） | **没施加**：`chosen=(1,7) effective=(1,7)`、`chosen=(3,7) effective=(3,7)` | — | 不适用（没有可补的） | 不适用 |

原样两行：

```
G1 kind=first_file_after_2_mounts nth=0 txg=5 chosen=(2,4) effective=(2,5) record_applied=true differing_fields=["allocation_record_tree_root"] rebuilt_bytes_equal=false three_fields_corrected_bytes_equal=true red_crash_state=[] red_after_fill_rebuilt=["I-3.9", "I-3.11"] red_after_fill_three_fields=[] red_after_fill_true_root=[]
G1 kind=row_on_file_version nth=0 txg=8 chosen=(1,7) effective=(1,7) record_applied=false differing_fields=["instance", "checkpoint_txg", "tree_table", "instance_table", "mapping_root"] rebuilt_bytes_equal=false three_fields_corrected_bytes_equal=false red_crash_state=[] red_after_fill_rebuilt=[] red_after_fill_three_fields=[] red_after_fill_true_root=[]
```

### 三种逐个

1. **写行：三个字段补不了，因为写行那次的记录从来不被施加。** 写行是新实例的第一次发布，它崩在记录已持久、根槽没持久时，下一次挂载的所选根一定是上一个实例的，而恢复「实例代号与所选根不同即停」：「注 1：实例代号与所选根不同即停」（`crates/singlefs-replay-prefix/src/lib.rs:125`），判法本身「record.instance == root_instance && record.checkpoint_txg.0 > root_checkpoint_txg.0」（`crates/singlefs-replay-prefix/src/lib.rs:131`）。两种池上量到 `record_applied=false`（上表末行）。所以写行那次留下的洞**就是 K1 那一类**，用户第 1 条里「写行」那一种与第 2 条是同一类洞；`effective_root` 补洞那条路对它一字节都写不出来。
   - 要真补它，只能另起一条「按别的实例的记录重建它的根」的路：那正是实例表指针这个字段唯一用得上的地方。但那样补进去的根 (j, h) 引用 j 写行那次写下的单元，而下一次挂载 k 的分配器从所选根 h − 1 的账重建、不认这些单元：量到 j 写下的 11 个单元在 k 挂载做完之后一个不剩（`results/g4-k1-s4.out` 第 1、2 行，`orphans=[(1, 12), (2, 11)]` 到 `orphans=[(1, 48, …)]`，实例 2 那一项没了，见 G4）。这条根若按有效根对待，就是 G2 判据字面那一种「候选集里的根引用已复用的单元」（推的：没造这条补洞路，只量了单元被盖）。
2. **首个文件：差的只有分配记录树根，而且只在池上已有过不止一次可写挂载时差。** 第一次挂载之后的树表 0 条那一版分配记录树根是空的，与带文件的一版同为空，所以第一行 `first_file` 不差；第二次挂载起写行那次给树表 0 条那一版建了分配记录树，第一个文件那一版恒为空（「带文件的一版恒 [`NodePointer::empty_root`]」（`crates/singlefs-core/src/root_record.rs:67`）），施加时照抄所选根（「allocation_record_tree_root: root.allocation_record_tree_root,」（`crates/singlefs-core/src/recovery.rs:3173`））就差了。补错根之后 I-3.9、I-3.11 红（量过）。
3. **卸载：差的只有卸载记号。** 施加时恒写「unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,」（`crates/singlefs-core/src/recovery.rs:3163`）。补错根之后只在那一串第 1 次发布上 I-7.9 红，第 2、3 次不红（量过，三个起点都这样）。F 由新根段带着，与当初相同（`G1 detail` 行 `true_floor=7 effective_floor=7`）。

照跑前判据「三种发布里任一种施加出来的根与当初会写的不同 ⇒ 那一种不补」：今天的施加法下首个文件与卸载两种不同；加了字段之后两种都逐字节相同、checker 绿（量过）；写行那一种施加不出来，判据里「施加出来的根」没有对象——**写行那一种按字面不补，归 K1，交第 2 条**。

### 三个字段买到的是什么（按今天 `crates/` 的发布种类）

- 实例表指针：在会被施加的发布里没有一种改它（写行之外的发布都「Carry」上一版的实例表，`crates/singlefs-core/src/mount.rs:2715`、`:2841`、`:5433`），上表 `instance_table_same(chosen,true)=true` 在每个被施加的种类上成立。它只在写行那一种上差，而写行那一种不施加。**这 86 字节今天买不到任何一格**（量过：上表）。C597 删行若落在写行那次发布，仍是这样；若落在别的发布，才用得上（推的）。
- 分配记录树根：被施加的发布里只有「第一个文件」改它，改成的值恒为空。今天一位「这一版带文件 ⇒ 分配记录树根为空」就够（推的：只覆盖今天的发布种类；以后若有「删掉最后一个文件、回到树表 0 条」那种发布，这一位不够）。
- 卸载记号：一位。
- 所以今天要的只是两位。记录头的「记录标志」那个字节现在只用位 0，其余位写 0、读到非 0 当损坏（`.claude/kb/decisions/23-journal的角色与格式.md` 冻结副本第 152 行：「记录标志 1（位 0 = 本次发布末条，已定项 17；其余位写 0，读到非 0 当损坏）」；代码 `crates/singlefs-core/src/journal.rs:28`）。把位 1、位 2 用掉，记录头仍是 311 字节，点名项仍是 67。这是我提的形态，**只在我的模型上量过（字段差只有这两样）、被攻过零轮**；它也是格式改动（读者规则「其余位读到非 0 当损坏」要改）。

### 点名项 67 → 64 的后果（算术，三个字段按第一轮正推腿的 86 + 86 + 4 = 176 字节）

- 一条记录：(4096 − 311) ÷ 56 = 67.6 → 67；(4096 − 311 − 176) ÷ 56 = 64.4 → 64（`crates/singlefs-format/src/lib.rs:213` 的 67）。
- 写行那次点名整条实例表链，一片 369 行：单条记录装得下的行数 67 × 369 = 24 723 → 64 × 369 = 23 616，超过就是多记录发布。
- 一次发布的记录条数上限按默认环 805 306 368 ÷ 4096 ÷ 3 = 65 536 条：一次发布最多点名 65 536 × 67 = 4 390 912 → 65 536 × 64 = 4 194 304 项。
- 写请求按 67 切的那一档（`crates/singlefs-core/src/write_request_split.rs:201` 注释）跟着变。
- 两位方案上面四行都不变。

### 格式改动要同步的地方（`grep` 现查，冻结副本）

- 代码：`crates/singlefs-format/src/lib.rs`（`JOURNAL_HEADER_BYTES` 第 207 行、`JOURNAL_NEW_ROOT_SEGMENT_BYTES`、`JOURNAL_NAMED_ENTRIES_PER_RECORD` 第 213 行与它们的断言）、`crates/singlefs-core/src/journal.rs`（写与解析、67 那两处测试注释第 509–511 行）、`crates/singlefs-core/src/recovery.rs`（施加处第 3160–3174 行）、`crates/singlefs-replay-prefix/src/lib.rs`（新根段来源）、checker 的 `crates/singlefs-checker/src/lib.rs` 与 `walk.rs`（记录扫描、`versions_applied_only_by_records` 用新根段）、`crates/singlefs-checker-tier/src/crash_judge_tables.rs` 与 `src/bin/e165_replay_bounded_enumeration.rs`、用例 `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs`（`grep -rln 'JOURNAL_HEADER_BYTES\|JOURNAL_NEW_ROOT_SEGMENT_BYTES\|new_root_segment' crates` 命中这几份）。
- kb：D23 已定项 4（头的字段与偏移）、已定项 15（新根段与「实例表单元指针照所选根」）、已定项 17；`layout/01-first-txn.md`、`freeze-layer-membership.md`、`invariants.md` 的 I-8 一类，及 E23、E75 等引了 311 的实验页（`grep -rln 'JOURNAL_HEADER_BYTES\|新根段\|311 字节\|头 311' .claude/kb` 共 29 份）。两位方案只动 D23 已定项 4 与 17 的读者规则与 journal.rs 的标志位解析。

## G3　S-辨（推的为主，量过的只有一条段序列）

1. **S-辨 的条件里「另一槽是上一世代」读者判不了，等于没有这一句。** 系统配置槽的槽位 = 世代号 mod 2（第一轮攻方 H4 第 1 条），一块盘上自证过的那一槽世代 g 住 g mod 2，另一槽不管是「撕裂的 g + 1」还是「烂掉的 g − 1」都住 (g ± 1) mod 2 = 同一个槽，自证不过的槽又读不出自己的世代号。所以 S-辨 能分的只有「读得出但自证不过」与「设备读错」两样，与上一轮 L2 表里的甲-读错是同一个判别子（推的）。
2. **丢写回来多少：跟甲-读错同形，上一轮量的是 120（全是 `Zeros` 形），不超过它。** 上一轮那份模型（`unreadable-at-mount-r2-opus-model/c331-summary-with-dev.txt`，已归档，`git show dbcc0d2f:research/prompts/unreadable-at-mount-r2-opus-model/c331-summary-with-dev.txt` 读到）的 `jiaeio` 一节：`lost_cells=120 over_refused_cells=284 today_lost_now_not_lost=120`，丢的 120 格最少要 4 个故障区间（`fewest_fault_ranges=4`：两盘最新那一槽读成全零、最新根与一份数据也藏起来）。S-辨 若再加「另一槽自证过」这一条，只会比甲-读错多拒、不会多放（推的）。照跑前判据「超过 120 ⇒ 交用户重定」，没打中。这是旧数，没重跑。
3. **S-辨 写成「池里只有一槽坏」就漏掉一个合法崩溃状态：两块盘的轮换一起撕裂。** 发布末尾的系统配置轮换两块盘的写落在同一段里（量过：`results/k1-count/count-s4-off.out` 的段表第 5 段 `system_configuration_slot+system_configuration_slot`），层 0 的原地覆写三态让两块盘各撕一槽是枚举域里的状态（上一轮 L2 模型的 `torn_both` 前缀就是它）。S-辨 的字面「一槽撕裂」若按全池读，这一态照旧永远只读，C596 没解决；要按盘读（每块盘至多一槽自证不过、另一槽自证过），就是上一轮攻方提的「甲-读错-每盘」（零轮，那一轮量到多拒 98、丢写同为 120）。**这一句的读法要在写回时定**。
4. **一槽持续设备读错（S-辨 管不到的那一格）**：S-辨 把设备读错留在「读不出」那边，所以这一格仍是永远只读（上一轮 L2 表「甲-读错」那一行最后一列）。出路有两条，都没量：
   - 按盘读的甲-读错-每盘：一块盘一槽设备读错、另一块盘两槽都自证过时放行（上一轮多拒 98、丢写 120，零轮）；
   - 照第一轮攻方 H4 第 1 条，取号写本来就落在坏的那一槽，放行之后那一槽被重写一次；若设备读错是持续的坏扇区、重写后仍读错，每次挂载照旧走同一个判。
   两条都是推的；我倾向前者，因为后者对「持续」那一半没有作用。

## G4　清扫器与删行

### 「连崩挂载留下的 (j, 0, 0) 行正是删不掉的那一类」——一半成立

量的（`orphan_units_probe`，S = 4，K1 与连崩两次两段历史；判「实例 j 的未发布单元」= 单元头读得出、写序里的实例代号 = j、且不在环里任何一条读得出的根引用的槽里，扫盘 0 的 2000 个槽）：

```
PROBE s=4 history=k1 step=crashed rows=[] orphans=[(1, 12), (2, 11)]
PROBE s=4 history=k1 step=mounted instance=3 rows=[(1, 15), (2, 0)] orphans=[(1, 48, [50180, 50184, 50186, 50188, 50240, 50242])]
PROBE s=4 history=k1x2 step=crashed rows=[] orphans=[(1, 12), (3, 11)]
PROBE s=4 history=k1x2 step=mounted instance=4 rows=[(1, 15), (2, 0), (3, 0)] orphans=[(1, 36, [50180, 50186, 50188, 50240, 50242, 50244])]
```

（`results/g4/g4-k1-s4.out` 第 1、2 行，`results/g4/g4-k1x2-s4.out` 第 1、2 行；实例 1 那一项是已发布过、出了环的旧单元，不是未发布单元。）

- 成立的一半：崩溃之后、下一次挂载 k 的写行之前，j 写行那次的 11 个单元都在盘上；k 在取号之前清扫，看到它们，(j, 0, 0) 这一次删不掉。
- 不成立的一半：k 的写行那次把 j 的单元全部盖掉了（k 的分配器从同一个所选根重建，提交内生块落在同一个「最低的全空段」，「开放段没有或装不下时这块盘给提交内生块的去处：最低的全空段；没有全空段就回落；连回落的空槽都没有就 `None`。」（`crates/singlefs-core/src/allocator.rs:794`）），挂载做完时实例 2（连崩时实例 2、3）的未发布单元为 0 个。所以 (j, 0, 0) 在 k 之后的下一次可写挂载就满足 I-3.8 草案的前一半，晚一次挂载删得掉。连崩时（每次写行都崩）一行都删不掉，但那时也没有一次发布做成。
- 只扫了盘 0、2000 个槽、两段历史；j 写下的单元比 k 多时（推的：k 的实例表只会更长，不会更短）会剩下几个。

### 最小形态（推的，按今天的代码谈）

- **读什么**：每块盘单元区里每个槽的单元头（写序里的实例代号与事务号）；根环每一槽；最新根的实例表。判「实例 i 的未发布单元」= 写序 (i, n) 而 n > 行 (i, Ti, Wi) 的 Wi，或 i 的行是 (i, 0, 0)。
- **什么时候跑**：可写挂载读阶段之后、取号之前（只读，取号之前拒的承诺「盘上逐字节不变」不受影响）；删行并进这次挂载的写行那次发布（写行那次本来就 COW 重写整条链）。
- **跑一次多久**：正比于单元区的槽数。副本的盘 4 GiB，单元区从第 50 176 槽起，单元区槽数不少于 4 GiB ÷ 16 KiB − 50 176 = 211 968（算的；50 176 是 `window_static_check` 调试输出里 R 引用的最低槽号，单元区起点不高于它）；真实盘按容量线性放大（推的，没计时）。
- **读失败、设备不全**：I-3.8 草案要「那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功」，任一不成立这一次不删、下一次挂载再扫。撕裂的单元头读不成单元，不算「该实例的单元」，也不算读失败——它的字节已经不是一个单元，删行不会让它被谁当成已发布（推的）。
- **崩在清扫与删行中间**：清扫只读，崩了什么都没留下；删行在写行那次发布里，崩在它的记录已持久、根槽没持久时就是 K1，记录不被施加，表原样，下一次挂载重扫重删。
- **删错行的形状**：没找到。删 (i, Ti) 之后 i 的根按「表里没有 i 的行 ⇒ 有效」变成有效，所以「根环里没有该实例的根」那一条必须在同一次读里判，与删行落在同一次发布、中间没有 i 的新根写进环（i 是死实例，推的）。

## 几个改法各修哪一格

「量过」= 副本这一次的产物里有那一格的数；「推的」= 按代码推、没实现或没跑。我提的形态只在我的模型上量过或只是推的，被攻过零轮。

| 改法 | G1 首个文件 | G1 卸载 | G1 写行 | G2 候选集引用已复用单元 | G2 合法盘面 checker 红 | K1 放槽 |
|---|---|---|---|---|---|---|
| 今天（不补、不开窗口） | 不补 | 不补 | 不补 | 不中（量过，锚无关） | 0（量过） | 0 |
| 补，新根段三个字段（用户第 1 条） | 逐字节相同（量过） | 逐字节相同（量过） | 施加不出来，补不了（量过） | — | — | 0（K1 补不了） |
| 补，记录标志位 1、2 两位（我提的，零轮） | 相同（量过：差的只有这一项，换掉即同） | 相同（量过，同左） | 补不了（同左） | — | — | 0 |
| 一圈窗口，锚在环里最新根（用户第 2 条，按上面的式子） | — | — | — | 不中（量过，见扫描） | 0，checker 同改（量过）；checker 不改 9/17、21/29 步红（量过） | 峰值差 144 / 284（S = 4 / 8，量过） |
| 一圈窗口，挂载时锚在 first_txg − 1 | — | — | — | 不中（量过，今天的落点政策下）；落点政策改了会中（推的） | 0（量过） | 同上 |
| 一圈窗口，回退候选集不跟着改 | — | — | — | 回退到 R：今天的回退核拒（量过，见下） | — | — |
| 一圈窗口，影子账候选过滤不跟着改 | — | — | — | 被抛弃根与 R 共用的槽会被放出（推的） | — | — |

## 没打中的形状

- G2 字面（候选集里的根引用已复用的单元）：两批全域静态判法共 900 段历史、220 788 次单元写（两种锚各算），锚在环里最新根与锚在 first_txg − 1 各一半，0 处（见 G2「扫描」）。历史：K1、连崩两次、可补而不补的后洞；S ∈ {4, 8, 16}、洞 txg 取满一圈多；被判那次挂载之后接 0 或 3S + 4 次覆盖写、再接或不接「回退到窗口里最旧的候选根 + 2 次覆盖写」。
- 锚在 first_txg − 1 的陷阱：推得出（挂载时就把 R 放出去，而崩在写行那次时环里最新根仍是 h − 1、R 还在窗口里），量不出：写行那次只写提交内生块，提交内生块只进「最低的全空段」，窗口在挂载时多放的那一代单元从来没把一整段放空；用户数据走「起点槽号最小」的落点（`crates/singlefs-core/src/allocator.rs:729`），但第一次用户写在写行那条根落盘之后，那时 R 已出窗口。想用小盘逼分配器走回落落点，mkfs 拒「journal 环超过设备容量的四分之一」，没做成。
- 回退到 R（回退候选集不加窗口）：今天的回退核在 R 的单元被复用之后每一次都拒（`UserVisibleUnitStillAllocatedUnderAnotherGeneration`、`ResurrectedUnitCopyUnreadableOrMismatched`、`TargetVersionUnreadable`），复用之前放行而 checker 绿；没找到放行而读错的那一步。
- G1：七种被施加的发布里，差的只有「首个文件的分配记录树根」与「卸载记号」两样；没找到第三样。
- G4：没找到「清扫在合法状态下删掉仍被要的行」的形状；只造了 K1 与连崩两段。

## 这条腿自己的限度

- 全部是副本上的数，`crates/` 没动；副本的窗口是我写的原型（`core-and-checker.diff`），不是用户第 2 条的实现。
- G2 的全域判法依赖两条前提：含单元写的段里只有单元写（发布路径按单元、屏障、记录、屏障、根槽 FUA、轮换、屏障的次序写，见 `persist_publish_writes`（`crates/singlefs-core/src/transaction.rs` 第 1057 行起）；`results/k1-count/` 的段表照此）；一条根引用的槽取它那一版账里仍分配的记录加实例表、树表两条指针（`placements_of`）。账读不出的根会少算引用。
- 判法的会红自证：把候选集的窗口拿掉（`STATIC_NO_WINDOW=1`），K1、S = 4、覆盖写 16 次那段历史上 32 次单元写打中 R（txg 4），那一个状态上今天的 checker 红 `I-2.1、I-3.1、I-3.10、I-4.8、I-7.4`（`results/selfred/selfred-k1-s4.out`）。
- 「没打中」只抽了一次样（一条攻方腿），照规则不支撑否定结论。
- 负载只有单个 3000 字节文件的覆盖写；没有被抛弃根、没有读错注入、没有多文件、没有删除。
- G3 全部是推的加上一轮的旧数；G4 只扫了盘 0 的 2000 个槽。
- G1 的「两位就够」只覆盖今天 `crates/` 的发布种类。

## 什么现象会推翻这些结论

- G1 写行补不了：恢复改成施加别的实例的记录；或者在入库装置上造出一次写行那次的记录被施加。
- G1 两位就够：出现一种会被施加、又改实例表指针或把分配记录树根改成非空的发布（例如 C597 删行落在写行之外，或「删掉最后一个文件」）。
- G2 没打中：在入库实现上找到一段历史，某个单元写盖住的槽被那一刻候选集里的根引用；或落点政策改成提交内生块也取最低空槽之后，锚在 first_txg − 1 的那种在 K1 历史上中。
- G2 放槽量：换负载（多文件、大文件）之后同一步之差不是这个量级。
- G4 一半成立：在入库装置上 k 挂载之后仍扫得到 j 的未发布单元。

## 没做什么

- 没跑 checker 档测试、门禁、变异、入库的 E156；没改仓里 `crates/` 与 kb，没做 git 写。
- 层 0 的逐状态枚举（`window_k1_states`）只做了计数：K1、S = 4 的挂载 k 只截到写行那次发布就有 4 194 324 个状态（写行那次 22 个单元写在一段里），超过约 10⁶，照派发的上限不跑；G2 用的是上面的全域静态判法，它判的是同一个枚举域，但不经枚举器自己的恢复、oracle 与记录核对器。这一条腿跑的全量状态数合计 0。
- 没造被抛弃根、读错注入、多文件、删除的历史；影子账那一格是推的。
- 没实现 P1 补洞，也没实现三个字段或两位标志；G1 的「加字段之后相同」是把三个字段从当初的根换进施加出来的根再比字节，不是改了记录格式再跑。
- G3 一格没量；用的是上一轮已归档的旧数。
- 没抽第二次样（`-s2`）。不判辩方那几格，不替主 agent 采纳。
- 跑批：G2 第一批 612 段与第二批 288 段各一个后台批，每段一次进程、约 2 秒（先跑一段计时：`window_static_check 8 k1x2 40` 2.02 秒、峰值 69 508 KB），没设限时。第一批头一次起跑时内存包装排不上（`rc=252`，上限写了 8G、整机账上占满），156 段里 62 段没跑，整批作废重跑；作废的那份原样留在 `results/sweep/sweep1.aborted-wrapper252.txt`。
- 第一批跑到 S = 16 时我重编了两次原型（加了 `STATIC_NO_WINDOW`、`PROTO_IMAGE_BYTES`、`SINGLEFS_ROLLBACK_NO_WINDOW` 三个开关）；三个开关都不设时行为不变，S = 16 那 324 段有一部分是新编出来的二进制跑的。
- 草稿清理：删了 `/tmp/claude-1000/m3-ring-r2-opus/target`（编译目录，1.1G）、`/tmp/claude-1000/m3-ring-r2-opus/repo`（仓副本，21M）、`/tmp/claude-1000/m3-ring-r2-opus/replay`（复跑验证用的仓副本连编译目录，1.1G）。留下的只有日志、脚本与产物原件（784K，产物已拷进模型目录）。前任没建过编译目录与仓副本。
- 引文核对：`python3 research/scripts/cite-check.py research/prompts/m3-ring-r2-opus-output.md --root /tmp/claude-1000/m3-batch4-freeze/tree` 末行「✓ 核了 11 处引文，对上 11 处，没判 0 处（逐处列在上面）」。不带 `--root` 时 `crates/singlefs-replay-prefix/src/lib.rs` 那两处对不上：主工作区这份文件在这一条腿开工之后被别的会话改过（第 125、131 行移到第 127、133 行），被判的是冻结副本，行号照冻结副本写。
