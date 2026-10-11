# 第 10z 批第十四、十五块代码轮第一轮：正推腿报告（2026-10-09）

轮名 m3-batch10z-no-code-r1，立场正推：从条款、判决与用户定案推「应该是什么」，再与冻结副本 `~/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree/` 的实现逐格比。判据格 K1（1–3）、K2、K3（1、2）、K4、K5（1、2）。草稿目录 `~/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet/`，模型目录 `research/prompts/m3-batch10z-no-code-r1-sonnet-model/`（README 写了每个文件干什么）。没读这一轮攻方腿的提示与产出。

## 0 开工核对与跑法

冻结副本对快照：

```
$ cd ~/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree; sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-batch10z-no-code-r1-snapshot/crates-sha256.txt 2>&1 | awk '{c[$NF]++} END{for(k in c) print k, c[k]}'
OK 336
```

跑法：每条编译与测试经 `run-with-memory-cap.sh` 带 12G（同时起的件数 × 上限不超过 12G：探针 6 件 × 1700M 或 4 件 × 3G）、线程上限 8（探针件 1）；开跑前 `ps` 看到别的会话在跑 `cargo test -p singlefs-checker-tier --bin e165_replay_bounded_enumeration`，没有 qemu、vm-bench、fio、e152，全部加了 `nice -n 19`。没跑任何门禁阶段，没跑 checker 档测试，没起 GPU 设备，没跑带 gpu 特性的 cargo test。扫源码用例（checker 档两条）用我自己写的模型逐字搬出来跑（`scan-model/`），不是跑原用例。

模型从上一轮正推腿的 `s2_probe` 改到新接口：加了三个独立读数（都不经被测的方法算）——`both`（分配位与隔离位同时置着的槽数，即新访问器）、`c_unblocked`（根环表里还记着的 C 落点盖住的槽里，分配、隔离、抬 F 扣住三位一个都没置的槽数）、`c_handed_out`（C 落点盖住的槽被这个进程新发出去的未释放记录压着的个数）；加了动作 `U`（正常卸载）与故障 `rroot`（R 的根槽读回全零，配 `S2_PERSIST` 让故障挂载之后一直在）。探针同时编在改前树（上一轮冻结副本 `~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree/`，与本轮冻结副本只差被判的 6 个 .rs 文件，用 `find … | sha256sum` 对过）上做改前对照。

## 1 K1　第十四块 双扣改法做到没有

### 1.0 应该是什么（从条款推）

D28 已定项 1 第 27 行第九项，整条（`awk 'NR==27'` 原样）：

```
- **被抛弃根独占量**（第九项）：取值 = 分配器实际隔离的集合的槽数：只被被抛弃根引用的槽（被抛弃根引用的槽 − 回退候选集里的根引用的槽，下界 0），加可写挂载时预隔离的槽：水位之上没施加的 journal 记录点名的落点、那次发布的账里的落点与根环里比所选那一版新、按实例表有效、读得出的根的账里的落点（账读不出时照 D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」从中央映射现算出来的落点也在内），不论它们有没有被回退候选集里的根引用，只除去现行账里仍分配着的槽（那些算在「已分配」里，不在这里再算一次；这次挂载里释放、回收时分配器照补隔离）（D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」），与分配器实际隔离的集合是同一个数；各条根引用的槽从它的分配记录树读、不加盘上字段；每次挂载与每次抬 F 时算出、只住内存、按设备算，被抛弃的根被轮转覆写时清零；被抛弃的根今天只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。影子账开着时，分配记录树两块盘各多写一片叶，准入读数两边之差 = 被隔离的槽 + 这几个节点（实二一报告「停下交主 agent」第 8 条）。
```

逐句读成三件事：（甲）第九项的数就是分配器实际隔离的集合的槽数（「与分配器实际隔离的集合是同一个数」）；（乙）预隔离那一部分「只除去现行账里仍分配着的槽」，括注给两个理由：「那些算在「已分配」里，不在这里再算一次」与「这次挂载里释放、回收时分配器照补隔离」；（丙）被抛弃根独占那一部分写的是「被抛弃根引用的槽 − 回退候选集里的根引用的槽」，条款没有给它加「只除去仍分配的」。
判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第 28 行把 H2 判为打中，第 42 行给验收，要钉住的是「同一个槽不同时算进已分配与已隔离」（`research/prompts/m3-batch10z-lm-code-r1-main-verification.md:42`）。

「已分配」的口径：准入读 `allocated_slots()`，注释写明含已释放还在窗口里的——「已分配 = 占着的槽数（含已释放还在窗口里的）」（`crates/singlefs-core/src/admission.rs:560`）；`mark_reclaimed` 才把它减掉：「self.allocated_slots -= span;」（`crates/singlefs-core/src/allocator.rs:717`）。所以 defer 窗口里的槽（记录已释放、分配位还置着）算在「已分配」里，括注（乙）的第一个理由对它们成立。

### 1.1 K1-1　回收路每一条都补得上隔离吗

实现的回收入口（冻结副本里 production 代码对 `reclaim_released_*` 与 `mark_reclaimed` 的全部调用，原样输出）：

```
$ grep -rn 'reclaim_released_up_to(\|reclaim_released_records_up_to(\|reclaim_released_up_to_isolating_what_the_isolation_still_protects(\|reclaim_released_in_the_sequence_raising_the_floor_up_to(' --include=*.rs crates/singlefs-core/src | <去掉注释与测试段>
crates/singlefs-core/src/mount.rs:2986:        allocator.reclaim_released_up_to(released_immediately_up_to, ReclaimedReuse::Immediately);
crates/singlefs-core/src/mount.rs:2988:    allocator.reclaim_released_up_to(
crates/singlefs-core/src/mount.rs:4499:            reclaim_released_in_the_sequence_raising_the_floor_up_to(
crates/singlefs-core/src/mount.rs:4517:                    reclaim_released_in_the_sequence_raising_the_floor_up_to(
crates/singlefs-core/src/mount.rs:4526:            reclaimed.extend(reclaim_released_in_the_sequence_raising_the_floor_up_to(
crates/singlefs-core/src/mount.rs:4730:            .reclaim_released_up_to_isolating_what_the_isolation_still_protects(floor, reuse),
crates/singlefs-core/src/mount.rs:4731:        ShadowLedger::Off => allocator.reclaim_released_up_to(floor, reuse),
crates/singlefs-core/src/allocator.rs:1844:                    self.reclaim_released_up_to(release_generation, ReclaimedReuse::Immediately);
crates/singlefs-core/src/allocator.rs:2305:            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);
$ grep -rn 'mark_reclaimed(' --include=*.rs crates
crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:1506:            map.mark_reclaimed(SlotNumber(*slot), 1);
crates/singlefs-core/src/allocator.rs:701:    pub fn mark_reclaimed(&mut self, slot: SlotNumber, span: u64) {
crates/singlefs-core/src/allocator.rs:2136:            device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));
```

逐条：
- 挂载重建 `mount.rs:2986`、`mount.rs:2988`：在隔离之前回收，C 圈隔离时这些槽分配位已清，照隔离（所以不用补）。次序证据：隔离调用在其后，`let shadow_ledger_computed = match shadow_ledger {` 之后才进 `isolate_slots_referenced_only_by_abandoned_roots`。
- 挂载内记根 `allocator.rs:2305`：回收后紧跟「self.isolate_reclaimed_slots_the_isolation_still_protects(」（`crates/singlefs-core/src/allocator.rs:2306`），护着的集合在这一槽的 C 组摘掉之后重算（`placements_the_isolation_still_protects` 在 `remove(&ring_slot)` 之后取），所以刚解开的那组不再补，别的组补。
- 抬 F 与卸载这一串 `mount.rs:4499`、`4517`、`4526`：三处都走包装 `fn reclaim_released_in_the_sequence_raising_the_floor_up_to(`（`crates/singlefs-core/src/mount.rs:4722`），影子账开着走新方法（`mount.rs:4730`），关着只回收（`mount.rs:4731`，关着时 C 表是空的，什么都不护）。
- 复用窗口置 0（`allocator.rs:1844`）当场回收不补隔离：正文第二节「已知没做」，只供测试，不算。
- `mark_reclaimed` 只有 `allocator.rs:2136` 一处产品调用（e168 是装置里的模型）。

所以静态上没有漏的入口。动态上用探针扫（改前树与新树各一遍）：

形的写法（`S2_*` 环境变量，全在 `scripts/run-mut.sh`）：s1 = 1536 槽盘、前缀覆盖写 40 次、回退到倒数第 3 或 5 版得 R、R 的单元或账读回全零（挂载那一刻）；s5、s6 = 1536 / 3072 槽盘、`rroot` 加 `racct,rlin` 或 `rall`（R 的根槽也读回全零，`S2_PERSIST` 让故障挂载之后一直在，这一串抬 F 读不出 R、护着复活单元的只有 C 圈，是本块唯一保护的那一形）；s7 = 同 s5，用户动作串短、末尾 `U` 正常卸载；用户动作 `o` 覆盖写、`n` 建 inode、`s` 顺序写、`F` 抬 F 到准入上限、`b` 回退一版、`U` 卸载。每一步之后读三个独立读数。下表是「没变异」与 9 个变异（名字解释在 `scripts/mutate.py`）各跑同一批形的汇总，输出原样（`handed_out` = 有 C 落点盖住的槽被新发出去的历史数，`both` = 有分配位隔离位同置的历史数，`unblocked_grew` = C 落点盖住的未挡槽比挂载那一刻多的历史数）：

```
none                           s1  histories=60  handed_out=0   both=0   unblocked_grew=0  
none                           s5  histories=60  handed_out=0   both=0   unblocked_grew=0  
none                           s6  histories=40  handed_out=0   both=0   unblocked_grew=0  
none                           s7  histories=72  handed_out=0   both=0   unblocked_grew=0  
m1_raise_plain                 s1  histories=60  handed_out=0   both=0   unblocked_grew=0  
m1_raise_plain                 s5  histories=60  handed_out=36  both=0   unblocked_grew=40 
m1_raise_plain                 s6  histories=40  handed_out=40  both=0   unblocked_grew=40 
m2_newmethod_noiso             s1  histories=60  handed_out=0   both=0   unblocked_grew=0  
m2_newmethod_noiso             s5  histories=60  handed_out=36  both=0   unblocked_grew=40 
m2_newmethod_noiso             s6  histories=40  handed_out=40  both=0   unblocked_grew=40 
m3_record_root_noiso           s1  histories=60  handed_out=32  both=8   unblocked_grew=32 
m3_record_root_noiso           s5  histories=60  handed_out=0   both=0   unblocked_grew=0  
m3_record_root_noiso           s6  histories=40  handed_out=0   both=0   unblocked_grew=0  
m4_skip_alloc_check_removed    s1  histories=60  handed_out=0   both=60  unblocked_grew=0  
m4_skip_alloc_check_removed    s5  histories=60  handed_out=0   both=28  unblocked_grew=0  
m4_skip_alloc_check_removed    s6  histories=40  handed_out=0   both=20  unblocked_grew=0  
m5_reiso_first_slot_only       s1  histories=60  handed_out=0   both=0   unblocked_grew=32 
m5_reiso_first_slot_only       s5  histories=60  handed_out=0   both=0   unblocked_grew=40 
m5_reiso_first_slot_only       s6  histories=40  handed_out=0   both=0   unblocked_grew=40 
m6_protect_abandoned_only      s1  histories=60  handed_out=0   both=0   unblocked_grew=0  
m6_protect_abandoned_only      s5  histories=60  handed_out=36  both=0   unblocked_grew=40 
m6_protect_abandoned_only      s6  histories=40  handed_out=40  both=0   unblocked_grew=40 
m7_immediately_arm_plain       s1  histories=60  handed_out=0   both=0   unblocked_grew=0  
m7_immediately_arm_plain       s5  histories=60  handed_out=0   both=0   unblocked_grew=0  
m7_immediately_arm_plain       s6  histories=40  handed_out=0   both=0   unblocked_grew=0  
m7_immediately_arm_plain       s7  histories=72  handed_out=0   both=0   unblocked_grew=48 
m8_extend_arm_plain            s1  histories=60  handed_out=0   both=0   unblocked_grew=0  
m8_extend_arm_plain            s5  histories=60  handed_out=0   both=0   unblocked_grew=0  
m8_extend_arm_plain            s6  histories=40  handed_out=0   both=0   unblocked_grew=0  
m8_extend_arm_plain            s7  histories=72  handed_out=0   both=0   unblocked_grew=0  
m9_clear_before_reclaim_all_c  s1  histories=60  handed_out=32  both=8   unblocked_grew=32 
m9_clear_before_reclaim_all_c  s5  histories=60  handed_out=0   both=0   unblocked_grew=0  
m9_clear_before_reclaim_all_c  s6  histories=40  handed_out=0   both=0   unblocked_grew=0  
```

读法：没变异的一行行全 0（s7 只对 none、m7、m8 跑了）。每个会让某条回收路漏补的变异都被至少一批形抓到：m1（抬 F 那一串换回只回收）、m2（新方法里补隔离那一步拿掉）、m6（补隔离只看被抛弃根、不看 C）在 s5、s6 上 36–40 个历史里有 C 落点被发出去；m3（挂载内记根不补）、m9（挂载内记根时护着的集合取空）在 s1 上 32 个历史里被发出去；m4（C 圈不看分配位）被 `both` 抓到；m5（补隔离只补回收记录的第一个槽）被 `unblocked_grew` 抓到而没发出去；m7（卸载那一串的当场放开那一截回收换回只回收）只被 s7 的 `U` 抓到（48 / 72 个历史里卸载之后 C 落点盖住的槽有未挡的），没有一个历史把它发出去；m8（卸载那一串里扣住那一截换回只回收）一批都没抓到（见 K2）。这说明探针对「漏补」有判别力，没变异那一行的 0 才算数（evidence-discipline「校验路径本身也要证明它会红」）。

不变异的树上另扫了三个目录共 2 248 个历史（A 960、D 648、C 640；形：1536 / 768 / 3072 槽盘，前缀 `o` × 10–63、`s` × 22、`ons` × 8，R = b2 / b3 / b4 / b6 / b8，故障 racct、rall、rlin、racct+rlin、oldacct，用户动作串随机 20–45 步、字母表 `onsFb`，不带 `S2_PERSIST`），汇总输出（目录 `runs/A`、`runs/D`、`runs/C`；A 四行行尾括号里的话是我后加的，其余原样）：

```
A/a1.log histories=240 both_max>0:0（这一批没有 handed_out 读数）
A/a2.log histories=240 both_max>0:0（这一批没有 handed_out 读数）
A/a3.log histories=240 both_max>0:0（这一批没有 handed_out 读数）
A/a4.log histories=240 both_max>0:0（这一批没有 handed_out 读数）
D/d1.log histories=324 handed_out>0:0 both_max>0:0
D/d2.log histories=324 handed_out>0:0 both_max>0:0
C/c1.log histories=160 handed_out>0:0 both_max>0:0
C/c2.log histories=160 handed_out>0:0 both_max>0:0
C/c3.log histories=160 handed_out>0:0 both_max>0:0
C/c4.log histories=160 handed_out>0:0 both_max>0:0
```

（`runs/A` 那一批跑在加 `handed_out` 读数之前，只有 `both` 与 `c_unblocked`；它的 a4 里有 40 个历史 `c_unblocked_max` 大于 0，见 1.2。）

结论 K1-1：**一致（没打中）**。回收入口静态上全罩到，动态上 9 个漏补变异里 8 个被探针抓到、没变异的树上有 `handed_out` 读数的 D、C 共 1 288 个历史零个被发出去，A 的 960 个 `both` 零。
什么现象会推翻它：任一历史里 `c_handed_out_max > 0`（根环表还记着的 C 落点盖住的槽被这个进程新发出去），或池级 checker 在这条历史上红 I-7.4 / I-4.8；或在冻结副本里找到第 5 个 `mark_reclaimed` 的产品调用点。
只供测试的复用窗口置 0（`crates/singlefs-core/src/allocator.rs:1842`）回收不补，照正文第二节「已知没做」，不算打中。

### 1.2 K1-2　按槽跳过补全了吗；起点槽豁免叠上去有没有一形该隔离的被豁免

应该是（条款乙是槽一级的）：一个落点一部分槽分配位置着、一部分没置，没置的当场隔离，置着的回收那一刻补。实现：当场那一半 `isolate_the_slots_not_allocated` 逐槽判，「if !self.allocated[index] {」（`crates/singlefs-core/src/allocator.rs:606`）；补隔离那一半 `isolate_reclaimed_slots_the_isolation_still_protects`（`crates/singlefs-core/src/allocator.rs:2435`）对每条回收记录逐槽问护着的集合盖不盖它，盖法是「fn covers(&self, device: DeviceIdentity, slot: SlotNumber) -> bool {」（`crates/singlefs-core/src/allocator.rs:980`），按跨度盖，不按起点。两半合起来，一个落点里先隔离的和后补的拼得满。变异证据：只隔离落点第一槽（z14）被入库用例 2 条抓到，只补回收记录第一槽（m5）被 2 条抓到（K2 的表）。

起点槽豁免：`referenced_by_the_current_account` 取现行账里未释放记录的（盘、起点槽），建在「.filter(|record| !record.is_released)」（`crates/singlefs-core/src/mount.rs:1573`），C 圈用它整个落点跳过：「if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {」（`crates/singlefs-core/src/mount.rs:1652`）。按槽跳过上线之后，这个豁免对「分配位置着」的部分是多余的，只会在「同一起点、跨度不同」时多豁免：未释放记录是 (S, 1)、C 落点是 (S, 2) 时 S+1 没隔离也没分配。我推它在协议里到不了（读码推导，没造出反例）：未释放记录属于所选那一版 V 的账，要在水位之上的发布里换成别的跨度，它得先释放（释放代大于 V）再回收，回收门槛不高于环里最旧有效根，而 V 自己就在环里、有效，门槛不会越过 V。要造它得手写盘上的账。判：没打中，写明未造。

同一行里的另一半 `!isolated.insert(key)` 也按起点槽去重，这一半我造出来了：

```
$ grep -E 'CLOOP' runs/dbg/d3.log   （1536 槽盘、前缀 o×40、回退 b2、故障 rall，用户动作 o；开 S2_DEBUG_CU 时在 C 圈打的行）
test s2_sweep ... CLOOP dev=0 slot=1544 span=1 in_current_account=false already_isolated_key=false
CLOOP dev=1 slot=1544 span=1 in_current_account=false already_isolated_key=false
CLOOP dev=0 slot=1544 span=2 in_current_account=false already_isolated_key=true
CLOOP dev=1 slot=1544 span=2 in_current_account=false already_isolated_key=true
$ grep -E 'SRC1544' runs/dbg/d9.log   （同一形，C 表里起点 1544 的两份各从哪来）
SRC1544 span=1 txg=44 counter=44 named=[(2, 1541), (2, 1542), (2, 1543), (2, 1544), (3, 1546), (2, 1548), (2, 1549), (2, 1550)] pointers=[tree=1550 map=1549 inst=1408 alloc=0]
SRC1544-NEWERROOT ring_slot=RootRingSlot { region: 2, slot: 6 } span=2
$ grep 'WATCH' runs/dbg/d5.log | 盘 0 | 隔两行取一行   （同一形，动作 n × 26，每步之后槽 1545 的三位 (分配, 隔离, 扣住) 与账里盖着它的记录）
WATCH step=0n dev=DeviceIdentity(0) slot=1545 bits=(false, false, false) recs=[(1544, 2, 11, true)]
WATCH step=2n dev=DeviceIdentity(0) slot=1545 bits=(false, false, false) recs=[(1544, 2, 11, true)]
WATCH step=20n dev=DeviceIdentity(0) slot=1545 bits=(false, false, false) recs=[(1544, 2, 11, true)]
WATCH step=22n dev=DeviceIdentity(0) slot=1545 bits=(false, false, false) recs=[(1544, 2, 11, true)]
WATCH step=24n dev=DeviceIdentity(0) slot=1545 bits=(true, false, false) recs=[(1545, 1, 71, false)]
```

读法：C 表里起点 1544 有两份——第一份 (1544, 1) 来自记录点名的单元（R 把 1544 重新分配成一个一槽节点，`named` 里类标 2 的单元占一槽，`span_in_slots_of_a_named_unit` 在 `crates/singlefs-core/src/recovery.rs:372`），第二份 (1544, 2) 来自较新可读根的账那一支（`SRC1544-NEWERROOT`，宽护并进现行账里全部已释放的落点，现行账里那条已释放记录是 1544 起的一个两槽数据单元）。第一份先进 `isolated` 集合，第二份被 `!isolated.insert(key)` 跳过，槽 1545 一位都没置，挂载那一刻 `c_unblocked_mount=2`（两块盘各一槽）。这一形在 1536 与 3072 槽盘 R = b2、b4、故障 rall 与 racct+rlin 的 64 / 640 个历史上出现，改前树同形 64 / 640（`runs/Cold`），不是这一块引入的；整个动作串里 1545 要到第 25 个 `n` 才被发出去（上面 WATCH 的最后一行，`bits=(true, false, false)` 记录 `(1545, 1, 71, false)`），而同一形带池级 checker 的复跑里 R 在第 21 步已离开根环（`r_left_at=21`，`runs/dbg/d4.log`），`first_red=-`。
为什么不判成 I-7.4 红：(1544, 2) 是宽护的超集——1544 在现行账里的两槽单元早已回收，R 自己把 1544 重用成一槽节点，没有任何候选根还合法引用 1545。所以这是「C 表比实际隔离的集合多罩一槽」的同族观察，当前没有一个读数红。它会变成红的条件：同一起点上较宽的那份是某条合法候选根引用的、较窄的那份先到。我按协议推这两份不会同时合法引用（同一槽在两条根里跨度不同，必有一条已被回收门槛放过），没造出反例。

结论 K1-2：**一致（没打中）**，附上面一条同族观察（C 表与实际隔离集合在「同起点不同跨度」上不一致，改前就有）。
什么现象会推翻它：造出未释放记录与 C 落点同起点、跨度不同而 C 落点多出来的槽被发出去的历史；或 `c_handed_out_max > 0` 且被发出去的槽属于 C 表里较宽的那一份。

### 1.3 K1-3　第九项与「已分配」还重复扣吗；少扣吗

先逐支对条款（实现的 `isolate_slots_referenced_only_by_abandoned_roots` 在 `crates/singlefs-core/src/mount.rs:1558`，三个隔离圈）：

| 支 | 条款（上面第 27 行）要的 | 实现 | 比 |
|---|---|---|---|
| 被抛弃根的落点 | 被抛弃根引用的槽 − 候选集里的根引用的槽 | 整个落点 `isolate`（`allocator.isolate_abandoned(device, slot, span_slots);`，`crates/singlefs-core/src/mount.rs:1622`），豁免 = 候选根引用的 ∪ 现行账里未释放的起点 | 一致；defer 窗口里的槽被抛弃根独占时仍两项各扣一次，条款 丙 没要求除去它，是正文第二节「已知没做」（第十六块） |
| 根槽读坏的被抛弃根 | 同上 | 同一个豁免、同一个整落点 `isolate` | 一致；同上 |
| C 圈（预隔离） | 只除去现行账里仍分配着的槽，那些算在「已分配」里 | 逐槽跳过分配位置着的（`allocator.rs:606`），回收时补 | 见下 |

C 圈那一格条款的字面是「现行账里仍分配着的槽」，实现跳的是分配位置着的槽——后者多包含已释放而还在 defer 窗口里的（记录 `is_released = true`，分配位还置着，`admission.rs:560` 那句注释说它们算在「已分配」里）。括注的两个理由里，「那些算在「已分配」里，不在这里再算一次」对 defer 窗口里的槽字面成立；「这次挂载里释放、回收时分配器照补隔离」对挂载之前就已释放的 defer 槽字面不是「这次挂载里释放」，但它们回收时同样由分配器补（`allocator.rs:2306`、`mount.rs:4730`）。读法依赖括注：按括注是**同义**，只按「仍分配着」四个字（记录未释放）实现是更宽的一支。我把这格记成「规则没说得死」，不记冲突；写回条款时建议把「仍分配着」落成「分配位置着」，由主 agent 定。

数（改前改后各一件，输出原样；`both` = 分配位与隔离位同置的槽数，`iso_mount` = 挂载那一刻两块盘隔离的槽数，`refused_steps` = 之后六步里被准入拒的步数；384 槽那一形与入库用例文字里的 720、616、104 对得上）：

```
== EV/e-old
selected=43 c_r=774 iso_mount=838 iso_end=838 both_mount=540 both_max=540 c_unblocked_mount=2 refused_steps=0
== EV/e-core
selected=43 c_r=774 iso_mount=298 iso_end=326 both_mount=0 both_max=0 c_unblocked_mount=2 refused_steps=0
== EV/e384-old
selected=48 c_r=660 iso_mount=720 iso_end=720 both_mount=104 both_max=104 c_unblocked_mount=0 refused_steps=6
== EV/e384-core
selected=48 c_r=660 iso_mount=616 iso_end=616 both_mount=0 both_max=0 c_unblocked_mount=0 refused_steps=6
```

1536 槽盘：改前 838 − 同置的 540 = 改后 298；384 槽盘：720 − 104 = 616。两个减法都对得上，说明访问器数的就是被双扣的那几槽，也说明新方法没有少隔离分配位没置着的槽（隔离数恰好少了同置的那些）。同一批形的汇总对比（改前树 `runs/Cold`、新树 `runs/C`，各 4 件 640 个历史，输出原样）：

```
Cold/c1.log histories=160 handed_out>0:0 both_max>0:128 c_unblocked_max>0:32
Cold/c2.log histories=160 handed_out>0:0 both_max>0:128 c_unblocked_max>0:0
Cold/c3.log histories=160 handed_out>0:0 both_max>0:128 c_unblocked_max>0:0
Cold/c4.log histories=160 handed_out>0:0 both_max>0:128 c_unblocked_max>0:32
C/c1.log histories=160 handed_out>0:0 both_max>0:0 c_unblocked_max>0:32
C/c2.log histories=160 handed_out>0:0 both_max>0:0 c_unblocked_max>0:0
C/c3.log histories=160 handed_out>0:0 both_max>0:0 c_unblocked_max>0:0
C/c4.log histories=160 handed_out>0:0 both_max>0:0 c_unblocked_max>0:32
```

少扣一截：第九项就是隔离位的槽数（准入读 `isolated_slots()`：「BytesOnOneDevice::of_slots(device_map.isolated_slots()),」（`crates/singlefs-core/src/admission.rs:586`），定义上不会少于分配器实际隔离的集合。回收那一刻槽从「已分配」挪进第九项：`allocator.rs:717` 把已分配减 span，补隔离把隔离加同一批槽，可用净变化 0（没有「该算进第九项的被当成已分配」的缝）。探针里新树 640 个历史、每一步之后 `both_max` 全 0。
我没造出的：被抛弃根（崩溃恢复造出的，不是 R 未见证）独占的 defer 槽那一形（上表第一行）——我的形都是 R 未见证 / 回退，不是崩溃抛弃，所以 `both` 的 0 不覆盖它；这一形已知没做。

结论 K1-3：**一致**（读法依赖括注，见上）；没看到少扣；被抛弃根独占 defer 槽的双扣是已知没做、我没造。
什么现象会推翻它：新树上任一历史 `both_max > 0`（C 圈或回收补隔离造出的同置）；或 `iso_mount` 不等于「改前 `iso_mount` − `both_mount`」；或一个槽既不在「已分配」又不在隔离、却被 C 表罩着（`c_unblocked_max > c_unblocked_mount`，m5 变异下出现）。

## 2 K2　第十四块 判别力

### 2.0 应该是什么

判决第 42 行第 1 条的验收是把攻方 2242 槽那一形与正推 384 槽 720 槽那一形做成 harness 档用例，钉住「同一个槽不同时算进已分配与已隔离」（`research/prompts/m3-batch10z-lm-code-r1-main-verification.md:42`）；K2 另要读盘一层两处与墓碑片读不出的用例会红。「改回去」要被拦；访问器自己不能成为用例唯一的信号。

### 2.1 实现员那 12 行（10 行追加 + 2 行整行替换）能不能拦住

逐行按实现员给的旧串 / 新串套在冻结副本拷贝上、按行里的命令跑点名的用例（`scripts/rowrun.py`，读盘一层与替换两行）；H2 那五行我用文字相同的变异（m0 = 第 1、2 行同一个文本；m4 = 第 3 行；m1 = 第 4 行；m2 = 第 5 行）在入库的整个用例文件上跑（含 ignored，24 条），输出原样：

```
$ python3 scripts/rowrun.py gpu/n14-rows.json n14-5 n14-6 n14-7 n14-8 n14-9 n14-10 n14-11   （读盘一层 5 行、替换 2 行；每行 rc 101 = 点名的用例红）
n14-5 第10z批第十四块 K2 读盘一层只看第一个打包单元（删除发布墓碑片读不出那一形） | rc 101 | ok 0 failed 1
     FAILED: a_deletion_whose_tombstone_unit_and_account_read_back_zero_at_the_mount_takes_every_released_placement ... FAILED
n14-6 第10z批第十四块 K2 读盘一层只看第一条记录点名的打包单元（顺序写末条点名叶容器那一形） | rc 101 | ok 0 failed 1
     FAILED: a_sequential_write_whose_inode_unit_named_by_its_last_record_and_account_read_back_zero_at_the_mount_takes_every_released_placement ... FAILED
n14-7 第10z批第十四块 K2 读得出而解不开的打包单元当解开了（旧写法 ok() 吞掉） | rc 101 | ok 0 failed 1
     FAILED: mount::packed_units_named_by_the_records_of_a_publish_as_read_tests::every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart ... FAIL
n14-8 第10z批第十四块 K2 读盘一层只看第一个打包单元（读盘一层的库用例） | rc 101 | ok 0 failed 1
     FAILED: mount::packed_units_named_by_the_records_of_a_publish_as_read_tests::every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart ... FAIL
n14-9 第10z批第十四块 K2 读盘一层只看第一条记录点名的打包单元（读盘一层的库用例） | rc 101 | ok 0 failed 1
     FAILED: mount::packed_units_named_by_the_records_of_a_publish_as_read_tests::every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart ... FAIL
n14-10 步 5：抬 F 回收的槽不扣住、生效之前就能发出去 | rc 101 | ok 0 failed 1
     FAILED: slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect ... FAILED
n14-11 C518（一次挂载之内环转过一圈之后不回收）：挂载内回收出来、被抛弃根还引用着的槽不补隔离（D23 已定项 14 主句） | rc 101 | ok 0 failed 1
     FAILED: slot_reclaimed_while_an_abandoned_root_still_references_it_stays_isolated_until_that_root_leaves_the_ring ... FAILED
$ 入库用例文件（24 条含 ignored）在 H2 五行的等价变异与我加的变异上，runs/RM/<名>.log 汇总：
none passed=24 failed=0
m0_c_loop_isolate_abandoned passed=21 failed=3
    FAILED: revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero
    FAILED: wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
m1_raise_plain passed=23 failed=1
    FAILED: revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero
m2_newmethod_noiso passed=23 failed=1
    FAILED: revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero
m3_record_root_noiso passed=23 failed=1
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
m4_skip_alloc_check_removed passed=21 failed=3
    FAILED: revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero
    FAILED: wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
m5_reiso_first_slot_only passed=22 failed=2
    FAILED: revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
m6_protect_abandoned_only passed=23 failed=1
    FAILED: revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero
m7_immediately_arm_plain passed=24 failed=0
m8_extend_arm_plain passed=24 failed=0
m9_clear_before_reclaim_all_c passed=22 failed=2
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
    FAILED: units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written
z2_accessor_zero passed=24 failed=0
z3_exempt_released_too passed=22 failed=2
    FAILED: wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
z5_isolated_counter_not_bumped passed=18 failed=6
    FAILED: a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount
    FAILED: session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements
    FAILED: revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero
    FAILED: wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices
    FAILED: units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
z13_skip_held_slots passed=24 failed=0
z14_c_only_first_slot passed=22 failed=2
    FAILED: a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
m0_c_loop_isolate_abandoned+z2_accessor_zero passed=23 failed=1
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
m4_skip_alloc_check_removed+z2_accessor_zero passed=23 failed=1
    FAILED: wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
```

读法：12 行的点名用例全红，**这 12 行都拦得住**。五个 H2 变异里：C 圈改回整落点隔离（m0 / 第 1、2 行）、分配器那一判拿掉（m4 / 第 3 行）、抬 F 回收改回只回收（m1 / 第 4 行）、新方法里补隔离拿掉（m2 / 第 5 行）各被 1–3 条抓到，m3（挂载内记根不补）被 1536 槽慢档那一条抓到，m5、m6、m9、z3、z5、z14 也都有红。

### 2.2 访问器会不会算错让用例照绿：会（冲突，窄）

判决要的是「钉住」，钉的信号是 `slots_counted_both_as_allocated_and_as_isolated`：访问器的实现是「.filter(|(is_allocated, is_isolated)| **is_allocated && **is_isolated)」（`crates/singlefs-core/src/allocator.rs:630`），我读它是对的，改前树同一份代码在 1536 槽那一形数出 540、384 槽那一形数出 104，与 `iso_mount` 的差对上（1.3 的两个减法）。但入库用例里它的全部 6 处调用都是同一种写法——「assert_eq!(」后一行 `slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),` 或 `(&pool.session().allocator),`，下一行 `0,`（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:2588`、`2769`、`2815`、`3145`、`3215`、`3231`），没有一处断言它大于 0（正控），库里也没有它的单测（`grep -rn` 只在 `allocator.rs:625` 的定义与这个用例文件里命中）。于是访问器恒返 0（z2）整个文件 24 条照绿（上表 `z2_accessor_zero`）；把 H2 的主改法改回去再叠上 z2（`m0_c_loop_isolate_abandoned+z2_accessor_zero`、`m4_skip_alloc_check_removed+z2_accessor_zero`），**只剩 1536 槽慢档（带 ignored）那一条红**，快档的两条（384 槽那一条、复活那一条）照绿。

并排：
- 判决：要钉住的是「同一个槽不同时算进已分配与已隔离」（`research/prompts/m3-batch10z-lm-code-r1-main-verification.md:42`）；
- 实现：快档两条用例钉它只靠一个只有 `== 0` 断言的访问器（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:2588`、`3145` 为快档两处），慢档那条另有「一次都不拒」的信号才抓得住。
什么现象会推翻它：库里或用例文件里出现一条让访问器返回大于 0 的正控（例如手工同置一槽再断言返 1），或把 z2 套上后快档仍有一条红。

### 2.3 第二处：卸载那一串当场放开的那一截没有用例

m7（卸载那一串里「当场放开」那一截的回收换回只回收，`crates/singlefs-core/src/mount.rs:4506` 那一支）与 m8（同一支里扣住那一截的 `reclaimed.extend(reclaim_released_in_the_sequence_raising_the_floor_up_to(`，`crates/singlefs-core/src/mount.rs:4526`）两个变异，入库 24 条全绿（上表）。探针加动作 `U` 之后抓得到 m7：s7 批 72 个历史里 48 个在卸载之后有 C 落点盖住的槽未挡（1 节的表），没有一个被发出去（为什么没有我没查：可能卸载那一串后面的发布不再分配单元）。入库的卸载用例（`fn unmount_expecting_the_floor_raised(`，`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:516`）返回卸载之后的分配器并断言 C 落点不空闲，被它调的三处（用例文件第 551、737、988 行）在 m7、m8 上照绿；这一块新加的用例（第 2380 行之后）没有一处卸载。三处的形是否含「挂载时 defer、卸载时才回收」的 C 槽我没逐条核，只看到 m7 在它们上存活。
m8 探针也没抓到（s7 的 72 个历史 `unblocked_grew` 为 0）。原因我没查清：可能是这些形里扣住的那一截是空的，也可能是扣住位在卸载结束时仍置着；不判等价也不判缺口。m7 是当场放开的那一截，卸载那一串里后面还有发布，窗口真在，我不把它当等价。
我另造的 z13（C 圈把「抬 F 扣住」位置着的槽也跳过）入库 24 条照绿：对应「卸载中途崩溃之后的下一次挂载只放开一截」那一形，挂载那一刻分配位已清、扣住位置着；这是我造的变异，不是实现员的行，当低价值记。

结论 K2：**冲突（窄）**——12 行都拦得住；缺口两处：访问器没有正控（改回去 + 访问器坏 → 快档照绿，只有慢档那一条抓），卸载那一串当场放开那一截没有用例（m7 存活）。

## 3 K3　第十五块 借之前那一步看每张卡（显存账）

### 3.0 应该是什么

用户原话（派发提示转述）：全局的任务只管会不会对指定的电脑造成 CPU 或 GPU 的 OOM，不压并发。判决 K3-3 的验收：「借之前也让账知道丢了（或改按读数分），不多给，借得到时不在重起时被拒」（`research/prompts/m3-batch10z-lm-code-r1-main-verification.md:46`）。从机制推：账给的量 = 能占的 − 借着的，能占的 = 现读空闲 + 这个进程实占里属于它许出去的那一部分；只要有一台丢了的设备的缓冲或驱动那一份还可能占在卡上，「属于许出去的那一部分」必须取 0，否则把别人（丢了的设备）占的算成自己的、多给。所以每一次借之前，账必须先对每张卡的丢没丢表态，再算量；不表态只会发生在丢了而账还没看见的窗口里。

### 3.1 实现

借之前那一步：「pub fn take_in_the_devices_kept_before_booking<Device>(」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1448`）；循环对池里每张卡：「for card_position in 0..card_count {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1457`）；留着的设备丢了就「if is_lost(&kept) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1468`）告诉账（`take_in_that_the_device_was_found_lost`，只在「if books_of_the_card.device_in_use == Some(self.device) {」时动账，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1688`），动账就是重起那一套 `take_back_the_driver_share_of_a_lost_device`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:868`）；没丢、这张卡列出来了才「if found_idle_by_a_poll(&kept) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1478`）。列卡在循环之前一次。

### 3.2 K3-1　多给

我对机制读了这几种交错，全部单调（只会少借，不会多借）：
1. 丢了的设备旧克隆还占着卡：账把它记进 `lost_devices_not_released`，新设备起好后进 `DeviceStartedAgainWhileTheLostDeviceRemains`，占着的部分取 0；放掉（最后一个克隆丢掉）才回到 `DeviceStarted`。入库用例 `the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again`（5694 行）管；我的变异 G12（丢了的不记进还没放掉的那一堆）被 5 条抓到。
2. 两线程：线程 A 看到丢了、线程 B 已重起好。A 的 `take_in_that_the_device_was_found_lost` 比的是设备号，B 起好的是新一台，号对不上，A 什么都不改（`a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone`，5421 行；我的变异 G7 把号比较换成恒真，被它抓到）。反过来 B 的 `prepare…` 先把在用的记成丢了，A 再来时 `device_in_use` 已是 `None`，也是空操作；`record_the_device_started_on_the_card` 里「在用的那一台没记成丢了就又起了一台」的断言在两种次序下都不触发（A 先动：`device_in_use` 变 `None`；B 先动：A 空操作）。
3. 重起正在进行：取留着的设备不等正在起的（`started_now`），那一格没有就跳过、继续看后面的卡（`a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease`；实现员行 `repo2` 与我的 G1 都被抓到）。
4. 残余窗口：借的线程看过 `is_lost` 之后设备才丢，账不知道，这一笔按旧账多给至多驱动那一份（模型里 600 MiB）；借走之后重起时 `driver_share_of_a_new_device_fits_now`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1347`）照此刻再判，放不下就拒、不起新设备，报本次未跑——被重起登记兜住，没有 OOM。这一窗口在这一块之前就有（原先到重起才知道），这一块把「列不出来的卡」也看了，窗口只剩看与借之间。它是「账不可能与丢设备原子」的下限，我不记成打中。

模型证据：入库库单测 `cargo test --release -p singlefs-harness --lib gpu_memory_ledger`（`crates/singlefs-harness` 是 harness 档，不是 checker 档）123 条全过（`logs/gpu-lib-baseline.log`）：

```
$ <lib 测试二进制> gpu_memory_ledger   （经 run-with-memory-cap.sh 4G、capped.sh 4）
test result: ok. 123 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.50s
```

其中随机模型 `random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:5977`）4 000 个种子、每串 14 步，设备没丢或丢了而账已看见时每一步之后「能借的不超过卡上真能再给的」。
借之前那一步的变异（实现员落在显存账的 9 行——追加 7 行、整行替换 2 行——全跑；再加我自己写的 9 个；`gpu/gmut.py`，每个变异在冻结副本 `gpu_memory_ledger.rs` 上套一次、编库单测、跑 `gpu_memory_ledger`），输出原样：

```
$ python3 gpu/gmut.py gpu/repo-rows.json 0 1 2 3 4 5 6 16 17   （实现员落在显存账的 9 行：追加 7 行 + 整行替换 2 行，编号是 gpu/repo-rows.json 里的下标）
0 repo0 | rc 101 passed 121 failed 2
     FAILED: a_device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart
     FAILED: random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss
1 repo1 | rc 101 passed 121 failed 2
     FAILED: a_device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart
     FAILED: random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss
2 repo2 | rc 101 passed 122 failed 1
     FAILED: a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease
3 repo3 | rc 101 passed 121 failed 2
     FAILED: random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss
     FAILED: the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again
4 repo4 | rc 101 passed 122 failed 1
     FAILED: a_lease_kept_after_the_listing_survives_the_poll_before_booking
5 repo5 | rc 101 passed 120 failed 3
     FAILED: a_device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left
     FAILED: a_device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart
     FAILED: random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss
6 repo6 | rc 101 passed 120 failed 3
     FAILED: a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease
     FAILED: a_card_without_a_kept_lease_is_not_polled_before_booking
     FAILED: every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease
16 repo16 | rc 101 passed 119 failed 4
     FAILED: a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease
     FAILED: a_card_without_a_kept_lease_is_not_polled_before_booking
     FAILED: a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion
     FAILED: every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease
17 repo17 | rc 101 passed 122 failed 1
     FAILED: a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion

real	1m55.618s
user	8m18.468s
sys	0m15.252s
$ python3 gpu/gmut.py gpu/my-rows.json all   （我的 9 个：G1 借之前丢了不 continue、G2 只核账不核卡位、G7 找到丢了不比设备号、G8 poll 结局取反、G13 丢没丢恒假、G14 列卡恒空、G12 丢了的不记进还没放掉的、G15 找到丢了不销没等成的借据、G16 没列出来的卡也拿别的卡列出来的去 poll）
0 G1_no_continue_after_lost | rc 101 passed 119 failed 4
     FAILED: a_device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left
     FAILED: a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone
     FAILED: a_lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again
     FAILED: the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again
1 G2_assert_only_ledger_not_card | rc 101 passed 122 failed 1
     FAILED: a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion
2 G7_found_lost_ignores_device_number | rc 101 passed 122 failed 1
     FAILED: a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone
3 G8_negate_poll | rc 101 passed 117 failed 6
     FAILED: a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease
     FAILED: a_card_without_a_kept_lease_is_not_polled_before_booking
     FAILED: a_device_still_busy_at_the_poll_before_booking_keeps_its_kept_lease_on_the_books
     FAILED: a_lease_kept_after_the_listing_survives_the_poll_before_booking
     FAILED: every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease
     FAILED: random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss
4 G13_is_lost_ignored | rc 101 passed 117 failed 6
     FAILED: a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone
     FAILED: a_device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left
     FAILED: a_device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart
     FAILED: a_lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again
     FAILED: random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss
     FAILED: the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again
5 G14_listing_empty | rc 101 passed 118 failed 5
     FAILED: a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease
     FAILED: a_card_without_a_kept_lease_is_not_polled_before_booking
     FAILED: a_device_still_busy_at_the_poll_before_booking_keeps_its_kept_lease_on_the_books
     FAILED: a_lease_kept_after_the_listing_survives_the_poll_before_booking
     FAILED: every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease
6 G12_lost_not_remembered | rc 101 passed 118 failed 5
     FAILED: a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
     FAILED: a_lost_device_still_held_outside_the_leases_keeps_the_card_unsplittable_until_it_is_released
     FAILED: a_second_loss_while_the_first_lost_device_remains_keeps_the_card_unsplittable_until_both_are_released
     FAILED: random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss
     FAILED: the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again
7 G15_found_lost_keeps_kept_leases | rc 101 passed 118 failed 5
     FAILED: a_device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left
     FAILED: a_lease_kept_after_a_failed_wait_leaves_the_books_at_the_restart_and_the_card_is_not_left_short
     FAILED: a_lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again
     FAILED: a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more
     FAILED: the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again
8 G16_poll_before_lost_check_order | rc 101 passed 120 failed 3
     FAILED: a_card_without_a_kept_lease_is_not_polled_before_booking
     FAILED: a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease
     FAILED: every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease

real	1m53.875s
user	8m10.098s
sys	0m14.359s
```

读法：18 个变异（实现员 9 + 我 9）在 123 条库单测上全红，每个都至少被 1 条抓到，而且抓它的用例都在「看每张卡」「丢了告诉账」「只销列出来的」「设备号比较」这几处各自的名下，不是同一条在兜。**K3-1 没打中；K4 的显存账一侧拦得住。**
什么现象会推翻 K3-1：库单测或随机模型出一条「能借的 > 卡上真能再给的」，且发生在账已看见丢了之后；或用真卡 / 假卡两线程交错造出「看过之后丢」以外的多给。

### 3.3 K3-2 / K5-2　卡放得下却报本次未跑、判红、整趟弃卡、并发被压

- 并发被压：这一步的代价是列卡一次锁、取卡数一次锁，每张卡 `is_lost`（一把互斥量）；丢了的卡再一次锁；poll 只对列出来（有等设备没等成而照旧记着的借据）的卡做，且在账的锁外（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1411` 的列卡、`:1478` 的 poll）。没列出来的卡不 poll 由 G16 钉死（3 条红）。借的判与记仍在一次锁里（`book_in_the_lock`），没有加新的串行段。
- 少借（用户已认）：丢了之后「占着的取 0 + 借着的整笔扣」会比卡上真能再给的少（活着的借据已经体现在读数的占用里，又被扣一次），这是正文第二节「用户已认」的那一类，我没把它记成打中。
- 实现员报告第三节说同一个攻方随机模型上改前改后「卡放得下却未跑」合计都是 59 678（改前 57 479 次借不到 + 2 199 次借到之后重起被拒，改后 59 678 次借不到 + 0 次重起被拒）：那是把原来重起时才拒的 2 199 次提前到借的时候拒，总数不变。那是上一轮攻方的随机模型，我没拷过来改接口复跑：**没复核**（我用库单测里的随机模型和上面的变异代替，它们答的是「多给」，答不了这个总数）。
- 判红 / 整趟弃卡：那一层在 checker 档（卡池接线与借不到之后的处置），我没跑；读 `take_in_the_devices_found_idle` 的接线只是把四样交给账，没有新的失败出口。

结论 K3-2、K5-2：**一致（没打中）**；K5-1 同 K3-1：**一致（没打中）**，残余只有 3.2 第 4 条的窗口。
什么现象会推翻：任一条借之前那一步的路径，让「池里放得下的卡」的借变成「借不到」且不属于丢了而账不分两态的那一类；或这一步在锁里 poll / 在每次借时对没有借据的卡 poll。

## 4 K4　第十五块 判别力

### 4.0 应该是什么

判决 K4 要的是：扫源码只看文本，拦不住改了行为而文本仍对的写法，所以下游各层叫停与卡池接线换成行为用例，换不掉的在用例旁写明它拦不住哪几种——原文「换不掉的在用例旁写明它拦不住哪几种写法」（`research/prompts/m3-batch10z-lm-code-r1-main-verification.md:34`）。从机制推「写全了」的判据：扫源码用例钉住一段文本，它拦不住的恰是「文本不变而行为变」的那几类；每一类要么有别的用例管，要么写在用例旁。

### 4.1 行为用例一侧

- 账层：正推 T1–T5 进了库单测——T1 `a_lease_kept_after_the_listing_survives_the_poll_before_booking`（5653 行）、T2 `the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again`（5694）、T3 `a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease`（5733）、T4 `a_device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart`（5776）、T5 随机模型（5977）。上面 3.2 的 18 个变异里，Z1（repo2）、Z6（repo3）、Z8（repo4）这三个上一轮「照绿」的都红了。
- 叫停两层（checker 档，我不跑，只读断言）：`the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken`（`crates/singlefs-checker-tier/src/crash_amplification.rs:4849`）用假的判一段与写一块，叫停在第三次被问时交真，断言被问 3 次、写进 [0, 2]、判了 2 块。我逐个推：领块之前问两次（第 2 块前就问到真）→ 写进 [0] 红；问在写之后 → 写进 [0, 2, 4] 红；`break` 换成 `continue` → 被问 4 次红；领块之前不问 → 被问 0 次红；`if false && stop_is_requested()` → 被问 0 次红。`check_blocks_where_until` 那一层由 `a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest`（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:359`）管，叫停在第二次被问时交真，断言第一趟只判 1 块、第二趟判剩下 1 块；往下交 `&|| false` → 第一趟判 2 块红。这三条我**没跑**（checker 档），是读断言推的。

### 4.2 扫源码一侧（我自己的模型）

`scan-model/` 把两条扫源码用例逐字搬出来（`the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking`，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2417`，方法体去掉注释行与空白之后整句比 `let expected_body = concat!(`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2442`）；`every_layer_handed_the_stop_request_hands_it_on_or_asks_it`，`crates/singlefs-checker-tier/src/crash_amplification.rs:4915`，四层逐行比），喂冻结副本和套了变异的副本。基线两条 GREEN。实现员 `repo7`–`repo19` 里落在这两个文件的行与我加的 N1（下面 4.3），输出原样：

```
$ python3 scripts/scanmut.py   （repo<下标> 是 gpu/repo-rows.json 的下标，名字只留尾 40 字）
baseline gpu  : GREEN
baseline crash: GREEN
repo7 z批第十五块 GPU 取卡 K4 借之前那一步方法体最前面整句返回（正推 A2） | gpu | RED: 方法体整句不等于钉住的那一句
repo8  GPU 取卡 K4 借之前那一步丢没丢与 poll 两个闭包对调（正推 A3） | gpu | RED: 方法体整句不等于钉住的那一句
repo9  GPU 取卡 K4 借之前那一步整个包进 if false（攻方 gpu-2） | gpu | RED: 方法体整句不等于钉住的那一句
repo10 evice_found_idle_by_a_poll 恒交真（攻方 gpu-3） | gpu | RED: 方法体整句不等于钉住的那一句
repo11 cks_where_until 往下交永不叫停、另提一句叫停的名字（正推 A7） | crash | GREEN
repo12 e_until 往下交的闭包里提一次叫停的名字而永不叫停（攻方 crash-2） | crash | GREEN
repo13 ocks_in_pieces 领块之前那一问短路成永不问（攻方 crash-3） | crash | GREEN
repo14 的闭包里提一次叫停的名字而永不叫停（攻方 crash-2 那一形挪到要卡的一层） | crash | RED: judge_my_blocks_on_gpu_cards 用到叫停的行不等于钉住的：["&|| {
repo15 ocks 往下交永不叫停、另提一句叫停的名字（正推 A7 那一形挪到要库的一层） | crash | RED: check_selected_blocks 用到叫停的行不等于钉住的：["{ let _ = st
repo18 K5 check_blocks_where_until 往下交的叫停换成永不叫停 | crash | GREEN
repo19  check_pending_blocks_in_pieces 领块之前不问叫停 | crash | GREEN
N1_decoy_if_false_branch_check_selected_blocks | crash | GREEN
```

读法：
- 卡池接线 4 行（A2 整句 return、A3 两个闭包对调、gpu-2 包进 `if false`、gpu-3 另起同名恒真）全红，整句比拦得住。
- 叫停 `repo14`（`judge_my_blocks_on_gpu_cards` 闭包里提一次名字）、`repo15`（`check_selected_blocks` 提一次名字）红。
- `repo11`、`repo12`（`check_blocks_where_until` 的两种写法）、`repo13`（`check_pending_blocks_in_pieces` 的 `if false &&`）、`repo18`、`repo19`（上一块的两行）在扫源码模型上 GREEN，**这是设计**：这两层已经不由扫源码管，由 4.1 的行为用例管（我没跑它们，所以这五行我只能说「扫源码这一侧放行，行为用例这一侧读断言应红」）。

### 4.3 「拦不住的写法」写全了没有

用例旁写明的（`crates/singlefs-checker-tier/src/crash_amplification.rs:4911` 起三行）：「整行比拦得住上面三种写法（多一行、或那一行不是原样）。拦不住的：同一行原样交下去而下一层的函数体里不用」，接着「把整次调用换成另一个同签名的函数；在函数上面加 `#[cfg]` 让另一份同名函数编进来；宏展开出来的调用」。卡池一侧（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2411`）：「整句比也拦不住的（文本都在这个方法体之外）」，列了改 `device_found_idle_by_a_poll`、`GpuCard::device_condition`、`started_card`、`started_now`、`is_lost`、`#[cfg]`、四个入口不调。

我找到一种没列的、编得过的（N1，叫停那一侧）：在 `check_selected_blocks` 里留着真调用但把它的最后一个实参换成 `&|| false`，再在它前面加一个 `if false { let _ = check_blocks_where_until(…, stop_is_requested,\n        ); }` 的死分支，让钉住的那一行 `stop_is_requested,` 和它后面的 `)` 原样留在死分支里。整行比只看「用到叫停名字的行」，这一行还在、下一行还是右括号，两条断言都过；`check_selected_blocks` 又没有行为用例（只有 `run_from_environment` 调它）。编不编得过：在 `cargo check -p singlefs-checker-tier --lib`（草稿拷贝，`CARGO_TARGET_DIR` 自己的，不带 gpu 特性——这一层不在 gpu 特性后面）上 `Checking … Finished`，0 个 error；扫源码模型 GREEN（上表最后一行）。它与「把整次调用换成另一个同签名的函数」同族（都是让钉住的那一行不再是真调用的实参），但落在「真调用还在、只换了实参」，名单没写到。
卡池一侧我另看到一处没列的：方法体里钉的是「started.device().poll(wgpu::PollType::Poll)」，`GpuCard::device()` 返回哪台设备（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:568`）不在整句里，名单里列了 `device_condition` 与 `started_card` 而没列 `device()`；它更像别的用例管的（用设备的各处），我没造变异，记「名单可以再加一条」。

结论 K4：**冲突（窄）**——账层与行为用例这一侧拦得住（18 / 18，叫停两层读断言推得出），卡池接线 4 行扫源码全红；缺口是用例旁的名单没写全（N1 一条、`GpuCard::device()` 一条）。
并排：判决要「换不掉的在用例旁写明它拦不住哪几种写法」（`research/prompts/m3-batch10z-lm-code-r1-main-verification.md:34`）；用例旁写的是上一段引的那三行（`crates/singlefs-checker-tier/src/crash_amplification.rs:4911`）。
什么现象会推翻它：N1 在带 gpu 特性或不带的 `cargo test` 里被现有用例拦下（我没跑 checker 档，只是静态上看到两条断言都过）；或名单里有一句我没读到的把它罩住。

## 5 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 回收路补隔离 | 一致（没打中） | 产品回收入口 5 处全罩到；9 个漏补变异探针抓到 8 个（m8 没抓到），没变异的树上有 `handed_out` 读数的 1 288 个历史（D 648 + C 640）零个 C 落点被发出去 |
| K1-2 按槽跳过与起点槽豁免 | 一致（没打中），附同族观察 | 按槽两半拼得满；起点槽豁免协议内到不了多豁免（读码推，没造）；C 表按起点槽去重让同起点较宽那份少罩一槽（改前就有，当前被罩的槽没有候选根合法引用） |
| K1-3 第九项与已分配 | 一致（读法依赖括注） | 改前双扣 540 / 104 槽，改后 0，减法对得上；条款字面「仍分配着」与实现跳「分配位置着」差 defer 窗口里的槽，靠括注同义；被抛弃根独占 defer 槽的双扣是已知没做、我没造 |
| K2 判别力 | 冲突（窄） | 12 行全拦得住；访问器只有 `== 0` 断言、没有正控，改回去叠访问器坏只剩慢档一条红；卸载那一串当场放开那一截（m7）存活 |
| K3-1 多给 | 一致（没打中） | 18 个变异全红、随机模型 4 000 种子过；残余只有「看与借之间才丢」的窗口，被重起登记兜住 |
| K3-2 少借 / 压并发 | 一致（没打中） | 没列出来的卡不 poll 由 G16 钉死；没加新的串行段；用户已认的少借不记 |
| K4 判别力 | 冲突（窄） | 账层与卡池接线拦得住；用例旁名单缺 N1（真调用换实参 + 死分支留原行）与 `GpuCard::device()` 两条 |
| K5-1 账给的量大于此刻真能用 | 一致（没打中） | 同 K3-1 |
| K5-2 放得下却未跑 / 判红 / 弃卡 / 压并发 | 一致（没打中） | 同 K3-2；判红与弃卡那一层在 checker 档，没跑 |

冲突两格并排整行抄（原样输出，行号去原文件里现取）：

```
$ awk 'NR==42' research/prompts/m3-batch10z-lm-code-r1-main-verification.md
  1. H2：挂载那一刻给 C（含宽护、回退复活、丙并中间各支）的落点置隔离时，跳过分配位置着的槽，靠回收时补隔离；准入第九项与「已分配」不再重复扣。验收：攻方 2242 槽那一形与正推 384 槽 720 槽那一形做成 harness 档用例，钉住「同一个槽不同时算进已分配与已隔离」；I-7.4 照旧不红（回收之后补的隔离照罩着）。

$ awk 'NR==2588 || NR==2589 || NR==2590' crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs   （冻结副本）
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
        0,
        "挂载那一刻没有一个槽同时算进「已分配」与第九项（D28（挂载期承诺量） 已定项 1 第九项：现行账里仍分配着的、已释放而还在 defer 窗口里的不在这里再算一次）"

$ awk 'NR==34' research/prompts/m3-batch10z-lm-code-r1-main-verification.md
| K4 第十三块判别力 | 正推：入库 20 行全红，Z1、Z6、Z8 照绿（补了 T1–T5 账层用例拦下，用例在模型目录）；扫源码用例被三种改了行为、编得过的写法骗过（整句 return、两个同型闭包对调、`let _ = stop_is_requested`）。攻方：另四种（闭包里提一次名字而永不叫停、`if false &&` 短路、借之前那一步整个 `if false` 包起来、同名 `device_found_idle_by_a_poll` 恒真遮蔽），副本上 `cargo check` 都编得过 | **打中**：Z1 Z6 Z8 补用例；扫源码只看文本、看不到行为，拦不住改了行为而文本仍对的写法——下游各层叫停与卡池接线换成行为用例（不要卡的 CPU 判器路径能数叫停被问了几次的就用它），换不掉的在用例旁写明它拦不住哪几种写法 |

$ awk 'NR==4911' crates/singlefs-checker-tier/src/crash_amplification.rs   （冻结副本）
    /// 整行比拦得住上面三种写法（多一行、或那一行不是原样）。拦不住的：同一行原样交下去而下一层的函数体里不用（下一层由它自己那一条管，
```

## 6 要主 agent 跑的命令（checker 档，我按规矩不跑）

三条行为用例我只读了断言，没跑；N1 我只在扫源码模型上放行、在 `cargo check` 上编过。要核的话，在仓的一份拷贝上（或提交时）：

1. 工作目录：仓根拷贝；环境前缀 `SINGLEFS_HEAVY_TESTS=user-request`（或 `=commit`）；内存上限 12G、线程 8。
   `bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 8 cargo test --release -p singlefs-checker-tier --lib --features verdict-store,gpu -- the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken every_layer_handed_the_stop_request_hands_it_on_or_asks_it the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking`
   看 `test result: ok. 3 passed`。
2. 同前缀：`… cargo test --release -p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest`，看 `1 passed`。
3. 核 N1：把 `research/prompts/m3-batch10z-no-code-r1-sonnet-model/gpu/scan-extra.json` 里的 `old` 换成 `new`（`crates/singlefs-checker-tier/src/crash_amplification.rs`，`check_selected_blocks` 的函数体）后再跑第 1 条里的 `every_layer_handed_the_stop_request_hands_it_on_or_asks_it`；我的推断是它仍 `ok`（扫源码放行）。再让 `crash_amplification_pipeline_records_checks_and_reuses` 整个目标跑一遍，看有没有一条在这个变异上红（我推断没有，因为没有用例调 `check_selected_blocks`）。
4. 核 m7（卸载当场放开那一截不补隔离）：把 `research/prompts/m3-batch10z-no-code-r1-sonnet-model/scripts/mutate.py` 里 `m7_immediately_arm_plain` 的旧串 / 新串套在 `crates/singlefs-core/src/mount.rs` 上，跑 `cargo test --release -p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- --include-ignored`（harness 档），看是否 24 条全 `ok`（我跑过：全过）；再用模型目录的 `src/s2_probe.rs.txt`（放进 `crates/singlefs-harness/tests/s2_probe.rs`，并按 `src/core-debug-edits.diff` 加 `debug_bits`）跑 s7 那批形（`scripts/run-mut.sh` 的 s7 行），看 `unblocked_grew`。

## 7 复核不了

- 三条行为用例与 N1 的 `cargo test`：checker 档，按规矩不跑（第 6 节）。
- 真卡：没起 GPU 设备；显存账全部用库单测里的假卡与随机模型。「借之前每次多看一遍卡」在真卡上的耗时没量。
- 实现员报告里的 59 678 / 57 479 / 2 199 一组数：攻方随机模型上的数，我没拷模型复跑。
- 实现员的 H1 重量表（1536、6144 两档每一形 0 拒；2242 → 252 槽、40 拒 → 0；384 槽盘 22 形仍被拒）：我只在 1536 / 3072 / 768 / 384 四档上扫了自己的形，6144 档没扫，22 形没数；384 槽盘仍被拒是第十六块的事。

## 8 没做什么

- 不判别的腿的格、不替主 agent 采纳或出判决；上面的「一致 / 冲突」是我对条款与实现逐格比的结果，不是判决。
- 没跑门禁阶段、checker 档测试、GPU 测试、崩溃枚举；没起 GPU 设备。
- 大扫（形的矩阵）为快用了 `S2_NOCHECK`，没开池级 checker；I-7.4 / I-4.8 的独立信号用的是我自己的三个读数（`c_handed_out`、`c_unblocked`、`both`），池级 checker 只在 smoke 与 `runs/dbg/d4` 的少数历史上开过（`first_red=-`）。读数对「漏补」的判别力用 9 个变异证过（1 节），对「池级 checker 会红的别的形」没有独立证明。
- 没造的反例：起点槽豁免在「同起点不同跨度」上多豁免（读码推协议内到不了，要手写盘上的账才造得出）；崩溃抛弃根独占的 defer 槽双扣（已知没做）；`resurrect_released_record` 对已回收且被补隔离的槽同置两位（读码推协议内到不了：被回收的槽的释放代不高于回收门槛，而候选根的 txg 低于它的释放代，二者不并存）。
- m8（卸载那一串扣住那一截）与 z13（C 圈跳过扣住位置着的槽）没查到底，原因见 2.3。
- 没在 6144 槽盘上扫；没扫崩溃抛弃（而不是 R 未见证）的形。
- 不整份改仓里别的文件；报告与模型目录之外只在草稿目录里写。

## 9 草稿产物与清理

交回前删了（删前 `du -sh`）：`~/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet/` 下的编译目录 `target-chk` 374M、`target-old` 519M、`target-core` 642M、`target-mut` 1.1G、`target-scan` 2.7G（一次误在仓根里按我的 `CARGO_TARGET_DIR` 编了工作区，没动仓里文件，`git status --short Cargo.lock` 仍是原来的 `M ` 暂存态）、`target-scan2` 556K；仓副本 `tree-chk`、`tree-core`、`tree-mut`、`tree-old` 各 22M。
没删：`~/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet/`（现 4.8M）里的 `runs/`、`scripts/`、`gpu/`、`scan-model/`、`scanmut/`、`model-src/`、`build1.log`——是报告引的原始输出与我自己的脚本，主 agent 核对时要看；全部已拷进 `research/prompts/m3-batch10z-no-code-r1-sonnet-model/`（那一份是入库用的）。

## 10 引文核对

```
$ python3 scripts/agents/cite-check.py research/prompts/m3-batch10z-no-code-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree --root /home/fy5090/code/singlefs   （末行）
  ✓ 核了 19 处引文，对上 19 处，没判 0 处（逐处列在上面）
```
