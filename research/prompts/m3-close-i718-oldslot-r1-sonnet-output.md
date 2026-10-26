# m3-close-i718-oldslot-r1：云端正推腿（Sonnet）报告

日期：2026-10-23。立场：从附录条款与第二节的代码今天的样子正推，三个候选（甲、乙、丙）各自改哪几处、各自判别力，推出该取哪一个。
分段落盘；引 kb 写 kb 文件自己的行号（现查）；量的数标「量的」、推的标「推的」。报告尾「cite-check 末行」是交回前跑的结果。

## 0 结论（先写，依据在后面各节）

1. **取甲，但要带「种子边」一处修正**（`next_txg >= 环长`，不是投稿补丁的 `>`）。丙不取。乙是甲的轻量替身，可作退路，不是首选。
2. 依据一句话：目标态（满环、最新根 N 之后那一槽读不出自证过的根）上，今天池级 checker 里 I-7.18 与 I-3.1 **没有一个 t 能同时判绿**（量的：N=14 的镜像上 I-7.18 只认 t=4，I-3.1 只认 t=3）；
   甲让 I-7.18 在这一态多认一个值（N+1−3S），两条不变量的交集正好是写者写的那个 t；乙让 I-7.18 在这一态不判，t 由 I-3.1 独自钉住（量的：这几族历史里 I-3.1 钉得住，p 谁都钉不住）；丙留一个单故障合法态上的红。
3. 实现落点：只动池级 checker 与 GPU 走读内核各一个函数，写者（`crates/singlefs-core`）一行不动。三份补丁（CPU 与 GPU 各一份）和量具都在模型目录，补丁在冻结副本上量过、在今天的主工作区上 `patch --dry-run` 过。
4. 这一轮量到的、会改变主 agent 决定的三件事：
   - 投稿的甲补丁漏一个边：N+1 = 3S（环第一次转满、写掉 mkfs 种的第 0 代根）那一态甲仍红（量的：N=11，S=4）；`>=` 修掉它。
   - 事后读坏「环没满时最旧那条根」的镜像三个候选都罩不住（量的：N=3、5、7、9、10 全红）；这不是 N+1 的槽，题面里的三个候选都不是为它设计的。
   - 甲的「两种现算之一」放过的错 t 只有一个值（t = N+1−3S），而且在目标态上这个值正是 I-3.1 要求的那个；「p 写成 255 而其实钉着下一槽」那一形只在第一圈（N+1 ∈ [3S, 6S)）放过（量的）。
5. 推翻本节结论的现象：见 §7。

## 1 机理正推（读过的实现，行号按主工作区，今天现查）

| 编号 | 事实 | 出处 |
|---|---|---|
| E1 | 写者记一条根时先把这条根写进内存里的根环表，盖掉它那一槽的旧住户，再取表里最小的 txg 当回收门槛：满环时记 N，表里剩 N−3S+1…N，门槛 = N−3S+1 | `crates/singlefs-core/src/allocator.rs:2742` 「.insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」 |
| E2 | 钉槽 p 在同一刻采样：记这条根之后还钉着的槽里钉值最小的那个 | `crates/singlefs-core/src/allocator.rs:2756` 「let lowest_pinned_txg = occupancy」 |
| E3 | 根记录的 t = max(分配器记根时的门槛, 这条根自己带的 F) | `crates/singlefs-core/src/allocator.rs:1806` 「self.reclaim_floor.max(rollback_floor_of_the_root)」 |
| E4 | 钉值是「这个槽可能住过的最旧 txg」，实现取 [0, 3S) 里同一槽的最小 txg，所以钉值就是槽位置，永远 < 3S，装得进一个字节 | `crates/singlefs-core/src/mount.rs:2883` 「一个槽最小的 txg 落在第一个周期里」 |
| E5 | 池级 checker 现算：镜像上全部自证过、按最新根实例表仍有效的根的最小 txg，并上钉值，再与 F 取大；求等 | `crates/singlefs-checker/src/walk.rs:8525` 「newest_root.reclaim_threshold == recomputed_threshold,」；共用函数 `crates/singlefs-checker/src/walk.rs:8366` 「fn deferred_slots_reclaim_threshold(」 |
| E6 | 最新根按 (txg, 实例) 取最大，不滤被抛弃的 | `crates/singlefs-checker/src/walk.rs:18091` 「let newest_index = roots」 |
| E7 | I-3.1 另有一道「钉带出来的那一截」：最新根带的 t 低于不带钉现算的门槛时，释放代落在 (t, 门槛] 的已释放槽并进遍历 | `crates/singlefs-checker/src/walk.rs:18813` 「if accounting_seen && newest_root_view.reclaim_threshold < deferred_threshold {」 |
| E8 | GPU 走读内核同一个函数，现算的写法与 CPU 逐句对应 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4606` 「judge(INVARIANT_I_7_18, eq64(reclaim_threshold, recomputed_threshold));」 |

推（由 E1–E6）：满环时写者记 N 的 t 是 N−3S+1；N+1 的发布把根槽写在 N−3S+1 那条根住的槽上。这次写成 0 或撕裂、进程崩，镜像上的环只剩 N−3S+2…N，
checker 现算 N−3S+2，比 t 大 1。D16（发布语义）已定项 1 的 `.claude/kb/decisions/16-发布语义.md:36` 「写失败的槽按旧内容算，在飞、没持久的发布不算」是写者一侧的口径，镜像上看不到被写坏的旧内容。
量的（§2 的量具）：S=4、环长 12 的池，N=14 时写者的 t=3，目标态镜像环最小 4，I-7.18 红，**其余不变量一条不红**（`other_red=[]`，24 个 N 里每一格都核了）。

## 2 量具与复跑

- 冻结副本：开工时从主工作区拷的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`（不带 target）。拷的时候 `walk.rs` 的 sha256 是 `e30ca36bc489a5c9a1b97ae87f6e51501d93c67504289f2180548078f44ecbc3`，
  `crash_judge_checker.wgsl` 是 `830adca581da546bc17a537e7581ae8edddacfb4a476611d14703cddccab9f00`，`allocator.rs` 是 `3d5a0296f97192ae064bd09f0996dc1b05c1663fdfeabcb2b095d95ba473d79a`。
  交回前再核：`walk.rs`、`allocator.rs` 与主工作区逐字节相同；`crash_judge_checker.wgsl` 在主工作区被别的会话改过（别处加了几行，与 I-7.18 那个函数无关），三份 GPU 补丁对今天的主工作区 `patch --dry-run` 全过（「Hunk #1 succeeded at 4603 (offset 6 lines)」）。
- 模型目录 `research/prompts/m3-close-i718-oldslot-r1-sonnet-model/`：`patches/`（候选补丁与量法开关补丁）、`probe/`（量具两份）、`tools/`（`rerun.sh`、`summarize.py`、`closure.py`、`naga-check/`）、`out/`（汇总、交叉核、naga 校验、补丁 sha256、逐行原始输出 `out/rows/`）。
- 复跑命令（整条经内存包装；全跑十几分钟，前台有超时的会话用 `RERUN_ONLY=<段>` 分段，段名见 `tools/rerun.sh` 里 `wanted` 一段的注释）：
  `bash scripts/hosts/run-with-memory-cap.sh 12G bash research/prompts/m3-close-i718-oldslot-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>`
  跑到最后打汇总，并与 `out/summary.txt` 逐行比。量法开关 `I718_VARIANT`：空或 baseline = 基线；`a` = 投稿的甲（`>`）；`a_seed` = 甲含种子边（`>=`）；`b` = 乙（`>=`）；`b_strict` = 乙按 `>`。
  同一份编译里切开关省了反复编；每个候选自己的补丁（不带开关）另编一份、跑同一组量具，输出与开关版同名变体逐行比（量的：`out/cross-check.txt` 21 组全部「逐行相同」，0 组不同）。
- 补丁与量具的 sha256（`sha256sum` 现算）：
```
5984e46e9ef49452ea8a284285c86489a091a7bbdd0c53b30c8ba2c181bc3894  README.md
d01d5d28648998ed364d7ad27b99c9fb84da1658c3908fb0e11dab9f4e8077f7  patches/candidate-a-cpu.patch
365c469b59fae1c9a1080088f9df3180b9b113bedd839f94ce3a61681f7480d3  patches/candidate-a-gpu.patch
4ba60b4897433d9d4cfd9684af2a4464d45bc2a2da44e00148f74b5b174b5455  patches/candidate-a-seed-edge-cpu.patch
6e724f7521ce488e30ec36483351cb716c2656640a2100a9af53dba9356db966  patches/candidate-a-seed-edge-gpu.patch
19423c623c2579ab4b86b226120df8e0d0c9339818ac3b1a5ac0e28a04e84458  patches/candidate-b-cpu.patch
4d9de3f0b1fc819deb63971647715afaef521c421882c194f88c815476c59ac4  patches/candidate-b-gpu.patch
c2610d3c72cf76902977adf5dbce5aa8a120a968a6aaf6609895c268643cdcea  patches/measurement-switch.patch
21c31c1d09835b739d7d8904a3210c7e102f500d6bbb5dc5beee9a04fabeb5c5  probe/probe_i718_baseline.rs
e03efa514287364c66c0d69595210cbdfb4b07e899cc211abc76bbcd21592aff  probe/probe_i718_oldslot.rs
e57c99157e1f7a5da5f04625532dc873d2c5fa92621c3c0e17618789c1194ca1  tools/rerun.sh
19b5752c619b0797c5e85f9a71dc20fc335da53e5a99fb7d971f379a6daf42ea  tools/summarize.py
76e393418613f812aa6f33bb8b9d39732ff3e683f729627aa1c46f272e239eef  tools/closure.py
92435ea2a35dddf305ccd4ed1e9f804f40eb97a25684a74f76a8bc099539187e  tools/naga-check/src/main.rs
b4221dab94a6d5b81ab52c9645f92756b1e6aca10dfe6257597b56e30403d8c7  tools/naga-check/Cargo.toml
64c79f687ad45b5aae7b5fcd15449b8a3b326b7089ed210f313adf253a65d5b6  tools/naga-check/prelude-extra.wgsl
feb78931fc848dec69f32e63b788cbedeb4d07b9982c39050117c25470ac0804  out/summary.txt
dbb62817ee2f9dfaec4c4bb66314aaaf3c353ea5b7076b2f757a565e0c9fe2bb  out/cross-check.txt
e3db7c76c892372a4bbec190dcbf0fe51c47e65cb71953d7c54bc28950a2ad4a  out/naga.txt
f8b0571bf096631dc2fca76758665486ecda1fe2b49a3fa5bb51208682f63859  out/patch-sha256.txt
```

（`out/summary.txt` 等产出文件的 sha256 在交回正文里再给一次；`rerun.sh` 末尾对 `out/summary.txt` 逐行比，对不上会指到差异文件。）

## 3 V1 判别力

**判定**：甲在目标态上多认一个 t（N+1−3S），别处一格不动；乙在目标态上整条 I-7.18 不判，t 改由 I-3.1 独自钉住（这几族历史里钉得住）、p 无人钉；丙一个都不多放，但对合法的单故障态判红。三者里甲放过的最少。

### 3.1 量的：错 t 镜像上各变体判成立的 t（`probe_i718_oldslot.rs` 第三、五、七组；最新根的 t 与钉槽 p 改成给定值并重封，只改根槽，不改 journal 记录）

| 盘面（S=4，环长 12） | 真写者 t | 基线 / 丙 | 甲（`>`） | 甲含种子边 | 乙 | I-3.1 单独判绿的 t（p=255） |
|---|---|---|---|---|---|---|
| N=14，N+1=15 的槽写成全零，p=255，t 扫 0..16 | 3 | {4} | {3, 4} | {3, 4} | 不判（17 个 t 全不判） | {3} |
| 同上，p=3（钉的就是 15 的落点，位置 3） | 3 | {3} | {3} | {3} | 不判 | — |
| 同上，p=4、5、11、12 | — | {4} | {3, 4} | {3, 4} | 不判 | I-7.19 另红（p 指着的槽住着本实例的根，或 p 不小于环长） |
| N=14，N+1 的槽完好（对照） | 3 | {3} | {3} | {3} | {3} | {3, …, 16} |
| N=14，第二旧的槽（txg 4）写成全零（对照） | 3 | {3} | {3} | {3} | {3} | 无 |
| N=26（第二圈），N+1=27 的槽写成全零，p=255，t 扫 0..18 | 15 | {16} | {15, 16} | {15, 16} | 不判 | {15} |
| 同上，p=3、p=4 | — | {3}、{4} | {3}、{4} | {3}、{4} | 不判 | — |
| 偶数 txg 空发布的历史，N=14 / N=15，p=255 | 3 / 4 | {4} / {5} | {3, 4} / {4, 5} | 同甲 | 不判 | {3} / {4} |

出处：`out/summary.txt` 第三、五、六之二节；逐行原始输出 `out/rows/group_wrong_t_and_p_mirrors-*.txt`、`out/rows/group_second_revolution_wrong_t_and_p_mirrors-*.txt`、`out/rows/group_wrong_t_mirrors_on_a_history_with_empty_publishes-*.txt`。
读法：基线在目标态上认的是 N+2−3S（环里最小的现算值），写者写的是 N+1−3S，**两者不等，所以基线对所有诚实的写者判红**；I-3.1 单独只认 N+1−3S。

### 3.2 甲的「两种现算之一」把错 t 判成对的镜像

推的形式（由 CPU 补丁 `candidate-a-seed-edge-cpu.patch` 的式子）：多认的值是 max(F, min(镜像有效根 ∪ 钉值 ∪ {N+1−3S}))。它与基线不同，当且仅当下面四条同时成立：
最新根之后那一槽读不出自证过的根；N+1 ≥ 3S；N+1−3S 低于镜像有效根与钉值的最小值；N+1−3S 高于 F。四条之外一个值也不多认（量的：上表对照盘面与 t 扫描里，成立的 t 集合与基线相同）。
放过的真错写者（量的 = 表里的 t；写者怎么错的 = 推的）：
- 把「记根时不算有效」的住户当成有效根并进门槛，写出 N+1−3S 而真值是 N+2−3S：目标态镜像上与诚实写者逐字节相同；同一段历史里 N+1 的槽还完好的那张镜像基线与甲都判红（量的：对照盘面只认 3），所以只在「只有目标态这一张镜像」时放过。
- p 写成 255 而其实钉着下一槽：第一圈内（N+1 ∈ [3S, 6S)）钉值 = 槽位置 = N+1−3S，坏写者与没有钉的诚实写者写出同一份字节，甲放过（量的：N=14 的 p=255 认 3）；第二圈以后钉值（槽位置）小于 N+1−3S，两个值分得开（量的：N=26 的 p=255 只认 {15, 16}，坏写者会写 3，不认；p=3 只认 3）。
- 拿错环：t 扫描里除了 N+1−3S 与 N+2−3S 一律红（量的）。
**甲不能收成只认 N+1−3S**（推的，没造镜像）：下一槽的住户记根时若是被抛弃的根（不算有效），写者的 t 是 N+2−3S 一支；两种住户在镜像上都读不出，收成单值会把这一支合法写者判红。
这一格还有个量的边：N+1 = 3S（环第一次转满，写掉 mkfs 种的第 0 代根）上，条件写 `>` 的投稿甲仍红，写 `>=` 的认（N=11：基线 R、甲 R、甲含种子边 Y、乙 N、乙按大于 R；`out/summary.txt` 第一节）。

### 3.3 乙：不适用的判据只看镜像就判得出

是。CPU 补丁只用几何（区域数 R、每区域槽数 S）和 `roots` 列表；GPU 补丁只用 `root_count` 循环和现成的 `checker_root_region` / `checker_root_slot_in_its_region`，不读实例表、不读 journal、不加交接段的字。
判据的粗处：「读不出自证过的根」分不出写失败、事后读坏、环里本来就空的槽；本来就空（N+1 < 3S）已被 N+1 ≥ 3S 排除（量的：N=3…10 的 N+1 槽写成全零，五个变体都成立）。乙放过的是：目标态上 I-7.18 的 t 与 p；
t 由 I-3.1 钉住（量的：见 3.1 末列，覆盖写历史与偶数 txg 空发布历史各一族）。I-3.1 盲的条件（推的）：这一态上没有释放代落在 (N+1−3S, N+2−3S] 的已释放槽。每个建账的发布都重写记账树、释放上一版的记账单元，所以区间里恒有槽；量的只覆盖了这两族，没覆盖零单元发布的历史（那种历史上 I-7.18 本身就不判）。
乙还多一个耦合：目标态上 t 的校验只剩 I-3.1 一家，它那一截（`crates/singlefs-checker/src/walk.rs:18813`）被改坏时，这一态没人看 t。

### 3.4 丙

量的：三个场景（plain、torn、zeroed）共 71 张镜像，基线判红的恰三张，每个场景一张「崩在重发之前、恢复之前」，红的只有 I-7.18（`out/summary.txt` 第七节），其余 68 张全绿。
丙不放过任何东西，但那三张是单故障合法态（只注入一次根槽写故障）。再加 §0 的量的事实：这一态上 I-7.18 与 I-3.1 没有共同绿的 t，所以「改写者让 I-7.18 绿」不是出路——写 N+2−3S，I-3.1 红。

**什么现象会推翻 V1**：(1) 找到一个目标态镜像，其 t 不等于 N+1−3S 也不等于 N+2−3S 而甲判成立；(2) 找到一段历史，目标态上 I-3.1 对 t 不敏感（乙在那里就放过任意 t）；(3) 非目标态镜像上甲或乙的成立 t 集合与基线不同。

## 4 V2 这一态的范围

**判定**：到得了这一态的合法历史，量到的只有「下一次发布的根槽写失败（写成 0 / 撕裂）后进程在重发之前崩」，和事后读坏同一个槽（镜像字节相同）；纯掉电与扇区边界撕裂到不了；
「事后读坏环没满时最旧那条根」是同族的另一个合法态，三个候选都罩不住。甲、乙罩住的范围相同（同一个条件），丙一个也不罩。

| 历史 | 到不到这一态 | 基线 / 丙 | 甲含种子边 | 乙 |
|---|---|---|---|---|
| 下一次发布的根槽写成全零 / 写一半，崩在重发之前 | 到。量的：N=11…26 共 16 个 N × 2 种故障 = 32 张，I-7.18 全红，其余不变量全绿（`other_red=[]`） | 红 | 成立（32/32） | 不判（32/32） |
| 同一个 N，那次根槽写完整 / 根槽写之前就崩 | 不到（槽里是新根或完好的旧根）。量的：48 张全成立 | 成立 | 成立 | 成立 |
| 纯掉电：N+1 那次发布写表的每个前缀（崩点） | 不到。量的：N=11、12、14、15 各 26 个前缀盘面，I-7.18 全成立，五个变体相同。前提是根槽一个原子扇区装得下：`crates/singlefs-checker/invariants/02-根环.md:12` 「每个根槽的起始偏移是判定宽度的整数倍」，判定宽度是声明值未探测（C388，推的） | 成立 | 成立 | 成立 |
| 物理块 4096，根槽撕在扇区边界 | 不到。根记录 466 字节全在头一个扇区，撕出来要么新根整槽要么旧根整槽。量的：既有用例（`crates/singlefs-harness/tests/root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs:605`）在五个变体下都过 | 成立 | 成立 | 成立 |
| 挂载时读坏 / 读回全零的槽（诚实写者带 p） | 不到：钉进了 t 与 p。量的：三个场景恢复之后的 68 张全绿，五个变体相同；C773 的 9 条回归、既有坏镜像 276 条（另 2 条 ignored）、池级 checker 库测试 101 条在五个变体下都过 | 成立 | 成立 | 成立 |
| 事后读坏（位腐）环满时最旧的槽，之后到崩没有新发布 | 到，镜像字节与第一行相同（推的：镜像分不出写失败与位腐） | 红 | 成立 | 不判 |
| 事后读坏环没满（N+1 < 3S）时最旧那条根（含 mkfs 种的第 0 代根） | 到。量的：N=3、5、7、9、10 五格，t=0，环最小变 1 | 红 | **红** | **红** |
| N+1 = 3S（环第一次转满，写掉第 0 代根）的目标态 | 到。量的：N=11 | 红 | 成立（投稿的甲写 `>`：红） | 不判（按 `>`：红） |
| 掉一块盘的降级镜像 | 量的，只当旁证：N=14…17 拿掉盘 0 / 盘 1 共 8 张，基线 6 张 I-7.18 红；甲 8 张成立；乙 6 张不判、2 张成立。这 8 张 I-3.1 一律红（拿掉盘本来就不是干净镜像） | 6 红 | 8 成立 | 6 不判 |
| 撕裂域里别的槽（槽宽小于原子单元，撕到邻槽） | 推的：被 I-7.5 的前提排除；判定宽度声明错（C388）时邻槽可能是最新根，最新根换人，甲乙按新的最新根重判，条件同样只看镜像。没量 | — | — | — |
| 下一槽的住户记根时是被抛弃的根 | 推的：写者 t 是 N+2−3S 一支，基线与甲都认，丙也不红；这是甲必须保留「两种现算之一」的原因（§3.2） | 成立 | 成立 | 不判 |

出处：`out/summary.txt` 第一、二、四、六、七、八节；逐行原始输出 `out/rows/`。

两件要主 agent 知道的事：
1. **环没满时最旧那条根事后读坏**是同一类「记账那一刻的环与镜像上的环不一样」，不在 N+1 的槽上，题面的三个候选都不管它。推的出路（没写没量）：把「那一槽」推广成「环里每个读不出的位置」，
   每个缺位的候选 txg 是同位置在 (N−3S, N] 里的那个 txg（第 0 代根按 txg 0），t 等于基线现算或「并进恰一个缺位后」的现算之一。这是另一个候选，题面没列；提出来只为说明甲不是这一类的终点。
2. 事后读坏与写失败在镜像上分不出，所以「这一态」不能靠原因来判；只能靠位置（N+1 的槽）。如果要把范围收窄到「有 journal 记录点名 N+1 却没有对应根」，要在判器里读 journal，
   CPU 判器的 `walk.rs` 里有记录列表，GPU 走读内核有没有现成的交接要另核（没核）。

**什么现象会推翻 V2**：(1) 找到一个纯掉电的前缀盘面，I-7.18 红；(2) 找到一个诚实写者造出的镜像，下一槽有自证过的根而 I-7.18 红（那就不是这一类，而是写者或判器的另一个缺口）；(3) 4096 字节物理块上撕出一个自证不过的根槽。

## 5 V3 与已定条款的关系

出处都是 kb 文件自己的行号（现查）。

| 条款 | 它说的 | 甲含种子边 | 乙 | 丙 |
|---|---|---|---|---|
| 16-发布语义.md 第 36 行 | 「写失败的槽按旧内容算，在飞、没持久的发布不算」（`.claude/kb/decisions/16-发布语义.md:36`） | 这是写者一侧的口径，不动 | 同 | 同 |
| 同一行 | 「六处都不再按一圈窗口滤根」（`.claude/kb/decisions/16-发布语义.md:36`）；六处里点名「checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛（max(F_生效, 环里最旧有效根)）」（`.claude/kb/decisions/16-发布语义.md:36`） | **要补一句**：目标态上镜像读不到写失败槽的旧内容，checker 一侧只有 I-7.18 的现算并进它，丁-defer 门槛与候选过滤照镜像，差额由 I-3.1「钉带出来的那一截」按 t 补。不补，读者会把并旧内容也加到丁-defer 门槛上（推的：那样 I-3.1 那一截为空，记账多于遍历的红回来） | 建议同补一句（说明镜像读不到），条款本身不矛盾 | 不改；这一行在目标态上对 checker 的三处今天就不是逐字成立 |
| 第 39 行 | 「钉也落盘：根记录带这次发布用的回收门槛 t 与钉槽 p，写者取 t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)；钉着的槽不得住着本实例写的根」（`.claude/kb/decisions/16-发布语义.md:39`） | 不动 | 不动 | 不动 |
| 02-根环.md 第 25 行 I-7.18 | 「checker 取这条根自己带的 F 现算回收门槛，要求等于 t」（`crates/singlefs-checker/invariants/02-根环.md:25`） | 补一支：最新根之后那一槽读不出自证过的根且 N+1 ≥ 3S 时，再按「那一槽按旧内容算」现算一次，t 等于两者之一即成立 | 补一支：这一态报不适用 | 不改，把这一态写进已认代价 |
| 同一行 | 「零单元发布与无文件版本上的复制根，t、p 抄自上一版，环转满之后求等会误红，不判」（`crates/singlefs-checker/invariants/02-根环.md:25`） | — | **有现成先例**：同一句已经用「不判」处理「镜像上的环不是记账时的环」 | 与这个先例的做法相反 |
| 同一行 | 「后者现算的门槛不带钉、比 t 高，求等红」（`crates/singlefs-checker/invariants/02-根环.md:25`）：p 写 255 而其实钉着，判得出 | 第一圈内、钉着的槽恰是下一槽时这句不再成立（量的，§3.2），要加限定 | 目标态上这句不成立，要加限定 | 成立 |
| 02-根环.md 第 26 行 I-7.19 | 「两样故障叠在一起（读坏钉着的槽，这个进程对它的写报失败而实际落了盘）时这一条会误红，超出单故障」（`crates/singlefs-checker/invariants/02-根环.md:26`）：误红只许出在超出单故障 | 一致 | 一致 | **不一致**：单故障下 I-7.18 红 |
| 同一行 | 「用户 2026-10-20 定不记成已认代价、要做成判得出」（`crates/singlefs-checker/invariants/02-根环.md:26`） | 新增一类「镜像上逐字节相同」的小盲点（第一圈内 p），要照 C863 登记 | 盲点更大（目标态上 t 与 p），也要登记 | **与用户 2026-10-20 的态度相反**：丙就是已认代价 |
| checks-owed.md C560 | 「当合法崩溃状态判成立」与「认成已知红」两支；「判别力自证：换回现行判法，那条用例必须由绿转红」（`.claude/kb/checks-owed.md:351`） | 选前一支；自证成立（量的：换回基线，三个场景各一张由绿转红） | 同 | 选后一支；**自证不成立**：现行判法就是红，没有可换回的 |
| 同一行 | C560 行里写的红是 I-3.1 | 要改写：C773 之后这一格 I-3.1 已绿（量的：t=N+1−3S 时 I-3.1 绿），红挪到了 I-7.18 | 同 | 同 |
| checks-owed.md C773 | 「射程：两样故障下（写失败而实际落了盘）钉槽那一条会误红，超出单故障」（`.claude/kb/checks-owed.md:497`） | 一致 | 一致 | 单故障下 I-7.18 红，与这句的口径不一致 |
| checks-owed.md C863 | 「这两类在镜像上与合法的钉逐字节相同（读故障不留在盘上）」（`.claude/kb/checks-owed.md:548`）；用户定不记成已认代价（`records/2026-10-18-里程碑三收尾与近期欠账.md:386` 「不记成已认代价，另立一笔新欠账」） | 同类盲点 +1（小） | 同类盲点 +1（大） | 已认代价 +1 |
| 05-空间记账.md 第 8 行 | 「钉最旧 txg 的代价（用户认的）」（`crates/singlefs-checker/invariants/05-空间记账.md:8`） | 三个场景在甲 / 乙下零红，这段是 C773 之前的现状，要改（候选无关） | 同 | 丙下 I-3.1 也已绿，同样要改 |

别的分项：D22 已定项 7 字段表、D13 比对表、`.claude/kb/layout/01-first-txn.md` 都只把 I-7.18 当作判这个字段的不变量，不复述现算式，三个候选都不用改。D16 在 `.claude/kb/decisions/16-publish-semantics.en.md`、`16-パブリッシュセマンティクス.ja.md` 各有译文，甲的补句涉及中、英、日三份文件，译文要不要同步按 kb 的译文约定（没查）。

**对 `crates/mutations.tsv` 的连带**：I-7.18 有两行变异锚在被改的那一行上（`crates/mutations.tsv:5214` 锚「newest_root.reclaim_threshold == recomputed_threshold,」，`crates/mutations.tsv:5220` 锚「judge(INVARIANT_I_7_18, eq64(reclaim_threshold, recomputed_threshold));」）。
变异表按子串、命中次数必须恰为 1 找原文（`scripts/mutation/lib_mutation_units.py:316` 「if text.count(original) != 1:」）。甲把这两行改了写法，两行会命中 0 次，要重登记（推的：没跑变异表）；乙只把那两行包进分支、缩进变深，子串仍命中 1 次，锚得住（推的）。丙不动。

## 6 V4 两侧判器

**判定**：GPU 走读内核可以照同一写法改，只用现成助手，不加常量、不加交接段的字；甲约 27 行，乙约 17 行。GPU 补丁没在 GPU 上跑，CPU 与 GPU 对拍（checker 档）我没跑，只做了 naga 的解析与类型校验。

- 今天的函数：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4575` 起的 `judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root`，现算后一句 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4606` 「judge(INVARIANT_I_7_18, eq64(reclaim_threshold, recomputed_threshold));」。
  跳过被抛弃的根那个循环在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4590` 「if (w_root_abandoned(r.instance, r.txg)) {」，CPU 一侧对应的是 `standing_in_the_newest_roots_table`。
- 甲在 GPU 里：算 `next_txg = txg + 1`，`divmod64` 两次得落点（区域、槽），循环 `root_count` 用 `checker_root_region(i)` / `checker_root_slot_in_its_region(i)`（定义在 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:393`、`:397`）看那一槽有没有自证过的根；
  没有且 `!lt64(next_txg, 环长)` 时，用已经算好的 `oldest`（已含钉值）与 `sub64(next_txg, 环长)` 取小、与 F 取大，得第二个现算值；`accepted = eq64(t, 现算) || eq64(t, 第二个现算)`。补丁 `candidate-a-seed-edge-gpu.patch`（+27 −1）。
- 乙在 GPU 里：同样的落点循环，条件成立时不调 `judge(INVARIANT_I_7_18, …)`（GPU 一侧「不判」就是不调 `judge`，与 CPU 的 `not_applicable` 对应）；`candidate-b-gpu.patch`（+17 −1）。I-7.19 照判。
- CPU 一侧的 `roots` 来自 `crates/singlefs-checker/src/image.rs:1631` 「pub fn valid_roots(reader: &dyn ImageReader, geometry: &PoolGeometry) -> Vec<(u64, u64, RootView)> {」（I-7.1 的集合 S，不滤被抛弃的根）；GPU 一侧的 `root_count` 循环同样不滤，只在求最小值时跳过被抛弃的。所以「那一槽有没有根」两边同一个读法（推的：读了两边的循环，没跑对拍）。
- 代价（推的，没量）：每个状态多一次至多 R × S = 48 步的循环和两个 `divmod64`，相对整个走读可忽略。
- 校验（量的，`out/naga.txt`）：naga 30.0.1 解析、校验判这条不变量那个函数的依赖闭包（前言常量由 Rust 一侧生成，这里补假值，所以只校验闭包，不校验整份内核）。基线闭包 21 个函数通过；甲、甲含种子边、乙三份闭包各 27 个函数通过；
  阴性对照（把 `eq64(…)` 换成一个 vec2 直接参与 `||`）校验红（「Function [26] … is invalid」），所以这个校验分得出对错。

**CPU 与 GPU 对拍要加的盘**（推的建议，没造）。现有的 I-7.18 盘在 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7308` 与 `:7333`，都是「t 比现算高」的红例；目标态要新的流（满环、最后一次发布的根槽写成全零 / 写一半）。我的量具造这些镜像用的是 harness 档的 `probe_i718_oldslot.rs`，可以改成 given-states 交给 checker 档。
- 甲：目标态 t = N+1−3S 成立；t = N+2−3S 成立；t 低 / 高一格红；N+1 的槽完好而 t = N+1−3S 红（不该放宽）；N+1 = 3S 的目标态成立（盯 `>=`）；第二圈 N = 26、p=255、t=3 红。共 7 盘，每盘 CPU 与 GPU 的判定字相同。
- 乙：目标态 CPU 不判、GPU 不判（已判数的字要相同）；同一态 I-7.19 照判；N+1 的槽完好时仍判。共 3 盘。
- 丙：不加；场景期望照实写成「恰好 I-7.18 红」，GPU 对拍照旧。

## 7 三个候选各自的改法落点

三个候选共同要改的（候选无关）：场景期望。恢复之后重开那一张和之后 20 次覆盖写各一张（每个场景 21 张）里「恰好 I-3.1 红」的期望改成全绿（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs` 的
`hand_out_red_only_by_the_allocated_statistic_counting_more_than_the_walk`（第 164 行）及其用处，`overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_holds`（第 207 行））；这一半调查员已列在他的报告第五节 5.1，我没重复量，量到的只有：三个场景在甲 / 乙下零红，在基线下各红一张（§3.4）。

| 候选 | 文件:函数 | 改什么 | 量 |
|---|---|---|---|
| 甲含种子边 | `crates/singlefs-checker/src/walk.rs:8451` `judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root` | 在 8522 行之后加下一槽的落点与有没有根，第二个现算；8525 行的求等改成 `\|\|` | `candidate-a-seed-edge-cpu.patch`：+36 −1 |
| | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4575` 同名函数 | 4606 行那一句换成 §6 描述的块 | `candidate-a-seed-edge-gpu.patch`：+27 −1 |
| | `crates/mutations.tsv:5214`、`:5220` | 两行锚的原文被改写，要重登记；加两行：去掉 `\|\|` 后一支（目标态转红）、去掉 `>=` 改 `>`（N+1 = 3S 转红） | 推的，没跑变异表 |
| | kb | `crates/singlefs-checker/invariants/02-根环.md:25` 补一支并限定「p 写 255 而其实钉着」那句；`.claude/kb/decisions/16-发布语义.md:36` 补一句（§5）；`.claude/kb/checks-owed.md:351` C560 改写；`crates/singlefs-checker/invariants/05-空间记账.md:8` 那段（候选无关） | — |
| 乙 | 同一个 CPU 函数 | 目标态条件成立时 `not_applicable`，否则照判；求等那句缩进变深 | `candidate-b-cpu.patch`：+27 −9（多出来的是缩进） |
| | 同一个 GPU 函数 | 条件成立时不调 `judge(INVARIANT_I_7_18, …)` | `candidate-b-gpu.patch`：+17 −1 |
| | 变异表 | `:5214`、`:5220` 子串仍命中 1 次，锚得住（推的）；加一行：去掉条件（目标态转红） | 推的 |
| | 场景期望 | `PoolCheckerExpectation` 已有 `and_not_judged`（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/mod.rs:152` 「pub fn and_not_judged(mut self, invariant: &'static str) -> Self {」），三个场景的「恢复之前」写 `.and_not_judged("I-7.18")` 加 `.and_holds("I-3.1")` | — |
| | kb | 只改 `crates/singlefs-checker/invariants/02-根环.md:25`（紧挨「复制根不判」那句）与 `.claude/kb/checks-owed.md:351` | — |
| 丙 | 无 `crates/` 代码改动 | 场景期望写成恰好 I-7.18 红；I-7.18 条文记成已认代价 | — |
| | kb | `.claude/kb/checks-owed.md:351` C560 保留，改写成「I-3.1 已绿，I-7.18 认红」；C560 自己要的自证过不了（§5） | — |

写者（`crates/singlefs-core/src/allocator.rs:1806`、`:2742`、`:2756`）三个候选都不动：写者没错（§1 的 E1–E3 与 §2 的 `other_red=[]`）。

## 8 推出该取哪一个

**不取丙**：(1) 量的：目标态上 I-7.18 与 I-3.1 没有共同绿的 t，写者再怎么改都过不了——丙不是「暂留一个红」，是把一个无解的红写进条文；(2) 它是单故障合法态，和 `crates/singlefs-checker/invariants/02-根环.md:26` 「两样故障叠在一起（读坏钉着的槽，这个进程对它的写报失败而实际落了盘）时这一条会误红，超出单故障」的口径相反，
和用户 2026-10-20 对 C773 的态度（`records/2026-10-18-里程碑三收尾与近期欠账.md:386` 「不记成已认代价，另立一笔新欠账」）相反；(3) C560 自己要的自证（`.claude/kb/checks-owed.md:351` 「判别力自证：换回现行判法，那条用例必须由绿转红」）在丙下无从谈起。

**取甲含种子边，不取乙**：
1. 乙把目标态上 t 的校验整个交给 I-3.1；量的结果是这几族历史里 I-3.1 钉得住（只认 N+1−3S），但这是两条不变量碰巧互补，不是 I-7.18 自己的判别力；甲让 I-7.18 在这一态自己还认得出「t 既不是 N+1−3S 也不是 N+2−3S」（量的：t 扫描里 15 个值红）。
2. 甲放过的东西小而且说得清：一个 t 值（它同时是 I-3.1 要的值）和第一圈内「p 写 255 而其实钉着下一槽」一形（逐字节与诚实写者相同，属 C863 同类，要登记）。乙放过这一态上全部的 t 与 p。
3. 甲的代价是真代价：CPU +36 行、GPU +27 行，第二个现算式把「环长、N+1−3S、钉值并进」的语义抄了第二遍，写者一侧这些语义变了，这里会静默跟不上（推的）；变异表两行锚要重登记；D16 第 36 行要补一句。

**乙是对的退路，如果**：主 agent 要最小改动、或不愿意在判器里留第二份写者语义；乙与「复制根不判」同一风格（`crates/singlefs-checker/invariants/02-根环.md:25` 「零单元发布与无文件版本上的复制根，t、p 抄自上一版，环转满之后求等会误红，不判」），变异锚不用动，CPU / GPU 各 17 到 27 行。

**甲含种子边而不是投稿的甲**：条件必须是 `N+1 >= 3S`。量的：N+1 = 3S 那一格投稿的甲红、含种子边的成立（§4 倒数第四行）。

**什么现象会让我翻到乙或别的**：(1) 攻方证明「下一槽住户记根时是被抛弃的根」不可达——那时「两种现算之一」可收成单值，甲更强；(2) 找到 I-3.1 在目标态上对 t 不敏感的历史——那时乙放过任意 t，甲更优；
(3) 主 agent 认为第二份写者语义的维护成本高于一个 t 值的盲点——那时取乙；(4) 要求罩住「环没满时最旧根事后读坏」——那时三个候选都不够，要上 §4 末尾推的「每个缺位」版。

## 9 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1 甲 | 多认 1 个 t，只在目标态 | 量的：t 扫描里成立集合 {基线的值} ∪ {N+1−3S}；对照盘面一格不变；第一圈内 p 写 255 一形放过（逐字节相同） |
| V1 乙 | 目标态上 I-7.18 整条不判 | 判据只看镜像；t 由 I-3.1 钉住（量的，两族历史），p 无人钉 |
| V1 丙 | 不放过，但判红合法态 | 量的：三个场景各一张红；I-7.18 与 I-3.1 在这一态无共同绿的 t |
| V2 范围 | 到得了的合法历史只有「N+1 的槽读不出」一类 | 量的：纯掉电 104 个前缀盘面 0 红、扇区边界撕裂到不了、挂载时读坏的镜像全绿；环没满时最旧根事后读坏三个候选都罩不住；掉盘镜像只当旁证 |
| V3 与条款 | 甲要补两处条文并改两行变异锚；乙补一处；丙与用户态度、C560 自证、I-7.19 口径相反 | D16 第 36 行「六处同取」在目标态上今天就不逐字成立，甲要明写 |
| V4 两侧判器 | GPU 同函数可照改，用现成助手 | 量的：naga 校验闭包通过且阴性对照红；对拍没跑；甲 7 盘、乙 3 盘 |
| 取舍 | 甲含种子边；乙为退路；丙不取 | 翻面观测见 §8 |

## 10 要主 agent 跑 / 核的

1. 复跑（模型目录里的脚本）：把 `research/prompts/m3-close-i718-oldslot-r1-sonnet-model/` 拷到草稿目录，`bash scripts/hosts/run-with-memory-cap.sh 12G bash <拷来的>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新目录> --model <拷来的>`，
   全跑十几分钟，输出与 `out/summary.txt` 逐行比、sha256 比 §2 的表。
2. 判决要引的数，按 `.claude/rules/three-way-inference.md` 要在入库装置上重做一次：这些数都是在我的副本装置（量法开关补丁 + 量具）上量的，不进 kb。
3. checker 档、变异表轻型、GPU 对拍我都没跑；甲或乙落地后：变异表 `crates/mutations.tsv:5214`、`:5220` 两行的锚（甲要重登记）、GPU 对拍盘（§6）、`crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 里三个场景的期望。
4. 复核不了的：GPU 判器与 CPU 判器在目标态镜像上的逐位一致（要 GPU 与 checker 档）；naga 校验用的是假前言常量，只证明函数闭包的解析与类型。
5. 攻方该打的：「下一槽住户是被抛弃的根」这一支（我只推了，没造镜像）；`16-发布语义.md` 第 36 行「六处」补句的措辞（我推的：丁-defer 门槛并旧内容会让 I-3.1 那一截为空）；乙对 I-3.1 敏感性的依赖（我只量了覆盖写与偶数 txg 空发布两族）。

## 11 没做什么

- 没跑 checker 档任何测试、变异表、门禁、崩溃点重放、形式化；没跑 GPU。
- 没改主工作区、没改 `crates/mutations.tsv`、没写 kb；补丁与量具都在模型目录与草稿目录。
- 没量：被抛弃住户那一支；每个缺位的推广版（§4）；撕裂域邻槽；零单元发布的历史；S ≠ 4 的环（量的只有 S=4、环长 12）；GPU 代价。
- 没替主 agent 采纳或出判决；§8 是正推的结论，不是判决。
- 没读禁读清单里的任何文件。
- 一条意外：第一次排错时，同一个编译目录里先后编了几份不同的树，同名产物按修改时间判新旧，曾把第五、六组量具跑在了别的树的库上；发现后那两组的旧日志作废，整套重跑，每个候选的交叉核改成各用自己的编译目录（`tools/rerun.sh` 里 `target-cross-*`）。报告里的数全部出自重跑之后的日志。

## 12 清理

交回前删掉的（删之前 `du -sh`）：冻结副本 37M；开关版 / 候选自己补丁 / 对照用的树各 37M（共 4 份）；编译目录 1.6G；naga 校验用的编译目录 108M；rerun 全跑的工作目录 1.7G；验证 `rerun.sh` 能起的临时工作目录 112M；补丁编辑用的树 6.7M；naga 小程序的草稿副本 4.4M。
草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-i718-oldslot-r1-sonnet` 里剩 2.0M 的小脚本与日志，没有仓副本或编译目录。模型目录 `research/prompts/m3-close-i718-oldslot-r1-sonnet-model/` 约 800K，留给主 agent。

## cite-check 末行

```
✓ 核了 34 处引文、19 处裸行号，对上 34 处，没判 19 处（其中裸行号没有引文可比 19 处；逐处列在上面）
```
