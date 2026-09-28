# m3-prune-gpu-r1 云端攻方（Opus）报告（2026-09-28）

攻击面：U4、P6、P2、P3、P4、P5、G2、T2、T4、S2，另加主 agent 中途追加的 ignore 标记与崩溃注入「探路登记」。每个打中的都做成了一个 `must_be_nonzero=` 的世界，在 E161 装置的副本上跑，数全部出自副本，**不算入库装置上的数**。

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-prune-gpu-r1-opus-model/`：`attack.rs`（原型，挂在 E161 装置副本上的子模块）、`e161-hook.patch`（副本里 E161 装置只加两处挂接：`mod attack;` 与 `attack` 子命令）、`rerun.sh`、`outputs/`（十个世界的原样输出，去掉了 `LAYER0_*` 进度行）。

```
bash research/prompts/m3-prune-gpu-r1-opus-model/rerun.sh <仓根> <一个不存在的草稿目录>
```

它把仓拷进草稿目录、放进 attack.rs、打上补丁，经 `run-with-memory-cap.sh 8G` 与 `capped.sh 4` 编译并逐个跑 `attack t4|torn|collide|kinds|p5|order|interior|small|p6|reuse`，最后逐个与 `outputs/` 比（带 `seconds=` 的行不比）。

```
9ce2927884cab045dfe27486115e15f337f8e94d6f01c2b28c5b423d9a1fdff1  attack.rs
cf5e0beedfa2fd37316f25210e5b620078e83521ac1d7f2ecfd4e527ba3649d8  e161-hook.patch
68f3398ecc861c471859ef5576ee959ced25cea2eb1f0b170e5283a867172218  rerun.sh
02c0e3782e5323693e634b7aa74c2bcb5215b290b26adcb5dbd140f1df40fc05  outputs/collide.out
81bc1048f1a4448138f77002d5f795737bb5eebf7c9f8a196bd3e1f7025ea9f4  outputs/interior.out
ec2ee3eb38383a30f5bd63a5e226906087c2cd46fb013f34aeb8d7efa55253c9  outputs/kinds.out
d52a62b16c951b15012643bb6ab9e5d6ca710470e48e10f974269632f079597f  outputs/order.out
5ff51c1abaa862cc2489ebc4dbae6fcaf81662656bbbec9e2de6356d71cf2667  outputs/p5.out
58acc427ab3af8fb445058ed91168391a2430cf6b710f1b03ae3c35753a762a4  outputs/p6.out
140e4bdb0cd715980bf162b36cc25f750709e9041809598adfcc3e76bb58f82b  outputs/reuse.out
8e129e4ac124b6f25d303951e0bc6e5b8d8413a8cfc58e5cbf7f6f82de073ed4  outputs/small.out
1e2820b383bf20d54c6eceda21e754ea2adc641e9dd1bd100d7e5d4294985172  outputs/t4.out
33de93b66081b9c4bb1cf69478b4c9df3c5144564e27626ffd91a6f05f490626  outputs/torn.out
```

取样的做法（照定义 3b）：历史全在内存稀疏盘（`SparseBlockDevice`）上录，不建镜像文件；起点池各建一次。崩溃状态一律经 crates 的 `enumerate_layer0_selecting_versions_observing_each_state` 取（段内整写子集逐个展开，带第三态），每个世界跑前现算 `closed_form_state_count` 与带第三态的实数，断言不超过 10⁶（`attack_state_budget` 行）。十个世界全部原型跑合起来是 96×2 + 4128 + 33 + 45×4 + 63 = 4596 个状态（按各输出里的 `enumerated_states=` 加的），远低于 10⁷。**缩法（写明）**：p5、p6、t4、torn 这几段历史是录制流的前缀（截短了历史长度），它们各自的崩溃状态一个不落；order、interior、small、reuse 四个世界的历史里有 24 写的单元段（每段 2²⁴ 个状态），照派发提示「只跑小域」跳过了这些段的展开（`expand = 段长 < 16`），其余段一个不落。这是派发提示与定义 3b「留下的每段历史，它的崩溃状态一个不落」冲突的地方，照派发提示办，写在「这条腿自己的限度」一节。

## 各格判定一览

| 格 | 判定 | 世界（输出行） | must_be_nonzero |
|---|---|---|---|
| P5 按论证跳过 | **打穿**：只有单元写、没有根的段里，改坏的实现写出一个 I-7.8 会数的孤儿节点、段尾再盖掉。63 个状态里 16 个红；段首那个状态和整段全落的那个状态都绿。把扫描读去掉的键（P5 的前提）下，16 个状态同键异判 | p5.out:4 | 16 |
| P6 单元段的类数 | **打中「省下多少」**：只收单元写的段（新建文件那段的前 12 次写，4095 个状态），精确的 P6 叶子数 = 4095 = 状态数，判定只有 1 类。叶子数不降，是因为池级 checker 的扫描读随子集一起变；要降下来就得扔掉扫描读，扔掉之后就是 P5 | p6.out:2 | 4094 |
| P6 撕裂态／同位置被同段两次写罩住 | **镜像层打中，判定层没打中**：同一个系统配置槽同段先写 A、再写回 C，8 个状态里有 2 个状态的枚举器镜像与「按覆盖关系、物理地撕」的镜像不同。这个世界里五样判定逐项相同 | torn.out:2 | 2（镜像） |
| P4 增量枚举 | **打中**：只差一次写落法的 62 对状态里，有 10 对读到被翻的那次写罩不到的位置（自适应读） | small.out:5 | 10 |
| G2 与 CPU 共模 | **打中**：按代表态的读位置分类，44 个状态里 6 个判定与本类不同；GPU 与 CPU 两份实现的划分逐个相同（不一致 0），两边都没发现 | small.out:4 | 6 |
| 键罩不住输入（P2、P3、Q1 的键） | **在键函数上打中**：E161 那份 128 位摘要 40.7 秒造出两份 16 KiB 内容摘要相同（有 16 个字节不同）。**端到端没打中**：两份的 CRC32C 不同，在池里会让父指针也不同 | collide.out:1 | 1 |
| T2 物化 vs 叠加 | **打中两处**：①用户文件内容在 32K 数据单元第二个 16K 槽开头摆一个节点头：叠加 0 红、物化 12 红（I-7.8），整条全落那一态叠加绿、物化红；②同一个父结束镜像，子段枚举出来的状态数叠加 5、物化 3 | interior.out:13、interior.out:14、t4.out:2 | 12、1、2 |
| T4 指纹连锁 | **打中（方向与题面相反，见 T4 节）**：父结束镜像的稀疏扇区表与字节都相同、子节点写表相同，子节点的枚举域却是 5 与 3 | t4.out:2、t4.out:3 | 2 |
| U4 同一节点三条路径 | **没打中判定**：两条路径共有的 44 个状态，五样判定逐个相同；只有段 10 归哪次发布的标签不同 | small.out:3 | — |
| U4 物化后交给记录核对器的记录流 | **半中**：只交本节点写表时，提前复用的变异里记录核对器的 `claimed_state_missing_unit` 从 3 个状态掉到 0 个；但池级 checker 在同样的状态上照红，这个世界里没有漏成绿的状态 | reuse.out:18 | 0（整态） |
| S2 只表示等价类 | **引已有世界、没重跑**：记录核对器要持久集合，镜像逐字节相同、判定不同的两个状态在 crates 里已有用例 | 见 S2 节 | — |
| ignore 标记／崩溃注入探路 | **种类串那一半打中**：30 次覆盖写的历史里，一个写种类串底下并着「根槽第一次写」与「根槽回卷重写」两种条件。ignore 那一半是推的，没建模型 | kinds.out:1 | 1 |

## P5 与 P6：单元段上「精确」与「省」不能兼得

### P5 是循环论证（世界 p5）

被攻的条文：「当前段只有单元写、根没写时，全部子集判成同一类，不逐个评」（`research/prompts/_m3-prune-gpu-r1-background.md:56`），对照「⇒ 剪枝只许是精确的」（`research/prompts/_m3-prune-gpu-r1-background.md:11`）。

历史（`attack.rs` 的 `p5_history`）：第一条流前 7 段（到暖机第二代之后那次系统配置轮换），再接一段只有单元写的段（6 次写，没有根）：
1. 盘 0 空槽 60000 写一个码 2 节点，它是从新建文件那段里取来的一个真节点，把树 ID 抬到 2⁴⁰、诞生代号压到 1，头校验和重算过（`orphan_node_that_the_i78_scan_counts`）；
2. 新建文件那段的前 4 次单元写；
3. 同一槽写回 16384 个 0。

这就是一个改坏的实现：它在发布之前写出一个不该有的节点，又在同一段里把它盖掉。整段全落之后盘上看不出它，后面各段也看不出，只有段内的崩溃状态看得见。每一步许可它的条文：
- 枚举：「当前段每次写各取它的几态、任意组合，之后各段全部没持久」（`.claude/kb/decisions/13-验证路线.md:72`）；
- 扫描读得到那一槽：CrashImage 把持久了的单元写都列进候选，「slots.insert(write.offset.0 / SLOT_BYTES);」（`crates/singlefs-harness/src/memory_pool.rs:747`），扫描从候选取槽，「let slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {」（`crates/singlefs-checker/src/walk.rs:2405`）；
- 数进去：「if birth_txg <= newest_published_txg」（`crates/singlefs-checker/src/walk.rs:2419`），然后「judgements.judge("I-7.8", watermark > highest_seen, || {」（`crates/singlefs-checker/src/walk.rs:6251`）。

原样输出（副本上，没改坏的对照在前）：

```
E7RESULT name=attack_p5 world=p5_control enumerated_states=96 unit_segment=7 states_in_unit_segment=63 red_states_in_unit_segment=0 red_by_name={} segment_first_state_red=Some(false) all_persisted_state_red=false example_red_landings_in_segment_order=None tally_checker_violated={} walk_key_classes=1 walk_key_inconsistent=0 full_key_classes=32 full_key_inconsistent=0 must_be_nonzero=0
E7RESULT name=attack_p5 world=p5_mutant enumerated_states=96 unit_segment=7 states_in_unit_segment=63 red_states_in_unit_segment=16 red_by_name={"checker:I-7.8": 16} segment_first_state_red=Some(false) all_persisted_state_red=false example_red_landings_in_segment_order=Some([true, false, false, false, false, false]) tally_checker_violated={"I-7.8": 16} walk_key_classes=1 walk_key_inconsistent=16 full_key_classes=48 full_key_inconsistent=0 must_be_nonzero=16
```

16 = 2⁴：孤儿落了、写回没落，中间 4 次写任意组合。crates 枚举器自己的计数（`tally_checker_violated={"I-7.8": 16}`）与我逐状态评的一致。

四句：
1. **分不分辨臂**：分。同一个世界里，精确的读集键（恢复两遍 + checker 正常那一遍的整串调用）是 48 类、同键异判 0；只有「根没写就当一类」的 P5 漏掉。
2. **系统看不看得到**：看得到。I-7.8 的扫描在那 16 个状态上读到了孤儿；是 P5 不去评那些状态。
3. **满足的判据分句**：「一条剪枝规则要么证明被剪掉的状态与留下的某个状态在每一样判定上逐项相同……要么它就是抽样、不许进层 0」（`research/prompts/_m3-prune-gpu-r1-background.md:11`）的第一分句不成立。按主 agent 转来的用户新定案「我们不是抽样 我们是剪枝」，第二分句那条出路也没有了。
4. **改法还中不中**：P5 没有跑前条款给的改法。能想到的只有两种：逐个评，那就不叫 P5；或者证明单元写不改变扫描的判定，但那正是被验的结论。

### P6 在单元段上叶子数 = 状态数（世界 p6）

被攻的条文：「叶子数 = 类数，各叶子的子集数加起来要等于段的状态数（这一条就是它的全量证明）」「它是 2¹³⁶、约 2³⁰⁰ 那几段唯一可能精确全量的候选（推的）」（`research/prompts/_m3-prune-gpu-r1-background.md:57`）。

历史：第一条流前 7 段，接新建文件那段的前 12 次单元写（录制流前缀，4095 + 1 + 32 个状态，全展开）。

```
E7RESULT name=attack_p6 k=12 enumerated_states=4128 states_in_unit_segment=4095 red_states_in_unit_segment=0 distinct_judgment=1 distinct_recovery_keys=1 distinct_checker_normal_keys=4095 distinct_candidate_unit_slot_answers=4095 distinct_byte_position_sets=4095 leaves_recovery_plus_checker_normal=4095 inconsistent_under_that_key=0 leaves_if_scan_dropped_walk_key=1 inconsistent_if_scan_dropped=0 seconds=20.1 must_be_nonzero=4094
```

机理：池级 checker 的扫描方向逐个读候选槽，候选就是持久了的单元写所在的槽：「slots.insert(write.offset.0 / SLOT_BYTES);」（`crates/singlefs-harness/src/memory_pool.rs:747`）。所以光是 `candidate_unit_slots` 的答案，每个子集就不一样（4095 种）；读位置集合也是 4095 种。P6 的分叉按「这个读位置上落的是哪次写」走，而读位置本身由子集决定，叶子数就等于子集数。

- P6 本身**不错**：叶子内判定一致（`inconsistent_under_that_key=0`），叶子子集数之和也等于状态数。
- 打中的是「它省下多少」：在只收单元写的段上，精确的 P6 一个状态都省不下，对 2¹³⁶、约 2³⁰⁰ 的段同样如此（推的，按同一机理外推，那两段没跑）。
- 要让叶子数降到判定类数（这里是 1），就要把扫描读从分叉里拿掉（`leaves_if_scan_dropped_walk_key=1`），而拿掉扫描读正是 P5 的前提。世界 p5 在同一种段上量出，拿掉扫描读的键下有 16 个状态同键异判（p5.out:4 的 `walk_key_inconsistent=16`）。

四句：分辨臂——打中的是 P6 的收益，不是 P6 的正确性，与 P5 世界合起来分辨「P6 精确」与「P6 省」这两个读法；系统看得到——看得到；分句——共用问句②「它省下多少」（`research/prompts/_m3-prune-gpu-r1-background.md:98`）；改法——正文没给 P6 的改法。

我的改法（**只在我的模型上量过、被攻过零轮**；下面三行都是**推的**，没实现没跑）：
- 扫描方向在 checker 里本来就只是聚合：I-7.8 取最大树 ID，I-1.8 按类身份归并。P6 可以不在扫描的每个读上分叉，改成把扫描当一个按写符号求值的聚合：对每个候选槽算「落在这里的是哪次写」的分布，只对进得了聚合的那几种内容分叉。
- 这要把 checker 的扫描代码拆成「逐槽判进不进」与「聚合」两半，也就碰到了 D13（验证路线） 已定项 5「遍历与记账代码交集为空」那条边界。
- 它修的是 p6 那一格（叶子数）；p5 那一格靠「孤儿进得了聚合」分叉，照样抓得到。

## P6 撕裂态与「同一位置被同段两次写罩住」（世界 torn）

被攻的条文：P6「按写之间的覆盖关系算」相容子集数（`research/prompts/_m3-prune-gpu-r1-background.md:57`）。枚举器定义的撕裂镜像不是「按覆盖关系」撕出来的：「同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条层 0 流上这种组合一次都没有）」（`.claude/kb/decisions/13-验证路线.md:74`）。

历史：第一条流前 6 段，再接一段，里面是同一块盘同一个系统配置槽（盘 0 偏移 0）上的两次原地写：先写 A（段 6 那次轮换的字节），再写回槽里原有的那一份 C。每次写三态，共 8 个状态。对每个状态比两份镜像：一份是枚举器的；一份按覆盖关系物理地叠，撕裂那次写的旧半取这个状态下盘上实际的旧字节。

```
E7RESULT name=attack_torn enumerated_states=33 states_in_segment=8 states_where_enumerated_slot_differs_from_physical_tearing=2 states_where_judgment_differs=0 per_state=[… "NotPersisted/Torn:images_differ=true:judgments_differ=false:enumerated_red=[]:physical_red=[]", "Torn/Torn:images_differ=true:judgments_differ=false:enumerated_red=[]:physical_red=[]", …] must_be_nonzero=2
```

（per_state 只摘了两格，全文在 `outputs/torn.out` 第 2 行。）

（A 没落、C 撕裂）那一态：物理地撕，C 覆盖在 C 上，撕不出第三种内容，槽是好的 C；枚举器按「A 已落」当旧半，撕出一份坏槽。（A 撕裂、C 撕裂）那一态同理。

- 结论：P6 要与照跑逐状态相同，它的「覆盖关系」必须照搬枚举器那张带撕裂镜像与重放的写表（`TornWriteTable`、crates 的 `WritesWithTornImages`），不能从写的覆盖关系推。否则在这类状态上，P6 叶子里的镜像与照跑的不是同一个；叶子子集数之和照样对得上，这一条全量证明查不出来。
- 这个世界里五样判定逐项相同（`states_where_judgment_differs=0`）：系统配置有另一槽兜着，坏了一槽不改判定。判定层没打中，只在镜像层打中。
- 两条层 0 流今天走不到这种组合；U3 要补的 Z1–Z15 里有没有，我没查。

四句：不分辨臂（照跑与 P6 共用枚举器的约定时两边一致）；系统看得到镜像差；分句是 P6 定义里「按写之间的覆盖关系算」那一句本身；没有跑前改法。

## P4 与自适应读（世界 small 的 `attack_p4`）

被攻的条文：「状态按格雷码次序走，相邻两个只差一次整写，只重算被那次写罩到的读」（`research/prompts/_m3-prune-gpu-r1-background.md:55`）。

域：第一条流全部小段（段长 < 16），44 个状态。在同一段里找只差一次写落法的状态对，比两边按字节读的位置集合（恢复两遍 + checker 正常那一遍），扣掉被翻那次写罩到的位置。

```
E7RESULT name=attack_p4 pairs_differing_in_one_write=62 pairs_reading_positions_the_flipped_write_does_not_cover=10 example=Some("segment=0 flipped_write=1 kind=SystemConfigurationSlot NotPersisted->Torn positions_not_covered_by_the_flipped_write=4064") must_be_nonzero=10
```

例子：盘 1 的系统配置槽 0 从「没落」翻成「撕裂」，恢复与 checker 改用另一份配置，读的位置换了 4064 个，没有一个落在被翻那次写上。「只重算被那次写罩到的读」在这 10 对上漏重算。能用的只有「从第一个答案变了的读起整个重跑」，那就等于 P2 的读集记忆化，格雷码次序本身不省判定。撕裂态让每位是三进制，格雷码要换成混合进制的反射码，这一点推的，没写。

四句：分辨臂（P4 出局，P2 不受影响）；系统看得到（换了的读位置就在恢复与 checker 的读里）；分句是 P4 定义里「只重算被那次写罩到的读」；没有跑前改法。

## G2 与 CPU 分类共模出错（世界 small 的 `attack_g2`）

被攻的条文：「一个节点里每个状态（掩码）对这个节点的读位置逐个求「落在这里的是哪次持久写的内容」」（`research/prompts/_m3-prune-gpu-r1-background.md:76`）。

装置：每段取第一个状态当代表，它读过的字节位置就是「这个节点的读位置」。两份独立实现在这组位置上给每个状态出向量：
- 「GPU」一份按写表下标算，每个扇区上最后一次持久了的写表项；
- 「CPU」一份真的读字节、求摘要。
两边各自按向量分类；每类的判定取类里第一个状态的，再与逐状态照跑的判定比。

```
E7RESULT name=attack_g2 states=44 classes_by_representative_positions=38 states_whose_judgment_differs_from_their_class=6 gpu_cpu_partition_mismatch=0 must_be_nonzero=6
```

GPU 与 CPU 的划分逐个相同，两份对拍全绿；判定错了 6 个状态。原因在两份共用的输入：位置表取自一个代表态，而别的状态因为自适应读会读到代表态没读过的位置（机理同 P4 与调查员丙那 12 个 K4 不一致，都在 journal 环）。

**谁发现：抽样对拍（GPU 对 CPU）发现不了**，因为两边吃的是同一张位置表。能发现的只有「类的判定 vs 逐状态照跑的判定」这一种对拍。

跑前条款的改法：「GPU 与 CPU 怎么对拍（抽样逐格相同，不同就判红）」（`research/prompts/_m3-prune-gpu-r1-background.md:80`）。它在这 6 格上**不起作用**。

我的改法（只在我的模型上量过、被攻过零轮）：
- 位置表按类取不动点。每类跑一次代表，把它读的位置并进表，重分类，直到不动。这样每个状态与本类代表在代表读过的每个位置上都相同，照确定性重放，执行逐步相同。修的是 g2 那 6 格，**推的**，没实现。
- 对拍另加一条「类判定 vs 照跑」。按用户新定案，这一条不能是抽样，只能当门禁里的一个固定小域，**推的**。

四句：分辨臂——G2 与 G1 分得开，G1 不分类；系统看不看得到——两份对拍都看不到；分句——共用问句③「它坏了谁先发现」；改法——跑前那条对拍在打中的格上不中。

## 键罩不住输入

### 键用的摘要可以被用户数据撞上（世界 collide）

E161 的读集键、内容键都用自写的 128 位摘要（`DigestBuilder`，每喂一个 64 位字做一步可逆的乘、转、异或）。它两步之内就是一个 64 位的条件：前一个字的两种取法只要让一个 64 位函数值相同，第二个字就能直接解出来，让 128 位状态合上。所以用 Brent 找环，大约 2³² 次求值就撞出一对：

```
E7RESULT name=attack_digest_collision length=16384 differing_bytes=16 first_digest=91bb1b2affa83a939d4dd75c9917dd09 second_digest=91bb1b2affa83a939d4dd75c9917dd09 digests_equal=true rho_attempts=1 seconds=40.7 crc32c_first=1ec1ebae crc32c_second=d30c8dc9 must_be_nonzero=1
```

两份 16 KiB 内容只差两个 8 字节字，摘要逐位相同。文件内容是用户给的，数据单元的读内容就是它，所以用 P2、P3 的键、Q1 的类表键，或 U1 γ 按内容寻址的段结果缓存时：
- 只要键函数是这一份，同键异判就造得出来；
- 判据「文本相同 ⇔ 值相同，除去 128 位摘要撞车」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:278`）的概率上界，只对不挑输入的撞车成立。

**端到端没打中**：两份的 CRC32C 不同（1ec1ebae vs d30c8dc9），落进池里时父节点里存的校验和也不同，父节点那次读的摘要就不同，整个键不撞。
- 要端到端撞，得把 CRC 也拉平：做 33 个串联的碰撞块（Joux 多重碰撞，任选子集摘要都相同），每块翻转对 CRC 的贡献是一个固定的 32 位差，33 个里总有一个非空子集异或为 0，再在尾部补 4 字节把 CRC 定到事先取好的值。
- 代价按单块 40.7 秒外推约 22 分钟单线程。**推的，没做**。

改法：键用抗碰撞的散列（带随机盐的或密码学散列），或者键相同时逐字节比内容（E161 的内容表有这一步，读集键没有）。**推的，被攻过零轮**。它修的是 collide 这一格。

### 记录核对器与 oracle 的输入不在任何读集键里

E161 的 `k2_consult` 等臂「不一致 0」的结局只是恢复报告：「let consult_outcome = fingerprint_of_debug_text(&consulted);」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:2247`）。oracle 与记录核对器不在任何一条臂的结局里，所以这些「0」不能当「按读集分类后五样判定逐项相同」的证据。

- oracle 的「盘上最新根」从持久集合算，不从读取算（`newest_persisted_root`）。
- 记录核对器按条款要持久集合：「按镜像去重的提速对它不适用」（`.claude/kb/decisions/13-验证路线.md:135`）。
- 已有的同镜像异判世界是 crates 里的用例「fn the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing()」（`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:404`）。它在 checker 档，我没跑。

这对 S2 是直接的：「状态不单独表示，只表示等价类：类 = 读集键」（`research/prompts/_m3-prune-gpu-r1-background.md:66`）下没有持久集合可交，记录核对器没法跑。所以 S2 只能和「记录核对器照跑」并存，而照跑就得把状态逐个表示出来，S2 在记录核对器那一截退回 S1。

在我这几个域里，恢复两遍 + checker 正常那一遍的整串调用当键时同键异判都是 0（p5.out:4 的 `full_key_inconsistent=0`、p6.out:2 的 `inconsistent_under_that_key=0`）：键罩住了恢复与池级 checker，没罩住记录核对器，这个判断不变。

## T2：物化与叠加不是同一个判定对象

T2 两个候选今天都有现成形态：
- 叠加，层 0 的 CrashImage，基线停在 mkfs 之后；
- 物化，崩溃注入的做法：「base.apply_writes(&writes[writes_applied_to_the_base..first_write_of_the_segment]);」（`crates/singlefs-checker-tier/src/crash_injection.rs:801`），CrashImage 的基线换成物化的池、写表只交本段。

同一个崩溃状态在两种形态下字节逐位相同（order.out 与 interior.out 各段 `states_with_read_bytes_differing=0`），差在两处。

### ①候选槽的答案不同，判定跟着不同（世界 interior）

- 叠加：一次持久了的单元写只贡献它的第一个槽，「slots.insert(write.offset.0 / SLOT_BYTES);」（`crates/singlefs-harness/src/memory_pool.rs:747`）。
- 物化：物化的池按写过的扇区给槽，「.map(|sector| sector * SECTOR_BYTES / SLOT_BYTES)」（`crates/singlefs-harness/src/memory_pool.rs:644`）。32K 数据单元的第二个 16K 槽只在物化那一形上当候选。

扫描方向对每个候选读 16 KiB、看头（`crates/singlefs-checker/src/walk.rs:2405`）。

历史：第一条流，第一个文件换成 17000 字节的用户内容。我先量了内容在数据单元里从第 134 字节起，就在内容第 16250 字节处摆一个 I-7.8 会数的节点头（134 字节），落盘之后它正好在数据单元第二个槽的开头。这是一个合法的池：用户写什么内容都可以。

```
E7RESULT name=attack_materialized world=data_unit_interior_looks_like_a_node segment=10 states=8 red_full_stream=0 red_materialized_node_local_writes=8 judgment_differs_node_local=8 red_only_under_full_stream=0 judgment_differs_materialized_image_with_full_record_stream=8 states_with_read_bytes_differing=0 states_with_hint_answers_differing=8
E7RESULT name=attack_materialized_summary world=data_unit_interior_looks_like_a_node enumerated_states=45 red_names_full_local_localimage_fullrecords={"checker:I-7.8": (0, 12, 12)} judgment_differs_materialized_image_with_full_record_stream_total=12 must_be_nonzero=0
E7RESULT name=attack_t2_interior_final_state overlay_red=[] materialized_memory_pool_red=["checker:I-7.8"] must_be_nonzero=1
```

（summary 行的 `must_be_nonzero=0` 数的是「只在整流下红」，这个世界反过来，是物化下红。要看的数是 `judgment_differs_materialized_image_with_full_record_stream_total=12`：物化镜像配整条记录流，照样有 12 个状态判定不同。）

- 数据单元落盘之后的 12 个状态（段 8、9、10），叠加全绿，物化全红 I-7.8；整条全落那一态也是叠加绿、物化红。
- 同一个状态，层 0 这一侧（叠加）判绿，崩溃注入这一侧（物化）判红。I-7.8 的扫描把数据单元内部当节点头读，是池级 checker 在健康池上的假红（**顺带看到，不在我的攻击面**），层 0 今天看不见它。
- 健康的第一条流上字节与判定都相同，但段 8、9、10 的候选答案同样不同（order.out 第 9–11 行 `states_with_hint_answers_differing=3/1/8`）：两种形态交给 checker 的输入不是同一份，只是今天那一槽里恰好全是 0。

### ②原地覆写的认定看写表历史，物化把历史丢了（世界 t4，与 T4 同一个世界）

第三态的认定：「base_holds_nonzero_bytes_there || an_earlier_write_with_bytes_overlaps」（`crates/singlefs-checker-tier/src/crash.rs:1600`）。后一半看的是写表里更早的写，物化之后只剩本段的写表，这一半恒为假。

历史 b：第一条流前 7 段，接一段「在记录槽 16785408 写 4096 个 0x5A、再整段清零这 4096 字节」，再接新建文件那次发布的两条 journal 记录写。历史 a 没有中间那一段。

```
E7RESULT name=attack_t4 variant=a_plain parent_end_sector_maps_equal_to_a=true parent_end_bytes_equal_to_a=true child_writes_identical=true child_first_write_tearable_in_overlay=false child_states_overlay=3 child_states_materialized=3 overlay_minus_a_plain=0 overlay_minus_materialized=0 must_be_nonzero=0
E7RESULT name=attack_t4 variant=b_bytes_then_zero_fill parent_end_sector_maps_equal_to_a=true parent_end_bytes_equal_to_a=true child_writes_identical=true child_first_write_tearable_in_overlay=true child_states_overlay=5 child_states_materialized=3 overlay_minus_a_plain=2 overlay_minus_materialized=2 must_be_nonzero=2
```

b 的子段：叠加认定第一条记录写是原地覆写（更早有带字节的写罩过它），3 + 2 个状态；物化看不到那次更早的写，3 个状态，少了两个撕裂态。

可达性：今天 mkfs 之后没有整段清零（「整段清零不属于任何一次发布（今天唯一的清零是 mkfs 清 journal 环，在第一次发布之前）。」（`crates/singlefs-harness/src/memory_pool.rs:1015`）），两条层 0 流走不到。实现以后加「清 journal 槽」「截断成 0」（Z2）这类写就走得到。**推的**，没造出一条今天可达的历史。

四句：
1. **分辨臂**：①②都分辨叠加与物化。
2. **看不看得到**：①看得到，两形 checker 读的候选不同；②枚举器看得到写表，物化那一形看不到。
3. **分句**：T2「物化」一行的定义说它「不改判定」（P0「只改读的代价（O(整张写表) → O(当前段)），不改判定」，`research/prompts/_m3-prune-gpu-r1-background.md:51`），①②各给了一个反例。
4. **改法**：跑前没有改法。我的改法（**被攻过零轮**）：物化那一形的 CrashImage 的候选答案与第三态认定，都另从整条路径的写表算，不从物化的池算；或者反过来，把叠加的候选改成按写过的扇区给。修的是 interior 与 t4 两格，**推的**，没实现。

## T4：父结束镜像相同、下游的输入却变了

被攻的条文：「每个节点的输入指纹 = 父节点结束镜像的哈希 + 本节点写表 + 判法摘要，父变了下游指纹自动变」（`research/prompts/_m3-prune-gpu-r1-background.md:44`）。

题面要的方向「父结束镜像变了而下游指纹没变」，在这个定义下按构造走不到：父结束镜像是指纹的一个分量，它一变指纹就变。危险在反方向：**父结束镜像不变，下游判定的输入却变了**。这时指纹不变，复用的是旧结果。下游判定的输入里，不在「父结束镜像 + 本节点写表 + 判法」里的，我现查到四样：

| 输入 | 在哪里读 | 世界 |
|---|---|---|
| 第三态认定看的写表历史 | `crates/singlefs-checker-tier/src/crash.rs:1600` | t4.out:2（稀疏扇区表与字节都相同，子段 5 个状态 vs 3 个）、t4.out:3（字节相同、扇区表不同，同样 5 vs 3） |
| 叠加形态下的候选槽答案（按写表里的单元写给） | `crates/singlefs-harness/src/memory_pool.rs:747` | interior.out:13（推：同一份字节的父镜像，由一次 32K 写或两次 16K 写得来，候选答案不同，下游 I-7.8 的判定跟着不同；这一格没单独跑） |
| oracle 的「盘上最新根」按路径写表与持久集合算 | 「if let Some((newest_txg, newest_instance)) = newest_persisted_root {」（`crates/singlefs-checker-tier/src/crash.rs:982`） | 没造世界（推的） |
| 记录核对器按路径写表分发布 | 「RecordStreamContinuity::OneRecording => publishes_of_one_recording(writes, 0),」（`crates/singlefs-harness/src/memory_pool.rs:967`） | reuse.out:18（只交本节点写表时 `records:claimed_state_missing_unit` 由 3 变 0） |

「判法摘要」若只指 checker 与恢复的代码，那么测试自己的版本表（oracle 的期望内容）、盘的字节数（`MemoryPool.device_size_in_bytes`，不在扇区里）也不在指纹里。这两样只是指出，没造世界。

四句：
1. **分辨臂**：分。「按代码指纹整棵重跑」那一臂不受这类影响，它不复用。
2. **看不看得到**：看得到。每一样都在判定时实际被读。
3. **分句**：T4 定义「父变了下游指纹自动变」只管了一个方向。
4. **改法**：跑前只有两臂可选。我的改法（**被攻过零轮**）：指纹里的「父」换成「路径写表的哈希」（整条路径的写与它们的段），不用结束镜像。它罩住上面四样的前三样，第四样（记录核对器）本来就要整条路径的写表。代价是一个父节点的写表改了、结束镜像没变时，下游照样重跑。修 t4 两格，**推的**。

## U4：同一节点在两条路径上

### 判定口径（世界 small 的 `attack_u4_shared_node`）

路径 a：第一条流。路径 b：同一条流之后再做一次覆盖写（txg 4），版本表多一版。两条路径前 11 段逐字节相同（`shared_prefix_identical=true`）。我在两条路径上各评一遍 a 的 44 个小段状态：b 那边用 b 的整张写表与版本表，a 各段持久、b 多出来的写都没持久。

```
E7RESULT name=attack_u4_shared_node states=44 judgment_digest_differs=0 red_list_differs=0 segments_whose_publish_label_differs=["10:after_the_last_root->instance1_txg4"] enumerated_states=45
```

五样判定逐个相同，没打中。不同的只有段 10（txg 3 之后那次系统配置轮换）归哪次发布：a 里归「最后一条根之后」，b 里归 txg 4。所以「按路径各出一份计数、判定共用」在今天的共享前缀上站得住。计数那一半（按发布分的格子）要按路径出。第三条路径（并行线一）没造。

### 物化父镜像之后交给记录核对器的记录流（世界 reuse、order）

reuse：路径 b 之后，改坏的分配器另起一段，把 txg 3 那一版的数据单元（两盘两份）提前盖掉，再接一段无害的写。比三种形态：
- 整流，今天的做法；
- 物化、只交本节点写表；
- 物化镜像、配整条记录流。

```
E7RESULT name=attack_materialized_summary world=premature_reuse_of_the_txg3_data_unit enumerated_states=63 red_names_full_local_localimage_fullrecords={"checker:I-2.1": (5, 5, 5), "checker:I-4.8": (5, 5, 5), "checker:I-7.4": (5, 5, 5), "records:claimed_state_missing_unit": (3, 0, 3)} judgment_differs_materialized_image_with_full_record_stream_total=0 must_be_nonzero=0
```

- 只交本节点写表时，记录核对器的 `claimed_state_missing_unit` 从 3 个状态掉到 0 个：txg 3 那次发布不在本节点写表里，它的单元没人核。
- 同样那几个状态上池级 checker 照红（I-2.1、I-4.8、I-7.4），所以整态没有漏成绿（`must_be_nonzero=0`）。
- 物化镜像配整条记录流时，判定与整流逐项相同（0）。
- 结论：物化可以做，但记录核对器与 oracle 要整条路径的写表与持久集合，照崩溃注入 `HistoryThenWritableMountRecords` 那一形交。P0 省下的只是 CrashImage 的读，记录核对器与 oracle 那一截仍是 O(路径)。

order（**没打中 U4，顺带看到**）：把新建文件那次发布改成先写根（FUA）、后写两条 journal 记录。三种形态都 0 红，**整流下的记录核对器也不红**。原因：发布按写表次序归，根之后的记录归到下一次发布（`publishes_of_one_recording`），这次发布的记录表是空的，「根在而记录一条都不在」那条判据跳过空记录表。这个乱序今天没有一样判定抓得到，不在我的攻击面，交主 agent。

## ignore 标记与崩溃注入「探路、登记新节点」（主 agent 中途追加）

收到时我已写好模型、还没写报告；前面各节没有哪一格依赖「退到抽样」。G2 一节的改法里写明了「不能是抽样」。

### 按写种类串认节点，条件不同的发布并成一个（世界 kinds）

在内存池上做第一条流 + 30 次同长的覆盖写（33 次发布），每次发布取它的段种类串，再记三个条件：复用了更早的单元槽没有、根槽写落在更早写过的根槽上没有、journal 写落在更早写过的记录槽上没有。

```
E7RESULT name=attack_kind_string overwrites=30 publishes=33 distinct_kind_strings=3 kind_strings_with_several_condition_signatures=1 detail=["SS|UUUUUUUUUUUUUUUUUUUUUUUUUUUU|JJ|R|SS=>{\"reuse=false,root_slot_rewrite=false,journal_slot_rewrite=false\", \"reuse=false,root_slot_rewrite=true,journal_slot_rewrite=false\"}"] must_be_nonzero=1
```

同一个种类串底下有「根槽第一次写」与「根环回卷、盖掉旧根」两种条件（Z 列表里的「根环回卷」就是后者）。T3「按录制流的种类串」（`research/prompts/_m3-prune-gpu-r1-background.md:43`）当节点身份时，崩溃注入探到一条回卷的历史，会认成已登记的节点，**不登记新节点**。30 次覆盖写里没出现单元复用，journal 环也没回卷，那两个条件这里没量到。

改法（**被攻过零轮**）：节点身份 = 种类串 + 覆盖关系签名，写与路径上更早的哪一类写重叠，按单元、根槽、记录槽、系统配置槽分。修 kinds 这一格，**推的**。

### ignore 会不会让节点无声地永远不开（推的，没建模型）

- 快档对全量用例的处置是：「逐条核 .claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的用例各自那一格全绿标记」，标记不作数的报本次未跑、不判红（`.claude/rules/verification.md:33`）。「本次未跑」不会变红，ignore 的节点也一样：没有东西逼它开。
- 先例：「2026-09-27 之前这条放量用例没登记，谁都不跑」（`.claude/gate.d/stage-inputs.tsv:42`），后来靠 `crash-case-check.py` ③「标了 ignore 又没登记的谁都不跑」才补上。
- U6 候选说「开一个节点的判据写死（剪枝之后类数与挂钟过线）」（`research/prompts/_m3-prune-gpu-r1-background.md:35`）。但剪枝之后的类数只有跑过那个节点才知道。按 p6 世界，单元段上精确剪枝的类数就是状态数，那条判据对 2¹³⁶ 那几段永远过不了线，节点永远 ignore。
- 要不无声：
  - 登记表里每个 ignore 行写「缺哪一条剪枝或实现」，并点名一个会变的对象（某个改法的用例名）；
  - 门禁在那个对象变绿、而节点仍是 ignore 时判红；
  - 覆盖报告按节点列出 ignore 的状态数与它们占全树的比例。
  这三条都是**推的**，被攻过零轮。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| U4 同一节点两条路径判定不同 | 第一条流与「同一条流 + txg 4 覆盖写」共有的 11 段中的小段，44 个状态 | 0 个不同（small.out:3） |
| 恢复两遍 + checker 整串调用当键，同键异判 | p5 的 63 个、p6 的 4095 个单元段状态 | 0 个（p5.out:4、p6.out:2） |
| 撕裂态约定不同，判定跟着不同 | 同槽同段两次原地写，8 个状态 | 镜像 2 个不同，判定 0 个不同（torn.out:2） |
| 物化只交本节点写表，漏成整态绿 | 提前复用变异 63 个状态；先写根后写记录变异 45 个状态 | 0 个（reuse.out:18、order.out:24）；后者整流也不红，见 U4 节 |
| 摘要碰撞端到端撞出同键异判 | 没做（要把 CRC 一起拉平） | — |
| oracle 的「盘上最新根」在同一段内变 | 读代码：两条流上根槽写都自成一段，同一段内的状态最新根相同 | 今天的流上走不到，没造合成流 |

## 这条腿自己的限度

- 副本上的数，**不算入库装置上的数**。十个世界都是自己造的历史：前缀、改坏的实现、用户内容，没有一个是登记过的崩溃枚举用例。
- 取样与定义 3b 冲突：order、interior、small、reuse 四个世界的历史里有 24 写的单元段，照派发提示「只跑小域」没展开，其余段一个不落。打中的格（interior 的 12 个状态、g2 的 6、p4 的 10）都在展开了的段里，不受这个取舍影响；没打中的 U4 与 order 两格，没覆盖那些大段。
- 状态数合计：十个世界共 4596 个，另有中途修 torn 时那一次跑的 33 个，合 4629，远低于 10⁷ 的上限。
- 自己提的改法（P6 扫描当聚合、G2 位置表取不动点、键用抗碰撞散列、物化那一形另算候选与第三态、T4 指纹换路径写表、节点身份加覆盖签名、ignore 三条）**全部只在我的模型上量过或只是推的，被攻过零轮**。
- 时间：派发时的线程上限 4 照用；内存包装排队排了 292 秒（batch1）。第一次按世界分开起的那一批在队里等了十多分钟，我停掉了它（`proc.py stop`，逐个），改成一批里依次跑。

## 没做什么

- 没跑任何 checker 档测试、名字带 layer0 的目标、门禁 54、55、57、59 号、E162、E163；引到的 crates 用例（`record_checker_judges_absence_by_the_persisted_set.rs`）只读没跑。
- 没判正推、辩方那几格；没替主 agent 采纳；主仓一个字没改，只写了报告与模型目录。
- 没造 oracle「最新根」、版本表、盘大小进不进 T4 指纹的世界（T4 表第 3 行与表下那一段，推的）。
- 没造第三条路径（并行线一）上的 U4 比较；没在 2¹³⁶、约 2³⁰⁰ 的段上直接跑 P6（按 p6 的机理外推，推的）。
- 草稿与删除：删了 `/tmp/claude-1000/m3-prune-gpu-r1-opus/repo`（仓副本，325M）与 `/tmp/claude-1000/m3-prune-gpu-r1-opus/target`（编译目录，385M）；草稿目录里留着日志与批跑脚本，没有编译目录与仓副本。
