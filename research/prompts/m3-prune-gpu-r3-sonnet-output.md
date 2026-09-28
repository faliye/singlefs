# m3-prune-gpu-r3 云端辩方（Sonnet）报告（2026-09-28）

## 任务

替第一、二轮被判「打中」的十二处辩：第一轮 T2、T3、T4、P4、P5、P6「省」、G2，第二轮 B1、B2、B3、B7、B8——逐处判引的产物行撑不撑得住那一句、打中是不是同样打中所有替代方案（判据分不分辨臂）。
替第二轮站住的 C2、C4–C7 与 D1「各写一库按内容换编号导入」各找一个最可能的反例（反例同样当线索，不当已坐实的打穿）。

## 核法

- 十二处「打中」：对每一格，去被引的 `.out` 文件里用 `cat -n` 现读那一行，核字面是不是撑得住判决那一句；再读攻方腿报告（`m3-prune-gpu-r1-opus-output.md`、`m3-prune-gpu-r2-opus-output.md`）自己写的「四句」（分辨臂 / 看不看得到 / 分句 / 改法），核判决有没有漏抄这四句里的限定词。
- SHA256SUMS：两份模型目录 `sha256sum -c` 全 OK（见下）。
- 复跑：r1 攻方模型（不带 `verdict-store` 特性，编译轻）在草稿目录副本上重编重跑十个世界，与存档比对；r2 攻方模型要开 `verdict-store` 特性、编译期新起 RocksDB 的 C++ 构建，本机已有另一个会话在跑同一套编译（`ps` 看到 `/tmp/claude-1000/m3-prune-gpu-r3-opus/r2-rerun/` 下的 `librocksdb-sys` 编译，pid 590947 起一串），为避免与它抢 CPU、重复跑一次几分钟的 C++ 编译，r2 这十二处没有另起一份复跑，只核对 SHA256SUMS 与产物文件内容（下文「没做什么」一节写明）。

## 一、第一轮十二处里的七处（T2、T3、T4、P4、P5、P6「省」、G2）

| 格 | 判决那句 | 辩方判定 | 证据 |
|---|---|---|---|
| T2 | 「用户内容摆一个节点头时叠加 0 红、物化 12 红（interior.out:14）」 | **够不着，行号错**：`interior.out` 第 14 行是 `attack_t2_interior_final_state overlay_red=[] materialized_memory_pool_red=["checker:I-7.8"] must_be_nonzero=1`——这是「整条全落那一态」单独一行的红名单，字面上没有「12」这个数，也不是逐状态计数。真正带「12」的是第 13 行：`attack_materialized_summary ... red_names_full_local_localimage_fullrecords={"checker:I-7.8": (0, 12, 12)} judgment_differs_materialized_image_with_full_record_stream_total=12`——(0, 12, 12) 三元组里第一位是叠加（全流）红数 0，后两位是物化（局部镜像、配整条记录流）红数 12。数据本身撑得住「叠加 0 红、物化 12 红」这句话，只是判决贴错了行号（应是 `interior.out:13`）。不改变「打中」的结论 | `research/prompts/m3-prune-gpu-r1-opus-model/outputs/interior.out` 第 13、14 行（`cat -n` 现读） |
| T2 | 「第三态认定看写表历史，物化丢了历史，子段状态数 5 vs 3（t4.out:3）」 | **站得住**：`t4.out` 第 3 行 `variant=c_zero_bytes ... child_states_overlay=5 child_states_materialized=3` 字面吻合；`crash.rs:1600` 现查是 `base_holds_nonzero_bytes_there \|\| an_earlier_write_with_bytes_overlaps`，与判决描述一致 | `t4.out:3`；`crates/singlefs-checker-tier/src/crash.rs:1600` 现查确认 |
| T2 分辨臂 | 「正文 P0 行『只改读的代价，不改判定』在这两处不成立」 | **分辨臂站得住**：攻方报告明写「①②都分辨叠加与物化」——同一批字节（`states_with_read_bytes_differing=0`）下叠加 0 红、物化 12/5 红，两个候选（叠加 / 物化）被这组数据实打实地分开，不是两边一起中的通泛问题。round2 的 F1（`r2-f1-shifted.out`）另外证明叠加候选在**别的**构造下也会假红，但那是另一条历史（数据单元起点复用）打中叠加自己的漏洞，不是同一份 T2 证据回头打中叠加——两件事不冲突，T2 的「分辨臂」结论不受它影响 | `m3-prune-gpu-r1-opus-output.md:240-243`「四句」第 1 条；`r2-f1-shifted.out` |
| T3 | 「同一种类串下并着『根槽第一次写』与『根环回卷』（kinds.out:1）」 | **站得住**：`kinds.out:1` 的 `kind_strings_with_several_condition_signatures=1`、`detail` 里 `root_slot_rewrite=false` 与 `=true` 两个签名并在同一个种类串下；现查 `attack.rs:1262-1264` 的 `rewrites_a_root_slot` 定义（本次发布里一条根写与更早一条根写在同一物理位置重叠）与「根环回卷」语义相符 | `kinds.out:1`；`m3-prune-gpu-r1-opus-model/attack.rs:1262` 现查 |
| T3 分辨臂 | 改法「种类串 + 覆盖关系签名」 | **分辨臂站得住，且已被 round2 复核**：round2 `r2-kinds.out:1` 的 `identities_merging_conditions=0`（改法版本）对照同一份历史下的 `kind_strings_merging_conditions=1`（原候选，仍旧照旧），这不是同一轮里自我宣称，是下一轮另一条腿在另一个世界上量出来的对照，比单轮自证更硬 | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-kinds.out:1` |
| T4 | 「父结束镜像逐字节相同、子节点写表相同，子节点枚举域 5 vs 3（t4.out:3）」 | **站得住，而且这条引文选得比表面看起来更硬**：`t4.out:3` 是 `variant=c_zero_bytes`，`parent_end_sector_maps_equal_to_a=false`（扇区表不同）但 `parent_end_bytes_equal_to_a=true`（逐字节相同）——专挑了「字节相同但扇区表不同」的那一支，排除了「指纹按扇区表算」这条容易被指出的漏洞，直逼「按字节算」的指纹一样会被打中。判决原句只写「逐字节相同」没提扇区表，字面没错，但没把这一手的分量写出来 | `t4.out:2`（variant=b，扇区表也相同）、`t4.out:3`（variant=c，扇区表不同但字节相同）两行对照 |

**独立复跑坐实**：把 `m3-prune-gpu-r1-opus-model/` 拷进 `/tmp/claude-1000/m3-prune-gpu-r3-sonnet/r1-rerun/`（`rsync --exclude target --exclude .git`），照 `rerun.sh` 在这份新副本上重新打补丁、重编（不带 `verdict-store`，29.76 秒编完）、跑十个世界，十个世界「与存档逐行相同（去掉带 `seconds=` 的行）」（原始命令输出全文，无删减）。上面 T2/T3/T4 与下面 P4/P5/P6/G2 引的六份 `.out` 文件（`interior`、`t4`、`kinds`、`small`、`p5`、`p6`）全部在这份独立副本里复现，不是只信存档。

| 格 | 判决那句 | 辩方判定 | 证据 |
|---|---|---|---|
| P4 | 「只差一次写的 62 对里 10 对读到被翻那次写罩不到的位置（small.out:5，自适应读）」 | **站得住**：`small.out:5` 字面是 `pairs_differing_in_one_write=62 pairs_reading_positions_the_flipped_write_does_not_cover=10 example=...`，与判决逐字对应 | `small.out:5`（复跑副本同行同值） |
| P4 分辨臂 | 「分辨臂（P4 出局，P2 不受影响）」 | **站得住**：P2 的候选键是「整串调用（位置 + 内容）」，本身不声称只重算被覆盖的读，P4 是在 P2 之上加的一层「只重算被翻那次写罩到的读」的增量优化，这条优化专门被打中，P2 自己的键定义没有这句承诺，攻击没有连带打中 P2 | `m3-prune-gpu-r1-opus-output.md:136` |
| P5 | 「63 个状态里 16 个红，段首与全落两态都绿（p5.out:4）」 | **站得住**：`p5.out:4` 字面 `red_states_in_unit_segment=16`、`segment_first_state_red=Some(false)`、`all_persisted_state_red=false`，与「16 个红」「段首、全落两态都绿」一一对应 | `p5.out:4`（复跑副本同行同值） |
| P5 分辨臂 | 「精确的读集键…48 类、同键异判 0；只有『根没写就当一类』的 P5 漏掉」 | **站得住**：同一个世界里，精确键（`full_key_classes=48 full_key_inconsistent=0`，见 `p5.out:4`）与被攻的 P5 启发式规则（按段首/全落两态代表整段）是两种不同的候选，精确键在这份数据上没有被打中，P5 的漏洞不是「这段历史本来就没法判」的通用限制 | `p5.out:4` |
| P6「省」 | 「单元段上叶子数 = 状态数 = 4095、判定只有 1 类（p6.out:2）」 | **站得住**：`p6.out:2` 字面 `states_in_unit_segment=4095`、`distinct_judgment=1`、`leaves_recovery_plus_checker_normal=4095`，与判决逐字对应；`crates/singlefs-harness/src/memory_pool.rs:747`、`crates/singlefs-checker/src/walk.rs:2405` 现查确认候选槽机理 | `p6.out:2`（复跑副本同行同值）；两处代码行现查确认 |
| P6「省」分辨臂 | 「精确站住；『省』被打中」 | **站得住，判决本身已经把『精确』与『省』分开**：`torn.out:2` 的镜像差异（`states_where_enumerated_slot_differs_from_physical_tearing=2`）只在「按覆盖关系推算撕裂镜像」这个具体实现细节上出现，且判定层 `states_where_judgment_differs=0`——攻方腿自己的结论是「照搬枚举器的撕裂写表就不会有这个差」，不是 P6 定义本身的漏洞。判决行文把这句一起放进 P6 那一格的攻方列，但结论列只用它佐证「精确要照哪张表算」，没有把 torn 的镜像差当「省」被打中的证据，读法准确。round2 `r2-b4.out` 用 P7 键把这个单元段从 4095 压到 4 类（`p7_key_classes=4`），证明「省不下」只是 P6 这个具体键法的问题，不是这类段本身没法省——这也反过来说明 P6「省」被打中确实分辨了 P6 与 P7 两个候选 | `torn.out:2`；`m3-prune-gpu-r2-opus-model/outputs/r2-b4.out:6` |
| G2 | 「位置表取自代表态，44 个状态里 6 个判错，GPU 与 CPU 两份划分逐个相同、对拍发现不了（small.out:4）」 | **站得住**：`small.out:4` 字面 `states=44 classes_by_representative_positions=38 states_whose_judgment_differs_from_their_class=6 gpu_cpu_partition_mismatch=0`，与判决逐字对应 | `small.out:4`（复跑副本同行同值） |
| G2 分辨臂 | 「G2 与 G1 分得开，G1 不分类」 | **站得住**：G1（本地攻方站住那一格）走的是「(a) 类占剩余每状态耗时上界」的成本核算，不涉及「代表态位置表」这种分类聚合，两个候选处理的是不同的对象（G1 是成本估算，G2 是位置表复用方案），没有共享输入使这个攻击连带打中 G1 | `m3-prune-gpu-r1-main-verification.md:64`（G1 行）对照 `m3-prune-gpu-r1-opus-output.md:161` |


## 二、第二轮五处（B1、B2、B3、B7、B8）

| 格 | 判决那句 | 辩方判定 | 证据 |
|---|---|---|---|
| B1 | 「并错：X 与载荷改坏的 Z 11 段 B1 身份逐段相同，段 10 的 8 个状态判定不同…拆错：写入时间 +1 秒的 Y，路径写表哈希从段 7 起不同、段 8–10 的 12 个状态判定逐个相同」 | **站得住**：`r2-b1.out:2` 字面 `segments_with_equal_b1_label=11 compared_states=44 states_equal_label_status_differs=8 in_segments={10}`；`r2-b1.out:3` 字面 `segments_with_equal_path_hash=7 states_path_hash_differs_status_equal=12 in_segments={8, 9, 10}`，与判决逐字对应 | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b1.out` 第 2、3 行现读 |
| B1 分辨臂 | 「没有一个候选两格都不中」 | **站得住，且判决没有夸大**：判决没有说「B1 打中了所有候选」，只是如实写出没有候选能同时躲过并错与拆错两个测试——这本身是个诚实的负面结果，不构成「打中不分辨臂」，因为它清楚区分了哪个候选在哪一格中（种类串 / 种类串+覆盖签名在并错中，路径写表哈希在拆错中） | `m3-prune-gpu-r2-opus-output.md:100`「四句」第 1 条 |
| B2 | 「版本表不在指纹里：12 个状态（r2-b2-versions.out）」 | **数据站得住，但判决的「打中」标注漏抄了攻方自己写明的一条限定**：`r2-b2-versions.out:2` 字面 `path_write_table_hash_equal=true states_whose_status_differs=12`，与判决数字对应。但攻方报告在这一格的「四句」第 1 条明写：「岔路单 B2 的三个候选…都不带版本表，三个一起中；带上版本表的那一版不中。**所以它不分辨岔路单里的候选，分辨的是『指纹带不带测试这一侧的期望』**…按『判据自己也会写错』那张表是**『打中不分辨臂』**：共用前提（指纹只罩实现与判法代码）另立一笔账先修。」`m3-prune-gpu-r2-main-verification.md` 与 `m3-prune-gpu-r1-main-verification.md` 全文里没有一处出现「分辨臂」三个字（`grep -n "分辨臂" 两份判决` 零命中）——这条限定没有被写进判决。这不是说 B2 没打中：版本表确实漏了，需要修；但把它记成「B2 候选专属的打中」而不加「三个候选一起中，不分辨」这句限定，容易让人误读成「路径写表哈希这个候选比另外两个候选更差」，实际上三个候选在这一点上一样差 | `r2-b2-versions.out:2`；`m3-prune-gpu-r2-opus-output.md:118`；`grep -n "分辨臂" research/prompts/m3-prune-gpu-r1-main-verification.md research/prompts/m3-prune-gpu-r2-main-verification.md` 零命中（现查） |
| B2 | 「下游写：六条历史 300 个状态 0（r2-b2-downstream.out，没打中）」 | **站得住**：`r2-b2-downstream.out` 六行 `states_whose_status_differs_full_versus_prefix=0` 覆盖 33+45+93+33+45+51=300 个状态，算术对得上「300」 | `r2-b2-downstream.out` 第 2、4、6、8、10、12 行现读并求和 |
| B3 | 「单点从物化的父镜像起算，4 个序号对不上」 | **站得住**：`r2-b3.out:1` 字面 `child_states_full_path=5 child_states_from_materialized_parent=3 ordinals_mapping_to_different_landings=4`，与判决对应；机理与 T4/T2 共用同一处 `crash.rs:1600`，属于同一族问题在 B3 上的复现，不是新发明的巧合 | `r2-b3.out:1` |
| B3 | 「一个段改成 ignore，其余 44 个状态的键全变（r2-b3.out）」 | **站得住**：`r2-b3.out:3` 字面 `states_before=45 states_after=44 states_in_both=44 ... states_in_both_whose_plan_hash_plus_ordinal_changed=44`，44 个状态确实全变 | `r2-b3.out:3` |
| B3 分辨臂 | 「它分辨的是岔路单 C5 的两个候选（从 KV 取父节点结束镜像 / 从根重放到父节点）」 | **站得住**：攻方明写「按整条路径的写表定每次写几态（第一轮改法 1），单点与全量逐号相同；只拿父结束镜像的不同」——即「从根重放、按路径写表定序号」这一支没有被这个反例打中，两支候选被明确分开 | `m3-prune-gpu-r2-opus-output.md:153` |


| 格 | 判决那句 | 辩方判定 | 证据 |
|---|---|---|---|
| B7 | 「计数格随路径变 8 个」 | **站得住**：`r2-b7_150.out:1` 字面 `states_whose_tally_cell_is_not_a_function_of_the_node=8`，与判决对应；这 8 个状态与第一轮 U4（small.out:3）是同一批段 10 状态，判定本身相同、只是计数格（哪次发布）不同——判决与攻方报告都点明了这一点，没有把「计数格不同」错写成「判定不同」 | `r2-b7_150.out:1`；对照 `small.out:3` |
| B7 | 「判定向量带恢复落到的根时 150 次覆盖写的长流上 307 种，真库在第 257 种上拒登记（r2-b7_150.out）；不带恢复落到的根时 7 种」 | **站得住**：`r2-b7_150.out:3` 字面 `distinct_status_vectors=7 distinct_vectors_with_landed_roots=307 real_store_refused_at=Some("257:MoreThan256VerdictVectorsNotDecidedInTheFirstVersion")`，与判决逐字对应 | `r2-b7_150.out:3` |
| B7 分辨臂 | 「1 字节编号中，2 字节编号（65536 种）在这条流上不中」 | **站得住，是个真候选对照，不是同一条测线自证**：307 远小于 65536，2 字节编号在这条具体流上确实不会拒登记；但要注意这只对这一条流成立，判决自己在「改法」一栏也写了「按项红绿（7 种）」与「恢复到了哪一版另存」两条，没有断言 2 字节编号能罩住所有流，读法准确 | `m3-prune-gpu-r2-opus-output.md:290` |
| B8 | 「十种放法、两种改定义（拆字段、同名改义）、159 个状态…（r2-b8.out 第 4–21 行）」 | **「十种放法」是个数错，应为九种；不影响打中本身**：现查 `r2.rs:1376-1394` 的 `enum VersionPlacement` 只有 9 个变体（`NoVersion`、`LibraryLayoutVersion`、`DefinitionVersionInKeyBumped`、`DefinitionVersionInKeyForgotten`、`SchemaHashInKey`、`CanaryStatesRejudgedAtOpen`、`CanaryOneStatePerStoredVector`、`CanaryOneStatePerStoredVectorLastOccurrence`、`RawStatusesDerivedAtRead`），`r2-b8.out` 第 4–21 行正好是 9 种放法 × 2 种改定义 = 18 行，与「九种」吻合、与「十种」不吻合。攻方报告自己的表格（`m3-prune-gpu-r2-opus-output.md:296-306`）也只列了 9 行。这处是判决文本自己数错了一个，`159 个状态`、`读错 4 或 16 个`、`两种改法下都 0 的只有…` 这几句本身字面仍然撑得住 | `r2.rs:1376-1394`（`enum VersionPlacement` 现查 9 个变体）；`r2-b8.out` 第 4–21 行（18 行 = 9×2）；`m3-prune-gpu-r2-opus-output.md:296-306`（表格 9 行） |
| B8 | 「库级布局版本拦不住调用方改编码（`crates/singlefs-checker-tier/src/verdict_store.rs:101`）」 | **站得住**：现查该行是「一个状态的判定向量，按调用方定的编码存成字节（这个模块不解释它）」，与判决描述一致 | `crates/singlefs-checker-tier/src/verdict_store.rs:101` 现查 |
| B8 分辨臂 | 「两种改法下都 0 的只有『抬了的定义版本进块键前缀』与『存逐项原始判定、读的一方现算』」 | **站得住**：`r2-b8.out` 第 6、15 行（`DefinitionVersionInKeyBumped`）与第 12、21 行（`RawStatusesDerivedAtRead`）四行都是 `states_read_with_the_wrong_meaning=0`，其余七种放法在至少一种改定义下读错；金丝雀三种放法里有两种在某一形下靠「拒开」躲过（不是「读对」），判决把它们与「读错 4 或 16」放在同一句里没有细分「拒开」与「读对」的差别，读法上略粗，但没有把拒开的格错记成读对的格，不影响「B8 打中」这条主结论 | `r2-b8.out` 第 6、10、11、12、15、19、20、21 行现读 |


## 三、给第二轮站住的六处各找一个最可能的反例

以下全部**推的，没有造世界、没有跑过**，标注为线索交主 agent 核实，不是已坐实的打穿。

### C2（核对判据收一处）

正文摸底 N7 记的是「结局判法各抄一份」：`crates-mutation-rows.py:649`、`prove-red.sh:75`–`:90`、`mutate.sh:536`、`mutation-sampling.md:80` 四处各写一份「抓到 / 无效 / 没红」的判法，这一轮刚修过 `prove-red.sh` 认不出 `- should panic` 的毛病，正是四份互相漂移的一个实例（`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:162`）。C2 的修法是把判法收成一个模块，54/59/`mutate.sh` 都调它。

**最可能的反例**：收成一个模块解决的是「多份拷贝互相漂移」，换不来「模块与各调用方的真实运行结果永远同步」——如果某个调用方（比如 `mutate.sh`）以后新增一种结局（新的内存包装退出码分档、新的 should-panic 变体），而这个模块的分类枚举没有跟着改，模块会把这个新结局落进一个默认类别（比如「无效」），调用方按模块给的分类继续判断，不会报错，只会静默分错类——这与今天「四份拷贝互相漂移」是同一形状的问题，只是从「四处互相漂」变成「一处与真实运行结果漂」，模块本身没有一种自动核对「它认识的结局种类」与「调用方实际会产生的结局种类」逐一对齐的机制。

### C4（四种跑法）

C4 的候选是「单机 CPU / 双机 / 单机 GPU / 双机 GPU，开关只从配置读；四种在重叠处结果逐项相同」。

**最可能的反例**：`crates/` 今天没有 GPU 代码（这一轮背景材料「实现今天的样子」一节已写明），GPU 核对的并发聚合顺序天然是这一轮 A9 段自己列出的风险（「反向检查：输入与节点代码都没变、重跑一遍输出却不同，判红（不确定性：并发次序、内存上限、未初始化的东西）」）——B4 节 P7 键要聚合的三项里有一项是「归并组（归并键、各成员载荷校验和、校验和不同时哪几份载荷读得出）与同一对象键下两组以上的定序」（`m3-prune-gpu-r2-opus-output.md:178`），这是一个排序敏感的判定：CPU 顺序扫描与 GPU 并行归约如果对「哪几份读得出」的判定依赖遍历到达的先后顺序，同一个状态在两条跑法下可能给出不同答案。这是「重叠域上判定向量逐状态相同」这条检查最该覆盖、但今天还没有 GPU 实现可以实测的一个缺口，标「推的，没有代码可验」。

### C5（一个入口、三档）

C5 的候选是「单条线、单点从哪里取父节点起点镜像；三档在重叠处与全量逐项相同」，判决备注它「与 B3 改法同向」。

**最可能的反例**：B3 第二部分（`r2-b3.out:3`）已经量出「一个段改成 ignore，其余 44 个状态的（计划哈希，序号）全变」——如果单点档的候选是「从 KV 取父节点结束镜像」（B3 第一部分打中的那个候选），那么当某个祖先节点因为 ignore 开关而被重新展开时，KV 里缓存的父节点镜像若不跟着失效，单点档会拿一份按旧 ignore 状态算出的镜像继续算，得到与全量档不同的结果——这正是 C5 自己问的「三档在重叠处与全量逐项相同」这条检查要专门盯住的场景，且直接沿用 B3 已经量出的机理（ignore 开关改变整条流的计划哈希），不是凭空编的。

### C6（复用与作废）

C6 判决已经附了条件（指纹按 B2 改法、库版本按 B8 改法），「不设时限」本身没被单独攻过。

**最可能的反例**：A9 段用户已定案「节点代码摘要」按「词法单元取哈希…别的任何改动（常量、宏、类型、格式、**依赖版本**）都算改了」——但这是第三轮才定的口径，C6 判决产生时（第二轮）还没有这条约束。B2 版本表漏了指纹这一格已经证明「指纹只罩实现与判法代码」这条前提本身会漏东西；同样地，如果判法摘要不覆盖某个间接依赖（比如 GPU 侧的核对库、或者被内联进判法但摘要没扫到的辅助函数），代码语义变了而摘要不变，「不设时限」意味着这个复用结果会被无限期地继续复用、不会因为「太旧」被重新核过——这与 B2 是同一族问题（判法摘要罩不全），只是换到了「没有时限兜底」这个环节上。

### C7（门禁 59 号）

C7 判决是「只共用调度与判定分类模块，不共用存储」。

**最可能的反例**：54 号（崩溃枚举）与 59 号（变异复跑）如果各自往自己的库里录判定，两边永远不共用存储，就没有一处能核「同一个崩溃状态在两条门禁上判定是否一致」——如果某次变异恰好改坏了一条真实的判定逻辑（不是变异本身要测的那一行，而是连带影响到的别处），59 号只在它自己的库里记录这次变异触发的新判定，54 号仍然读它自己库里没被这次变异触碰过的旧判定，两边各自为政，永远不会报出「同一状态两处判定不同」这类交叉不一致。这与本轮 D1「各写一份、按块原样导入」打中的问题是同一形状（分开存储导致缺交叉核对），只是把「两台机器」换成了「54 号与 59 号两道门禁」。

### D1「各写一库按内容换编号导入」

r2 判决量过 `r2-d1.out:4` 的 `merged_states_decoded_differently_from_single_machine=0`，即按判定向量内容重新登记编号后合完与单机一致。

**最可能的反例**：这个 0 是在两台机器跑**同一份代码**（真 `VerdictStore`，同一个二进制）时量出来的；如果两台机器的 checker 代码版本不同（例如一台还没重新编译、落后于另一台新增或改写过某条不变量），两边写出的「判定向量」在**语义**上已经不是同一个 schema——「按内容换编号」这个手法本身只按字节内容做匹配，不感知 schema 版本，字节相同就当同一个向量处理。这正好落在 B8 版本号那组问题的射程里：如果这时判定向量还是「按调用方编码存成字节，这个模块不解释它」（`verdict_store.rs:101`）而不是 B8 改法的「存逐项原始判定」，两台机器的字节巧合部分重叠时会被静默地当成同一个向量导入，产生与 B8「同名改义」那一类完全同形的误读。B8 的改法（存逐项原始判定）能不能同时堵住这个反例，判决与攻方报告都没有专门在「跨机器、跨 checker 版本」这个组合上量过，值得留意。


**C2 反例的先例**：`.claude/rules/mutation-sampling.md:80` 正是同一形状问题的既有记录（现查）：「变异表里一条本来好好的条目，会因为**别处改了一个常量**而从『被抓』变成『无效』，而没有任何东西报警。」——C2 把判法收成一处并不消除这条规则要防的漂移，只是把「谁与谁漂移」从「四份拷贝互相漂」换成了「一处模块与真实运行结果漂」，机理相同。

