# m4-exit5-relocation-proof-r1 攻方腿（Opus）报告

日期 2026-10-26。攻击面：正文第三节 X1–X4，第一轮角度（过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱）。代码行号都是冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/` 的；引到的文件除 `crates/singlefs-core/src/mounted_session.rs` 外与主工作区此刻逐字节相同（`cmp` 现比，那一份不按行号引）。副本上量出的数注明「副本」，都不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 打中的是什么（一句话） | 分不分辨臂 |
|---|---|---|---|
| X1 | **打中** | 出口 5 后半句「带 GC 的模型对拍与不跑 GC 逐项相同」在带管理员回退、抬 F 的历史上不成立：副本上同一段历史去掉整理，回退能退回的内容集合 12 格里 10 格不同，抬 F 的结局「做成 / 拒」也分叉；抬 F 那几格之后冷读出的现行文件两份逐字相同（P4 那一读照样成立） | 不分辨：三个定义都中，病根是三者共用的前提「整理不改用户看得见的东西」（判决 G7）。只有把回退窗口与 F 明写在抽象状态之外，三个定义才都不中 |
| X1 | **打中** | P4 的目的地前提只比起点槽号（注释写的是「两两不重叠」）：Verus 证出一个两槽目的地罩住一槽单元、前提照样成立的具体状态 | 分辨：只打「沿用 P4 前提」的候选（核外提第 ③ 样） |
| X2 | **打中** | 「罩到 `publish_relocation_batch` 那一路」切错了层：到今天为止 GC 让抽象状态变了的最重一次（判决 m4-segment-code-r1 的 T5 / F1）住在 `mount.rs` 的回退路径里，那几个函数不在 `publish_relocation_batch` 的单元闭包里（副本上量，0 次） | 分辨：(a)(b)(c) 三层只要以「发布入口那一路」划界都中；以「GC 写下的状态被谁读」划界的不中 |
| X3 | **打中** | `gc_crash_proofs` 的协议里没有「回退目标」这一类视图：Verus 证出「搬迁、checkpoint 之后立刻回收并覆写源槽」每步都合法、定理照样成立，而 core 的回退恰要复活这个源槽；把回退目标当读者塞进去，只要它是另一个逻辑版本，定理前提对任何 `logical` 都假 | 不分辨：核外提第 ④ 样、执行模型对拍、前提运行时核都拿同一份协议当 spec，一起中；core 直验只有在换掉这份 spec 时不中 |
| X3 | **打中（数）** | 副本上 `publish_relocation_batch` 闭包 836 个 fn、18057 行，在核里 2 个 / 12 行，其中跟 GC 有关的只有 `released_record_can_be_reclaimed` 6 行；核外提四样把最相关的 5 个函数整段搬进核也只到 (12 + 583) / 18057 ≈ 3.3%（按函数体行数算的上界），离恢复路径先例 69% 很远 | 只打「核外提够罩到搬迁执行」这一读法 |
| X4 | 给改法，不判 | G7 那一句与 D13 怎么改：攻方这边要求改写句至少交代四样（版本对应、回退窗口与 F、协议补持久的回退视图与多逻辑版本、目的地按区间不重叠）；`gc_crash_proofs` 自列的未连接前提里，「floor 不超过受保护读者与根」「不同逻辑版本」「多槽 span」三条被上面的打中坐实是要罩的，其余三条这一轮没攻 | — |

我提的改法一样都没实现：都是推的，被攻过零轮；只有反例本身是量过的（第六节表格逐格标）。

## 复跑

模型目录 `research/prompts/m4-exit5-relocation-proof-r1-opus-model/`。复跑命令（仓根起，草稿目录给一个新的；整条约 4 到 15 分钟，看本机负载与内存包装排队）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m4-exit5-relocation-proof-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
```

它做三件事：一、把冻结副本的 `gc_crash_proofs.rs`、`compaction_plan_proofs.rs` 原样拷来，与 `verus/attack_gc_protocol_and_p4.rs` 一起直接起 `verus`，另跑只有两份证明的基线与两份必红变体；二、把 `harness-twin/gc_twin_rollback_and_floor.rs` 放进冻结副本拷贝的 `crates/singlefs-harness/tests/`，`cargo test --no-run` 编出来直接执行；三、编 `unit-graph`，量两个入口的闭包。末了与 `out/summary.txt` 逐行比，不同退 1。交回前在新的登录 shell 里整份跑过一次：包装退 0，末行原样「name=rerun-compare out/summary.txt same=yes」，`out/twin-lines.txt`、`out/reachable-lines.txt` 与那一趟逐行 `diff` 为空。

`out/summary.txt` 全文（量的，副本）：

```
name=verus file=baseline rc_is_zero=yes verification results:: 32 verified, 0 errors
name=verus file=attack_gc_protocol_and_p4 rc_is_zero=yes verification results:: 36 verified, 0 errors
name=verus file=must_red_rollback rc_is_zero=no verification results:: 35 verified, 1 errors
name=verus file=must_red_overlap rc_is_zero=no verification results:: 35 verified, 1 errors
name=reachable-summary cases=12 differing=10
name=twin-summary cases=112 with_batches=56 differing=48
name=twin-differing-by-kind rollback=32 raise_floor=16 in_cases_without_a_batch=0
name=closure entry=transaction::publish_relocation_batch fn=836 fn_lines=18057 kernel_fn=2 kernel_lines=12
name=closure entry=mount::roll_back_by_a_forward_publish fn=1057 fn_lines=24494 kernel_fn=2 kernel_lines=12
name=membership fn=mount::mapping_keys_whose_target_locations_differ_from_the_current_mapping in_relocation_publish=0 in_rollback=1
name=membership fn=mount::version_to_publish_the_rollback_after in_relocation_publish=0 in_rollback=1
name=membership fn=transaction::relocation_checked_against_the_version in_relocation_publish=1 in_rollback=0
name=membership fn=gc_crash_proofs::released_record_can_be_reclaimed in_relocation_publish=1 in_rollback=1
name=lifted-upper-bound fns=5 body_lines=583
```

各文件 sha256（`sha256sum`，模型目录下相对路径）：

```
b62855d8b0a848e6a57bb1128aaf0b141de4e5ec064d31ed9fc997cf01ef3d98  ./harness-twin/gc_twin_rollback_and_floor.rs
d38b2de778a5fe17c41c0f6e8201cac5720efeb478d6771d42a9476f44ed4402  ./out/reachable-lines.txt
35ae9af08b6cfffdae60a132fce2454d531f00cdcee56cb8aae9ed7467e3ffea  ./out/summary.txt
5db496c85efd30c086a75c1d5d7f347fbb172dabeaa1b1ca3a8d79a0d44e601a  ./out/twin-lines.txt
0d2df0627596d20521b75745e784c946a7b4a82ce1dd1ad2f2c1dc3d1bfc844e  ./tools/rerun.sh
ef6e262640124876c1bbdda799d1359491ace8e55ab1f3013f952a2614b4f230  ./verus/attack_gc_protocol_and_p4.rs
```

取样与估时：对拍原型先在草稿上跑了一趟 84 格（测试体 9 秒），再扩到 224 格（测试体 52 秒），正式取样缩成 112 格（选段只留 0、1，理由在「这条腿自己的限度」）加 12 格可达集合（每格 16 段历史）；不带崩溃，没有崩溃状态数可算（见限度一节）。Verus 每份几十秒。

## X1　「抽象文件系统状态」取哪一个

### 打中一：出口 5 后半句在带回退、抬 F 的历史上不成立（三个定义一起中）

被判的那一句：「用户 2026-10-26 定：证明要罩到 core 的搬迁执行（`publish_relocation_batch` 那一路），抽象模型的证明加模型对拍不算证出」（`.claude/kb/milestone/04-fourth-txn.md:20`），同一行前半是「带 GC 的模型对拍与不跑 GC 逐项相同」。今天落实这半句的只有一处比法：同一段历史跑两份、一份去掉整理（`crates/singlefs-harness/tests/compaction_while_mounted_in_random_histories.rs:383` 起），而且两份结局只许差在空间不够——「两份结局不同只许是一份因空间不够被拒、另一份做成」（`crates/singlefs-harness/tests/compaction_while_mounted_in_random_histories.rs:444`）。它用的那一组比重（`crates/singlefs-harness/src/history.rs:1387` 起的 `COMPACTION_WHILE_MOUNTED`）不抽回退、不抽抬 F（第 1390–1426 行 `grep -c 'RollBackWhileMounted\|RaiseRollbackFloor'` 输出 0）；专为「整理夹着回退」加的那一组写明不进这个比法：「只交给按模型逐版比的用例，不进」（`crates/singlefs-harness/src/history.rs:1432`），理由在前两行：「整理多发的那几条根让它多半退到整理半路或整理之前的根上」（`crates/singlefs-harness/src/history.rs:1430`）。

**造的历史**（副本，原型 `harness-twin/gc_twin_rollback_and_floor.rs`，只用 `singlefs-harness` 的公开入口）：起点第一个文件 → 可写挂载 → 覆盖写 → 可写挂载（前缀 p1；p2 再加一次覆盖写与一次挂载）→ 挂着时整理（一批 1 或 2 个，选段 0 或 1，推到停）→ 用户定的最后一步。最后一步放开扫：回退到根环里从新往旧第 0..7 条根、抬 F 0..5 步。另一份是同一段去掉整理那一步。选段 0 在这两种前缀上一批都不发，是对照臂。

**每一步许可它的那一句**：回退候选集「候选集：根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ 带文件」（`crates/singlefs-core/src/mount.rs:9649`）；根环每区「= 8）。挂载时判 4..16 区间」（`crates/singlefs-core/src/system_configuration.rs:263`，mkfs 取 8）；D16 已定项 1「回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算」（`.claude/kb/decisions/16-发布语义.md:31`），同一行「**根环容量是这句的边界**」（`.claude/kb/decisions/16-发布语义.md:31`）；搬根算非空：「搬走的是用户可见树的根（inode 树根、extent 树根，活头的或快照条目抄的）时，树表里那条根指针换成映射位置、盘上字节变了，」（`crates/singlefs-harness/src/model.rs:254`），模型单测钉着「抬 F 的上限跟着从第 4 新的非空根 6 挪到 7（不算它时还是 6）」（`crates/singlefs-harness/src/model.rs:9972`）。

**量到的**（副本，复跑输出见「复跑」一节的 `out/`；原样摘行）：

```
name=twin case=p1-k1-r1-raise-floor-1 compaction="Applied(Compacted batches=3) units=3 stop=compaction_stopped_no_eligible_segment" with="Applied(RaisedFloor) new_floor=1 file:fcd1c8b89bf919b5" without="Refused(MountError::RollbackFloorAboveCeiling) file:fcd1c8b89bf919b5" same_outcome_class=false same_file=true
name=reachable case=p1-k1-r1-floor-0 with_floor="Applied(RaisedFloor) new_floor=0" without_floor="Applied(RaisedFloor) new_floor=0" with={"file:fcd1c8b89bf919b5"} without={"file:b2b817d52199c655", "file:fcd1c8b89bf919b5"} same_set=false
name=reachable case=p2-k1-r1-floor-1 with_floor="Applied(RaisedFloor) new_floor=1" without_floor="Refused(MountError::RollbackFloorAboveCeiling)" with={"file:1f7c2302f5a08bdb"} without={"file:1f7c2302f5a08bdb", "file:b2b817d52199c655", "file:fcd1c8b89bf919b5"} same_set=false
```

（三行都逐字取自 `out/twin-lines.txt`、`out/reachable-lines.txt`。）汇总行见「复跑」一节 `out/summary.txt`。读法：

- 抬 F 那一格：同一个「从现行 F 抬 1 步」，带整理那一份做成、去掉整理那一份被拒 `RollbackFloorAboveCeiling`。两份的 F 都从 0 起（`new_floor=0` 那一行），差在上限：整理那一批搬了 inode 根或 extent 根，按「照字节比，搬根算非空」算一个新状态，上限往上挪。
- 回退能退回的内容集合（不靠根环下标、不靠 txg 编号的比法：对第 0..7 条根各回退一次，收做成的那几次冷读出的文件）：p1 抬 F 0 步，去掉整理能退回第一个文件（`b2b8…`），带整理退不回。这一格 F 没动（`new_floor=0`），差在根环：整理那一趟多发了 4 条根（回退第 0 条那一行，两份最新根的 txg 是 12 与 8），加抬 F 那一条，8 槽的环把第一个文件那几版挤掉了（txg 差是量的；「是环挤掉的」是按 txg 数推的，没逐槽读环）。
- 逐格比（`name=twin`）：112 格里整理真发了批的 56 格，48 格两份结局类或回退之后读出的文件不同（回退 32 格、抬 F 16 格）；整理一批没发的 56 格 0 格不同。回退那 32 格里有一部分是「按根环下标点名」造成的（两份的环里根不一样多），不单拿它们当证据；不靠下标的是上面可达集合那 12 格（10 格不同）与抬 F 那 16 格。
- 抬 F 那几格之后冷读出的现行文件两份都相同（`out/twin-lines.txt` 里带 `raise-floor` 的行 `same_file=false` 0 行）：P4 那一读、`logical` 那一读、O1 的 POSIX 内容那一读在这些格上都说「恒等」。

**四句**：① 不分辨臂：P4、`logical`、POSIX 内容三个定义都只看「现行那一版读到什么」，三者在这几格上都说恒等，而用户看得见的回退结局变了；病根是三者共用的前提，判决 G7 那一句「RefFS 翻译整理那一步（判决 G7：整理不改用户看得见的东西，不进操作序列）」（`crates/singlefs-harness/tests/compaction_while_mounted_in_random_histories.rs:588`）。照证据纪律「打中不分辨臂」那一行，这一格不拿来判 X1 的三个候选，另立一笔账：出口 5 后半句的射程。② 判别子看得到：回退与抬 F 都是会话里的用户操作，两份历史各自做决定时看得到自己的根环与 F。③ 满足的分句：出口 5 原文「带 GC 的模型对拍与不跑 GC 逐项相同」的「逐项相同」；今天的比法（第 444 行那一判）若放进回退与抬 F，会在「两份结局不同只许是一份因空间不够被拒」那一分句上红。④ 改法（没有跑前条款，按正文第三节 X1 的三个候选逐个核）：三个定义换来换去都碰不到这几格；碰得到的只有两类——在抽象状态里明写「回退窗口与 F 不在内」并把出口 5 后半句的射程缩到不含回退、抬 F 的历史（推的），或改实现让整理的发布不占回退窗口（不算非空、不占根环槽，推的，要动 D16 已定项 1 与用户 2026-10-15 那条定案）。

**什么现象会推翻它**：入库装置上同一段历史两份的回退可达集合相同；或者找到一条条款把「回退窗口」明写成不属于用户可见状态、且出口 5 的「逐项相同」本来就只比现行内容。

### 打中二：P4 的抽象状态按起点槽号取字节，「两两不重叠」只写在注释里

P4 的具体状态是「映射（key 的身份 → 落点起点槽号），加上每个槽号上的字节」，搬一个单元只改目的起点那一个槽号上的值；目的地前提 `destinations_are_distinct_and_free` 只要求起点互异「0 <= earlier < later < count ==> destinations[earlier] != destinations[later]」（`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:568`）、起点不等于映射里任何一个值（第 555 行起 `destination_is_free_in_the_mapping`）。文件头却写「目的地那一半（不在 R 内、两两不重叠、跨度相同）按判决 G2 是分配器的性质」（`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:7`）。core 里两种跨度并存：「CommitGenerated(UnitFootprint::TwoSlotsAligned) => 2,」（`crates/singlefs-core/src/transaction.rs:2726`）、「CommitGenerated(UnitFootprint::OneSlot) => 1,」（`crates/singlefs-core/src/transaction.rs:2727`）。

**反例**（Verus，`verus/attack_gc_protocol_and_p4.rs` 的 `a_two_slot_destination_over_a_one_slot_unit_passes_the_p4_premise`）：映射 key 5 → 槽 3（一槽单元）、key 9 → 槽 6（两槽，要搬）；目的地起点 2。P4 前提 `destinations_are_distinct_and_free(mapping, [2], 1)` 证得出；按槽内容建的具体盘上把 key 9 的两槽写到 2、3，槽 3 上 key 5 的字节变了。原样输出（复跑 `out/summary.txt`）：攻方文件 `verification results:: 36 verified, 0 errors`，只有两份冻结证明时 `32 verified, 0 errors`，多出的 4 条就是这份文件里的 4 个 proof fn；把最后一句改成「槽 3 没变」的必红变体 `35 verified, 1 errors`。

**四句**：① 分辨：只打「核外提第 ③ 样（目的地约束）直接沿用 P4 的 `destinations_are_distinct_and_free`」那一读；照区间写不重叠的不中。② 判别子看得到：分配器取目的地那一刻手里有跨度。③ 满足的分句：注释「两两不重叠」与 spec「起点互异」不是同一句——这是「打中归错了判据」那一类的反面：spec 比注释弱。④ 改法：目的地前提改成区间不相交（每个映射值带跨度、目的区间与任何活单元区间不交），推的、没写。现实里分配器按空闲图取槽，这一条今天不是 core 的错；它说的是 P4 这份前提若被搬去当「目的地约束已证」，证的比字面少。

**什么现象会推翻它**：找到一条已证的引理把 P4 的槽号状态与按区间的盘面接上，且它的前提里有区间不相交。

### X1 没打中的形状

- `logical` 定义下「恒等」成立而对拍不同：除上面共用前提那一格外，没造出只打 `logical`、不打另两个定义的历史。`gc_crash_proofs` 自己写着「所有读者共享同一个 `logical`，连 key 的定义域也相同。」（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:9`），它与用户覆盖写交错的历史接不上，这一点归到 X3 打中一。
- 快照共享块：没造。`publish_relocation_batch` 文档写「被快照引用的块照样搬（留存的映射条目跟着换位置）。」（`crates/singlefs-core/src/transaction.rs:9662`），P4 的一张映射能不能代表活头与留存条目两份，这一轮没攻。

## X2　「罩到 core 搬迁执行」罩到哪一层

### 打中：以「`publish_relocation_batch` 那一路」划界，漏掉 GC 改变抽象状态的那条真路径

到今天 GC 让抽象状态变了的最重一次在回退路径上：判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第 29 行「整理搬过单元之后回退，回退那一版的中央映射照抄现行版、指向回退自己释放掉的目的落点」（`research/prompts/m4-segment-code-r1-main-verification.md:29`）。修它的代码在 `mount.rs`，文档原话「整理在两版之间搬过的单元就是这样：key 不变、现行那一版的条目指着目的落点，而回退释放的正是目的落点、复活的是源落点。」（`crates/singlefs-core/src/mount.rs:9623`）。

量的（副本，`unit-graph closure`，复跑 `out/summary.txt` 的 `name=membership` 四行）：`mount::mapping_keys_whose_target_locations_differ_from_the_current_mapping` 与 `mount::version_to_publish_the_rollback_after` 在 `publish_relocation_batch` 的闭包里各 0 次、在 `roll_back_by_a_forward_publish` 的闭包里各 1 次；`relocation_checked_against_the_version` 反过来（1 / 0）；`released_record_can_be_reclaimed` 两边都在。

**四句**：① 分辨：(a) 判定纯函数、(b) 可执行代码、(c) 写流与盘面三层只要都以「发布入口的闭包」划界，一起中；以「GC 写下的映射与释放代被哪些路径读」划界（回退、挂载续做、回收与 discard）的不中。② 看得到：回退那一刻手里有现行那一版与目标那一版两张映射。③ 满足的分句：用户原话「要罩到 core 搬迁执行」（`records/2026-10-13-里程碑四预开发.md:1118`），正文第三节 X2 列的最强读法 (c)「core 写出的写流与盘面（崩溃恢复之后读到的东西）」——回退之后读到的东西就是盘面的读法之一，F1 那一次红在 I-3.9、I-3.15，是盘面上的。④ 改法：X2 的层写成「`publish_relocation_batch` 写下的状态，加读这些状态的回退、挂载续做、回收与 discard」，推的；代价没量（`roll_back_by_a_forward_publish` 闭包 1057 个 fn、24494 行，见复跑输出，其中不在发布闭包里的 fn 237 个，草稿上 `comm` 数的）。

**什么现象会推翻它**：有一条已定条款把回退排除在出口 5 之外（比如「GC 恒等只对不含回退的历史」），那 X2 的切法与 X1 打中一一起缩。

### X2 里 D13 射程那一问

D13 已定项 1 射程「**射程**：refinement 的边际价值只在崩溃点重放枚举取不到的那几维」（`.claude/kb/decisions/13-验证路线.md:69`）。上面两处打中都不靠崩溃：回退窗口与 F 那一格在不崩的历史上就分叉，维度是「整理发了几次根」（批数、搬了几个根单元）与根环槽数——正是射程那一句点名的「槽数」、以及正文点名的「搬几批」。所以按射程句，证明要罩的维至少有批数与根环槽数这两样，而它们恰恰落在回退窗口上，今天两份证明都没有这个量。

## X3　办法与叠法

### 打中一：四个候选里三个拿来当 spec 的协议，没有「回退目标」这一类视图（最脆弱的一条）

`gc_crash_proofs` 的状态只有 `disk`、`current`、`durable`、`readers` 四样；`readers` 是进程内的，「AcquireReader 不是持久快照创建，Crash 会丢掉它。」（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:10`），崩溃一步「GcCrashStep::Crash => GcCrashState { current: s.durable, readers: Seq::empty(), ..s },」（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:107`）。core 的回退目标是根环里的旧根，崩了还在，回退要复活 GC 的源槽（上面 X2 引的 `crates/singlefs-core/src/mount.rs:9623` 那一句）。协议里保护源槽的只有「当前根、持久根、每个留存读者三类缺一不可」，而且「真实分配器必须另证其回收条件蕴涵这一条件；本模型没有从 floor 推出它。」（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:70`）；产品真跑的那一个核与它之间「与地址级 gc_slot_is_unreferenced 之间没有已证明的转换桥梁。」（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:255`）。

**反例**（Verus，`verus/attack_gc_protocol_and_p4.rs`，照冻结副本两份证明原样拷来引用它们的 spec）：

1. `reclaiming_the_source_after_a_checkpoint_is_enabled_and_breaks_the_pre_compaction_root`：初态 key 7 → 槽 10、字节 [42]，没有读者。Write 槽 20 → Relocate 7 到 20 → Checkpoint → Reclaim 槽 10 → Write 槽 10 写 [0]，五步每步 `gc_step_enabled` 证得出，`gc_all_views_match(s5, logical)` 证得出（定理那一读照样成立），而搬迁之前那一版的映射（core 回退复活的就是它）`!gc_view_matches(before, s5.disk, logical)` 证得出。必红变体（把最后一句改成「读得对」）`35 verified, 1 errors`。
2. `a_crash_drops_every_reader_while_the_rollback_target_survives_on_disk`：就算把回退目标当一个读者挂上，一次 Crash 之后读者清空，源槽的 Reclaim 就合法了。
3. `a_reader_of_another_logical_version_makes_the_premise_false`：反过来，把回退目标当读者常驻（不让 Crash 清掉），只要它的 key 集合与现行那一版不同（用户在它之后覆盖写过一次就是这样：core 的映射 key 里带出生 txg，「中央映射 key 里出生 txg 那 8 字节的起点：类标签 1 + 出生树 8」（`crates/singlefs-core/src/records.rs:589`），覆盖写出的新单元出生 txg 不同、key 跟着不同——这半句是推的），`gc_all_views_match` 对任何 `logical` 都假——定理前提空真。

**这对四个候选各意味着什么**：

| 候选 | 拿什么当被证的性质 | 这一格中不中 |
|---|---|---|
| 核外提（第 ④ 样「写序列的每个持久前缀对应协议的一段合法历史」） | `gc_crash_proofs` 的协议 | 中：core 在 GC 之后立刻回收并覆写源槽、回退读坏，这样的写流照样「对应一段合法历史」 |
| core 直验 | 正文没定 spec；沿用这份协议就中，另写带回退视图的 spec 就不中 | 推的 |
| 执行模型对拍 | 执行模型精化这份协议 | 中，理由同第一行 |
| 前提运行时核 | 在每个崩溃态上核 `gc_all_views_match` 与 `gc_step_enabled` | 两难：投影里放进根环旧根，就是第 3 条，覆盖写过的盘面上每个态都判红（假红）；不放，就是第 1 条，放过读坏回退的实现 |

**四句**：① 不分辨：三个候选共用这份协议当 spec，一起中；照证据纪律另立一笔账——协议本身先补「持久的回退目标视图」与「多个逻辑版本」两样，再拿它判候选。② 看得到：分配器回收那一刻手里有根环表、F 与释放代（`released_record_can_be_reclaimed(released, generation, floor)` 的三个入参）。③ 满足的分句：正文第四节攻方立场「某个候选声称罩到的那一层，在 core 里找一条它罩不到、却会让抽象状态改变的路径」——路径是回退复活源槽；它今天由 floor 护着（`allocator.rs` 里调 `released_record_can_be_reclaimed` 的两处，第 2561、4466 行），这份护法不在任何一份证明里。④ 改法：协议加一类「持久视图」（根环里仍是回退候选的根），Crash 不清它、Reclaim 与 Write 要避开它，每个持久视图各带自己的 logical；推的，没写，义务数没量。

**什么现象会推翻它**：协议里已有一条（或一条已证引理）把根环里的回退候选折进 `durable` 或 `readers` 并且崩溃之后还在；或条款把回退明写在出口 5 之外。

### 打中二：核内占比与义务数在副本上重量

量法照调查员报告 1.6（`unit-graph closure` 加 awk 按 fn 起止行求和），树是冻结副本，原样输出在复跑 `out/summary.txt` 的 `name=closure` 与 `name=lifted-upper-bound` 几行：

- `publish_relocation_batch`：闭包 836 个 fn、18057 行；落在 `singlefs-replay-prefix` 的 2 个 fn、12 行。调查员在他的副本上量的是 834 / 18027（树不同，差 2 个 fn、30 行）。这 12 行是 `gc_crash_proofs::released_record_can_be_reclaimed`（6 行）与 `singlefs_replay_prefix::records_at`（6 行，恢复那边的取前缀辅助，不是 GC 的），跟 GC 搬迁有关的核内行数是 6 / 18057。
- 核外提的上界：把搬迁这一路最相关的 5 个函数（`relocation_checked_against_the_version` 133 行、`prepare_the_version_publish_relocating_units` 267 行、`settle_the_allocation_record_tree` 124 行、`allocate_relocation_destination` 32 行、`persist_publish_writes` 27 行）整段当成核，合 583 行，核内占比上界 (12 + 583) / 18057 ≈ 3.3%。这是函数体行数的和，量的；「整段都能搬进核」是推的，而且偏宽（后两个是普通发布也走的）。恢复路径的先例 `replay_journal` 是 3171 / 4616 ≈ 69%（调查员量的，我没重量）。
- 读法：核外提把四样判定搬进核之后，闭包里 96% 以上的行（读树、节点编解码、记账与分配记录树重写、journal 编码、块设备）还在核外。用户 2026-10-26 原话「抽象模型的证明加模型对拍不算证出」（`.claude/kb/milestone/04-fourth-txn.md:20` 那一行）要的若是「core 跑的那份代码被证」，单靠核外提到不了；它够不够，取决于 X2 定的层是不是 (a)。
- 义务数：这一格我没给任何候选写原型核，没量 verified 数。core 直验的「下界约 834」在冻结副本上是 836（fn 个数，量的）；那是「每个 fn 至少一条」的下界，Verus 能不能编下这些 fn（泛型 `BlockDevice`、`BTreeMap`、闭包）没核。

**四句**：① 只打「核外提已经罩到搬迁执行」这一读法，不打 core 直验。② 不涉及判别子。③ 满足的分句：正文 X3「每种在副本上的核内占比与义务数」。④ 改法：无，这是给岔路单的代价数。

**什么现象会推翻它**：在副本上把核外提四样写成核、照同一条命令量，核内行数明显高于 583 + 12（说明我选的 5 个函数漏了大头）。

### X3 与用户 2026-10-01「Verus 跑真实代码」、D13 已定项 5、13、14 冲不冲

这几格这一轮只看了与打中相关的那一半：

- 前提运行时核与执行模型对拍的投影放在池级 checker 一侧，D13 已定项 5「只共享一份从 kb 生成的常量模块」（`.claude/kb/decisions/13-验证路线.md:15`）不挡；可是 X3 打中一说的两难与放在哪一侧无关。
- 执行模型对拍要立一对新的 spec × model（D13 已定项 13）。X1 打中一给了这对的第一笔「条款欠一句」事件的现成样子：spec（P4 / 协议）说恒等，model（O1 理想模型）在搬根那一批记新状态、上限挪一版——两边各照条款的一种读法（「恒等」对「照字节比，搬根算非空」），条款没写谁对。推的，计数器那一格我没碰。
- D13 已定项 14「哈希与 AEAD 在形式验证里只能当 uninterpreted function」：协议 Relocate 要求的是完整字节相等，core 在留存条目那一支按 CRC 取「两份取先核得上的那一份」（`relocation_checked_against_the_version`），属于已知的边界，这一轮没攻。

## X4　G7 那一句与条款怎么改

不判，给攻方这边对任何改写句的四条要求（每条都对上一格打中，「量过」「推的」见第六节）：

1. **版本对应**：「带 GC 的模型对拍与不跑 GC 逐项相同」要写明两份历史的版本怎么对应（txg 与根环下标在两份里都不同，见 X1 打中一的回退那几行），否则「逐项」没有定义。
2. **回退窗口与 F**：要么写明它们不在「抽象文件系统状态」里、出口 5 后半句只管不含回退与抬 F 的历史；要么改实现让整理的发布不占回退窗口。两条路都碰到已定条款：前一条与 D16 已定项 1「只数改过用户可见状态的根」的读法（GC 搬根照字节算改过）要对一次账，后一条要重开用户 2026-10-15「照字节比，搬根算非空」。
3. **协议补持久的回退视图与多逻辑版本**：不补，核外提第 ④ 样、执行模型对拍、前提运行时核证出来的性质都放过「回退读到被回收覆写的源槽」。这就是 `gc_crash_proofs` 自列的「floor 不超过受保护读者与根的版本」与「不同逻辑版本」两条未连接前提——这一轮坐实它们是承重的，不是边角。
4. **目的地按区间不重叠**：P4 那一句若被沿用，前提要从「起点互异」收严成区间不交。对上自列的「多槽 span」。

G7 原句「执行层靠 N6、模型对拍与必出反例的世界」（`research/prompts/m4-gc-r1-main-verification.md:38`）里的「模型对拍」，按 X1 打中一，今天的那份对拍不抽回退与抬 F，罩不到上面第 2 条。

`.claude/rules/verification.md` 两句、D13 新立一项还是补进已定项 1：这一轮没攻。K 未定（`CompactionBatchSize::NotDecidedByTheClauses`）：我的原型用的是只供测试的强制批大小 1、2，两档在 p1 上分别发 3 批与 2 批，回退那几格两档都分叉（见 `out/twin-lines.txt`），所以 X1 打中一不依赖 K 取哪一档；K 更大时整理发的根更少、根环那一半的差会变小，抬 F 那一半只要有一批搬了根就在（推的，没量更大的 K）。双盘副本、隔离、共享快照三条未连接前提：没攻。

## 第六节　改法逐格表（我提的改法，被攻过零轮）

| 改法 | 修哪一格 | 在打中的那几格上 | 标 |
|---|---|---|---|
| 抽象状态明写「回退窗口与 F 不在内」、出口 5 后半句缩到不含回退与抬 F 的历史 | X1 打中一 | 那几格不再算违例；代价是出口 5 不再说任何关于回退的话 | 推的 |
| 整理的发布不算非空、不占根环槽 | X1 打中一 | 抬 F 那几格应当不再分叉；根环那一格只有「不占槽」才修得到，D16 已定项 1 自己写着根环容量是边界 | 推的（没改代码试） |
| 协议加持久的回退视图（Crash 不清、Reclaim / Write 要避开），每个视图带自己的 logical | X3 打中一 | 第 1、2 条反例应当变成「Reclaim 不合法」；第 3 条两难消失 | 推的（没写 Verus） |
| P4 目的地前提改成区间不交 | X1 打中二 | 反例里的前提应当证不出 | 推的 |
| X2 的层改成「发布写下的状态，加读它的回退、挂载续做、回收与 discard」 | X2 打中 | F1 那类路径进射程；代价没量 | 推的 |

量过的只有反例本身（第三列描述的「现在会怎样」），改法一样都没实现。

## 没打中的形状

| 形状 | 试了多少 | 结果 |
|---|---|---|
| 整理一批也没发的对照（选段 0） | 2 种前缀 × 批大小 1、2 × 回退 0..7 与抬 F 0..5，56 格（`out/summary.txt` 的 `in_cases_without_a_batch`） | 两份逐格相同：分叉只出在真发了批的那几格，不是比法自己造出来的 |
| 抬 F 4、5 步（p1） | 2 格 | 两份都被拒、可达集合相同：上限两边都够不到 |
| 写序列某个持久前缀对不上协议历史（目的副本没落、根先落） | 没造；只读了 D13 已定项 4 的按设备切段：一段要等每块有写的盘都被屏障或 FUA 放行才关，根槽 FUA 落在副本写之后的段里 | 没打中，推的；没跑崩溃枚举（见「这条腿自己的限度」） |
| `RoleOfThisVersion` 那一支写目的地的是内存里的字节（`.unit_of_role(role)`，`crates/singlefs-core/src/transaction.rs:10239` 那一段），不是从源落点读回的字节 | 只读代码 | 协议 Relocate 要求的是盘上源与目的字节相等，内存与盘一致是另一条没连的前提；不带故障时造不出两者不一致的历史，没打中 |
| 快照共享块在 P4 一张映射里怎么表示 | 没试 | — |
| 双盘副本、隔离 | 没试 | — |

## 这条腿自己的限度

- **崩溃状态没枚举**：定义第 3b 步要求留下的每段历史的崩溃状态一个不落。我造的两类历史（带整理与去掉整理的对拍、Verus 的协议反例）都不带崩溃，打中的几格也不靠崩溃；但 X2 (c) 那一层（写流与盘面）我没用 `enumerate_crash_states` 去扫 GC 写流的任何一段，「写序列持久前缀」那一形只是读条款推的。这一半交主 agent 判要不要另派。
- 对拍原型住在 `singlefs-harness` 的测试目录里（副本），用的是 harness 的执行器与内存池，不是 `SparseBlockDevice` 直接建池；4 GiB 的盘由执行器自己建。
- 取样：前缀只有两种（一次、两次覆盖写），选段只取 0、1（草稿上跑过 2、3：2 与 0 同为一批不发，3 与 1 的 56 行逐行相同，`diff` 为空），回退下标 0..7、抬 F 0..5。没扫别的批大小、别的盘宽、带快照的前缀。
- 「根环那一格是 8 槽挤掉的」是按 txg 数推的，没逐槽读环；抬 F 那一格靠的「搬根算非空」是读 model.rs 与它的单测推到实现上的，我没在实现里单独确认那一批被算成非空（只看到上限挪了、结局分叉）。
- 复跑要排内存包装的队：本机 slice 常年接近满，复跑 `tools/rerun.sh` 前台一次超过 240 秒（交回前那一趟我用了 590 秒的前台上限，钩子会记一条检出）。
- 没读正推腿的产出（禁读）；X3 的「推荐叠法」我没有对象可攻，攻的是四个候选本身。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳。副本上的数都不是入库装置上的数。
- 没跑任何门禁阶段、没起 `scripts/gate/verus-proof-check.py`、没跑 checker 档测试、没起 GPU。
- 没给核外提、执行模型对拍写原型核，没量这些候选的 verified 数；core 直验没试编。
- `.claude/rules/verification.md` 两句、D13 怎么立项、双盘副本 / 隔离 / 共享快照三条前提，没攻。
- 要主 agent 跑的命令：无（轻型三条与重型都不涉及）。

## 草稿与清理

- 草稿目录 `~/.local/share/singlefs/work/m4-exit5-relocation-proof-r1-opus/` 里留着早先两趟扩样的原样输出（`twin1.txt` 84 格、`twin2.txt` 224 格、`reach1.txt` 24 格）与闭包清单，是推到正式取样之前的草稿，正文只引 `out/` 里复跑出来的那几份，所以没入库。
- 删了：仓副本与它的编译目录（`repo-copy/` 39M 加 `build/`）、交回前复跑那一趟的 `rerun-final/`（1.6G，含冻结副本拷贝与编译目录）、两份 Verus 草稿目录（124K、16K）。草稿目录现在 1.3M，没有仓副本与编译目录。
