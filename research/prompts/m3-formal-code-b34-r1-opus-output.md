# 第 34 批代码轮第一轮 攻方腿（Opus）报告（2026-10-06）

## 复跑

```
bash research/prompts/m3-formal-code-b34-r1-opus-model/tools/rerun.sh <草稿目录> [build crafted r1 r2 r3 r4 rfl l41 l45 l6 l57a l57b l65a l65b l3]
```

不给段名就全跑（编译约 25 秒；随机历史五批约 12 分钟；层 0 七段约 19 分钟；10 线程、内存上限 8G，每段一次经 `run-with-memory-cap.sh` 起）。`build` 从冻结副本 rsync 一份、核 `crates-sha256.txt`、从 `_m3-formal-code-b34-r1-diff.md` 切出 34-1 的 walk.rs 补丁反打出第 34 批之前的 checker（再打探针补丁），另拷一份打改法 FL 补丁，装进工作区编 `b34probe`；换过的源文件刷新修改时间、编译目录在草稿目录里，不复用旧产物。日志落 `<草稿目录>/out/<段>.log`，与模型目录 `out/*.txt` 里抄的行逐行比。我在自己草稿目录的新副本上照这个脚本跑过 `build crafted`：`crafted.log` 的 195 行 `B34X` 与原跑逐字节相同，两份改过的 walk.rs 与原副本逐字节相同。

模型文件 sha256（`research/prompts/m3-formal-code-b34-r1-opus-model/` 下）：

```
6f8020aab931ce186c1af6d77c6184b3b6bf24f374ab53eb353996ea7f4438c1  cargo/b34-checker-fl.toml
3c7bf78b6cbfd0452b71442c5fe88d0fb29df6b5015193929b655519977fa421  cargo/b34-checker-pre34.toml
55375a1b90bf959cd514d35b99efd08686c84602d5fa050dbe229d33014994f6  cargo/b34-opus-proto.toml
115b1ccc3683b866df087daf8d6814681acb53c4baee53f08c80fd22f1705909  out/crafted.txt
59981029628ed1ccee040dba62714607a835fb255d48d38e8112809f48b9630c  out/layer0.txt
740c8ea9dbcb40bb94d81e9e6103efb0b1bc68d19a27ed46263f892a1085ca2f  out/random.txt
78abd8517992615a63fc7c637fbbea7cb4eb967a2bf3116a4c84764a7e32afd1  patches/fl.patch
7aff4ec811f35b64036cb57ae7b09c466ab577d2a264f2834ee7366be8385984  patches/pre34-probe.patch
1499610b4e69cf5f430125d5d73e1bb5070bf9783a24e9d424b7ec334aba2da1  src/craft.rs
79d226f38282594d14d95ff2cc2c65315224f76b3c01b7bc30d66d90d0684370  src/drive.rs
c8052100c8c6cce2223a1686bc29955997bf488a9bb9d699ed240888042707c9  src/judge.rs
2497d6de07900a535ae1f80771199e4c62f1328b996270ab4349f7b90d71451b  src/layer0.rs
2e7fec0f578cfb958ac1f9a69fe17af0e8b9c2ce22e592644150b3c406b28a7e  src/main.rs
0c7805f4f8a02583f94a2827fb685c3896328c66e4f197b48dd76c1d366f7a92  src/p2.rs
49ad567e4c58e12dfed30e508a6a56e9e3c9f298a027fd672513c5359e32aca6  src/random.rs
8753642b5ef416b677f0f4b9c17f4a1c16e357c1c99a6f61c895fb0a9b518c35  src/tally.rs
648c38148a6ea89f5a098810961d218c98de96f5def4509c173e0ade84d2ebf1  tools/rerun.sh
```

装置是什么：两块内存稀疏盘（`SparseBlockDevice`，不建镜像文件），开着内容保留的录制流，按一串用户动作调 core 今天的公开入口（可写挂载、第一个文件、经会话的覆盖写 / 顺序写 / unlink / 建 inode、挂着时回退、抬 F、卸载）；崩溃 = 那一步之前的镜像叠上那一步录制流的一段前缀（停在它某次根槽写之前）。每张镜像交两份池级 checker 各判一遍逐条比：「新」= 冻结副本今天的 `singlefs-checker`，「旧」= 同一份 walk.rs 反打 34-1 补丁（第 34 批之前的判法，外加两个只读探针数它认出的两类由记录造出来的版本）。三份数都是副本上的数，不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1-a RV 在合法历史上多报 | 没打中（一次抽样） | 430 段随机历史 35,989 张合法镜像、层 0 六段 221,354 个崩溃状态，新 checker 的 I-5.4 违例 0、部分没判 0；新旧之差只有「旧不适用 → 新成立」那一格与五条的理由文字 |
| P1-b 两类版本的新根段取自哪条记录 | **打中（副本量过）** | checker 取同一次发布里第一条带提交标记的记录，core 施加带末条标志的那一条；两边一致只靠 D23（journal 的角色与格式） 已定项 15「每条记录都带同一份新根段」，没有一条不变量判它。第一条换成上一版新根段、再让这一版两条分配记录罩住同一个槽：新 checker 0 红，只读恢复与可写挂载都拒在「同一块盘上两条分配记录罩住同一个槽」 |
| P1-c GPU 扫描内核与 CPU | 没打中（读码，推的） | 同一组版本、同一个判法，同样取第一条记录；P1-b 的改法要两边一起改 |
| P2-a 五处在合法历史上误红 | 没打中（一次抽样） | 合法镜像上树表指针全零的根 0 条、记录 0 条（6,702 张里数的），五处没有对象 |
| P2-b 有没有一处整版放过 | 部分（副本量过，归 Q1） | 下一次挂载先施加的那一版树表指针全零：新 checker 0 红（I-5.4 只报部分没判），两种挂载都拒成「槽 0 读不出」；F 之下的根全零只 I-9.28 报部分没判、core 不择它，不算放过 |
| P2-c 「自证过的根」在每一处是不是同一个集合 | 不是（读码 + 副本量过） | 主走读、五处、I-9.28 三组集合各不同，见 P2 节表 |
| P2-d 改法与 core 读路径是不是同一结局 | 是（副本量过） | 最新根全零：checker 红 I-7.2 等、两种挂载都拒；更早的候选根全零：checker 红、core 挂上，回退到那条根时 core 拒成「槽 0 读不出」 |
| P2-e I-9.28 部分没判的说明 | 打中（文字，副本量过） | 前缀写「树表指针非零而条目没读全」，后面列的却是「树表指针全零」的根 |
| P3 34-2 根记录四条指针 | 没打中（读码） | core 读根槽的入口都经 `RootRecord::parse_slot`；两份改了的测试仍判原来那件 |
| P4 Q1–Q4 | 交用户，各给候选、翻面观测与量过的代价 | 四问都不挡这一批合入 |
| P5 变异行、I-5.5 理由 | 没打中（静态核 + 读码） | 61 行锚点各恰好命中一次；I-5.5 专门理由分得出「没有对象」与「一次都没判」；「认出来而没有数据单元」那一形在 6,702 张合法镜像上 0 次 |
| 越格线索 | 不归这一轮 | 合法历史上新旧两份 checker 同样红 I-3.1（4 段）、I-3.9（1 段）；可能是 C697 / C560 那一族（推的），复现种子见末节 |

## P1　RV

### P1-a 合法历史上会不会多报：没打中（一次抽样）

取样（用户动作不写死：每一步按会话状态随机抽，崩溃点按种子摆在那一步某次根槽写之前，崩后下一步同样随机抽）：

- 随机历史 5 批 430 段（`random` 段，盘宽在单元区 256 槽、384 槽、4 GiB 三档里抽；20 步 370 段、40 步 60 段），每一步之后的镜像、每一步里每次根槽写之前那一刻（记录全落、根没落）与只落了第一条记录第一份那一刻，共 35,989 张，两份 checker 对拍。认出「由记录施加出来的那一版」的镜像 15,853 张，认出「下一次挂载先施加的那一版」的 25,572 张（旧 checker 的探针数的，两份 checker 认法相同）。
- 层 0 六段（`layer0` 段）：随机前缀（带崩溃与重挂）之后接一步不崩的后缀，后缀那条录制流按层 0 的枚举域全展开（`judge_layer0_state_range` 配 `full_expansion`，与 `enumerate_layer0` 同一份 `Layer0StatePlan` 与 `judge_crash_image`；先用 `layer0_state_count_with_torn_in_place_overwrites` 算状态数，跑完断言判过的个数相等），每个状态的崩溃镜像一个不落；seed 41、45、6 三段每个状态上都认得出由记录施加出来的版本（逐状态探针），seed 57、65 两段在基镜像与后缀全持久的镜像上认得出（探针行在 `out/layer0.txt`），seed 3 那一段前缀没崩、只有 6 个状态认得出下一次挂载先施加的那一版。

产物原样（`out/random.txt`、`out/layer0.txt` 里同一行）：

```
r-b1.log:B34R images=7588 rv_images=3293 next_mount_images=5430 i54_on_rv_images={'H': 6025, 'N': 232} new_red={"I-3.1": 1} differences={"I-3.4 N>N": 1192, "I-5.4 N>H": 146, "I-5.5 N>N": 5166, "I-9.24 N>N": 1192, "I-9.5 N>N": 1192}
r-b2.log:B34R images=8862 rv_images=3868 next_mount_images=6344 i54_on_rv_images={'H': 7044, 'N': 266} new_red={} differences={"I-3.4 N>N": 1230, "I-5.4 N>H": 218, "I-5.5 N>N": 5614, "I-9.24 N>N": 1230, "I-9.5 N>N": 1230}
r-b3.log:B34R images=8345 rv_images=3729 next_mount_images=5806 i54_on_rv_images={'H': 6731, 'N': 142} new_red={"I-3.1": 13, "I-3.9": 3} differences={"I-3.4 N>N": 694, "I-5.4 N>H": 76, "I-5.5 N>N": 6853, "I-9.24 N>N": 694, "I-9.5 N>N": 694}
r-fl1.log:B34R images=4492 i54_all={'H': 4146, 'N': 346} rv_images=2026 next_mount_images=3202 i54_on_rv_images={'H': 3568, 'N': 138} new_red={"I-3.1": 2} differences={"I-3.4 N>N": 700, "I-5.4 N>H": 100, "I-5.5 N>N": 2909, "I-9.24 N>N": 700, "I-9.5 N>N": 700}
r-b4.log:B34R images=6702 i54_all={'H': 6188, 'N': 514} rv_images=2937 next_mount_images=4790 i54_on_rv_images={'H': 5325, 'N': 192} new_red={} differences={"I-3.4 N>N": 986, "I-5.4 N>H": 154, "I-5.5 N>N": 4540, "I-9.24 N>N": 986, "I-9.5 N>N": 986}
l0-41.log:B34L0T seed=41 images=20511 rv_images=20511 next_mount_images=16398 i54_on_rv_images={'H': 20511} new_red={} differences={}
l0-45.log:B34L0T seed=45 images=24618 rv_images=24618 next_mount_images=16401 i54_on_rv_images={'H': 24618} new_red={} differences={}
l0-6.log:B34L0T seed=6 images=20511 i54_all={'H': 20511} rv_images=20511 next_mount_images=16398 i54_on_rv_images={'H': 20511} new_red={} differences={}
l0-57.log:B34L0T seed=57 images=65548 rv_images=0 next_mount_images=0 i54_on_rv_images={} new_red={} differences={}
l0-65a.log:B34L0T seed=65 images=33000 i54_all={'H': 33000} rv_images=0 next_mount_images=0 i54_on_rv_images={} new_red={} differences={}
l0-65b.log:B34L0T seed=65 images=32548 i54_all={'H': 32548} rv_images=0 next_mount_images=0 i54_on_rv_images={} new_red={} differences={}
```

读法：`new_red` 里没有 I-5.4；I-5.4 在认出两类版本的镜像上只有成立（H）与不适用（N）两种，部分没判 0 次。I-5.4 一格新旧有别的只有 `N>H`（旧 checker 候选集里没有一棵分配记录树走到叶、报不适用，新 checker 判了由记录造出来的那一版、报成立），是 RV 多判到的，不是多报。seed 57、65 两段是只跑新 checker 的（`with_old=false`，探针数为 0 是没跑探针，不是没有版本）：探针另跑在基镜像与后缀全持久的镜像上，`B34L0-PROBE seed=57 base versions_applied_only_by_records=[(1, 2), (2, 6)] …`、`seed=65 … [(1, 3), (2, 7)]`。seed 57 那一行没带 I-5.4 种类分布（那一趟的二进制还没加这一项），只证违例 0。`new_red` 里的 I-3.1、I-3.9 新旧两份 checker 同样红（`differences` 里没有这两条），不是第 34 批带来的，见末节越格线索。

为什么推得出不会多报（正推，读码）：两类版本的单元在它们被认出来的那一刻都还没到回收的时候。由记录施加出来的那一版要满足第 ④ 条，环里最旧的有效根比它老，见 `crates/singlefs-checker/src/walk.rs:7741`：「let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;」；它的单元释放代大于它的 txg、回收门槛不超过它的 txg；主走读早就逐版走它、判 I-2.1 / I-7.4，单元被复用的话那两条先红。下一次挂载先施加的那一版的单元写在记录之前的那一段，记录落了单元就落了。所以合法写者写出来的这两类版本，分配记录树都还是它发布时那一份，重叠与否跟它作为候选根时一样。什么现象会推翻：一段合法历史上新 checker 的 I-5.4 违例或部分没判点名「由记录施加出来的那一版」或「下一次挂载先施加的那一版」。

缩了什么：层 0 全展开的单段在这台机器上每个状态约 2.7 ms（只跑新 checker）/ 6.3 ms（加旧 checker，10 线程，各取前 2,000 个状态估的），一次覆盖写的单元段 20–28 个写（2²⁰–2²⁸ 个状态），带它的后缀一段就超过约 10⁶；按 40 分钟的预算只留单元段不超过 16 个写的后缀（挂载、回退），在 seed 0–70 里按状态数挑了带崩溃前缀、≤ 65,548 个状态的六段，七次调用（seed 65 分两段序号）。随机历史那一半不缩崩溃状态，只是不全展开：它判的是三类具体的镜像，不是枚举域。

### P1-b 两类版本的新根段取自哪条记录：打中（副本量过）

两边各取哪一条（读码）：

- checker 由记录施加出来的那一版：按 (实例, 计数器) 升序扫，带提交标记、进得了前缀的记录里同一 (实例, txg) 先到的那条占位，`crates/singlefs-checker/src/walk.rs:7727`：「let applied_by_a_recovery = record.commit_marker == CommitMarker::Present」，`crates/singlefs-checker/src/walk.rs:7749`：「.or_insert_with(|| record.new_root_segment.clone());」。
- checker 下一次挂载先施加的那一版：同样按键序 `.find` 第一条带提交标记的，`crates/singlefs-checker/src/walk.rs:5520`：「record.commit_marker == CommitMarker::Present」。不要求这次发布的末条落了。
- core 恢复：`crates/singlefs-core/src/recovery.rs:3658`：「每条记录都带整次发布的新根段（已定项 15），施加末条那一份」，而且一次发布要走到带末条标志的那一条才整体施加。
- 两边取的是同一份字节，只在写者守 `.claude/kb/decisions/23-journal的角色与格式.md:452`：「一次发布切成几条记录时，每条记录都带同一份新根段」 时成立。判这一句的不变量没有：I-8.10 只比末条与同代根，`.claude/kb/invariants.md:99`：「带末条标志（记录标志位 0 = 1），且这块盘上同（实例代号, `checkpoint_txg`）带末条标志的记录只有这一条」；I-8.9 判序号与标志，不比新根段。

造的盘面（`crafted` 段，4 GiB 两盘）：挂载、第一个文件（txg 3）之后顺序写三个数据单元，那次发布（txg 4）切成三条记录（计数器 4、5、6，每条带提交标记），崩在它的根槽写之前。s1 只把第一条记录（两盘各一份）的新根段换成上一版（txg 3 那条根）的七段、重封、后面两条重链；s2 只让 txg 4 自己写的那片分配记录叶里两条记录罩住同一个槽、校验和沿引用链补到记录；s3 = s2 再做 s1。产物原样（`out/crafted.txt`）：

```
B34X label=s1-stale-first-record new_red=[] old_red=[] rv=[] next_mount=true
B34X mount label=s1-stale-first-record recover=FileRead (InstanceGeneration(1), CheckpointTxg(3)) bytes=65285 writable=ok
B34X label=s2-overlap new_red=["I-5.4"] old_red=[] rv=[] next_mount=true
B34X mount label=s2-overlap recover=Failed Some((InstanceGeneration(1), CheckpointTxg(3))) AllocationRecordOutsideThePoolGeometry { what: "同一块盘上两条分配记录罩住同一个槽" } writable=refused Recovery(AllocationRecordOutsideThePoolGeometry { what: "同一块盘上两条分配记录罩住同一个槽" })
B34X label=s3-overlap-and-stale-first-record new_red=[] old_red=[] rv=[] next_mount=true
B34X   s3-overlap-and-stale-first-record I-5.4 new=H[] old=H[]
B34X   s3-overlap-and-stale-first-record FL red=["I-5.4"] I-5.4=V[下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树、盘 0：槽 50176 跨 3 的记录与槽 50178 跨 1 的记录罩住同一个槽]
B34X mount label=s3-overlap-and-stale-first-record recover=Failed Some((InstanceGeneration(1), CheckpointTxg(3))) AllocationRecordOutsideThePoolGeometry { what: "同一块盘上两条分配记录罩住同一个槽" } writable=refused Recovery(AllocationRecordOutsideThePoolGeometry { what: "同一块盘上两条分配记录罩住同一个槽" })
B34X label=s4-stale-first-record-then-mount-and-overwrite new_red=["I-3.1", "I-3.5"] old_red=["I-3.1", "I-3.5"] rv=[(1, 4)] next_mount=false
B34X   s4-stale-first-record-then-mount-and-overwrite FL red=[] I-5.4=H[]
```

读法：s2 是第 32–33 批判决里 b3 那一形，RV 照设计红；s3 只多了「第一条记录的新根段是旧的」，新 checker 就判回 0 红，而只读恢复与可写挂载照样拒——RV 要拦的「下一次可写挂载重建分配器时两条记录罩住同一个槽」那件事原样放过。s1 单独一样东西都不红、core 照常挂上：这类写侧错误（违反已定项 15 末句）今天没有判据，它只在 checker 那一侧生效。s4（s1 之后可写挂载、再覆盖写一次，txg 4 成了由记录施加出来的那一版）新旧两份都红 I-3.1、I-3.5：主走读照第一条记录走那一版，那一版自己的单元没走到——这是第一条记录取错一版在别的不变量上漏出来的样子，不是 I-5.4 判出来的。

打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：

1. 分不分辨臂：分辨「取第一条」（今天）与「取末条」（改法 FL）：s3 上今天 0 红、FL 红 I-5.4；合法历史上两者逐张相同（下面 FL 那一批），所以合法历史分辨不了，只有坏写者分辨得了。
2. 被判的系统当时看不看得到判别它的东西：看得到。扫描到的记录带着记录标志那 1 字节（`record_flags_byte`），哪条带末条标志 checker 手里就有。
3. 满足的是判据字面的哪一个分句：I-5.4 那一行的「下一次挂载先施加的那一版（记录与最新根同实例、txg 比最新根大 1、带提交标记，根环里没有它的根）」与「树表指针与「树表 0 条那一版的分配记录树根」取自记录的新根段」——条文没说取哪一条记录，`.claude/kb/invariants.md:198`：「与下一次挂载先施加的那一版（记录与最新根同实例、txg 比最新根大 1、带提交标记，根环里没有它的根）」。这一格是「坏实现让它放过」那一类：两个写侧错误叠在一起（新根段写旧了、分配器留下重叠记录）。只有前一个错误时，core 的结局不变。
4. 跑前条款给的改法：背景材料第五节没给这一格的改法；下面两个是我提的。

| 改法 | 修哪一格 | s3 上 | 合法历史上 | 标注 |
|---|---|---|---|---|
| FL：两类版本都取带末条标志的那条记录的新根段（下一次挂载那一版要求末条已落、带提交标记；由记录施加出来的那一版有带标志的就用它、没有才退回第一条） | P1-b 的 checker 一侧 | 红 I-5.4（量过：上面 `FL red=["I-5.4"]` 那一行） | 4,492 张镜像上与今天逐条相同（量过：`r-fl1.log` 那一行的 `differences` 里没有一条以 `FL ` 起头的） | 只在我的模型上量过、被攻过零轮；补丁 `patches/fl.patch`（31 行）。s4 上 FL 把今天的 I-3.1、I-3.5 红也消掉了（`FL red=[]`），因为它与 core 走同一版 |
| FS：新立一条，同一块盘上同 (实例, txg) 的每条记录新根段逐字节相同（I-8.10 往非末条扩） | 写侧违反已定项 15 末句本身 | s1、s3 都红（推的，没实现） | 合法写者三处造记录都从同一个 `root` 取新根段（`crates/singlefs-core/src/transaction.rs` 1502、2039、8768 行，读码），推的 0 红 | 推的、没实现、被攻过零轮 |

两个改法修的不是同一格：FL 让 checker 判 core 真施加的那一版，但写旧了的第一条从此谁都看不见；FS 抓写旧了的那一条本身，s3 上照样红（红的是新的那一条，不是 I-5.4）。GPU 判器同一处要跟着改（P1-c）。什么现象会推翻这一格：core 的恢复改成取第一条记录的新根段，或条款写明 checker 只认第一条、末条与它不同算别的不变量红。

### P1-c GPU 扫描内核与 CPU：没打中（读码，推的）

- 版本表：GPU 走读内核拼 `R_DATA` 时用同一组谓词（带提交标记、进得了前缀、实例表行的 txg 不小于记录的、根槽读不出、不低于 F、环里最旧有效根比它老），同一 (实例, txg) 也取先到的那条，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3768`：「同键取计数器小的那一条（先到的）」。
- 下一次挂载那一版：扫描内核新拆出的函数在记录数组里取第一条合条件的下标，`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1071`：「fn record_index_of_the_version_the_next_mount_applies_first() -> u32 {」；数组若是按盘按槽排的，环绕回 0 号槽之后「第一条」可能是计数器大的那条，与 CPU 的键序不一定是同一条（合法写者下字节相同，推的）。
- 判法：每一版同一套（带树一版根记录那棵树必须全零、走到叶才判互不相交、分配记录树按根去重），GPU 只有违例位、部分没判不置位，与 CPU 的红位同；FL 若采纳，`R_DATA` 那一段与上面那个函数要一起改。GPU 跑不了，这一格全是推的。

## P2　全零树表指针（用户定甲）

### P2-a 合法历史上误红：没打中（一次抽样）

`r-b4` 那一批每张镜像都数了一遍树表指针整条全零的自证过的根与解得开的记录：`B34R-Q i55_reasons={"H:": 2162, "N:回退候选集里每条根与由记录施加出来的那几版，没有": 4540} all_zero_tree_table_pointer_roots=0 records=0`（6,702 张）。五处的新分支在合法历史上没有对象，5 批 35,989 张镜像与层 0 六段上五处一条都没红。什么现象会推翻：一段合法历史写出树表指针全零的根或记录新根段（今天 core 的写者三处造记录、根记录都从发布出来的树表取这条指针，读码）。

### P2-b、P2-d 造出来的几张盘面（`crafted` 段）

4 GiB 两盘：挂载、第一个文件（txg 3）、覆盖写两次（txg 4、5），把某条根的树表指针整条写零、重封根槽、照抄进同代记录的新根段（I-8.10 照旧成立）。产物原样（`out/crafted.txt`）：

```
B34X label=z1-newest-root-zero new_red=["I-2.1", "I-4.8", "I-7.2", "I-7.4"] old_red=[] rv=[] next_mount=false
B34X mount label=z1-newest-root-zero recover=Failed Some((InstanceGeneration(1), CheckpointTxg(5))) UnitUnreadable { slot: SlotNumber(0) } writable=refused Recovery(UnitUnreadable { slot: SlotNumber(0) })
B34X label=z2-older-root-zero new_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"] old_red=["I-3.1"] rv=[] next_mount=false
B34X mount label=z2-older-root-zero recover=FileRead (InstanceGeneration(1), CheckpointTxg(5)) bytes=10 writable=ok
B34X z2-rollback RollbackToTxg(4) → refused TargetVersionUnreadable { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, failure: UnitUnreadable { slot: SlotNumber(0) } }
B34X label=z3-first-file-root-zero new_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"] old_red=["I-3.1"] rv=[] next_mount=false
B34X label=z4-root-below-floor-zero-txg2 new_red=[] old_red=[] rv=[] next_mount=false
B34X   z4-root-below-floor-zero-txg2 I-9.28 new=P[比到的 11 棵树诞生 txg 都对得上；这几条读得出、自证过的根树表指针非零而条目没读全，没进比较：盘 0 上的根（实例 1、txg 2）：树表指针全零（core 的根记录读者照它读盘 0 槽 0、读不出）] old=H[]
B34X label=z6-next-mount-version-zero new_red=[] old_red=[] rv=[] next_mount=true
B34X mount label=z6-next-mount-version-zero recover=Failed Some((InstanceGeneration(1), CheckpointTxg(4))) UnitUnreadable { slot: SlotNumber(0) } writable=refused Recovery(UnitUnreadable { slot: SlotNumber(0) })
B34X label=z5-version-applied-only-by-records-zero new_red=["I-3.1", "I-3.5"] old_red=["I-3.1", "I-3.5"] rv=[(1, 5)] next_mount=false
```

- P2-d 与 core 同一个结局：最新根全零（z1）checker 红、两种挂载都拒成「槽 0 读不出」；更早的候选根全零（z2、z3）checker 红、core 照常择最新根挂上，而 core 真去读那条根（挂载之后回退到 txg 4）时拒成同一个「槽 0 读不出」。在 core 会读它的每一条路上两边同向。z2、z3 上 I-3.1 新旧都红，不是这一批带来的：那条根的走读断在树表，它下面的中央映射树也没走，遍历少算。
- P2-b 放过的只有一形，归 Q1：下一次挂载先施加的那一版（z6，记录里的树表指针全零）新 checker 0 红（I-5.4 只报部分没判），两种挂载都拒。由记录施加出来的那一版（z5）新旧两份都已经红 I-3.1、I-3.5，主走读照旧把它读成没有树表（「AllZeroTreeTablePointerReading::NoTreeTable,」，`crates/singlefs-checker/src/walk.rs:1446`），五处里 I-5.4、I-5.5 改报部分没判。F 之下的根全零（z4，抬 F 到 3 之后的 txg 2）只 I-9.28 报部分没判：主走读不走 F 之下的根，core 择根取最新、回退不收 F 之下的目标，读不到它（推的，没造「丢一块盘」那一形）。
- 被抛弃的根全零（没造盘面，读码，推的）：被抛弃根那一遍也走 `walk_root`（`crates/singlefs-checker/src/walk.rs:7809`：「walk_of_the_abandoned_root.walk_root(&root.record_bytes, false);」），所以它跟过去、记一次走读失败，红在 I-7.4，说明写的是「被抛弃的根……引用的单元已被重新分配或抹头」——判红对，说明不对。

### P2-c 「自证过的根」在每一处是不是同一个集合：不是

| 判法 | 判的版本集合 | 树表指针全零时 | 依据 |
|---|---|---|---|
| 主走读 | 最新根、回退候选集里每条根（不被抛弃、txg ≥ F_生效）；另走被抛弃的根、F 之下的根（后两遍判定丢掉，只取引用或判 I-7.4） | 根环里的根：跟过去、记走读失败；由记录施加出来的那一版：当没有树表；下一次挂载那一版：不走 | 读码；z1–z5 |
| I-5.4 | 候选根 + 由记录施加出来的那几版 + 下一次挂载那一版 | 三类都报部分没判 | 读码；z2、z5、z6 |
| I-5.5、删除态、墓碑与谱系 | 候选根 + 由记录施加出来的那几版 | 部分没判 / 射程里判不了 | 读码；z5 的 I-5.5 |
| I-9.28 | 根环里全部读得出、自证过的根（含被抛弃、F 之下），不含由记录造出来的版本 | 部分没判 | 读码；z4 |

所以「自证过的根」在五处与主走读之间差两块：一是由记录造出来的两类版本（五处当它读不出、主走读当它没有树表或不走），二是 F 之下与被抛弃的根（I-9.28 报部分没判、主走读的候选那一遍不走它）。前一块就是 Q1；后一块 core 读不到（推的），差在判定的口径上、不差在红绿上。checker 的「自证过」（`valid_roots`：整槽校验和与 fsid）与 core 的「自证过」（`RootRecord::parse_slot`，34-2 之后另判四条指针的加密、压缩与 extent 偏移）也不是同一个集合：四条指针头部不是第一版取值的根，checker 照走、红 I-2.4，core 当自证不过，这一格条款已写「两边各判各的」，不在这一轮。

### P2-e I-9.28 部分没判的说明自相矛盾：打中（文字）

`crates/singlefs-checker/src/walk.rs:5178`：「这几条读得出、自证过的根树表指针非零而条目没读全，没进比较」——34-1 把树表指针全零的根也放进这张单子，前缀没跟着改，z2、z4 上读出来是「树表指针非零而条目没读全，没进比较：盘 0 上的根（实例 1、txg 2）：树表指针全零」（上面 z4 那一行原样）。不改判定，只改说明；什么现象会推翻：这一句的前缀在合入树里已改。

## P3　34-2 根记录四条指针：没打中（读码）

- core 读根槽的入口都经同一个读者：只读恢复与择根（`recovery.rs` 的 `read_root_ring_slot`、`choose_root`）、可写挂载重建分配器读根环那一遍（`mount.rs` 区分全零槽那一个读法）、实例表清扫（`instance_table_sweep.rs`）、挂载透明比根记录（`recovery.rs` 3265 行附近），`grep -n 'RootRecord::parse_slot' crates/singlefs-core/src/*.rs` 在这几处各一次，没有绕开它直接切根槽字节取 txg 的读者（读码）。抬 F 预演、回退判候选读根环走的是同一组函数（读码，推的）。
- 只判 extent 偏移的读法 `read_judging_the_extent_offset_from` 在冻结副本里 `grep -rn` 0 次。GPU 恢复内核根记录那一判仍只判 extent 偏移（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:928`：「fn root_record_pointers_carry_no_extent_offset(r: RootFields) -> bool {」），背景材料第二节已记进第 35 批，不重报。
- 两份改了的测试：挂载透明那份把「整段全零的指针段」翻在头部出生 txg 那一字节，翻完那条根仍自证过、仍到得了判官，判官比的是同一段字节区间，判的还是那一段变没变；extent 偏移那份把根记录用的读法换成走读读者的入口取「判」那一种，断言的仍是 extent 偏移 7 那一个错误值。两份都判原来那件（读码，没跑）。

## P4　条款没写的四问（交用户，不替用户定）

| 问 | 候选 | 量过的代价（副本上） | 翻面观测 | 挡不挡这一批 |
|---|---|---|---|---|
| Q1 由记录造出来的版本里树表指针全零要不要也判损坏 | 甲 判损坏：两类版本照根环里的根那样跟过去、记走读失败；乙 照今天：主走读当没有树表、下一次挂载那一版不走，五处报部分没判 | 今天：z6（下一次挂载那一版）checker 0 红、两种挂载都拒；z5（由记录施加出来的那一版）checker 已红 I-3.1、I-3.5（新旧同）、core 照常挂上（它不再读那一版）。合法镜像上全零的记录 0 条 / 6,702 张，甲在取样里 0 误红（甲没实现，推的） | 找到一条合法写者路径把全零树表指针写进新根段（取样 0 次）⇒ 乙；层 0 的恢复 oracle 已在 z6 那类状态上记「恢复失败」（推的，没跑层 0 oracle 那一侧）⇒ 乙的代价只剩 checker 不报 | 不挡 |
| Q2 全零树表指针要不要同时红 I-2.5 | 甲 收窄豁免：根记录与新根段的树表指针不享全零豁免；乙 照今天整条豁免 | z1–z4 上 I-2.5 今天都成立；合法镜像上全零的根树表指针 0 条 / 6,702 张；豁免还罩着树表里三条 day-1 注册的条目与 mkfs、暖机根里的中央映射树根（`.claude/kb/invariants.md:134`：「整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定」），甲只能按字段收、不能整条收 | 出现一张别的不变量都不红、只有 I-2.5 能红的全零树表指针盘面（z4 那一形：F 之下的根，今天 0 红）⇒ 甲有用；否则只是多红一条 | 不挡 |
| Q3 GPU 判器要不要同改 | 甲 同改；乙 照今天（CPU 红、GPU 不红） | GPU 跑不了。读码：走读内核读码 2 节点遇全零指针直接返回、不记失败（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1481`：「if (p.all_zero) {」），z1–z3 那类状态上 CPU 红、GPU 不红（推的）；对拍流上这类状态 0 次（合法写者写不出，同上 0 / 6,702），今天的对拍不受影响 | 有一条对拍流造出全零树表指针的根 ⇒ 不改就对拍红 | 不挡 |
| Q4 I-5.5「认出了那一版但没有数据单元」要不要另写理由 | 甲 另写一句；乙 照今天落缺省理由 | 6,702 张合法镜像上 I-5.5：成立 2,162、专门理由 4,540、缺省理由 0；第一个文件写空内容那一版 I-5.5 照样成立（那一版仍有一个数据单元，`q4-empty-first-file … I-5.5 new=H[]`）。这一形在今天的写者上取样 0 次 | 一段合法历史上 I-5.5 落缺省理由 ⇒ 甲（缺省理由那时与「一次都没判」分不开） | 不挡 |

## P5　I-5.5 专门理由、B4、#9、变异行：没打中

- I-5.5 专门理由：只在一版都没认出来时设（`crates/singlefs-checker/src/walk.rs:4815`：「judgements.not_applicable(invariant, NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK);」）；部分没判压过不适用（`image.rs` 的 `into_report` 次序），函数一次都没跑时落缺省理由，看门用例只认专门理由，分得开。随机历史里专门理由出现 4,540 次、缺省理由 0 次（上面 `B34R-Q` 那一行）。
- 变异行：34-1 新加的 13 行、34-2 新加的 6 行与 8 行重取、34-3 一行、34-4 重取的 8 行，连同同批点名的几行共 61 行，按「原文在文件里恰好命中一次」在冻结副本上静态核，`bad 0`（脚本在草稿目录里，一次性的）。没改坏跑点名的测试，基线绿与「红在断言上」没验。
- B4 取样点 100、#9 三个钉值：没攻（读码看了 34-3 的补丁，取样点只加了 100；#9 归删除线重取，背景材料第二节已说这一轮不判钉值）。

## 越格线索（不归这一轮，交主 agent）

合法历史上新旧两份 checker 一样红（`differences` 里没有这两条，不是第 34 批带来的），都在带崩溃重挂、并进遍历的由记录施加出来的版本 ≥ 1 的镜像上：

```
r-b1.log:B34R-RED-FIRST I-3.1 seed=187957262 step=19 op=Overwrite(337248) after :: 盘 0：记账的已分配 Some(4292608)，遍历全部有效根得到 4177920（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 24、最新根 txg 27、环里自证过的根槽 18 个、最老的自证过的根 txg 5、遍历的候选根槽 18 个、并进遍历的由记录施加出来的版本 3 个、被实例表判抛弃的根槽 0 个、回退下界 F 0、低于 F 的根槽 0 个
r-b3.log:B34R-RED-FIRST I-3.9 seed=187957506 step=39 op=Mount cut=before-root@57 :: 盘 1 槽 50467 的分配记录释放代 34 不在 (25, 28] 里：还引用它的最新一条有效根是 txg 25，最早不再引用它的是 txg 28
```

复现：`b34probe random 14 1 20 1`（种子 187957262，复跑过一次，同一行）；I-3.1 另在种子 187957475（`random 227 1 40 1`）、187957674（`random 426 1 20 1`）上红，I-3.9 在 187957506（`random 258 1 40 1`）上红。像 C697（已分配统计在短环加大文件上误红）、C560（满环时根槽写坏的恢复前镜像上记账多于遍历）那一族，没核是不是同一形（推的）。

## 没打中的形状

- RV 多报：430 段随机历史（三档盘宽、用户动作按会话状态随机抽、每步四分之一的机会崩在那一步某次根槽写之前，崩在挂载里、第一个文件、覆盖写、顺序写、unlink、建 inode、回退、抬 F、卸载每一种步里都有：按日志数各 768、100、462、71、63、63、210、30、188 次），35,989 张镜像；层 0 六段 221,354 个状态（后缀是挂载、回退、挂载加被拒的抬 F）。一次抽样，按规则不算「没打中」的结论，要再抽一次。
- 全零树表指针在合法历史上：同上取样，0 条全零的根或记录。
- RV 在环转过之后的边界（④ 翻面那一刻）：40 步那 60 段里根环转过不止一圈，没单独盯 ④ 刚翻面的那几张，混在总数里。

## 这条腿自己的限度

- 层 0 只全展开了单元段 ≤ 16 个写的一步后缀；带覆盖写单元段（20–28 个写）的后缀一段就超过约 10⁶ 个状态，没跑。所以层 0 里「下一次挂载先施加的那一版」只出现在挂载与回退那几次发布上，覆盖写、顺序写、unlink 那几种发布的这一版只在随机历史的三类具体镜像上判过。
- 随机历史里崩溃只摆在根槽写之前那一刻（与只落第一条记录那一刻），没有摆在单元段中间；那些状态层 0 才罩得到。
- 四问的「甲」都没实现，只有 FL 实现并量过；FS、Q1 甲、Q2 甲、Q3 甲的效果都是推的。
- GPU 那几格全是读码。
- 坏镜像 s1–s4、z1–z6 都只在 4 GiB 两盘上造了一份，没扫盘宽与发布形状。

## 没做什么

- 没跑重型测试（checker 档的测试、GPU 对拍、`crates/mutations.tsv` 整表），没跑任何 harness 档测试二进制；原型是自己的包（`b34-opus-proto`），只调 checker 档的库函数。
- 没改冻结副本与主工作区；改法 FL 只在草稿目录副本里的另一份 checker 拷贝上。
- P3、P5 只读码与静态核，没改坏跑测试。
- 没判正推那几格，没替主 agent 采纳。

## 草稿目录

`/tmp/claude-1000/m3-formal-code-b34-r1-opus/`：各趟日志（`r-*.log`、`l0-*.log`、`l0p-*.log`、`x1.log`–`x3.log`、`r-repro14.log`）与 `pkg/`（补丁、汇总）留着；仓副本与编译目录交回前删掉（见交回）。

## 批次与估时

先跑小段估：10 段 × 20 步、10 线程单段最长 19.6 秒 ⇒ 100 段约 160 秒；层 0 各取前 2,000 个状态估每状态 2.7 ms（只新 checker）/ 6.3 ms（加旧 checker）。按估时定每批大小、不设限时，每批一次调用：

| 批 | 参数 | 段数 / 状态数 | 挂钟（秒） |
|---|---|---|---|
| r-b1 | random 0 100 20 10 | 100 段 | 164 |
| r-b2 | random 100 120 20 10 | 120 段 | 165 |
| r-b3 | random 220 60 40 10 | 60 段 | 237 |
| r-fl1 | B34_WITH_FL=1 random 400 60 20 10 | 60 段 | 75 |
| r-b4 | random 500 90 20 10 | 90 段 | 72 |
| l0-3 | layer0 3 6 2（加旧 checker） | 24,618 | 203 |
| l0-41 / l0-45 / l0-6 | layer0 种子 7 1（加旧 checker） | 20,511 / 24,618 / 20,511 | 170 / 150 / 111 |
| l0-57 | layer0 57 7 1（只新 checker） | 65,548 | 239 |
| l0-65a / l0-65b | layer0 65 7 1，序号 [0, 33000) 与 [33000, 65548) | 33,000 + 32,548 | 83 + 83 |

层 0 合计 221,354 个状态（单段都不超过约 10⁶，合计不超过约 10⁷）。缩前：同样的随机历史带两步后缀时一段就有 2,097,175 到 536,870,935 个状态（`B34L0 seed=14 … layer0_states=536870935` 这一类，草稿目录 `l0c-*.log`），按每状态 6.3 ms 估一段 3.7 小时以上；缩到一步、单元段不超过 16 个写的后缀。

cite-check 末行（交回前现跑）：`✓ 核了 15 处引文，对上 15 处，没判 0 处（逐处列在上面）`
