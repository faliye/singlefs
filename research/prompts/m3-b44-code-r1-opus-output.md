# 代码欠账批 4-4 代码轮第一轮攻方腿（Opus）报告（2026-10-16）

攻 K1、K3、K4、K5、K6，余力做了 K7、K8 的读码核对（正文「三、格」）。K1 的数全是**副本上的数**：冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r1/repo/` 的拷贝，只在 `mount.rs` 那一判外面包一层记录（`patch/instrument_k1.py`：记下那一判交回的 bool 与四个落点读出的实例代号，交回值原样），加攻方原型；判法本身是 W1 的实现原样，没装候选开关。不是入库装置上的数。其余各格是读码，标「推的」。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-b44-code-r1-opus-model/`（骨架照 C834 第三轮攻方原型改来，候选开关全部去掉）。复跑（整条经内存包装；本轮负载 67–290 时：编译 1 分 48 秒，批 0 580 秒、批 1 369 秒、批 10 766 秒，换几何 31 秒）：

```
bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/m3-b44-code-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-b44-code-r1/repo \
  --work <新的草稿目录>
```

末行判结果：`name=rerun_compare all_same`（`out/` 两份排序后逐行相同）。这条腿自己没有把 `rerun.sh` 从头跑一遍，`out/` 两份是逐批跑出来的输出用同一个 `aggregate.py` 与同一条 `grep | sort` 汇的（见「这条腿自己的限度」）。

| 文件 | sha256 |
|---|---|
| `.cargo/config.toml` | 891b14e98b472f513f13dea65ff5b2a7cf1429b97a29b2c488f06e90ee095fc5 |
| `Cargo.toml` | bd7eb19347861c1bbada44bf425d9762405af5959f8af6a45ccf85e56e819c39 |
| `out/agg-enumeration.txt` | 7457cea25126eb5679e6b4476aaf2f3174a23752d3dd451e9e2e4391848cbb0b |
| `out/survey.txt` | 6e99fd5bc83ec91a8c3f1fdcf8c2c79482a821d817a4032ace9f7bb1d3eb0a35 |
| `patch/instrument_k1.py` | 39c7826ada82475d8bdd715d6317821d375aeafb4a5a0b7f0aee4cb1a90bd9da |
| `src/main.rs` | 4f54c67f9b683d1206fdb35f4db6278b187c3d4ea8366b3663094f929f967ec3 |
| `src/prefix.rs` | fcb76beda8219f6d435830aecf441d564cde27333b103085003bc1c0b1c8299b |
| `tools/aggregate.py` | 521d1bfbda5692283aa67971a6e49a99918b3090994b1c011ae1d567d43b1f91 |
| `tools/faultsets.txt` | 8dc4a023ede24d5cfffacf81b62a5dcf3593f745d4ffe2347ffe651a78c0fd89 |
| `tools/make-pieces.py` | 39da82e26167c84d06145c0c2319e551ae24c3c09034883d2fe12d2f20395a70 |
| `tools/pieces.txt` | 6dd62125fae1fe90af785a13ec8ccfcff3fd5701bd6f6afa6bacd56d356999ff |
| `tools/rerun.sh` | be73ec7aff3f952ad20810a39ffd713a3a9023465abab8f4d212ad6b1f59f9b3 |
| `tools/run-batch.sh` | 2e3d6ebb617f134a969e5b3e24f4abca3dbcb1383d9e1f28c494c2f8b8d7a336 |
| `tools/survey-batch.sh` | 0e9a1be0e1734c032f3deddbc13930cf03e864f5b22edd4869b896fb027d7ff1 |
| `tools/survey-numeric.txt` | 8a28537f7db0c4849f5778b45d2c7dccfe918345e6bfe3e2841d46d91f368540 |
| `tools/survey-prefixes.txt` | 040b748a1b86ba6d38a2afc869ffa2d1d85ee346b6533423bf068ba0cb8c2100 |

读法：`before=walkable` 是被判那次挂载 M2 之前那个崩溃状态上池级 checker 不红 I-7.2（R 走得完）；`before=gone` 是已红 I-7.2。`judge=` 是那一判交回的（`pass` 放过、`refuse` 照「记录不齐」那一支办），没走到那一判的另记 `name=k1_not_reached_total`。打中的定义写死在 `tools/aggregate.py` 里：判放过、挂载做成、写行根落盘之前新写掉 R 的单元，且 `before=walkable`；汇总行带 ` HIT` 尾巴。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 |
|---|---|---|
| K1 R 走得完而它放过并写掉 | **没打中**。四段 `urJ` 一族历史，M1 写流的崩溃状态全量（90293 个，一个不落），被判那次挂载 M2 两序：走到那一判、R 走得完的状态它全拒（`walk_pass=0`），放过的全落在 R 已走不完的状态上。第三轮打中 `pointer_per_kind` 的那两形（k=5 `tt1+mr0+mr1+rec` 的 972 个、`tt1+mr1+rec` 的 324 个）今天两序都拒 | 量过（副本） |
| K1 入库 `urJ` 那一形它拒 | **没打中**。k=5 写行根之前全落那几个盘面、读故障 `none`：走到那一判的 4 个状态两序都放过；换几何 232 种（26 种 k、206 种前缀）的同一盘面、`none` 下走到那一判的 11 种全放过，没有一种在这一判上拒 | 量过（副本） |
| K1 读法逐句对定案 | 对得上：读树表与映射根四个落点、空指针不算、16 KiB 先解索引节点再 32 KiB 解数据单元与打包记录单元、比写序实例代号 `>` R 的、一个指针两份都要、不读系统配置的号、否则走「记录不齐」那一支。两处定案没写的细节（不判单元头里的 fsid；不判单元与位置条目的校验和）见第一节第 4 小节，推的不改判 | 读码 |
| K1 可用性一边 | R 已走不完、而有一份被读故障藏住时它拒（k=5 `tt1+mr0+mr1+rec` 11964 个等，第三轮判决交用户时已列为代价）；没有读故障时一个都没拒 | 量过（副本） |
| K3 误拒合法槽 / 该拒照择 | **没打中**。写者只写格式常量；读者把它当自证不过、与 checker 次序一致。相邻：节点大小坏且 S 越界的槽，改前整池拒、改后换槽，与「当自证不过」同一个处置，不算判错 | 推的（读码） |
| K4 合法写路径写出同盘两条非空指针 / 全零例外认错 | **没打中**。产品写路径的位置条目都由两块不同身份的盘排序得来，mkfs 断言两块盘、两盘时区域归属写死 0/1/0。相邻一形（零轮，推的）：三棵容器树（墓碑、谱系、退役）的容器指针与根之下节点指针不在重建时那道 I-2.5 判里，盘上一条同盘两条的非空指针今天在写者断言上 panic（改前 `<=` 放过），见第三节 | 推的（读码） |
| K5 产品调用方走到新断言 / 行为变化落在合法态 | **没打中**。`allocation_records_are_one_per_device` 的两处产品调用都交读者的盘表，零块盘走不到；`index_node_entry_capacity` 的调用方都交格式常量，条目宽 0 改前就是除零 panic | 推的（读码） |
| K6 错误交回之前已发写 / 三处 `match` 处置错 | 发布那一层**没打中**：错误在 `prepare_the_version_publish` 动分配器与写之前交回；三处穷举 `match` 处置对。挂载那一层有一形交主 agent（推的）：挂载续做里一次删除类发布报这个错时写行那次发布已落盘，挂载拒、每次可写挂载都再写一次行，见第五节 | 推的（读码） |
| K7 删掉的代码有调用方 / 注释说错 | **没打中**（抽查）：删掉的公开项在 `crates/`、`scripts/experiments/` 里零引用（同名命中都是各自的局部定义）；抽查的改注释与代码对得上 | 推的（grep + 读码） |
| K8 `leaves` 计数 / 常量合并漏字面量 | **没打中**：计数在上段那一趟之后、下段之前取；core 里 inode 树 key 宽不再有字面量 8 | 推的（读码） |

一句话：W1 的实现在第三轮攻方原型同一族历史与更宽的读故障组上，没有放过一个 R 走得完的状态；它放过的只有某个指针两份都已被更新单元盖掉的状态，那时 R 已走不完。

## 一、K1：`one_pointer_has_both_copies_overwritten_by_a_later_instance`

### 1. 造了什么

历史与第三轮攻方同一族（`research/prompts/c834-r3-opus-output.md` 第一、二节），许可每一步的条款与那里相同：R 照入库搭建 `common_unwitnessed_publish` 录（384 槽小盘、第一个文件、覆盖写 k 次或前缀动作，R 是一次覆盖写或顺序写三个数据单元），崩在 R 的根槽之后、系统配置轮换之前；M1 是入库 `urJ` 那一形的可写挂载（R 的根槽、单元、记录读回全零，M1 退回 P、取号 2、写行）；M1 的写流按崩溃点重放的枚举域逐个崩溃状态展开（`crash_state_enumeration_tables` + `extract_flow_facts`，每一段 `full_expansion`，原地覆写取三态）；每个状态上做被判的那次挂载 M2，读故障点名 R 的哪一份读回全零，两种交盘次序各一次。和第三轮不同的三处：被判的是冻结副本里的产品代码本身；每段历史的崩溃状态**全量**（第三轮只到写行根之前的 16400 个）；k=5 那一段 17 组读故障全量（第三轮 3 组全量、14 组只跑了序号 0–3719 与 16399）。

| 历史 | 崩溃状态（全量，`crash_state_count_with_torn_in_place_overwrites`） | 读故障组 | 被判挂载次数 |
|---|---|---|---|
| `urj_overwrite_k5`（入库 `urJ` 那一档） | 20516 | 17 组（`tools/faultsets.txt`，含 `none`） | 697544 |
| `urj_seq3_k5`（R 三条记录） | 20521 | 6 组（缩过，见第七节） | 246252 |
| `urj_overwrite_pooww`（树表槽上落打包记录单元） | 24623 | 6 组 | 295476 |
| `urj_seq3_poiww`（映射根槽上落打包记录单元的后半截） | 24633 | 6 组 | 295596 |
| 换几何：26 种 k、206 种前缀，写行根之前全落的那一个盘面 | 232 个盘面 | `none`、`rec`、`mr0+mr1+rec` | 1392 |

状态数是各段跑前 `count` 模式打出来的（`name=k1_history … state_count=20516` 等四行，草稿目录 `runs/` 下），每段都不到约 10⁶；四段合计 90293 个状态，远低于约 10⁷。

### 2. 结果（原样，`out/agg-enumeration.txt`）

第三轮判 `pointer_per_kind` 出局的那两形，今天两序都拒在任何写之前：

```
name=k1_cell history=urj_overwrite_k5 faults=tt1+mr0+mr1+rec order=forward before=walkable judge=refuse outcome=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=972 example_ordinal=5128
name=k1_cell history=urj_overwrite_k5 faults=tt1+mr0+mr1+rec order=reversed before=walkable judge=refuse outcome=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=972 example_ordinal=5128
name=k1_cell history=urj_overwrite_k5 faults=tt1+mr1+rec order=forward before=walkable judge=refuse outcome=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=324 example_ordinal=5384
name=k1_cell history=urj_overwrite_k5 faults=tt1+mr1+rec order=reversed before=walkable judge=refuse outcome=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=324 example_ordinal=5384
```

入库 `urJ` 盘面（读故障 `none`）走到那一判的 4 个状态两序都放过、没再写掉 R 的单元：

```
name=k1_cell history=urj_overwrite_k5 faults=none order=forward before=gone judge=pass outcome=mounted_r_kept states=4 example_ordinal=16395
name=k1_cell history=urj_overwrite_k5 faults=none order=reversed before=gone judge=pass outcome=mounted_r_kept states=4 example_ordinal=16395
```

全表计数（`grep` 在 `out/agg-enumeration.txt` 上现数）：`name=k1_cell` 114 行；带 `judge=pass` 的 42 行，其中 `before=walkable` 的 0 行；带 ` HIT` 尾巴的 0 行。两序逐格同数（汇总脚本按（历史，读故障，R 此前，判，结局）对两序，0 处不同）。按历史与读故障组（正向，反向同数）：

| 历史 / 读故障 | R 走得完、走到那一判：拒 | R 走得完：放过 | R 已走不完：放过 | R 已走不完：拒 |
|---|---|---|---|---|
| k5 / `none` | 0 | 0 | 4 | 0 |
| k5 / `rec` | 0 | 0 | 2808 | 0 |
| k5 / `tt1+mr0+mr1+rec` | 972 | 0 | 0 | 11964 |
| k5 / `tt1+mr1+rec` | 324 | 0 | 0 | 6148 |
| k5 / `tt0+tt1+mr0+mr1+rec`（Y2 六段） | 2924 | 0 | 0 | 13476 |
| k5 / 只藏树表或只藏映射根（加记录）的 6 组 | 0 | 0 | 2056–4104 | 0–7104 |
| k5 / 其余 6 组（树表与映射根各藏了至少一份） | 324–972 | 0 | 0 | 6148–11964 |
| `pooww` / `mr0+mr1+rec` | 0 | 0 | 4104 | 0 |
| `pooww` / `tt1+mr0+mr1+rec` | 4096 | 0 | 0 | 4104 |
| `seq3_k5` / `tt0+tt1+mr0+mr1+rec` | 2195 | 0 | 0 | 14205 |
| `poiww` / `rec` | 0 | 0 | 1032 | 0 |
| `poiww` / `tt1+mr0+mr1+rec` | 3072 | 0 | 0 | 5128 |

（区间那两行是几组各自的数，逐组的数在 `out/agg-enumeration.txt`。）`pooww` 的 `mr0+mr1+rec` 那 4104 个放过，靠的是树表两个槽上的打包记录单元：实现先按 16 KiB 解索引节点不成、再读 32 KiB 解打包记录单元成，与定案的读长一致。

换几何（`out/survey.txt`，1392 行）：读故障 `none` 下 232 种几何里走到那一判的 11 种（k=5 与 10 种前缀），两序全放过、`new=0`；其中 `piiiiw`、`poiiiw` 等 5 种是树表两槽还是 R 自己的（`tt0@d0s1544:1`）、映射根两槽是 M1 的（`mr0@d0s1543:2`），靠映射根那一个指针放过。没有一种几何在这一判上拒。

### 3. 为什么造不出「R 走得完而它放过」（推的，读码）

放过要某一个指针的两个落点都读出自证过、写序实例代号大于 R 的单元头（`crates/singlefs-core/src/mount.rs:2168`「.is_some_and(|instance| instance > root.instance)」）。R 写出或继承的单元，写序的实例代号都不大于 R 的；两个落点都是别的单元时，走读按位置条目里的校验和核每一份（`crates/singlefs-core/src/recovery.rs:401`「if crc32_castagnoli(&bytes) == location.unit_checksum {」），两份都对不上，那个单元两份都没了，R 走不完。要打破它得有一个单元的写序实例代号大于引用它的根：取号跨盘取最大，读码没找到这样的写路径（暖机、抬 F 推的根、回退那次发布的根都带写它的那次挂载的实例代号，指的单元都不更新）。推翻条件：造出一条根，它引用的某个单元的写序实例代号比根的大。

### 4. 定案没写到的两处细节（推的，不改判）

- 不判单元头里的 fsid：解析只核头校验和与载荷校验和（`parse_index_node` 等不比 fsid），所以一个别的池写的、实例代号更大的单元也算「更新的单元」。要它落在 R 的两个落点上，R 那两份已经被盖掉，R 一样走不完；放过不丢可读的东西。
- 不判单元与位置条目的校验和（比的只是实例代号）：同上，两份都是别的单元时 R 已走不完。
- 实现在这一判之前不重读（`crates/singlefs-core/src/mount.rs:2090`「None => continue,」那一支直接跳过这条根）：一次读故障让一份读不出时它交 `false`、走「记录不齐」那一支（重读一次），只往拒那边偏。

## 二、K3：`parse_slot` 判节点大小（推的，读码）

- 合法写路径只写格式常量：写者取 `SystemMutableConfiguration::node_bytes`（`crates/singlefs-core/src/system_configuration.rs:275`「u32::try_from(NODE_BYTES).expect("节点大小")」），这个类型没有字段，mkfs 不收参数。写到一半的槽整槽校验和不过，先拒成 `NotSelfDescribing`。所以一个合法写路径写出的槽不会在 `crates/singlefs-core/src/system_configuration.rs:579`「if u64::from(recorded_node_bytes) != NODE_BYTES {」上被拒。
- 处置：恢复把它与 `NotSelfDescribing` 并成一路（`crates/singlefs-core/src/recovery.rs:963`「| SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant { .. },」），换一槽、换一盘照择；别的读 `parse_slot` 的产品处（逐盘逐槽取最大那几处）都只收 `Ok`，同一个效果。池级 checker 在同一个次序上判（`crates/singlefs-checker/src/lib.rs:286`「if u64::from(read_u32(slot, SYSTEM_CONFIGURATION_NODE_BYTES_OFFSET)) != NODE_BYTES {」，在几何字段之前）。
- 试过、算不上打中的一形：节点大小坏、同时 S 越界的槽。改前它走到 `crates/singlefs-core/src/system_configuration.rs:589`「.map_err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange)?;」整池拒；改后先拒在节点大小上、换一槽。这与「当自证不过」同一个处置（整槽校验和不过而 S 越界的槽本来就是换槽），又只有不经写者、校验和自洽的坏镜像才有。要不要让「S 越界」压过「节点大小」交主 agent 定，我判不算判错。
- 推翻条件：找到一个产品写路径写出节点大小不是 16384 的槽。

## 三、K4：`pointer.rs` 写者断言（推的，读码）

**合法写路径**：产品里位置条目都由盘表两块盘的身份排序后各取一份（mkfs 的 `crates/singlefs-core/src/make_filesystem.rs:273`「let mut sorted: Vec<DeviceIdentity> = devices.to_vec();」，发布路径的 `location_entries` 同一个做法）。两块盘身份相同走不到：mkfs 断言两块盘（`crates/singlefs-core/src/make_filesystem.rs:478`「"第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"」），两盘时区域归属写死 0/1/0（`crates/singlefs-core/src/make_filesystem.rs:462`「if devices.len() == 2 && parameters.region_devices != FIRST_VERSION_REGION_DEVICES {」），身份只能是 0 与 1；挂载时交两块同身份的盘，区域 1 那块盘读不到，在写之前拒（推的，没造）。`transaction.rs` 里给映射 key 用的那条「位置条目全零、头不全零」的指针（`mapping_key_of_a_node_born_in_this_publish`）只进 `mapping_key_for_node`，那里不调 `write_to`，碰不到断言。

**全零例外**：读者与写者都用 `NodePointer::location_entries_ascend_by_device`，豁免只认整条等于 `empty_root()`（`crates/singlefs-core/src/pointer.rs:316`「if *self == Self::empty_root() {」）；写者在 `crates/singlefs-core/src/pointer.rs:326`「self.location_entries_ascend_by_device().is_ok(),」。读码没找到产品路径写「位置条目全零而头、实例代号或出生序号不全零」的指针；没造。

**相邻一形（零轮，推的，没造盘面）**：重建出来的那一版里按 I-2.5 判的指针清单（`crates/singlefs-core/src/recovery.rs:5121`「fn pointers_of_the_rebuilt_version_ascend_by_device(」，到 `crates/singlefs-core/src/recovery.rs:5157`「.chain(output.central_mapping_tree.pointers.iter());」为止）不含墓碑树、谱系树、退役树的容器指针与根之下节点指针（它们由 `crates/singlefs-core/src/recovery.rs:3213`「tombstone_tree_index_nodes: tombstone_tree_read.as_ref().map_or_else(」那几行装进这一版）。下一次重写这三棵树某个节点时，照抄的孩子指针原样进 `build_inode_internal_entry`（`crates/singlefs-core/src/transaction.rs:7759`「*child_pointer,」），走 `NodePointer::write_to` 的断言。所以一份校验和自洽、某个墓碑容器指针两条位置条目同盘（非全零）的坏镜像：改前 `<=` 照写回去，改后在写者断言上 panic；两条逆序的那一形改前就 panic。这是 C476（盘上内容可控时走得到的 panic）那一类，不是合法写路径，按 K4 验收字面不算打中。改法（只在读码上推的、被攻过零轮）：把三棵树的 `pointers_below_the_root` 与容器指针并进那张清单。

打中之后四句（对这一相邻形）：不分辨臂（这一轮没有臂）；被判的系统读那一版时看得到那条指针的两条位置条目；满足的不是 K4 验收的任一分句，落在 C476 那一类；跑前条款第三行「反例落在条款没写的地方」那一句可能适用，交主 agent。

## 四、K5：两处前提断言（推的，读码）

- `allocation_records_are_one_per_device` 的产品调用只有两处，盘表都取读者自己的（`crates/singlefs-core/src/recovery.rs:2894`「let device_identities = reader.device_identities();」与 `crates/singlefs-core/src/recovery.rs:5400`「let device_identities = reader.device_identities();」）。读者零块盘时，读树表、映射、分配记录树都读不到，在那之前报错返回，走不到 `crates/singlefs-core/src/recovery.rs:4469`「"盘表不许为空：产品调用方交读者的盘表，零块盘的池在读实例表单元那一步已经报错返回"」那一断言。「有记录、盘表空」只有直接调这个公开函数的测试造得出来；产品里的 `PoolReader` 实现只有切片、`Vec` 与挂载的读阶段缓存三种，都按盘表读。
- `index_node_entry_capacity` 的调用方（core 里五份文件六处、harness 的模型与用例）交的都是格式常量；`build_index_node` 经它判容量时传的条目宽也是常量。条目宽 0 改前就是除零 panic，key 宽过大改前是减法下溢 panic（工作区开着 overflow-checks），行为只是 panic 消息变了。
- 推翻条件：找到一个产品调用方把盘上读来的宽交给 `index_node_entry_capacity`，或一个产品 `PoolReader` 盘表为空而读得出单元。

## 五、K6：开新容器 checked 加法与三棵树交 `Result`（推的，读码）

**发布那一层**：墓碑树与退役树的错在普通分配那一半交回，谱系树的错在 `resolve` 里交回（`crates/singlefs-core/src/transaction.rs:5304`「.map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?」、`crates/singlefs-core/src/transaction.rs:5340`「.map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?;」）。`prepare_the_version_publish` 先调普通分配那一半（`crates/singlefs-core/src/transaction.rs:6605`「.ordinary_allocation_part(previous, &trees, deletion_and_lineage)?」），再进固定点；固定点里 `resolve` 跑在分配器的拷贝上。两处都在动分配器、读盘核与任何写之前，这一次发布盘上逐字节不变。三处穷举 `match`：`refusal_is_short_of_space` 把它归进不是空间不够的那一组（`crates/singlefs-core/src/mounted_session.rs:934`「| PublishError::NextContainerNumberPastTheTopOfItsRange(_)」，会话不为它推抬 F，对：推抬 F 不让号变小）；harness 的 `model_comparison.rs` 与 `history.rs` 两处只是归类与打名字。

**挂载那一层（交主 agent，推的，没造盘面）**：挂载续做走删除队列时，一次删除类发布被拒而不是空间不够就整个挂载报错（`crates/singlefs-core/src/mount.rs:6616`「if !refusal_is_short_of_space(&refusal) {」接 `crates/singlefs-core/src/mount.rs:6624`「return Err(MountError::Publish(Box::new(PublishSequenceFailed {」），而续做排在写行那次发布之后（注释 `crates/singlefs-core/src/mount.rs:6463`「写行、暖机、抬 F、删除续做之后，」）。删除批会给退役树追加记录（删空摘掉的叶容器身份），要开新退役容器而最右那片号是 `u64::MAX` 时，这一错在挂载里交回，那时写行那次发布已经落盘：这一次挂载拒、盘变了（取了号、写了一次行），下一次可写挂载再取号、再写一次行、再拒。这不违背 C841「开新容器那一次发布报这个错、读盘不变」的字面（变的是前面那次写行发布），改前是同一个位置上 panic、写行同样已落，所以不是这一批引入的回归；只有校验和自洽的坏镜像走得到。txg 到顶那一错（`NextCheckpointTxgPastTheTopOfItsRange`）在续做里也是这一个处置。

打中之后四句（对这一形）：不分辨臂；系统在续做那一刻看得到最右那片的号；满足的是 K6 验收「错误交回之前已经发了写（盘变了）的路径」那一分句的挂载层读法，不是发布层读法；跑前条款第三行（条款对挂载层没有一句话）可能适用，交主 agent。改法（推的、零轮）：挂载在取号之前预演续做第一步的计划（只调 `ordinary_allocation_part` / `resolve`，不写），这一错在取号之前拒；或续做里遇到它照「推满仍放不下」那样停下、挂载照样做成可写、队列留着。

## 六、K7、K8（余力，推的，grep + 读码）

- K7 删掉的公开项（`PublishShape` 与它的三个常量与 `rewritten_roles`、`ResolvedPublish::shape`、`ExtentTreePlan::roles`、`ContainerTreeShape::rightmost_path`、`tombstone_containers_one_root_node_holds`、`every_unlink_step_from` 挪进测试、`previous_index_of_a_carried_container`、`ExtentLeafRecordInMountState::inode_number` 与 `data_unit_pointer`、`OpenFileForRead::data_unit_records`）：在 `crates/`、`scripts/`、`research/` 下 `grep -rln --include=*.rs` 逐个搜，core 之外的命中只有 `PublishShape`（`e165_replay_bounded_enumeration.rs` 自己定义的同名结构）、`ROW_PUBLISH` / `EMPTY_PUBLISH`（各实验与 GPU 用例里自己定义的 `DEVICES_OF_THE_ROW_PUBLISH_FLOW`、`EMPTY_PUBLISH_ROLES` 这类局部常量）、`rightmost_path`（一份已归档原型的探针，按冻结快照编），没有一处调到被删的 core 项。`scripts/experiments/Cargo.toml` 不依赖 `singlefs-core`。
- K7 改过的注释抽查：`transaction.rs` 文件头「取号 = 屏障 → 逐盘一次系统配置槽写 → 屏障」与 `write_acquired_instance` 的次序对得上；`AcquisitionRollback::RolledBack` 那句「回卷写之后没有屏障」与 `roll_back_acquisition` 对得上（回卷写之前一道屏障、之后没有）；`admission.rs` 新注释点名的 `mount::publish_empty_after`、`mount::establish_instance`、`transaction::prepare_the_row_publish_on_a_version_without_file` 三个函数都在。没逐条核几十处注释，见「这条腿自己的限度」。
- K8 `leaves`：`level_of_the_node_read_last` 在上段那一趟（`find_upper_leaf_entry`）交回之后、读下段之前取，下段的读不会改到它；上段根就是层级 0 的叶时计 1。core 里 inode 树 key 宽的字面量 8 已全换成 `INODE_KEY_WIDTH_IN_BYTES`（`recovery.rs`、`mounted_read.rs`、`transaction.rs`、`inode_tree.rs` 四处引它）；池级 checker 与 harness 模型各留自己的常量，是有意的独立实现，不算漏。

## 七、没打中的形状与取样范围

- K1 试过的形：第一节那张表的四段历史（覆盖写 k=5、三条记录 k=5、`ooww`、`oiww` 两个前缀），每段 M1 写流的崩溃状态全量（含写行根落盘之后 R 被 M1 的实例表抛弃的那些，那时走不到这一判）；读故障组：k=5 17 组（`tt0`、`tt1`、`mr0`、`mr1` 的全部子集加 `rec`，与 `none`），另三段 6 组（`none`、`rec`、`mr0+mr1+rec`、`tt1+mr1+rec`、`tt1+mr0+mr1+rec`、`tt0+tt1+mr0+mr1+rec`）；交盘两序；换几何 232 种盘面、3 组读故障。几何都是 384 槽小盘、两块盘；M1 一律是 `urJ` 那一形。
- 没造的（推的）：多层中央映射树（第三轮攻方第一节第 5 小节的理由：小盘装不下 295 个进映射的单元）；撕裂写落在指针槽上（崩溃点重放的枚举域单元写不取撕裂态；撕裂的槽解不出，只会让这一判多拒）；R 两份落在同一块盘上（K4 那一节：产品写不出）；M1 之外再多几次没写成行根的挂载（每次取的号都更大，落在 R 槽上的都是更新的单元，与一次同形，推的）。
- K3：试过节点大小坏 + S 越界那一形（读码）；没造盘面。
- K4：试过「同身份两块盘」「位置条目全零而头不全零」两形（读码），产品写不出；相邻的容器树那一形没造盘面。
- K5：试过零块盘的读者、盘上读来的宽进 `index_node_entry_capacity` 两形（读码）。
- K6：试过发布层（读码，错误早于一切写）与挂载续做那一层（读码，第五节）；没造最右容器号是 `u64::MAX` 的盘面。

### 取样、估时与分批

- 起跑前 `ps -o pid,args -u "$(id -u)"`：别的会话有 `cargo test -p singlefs-checker-tier`（变异复跑的 baseline、GPU 那一份）在跑，没有性能测量（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）；加 `nice -n 19` 照跑，没等锁。负载在 35–290 之间变。
- 小段估时：k=5 序号 1030–1040 × 17 组 × 两序（340 次被判挂载）3.98 秒，含录 R 与 M1 约 2.4 秒。按它推 k=5 全量单线程约 55 分钟，31 件并行约 2 分钟；实际负载下批 0（k=5 序号 0–10257，31 件）580 秒、批 1（其余 10258 个，31 件）369 秒。
- 按批 0 的实测，另三段若也跑 17 组要再约 50 分钟，超过 40 分钟那一线：**缩了读故障组**，另三段从 17 组缩到 6 组（留下的是第三轮在这三段上走到那一判的几组与 Y2 六段、`none`）；状态一个没缩。缩后三段并成一批 31 件（批 10），实际 766 秒。缩前三段 17 组 × 两序约 237 万次被判挂载，缩后 837324 次。
- 每批不设限时，`.rc` 文件数与件数逐批对上、退出码全 0（`batch=0 nonzero=0`、`batch=1 nonzero=0`、`batch=10 nonzero=0`；换几何 7 + 1 小批 `nonzero=0`）。三批超过 240 秒被工具挪到后台，用 `proc.py wait <pid>` 等，没写轮询。
- 状态数：每段跑前 `count` 模式算（20516、20521、24623、24633），合计 90293，加换几何 232 个盘面。

## 八、这条腿自己的限度

- K1 的数全是副本上的：被判的产品代码原样，副本只多一层记录；原型是攻方写的、骨架照第三轮攻方原型。要引进判决得在入库装置上重做（`.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做`）。
- 「R 走得完」按状态上池级 checker 不红 I-7.2 判，与第三轮判决用的是同一把尺。
- `rerun.sh` 没从头跑一遍；`out/` 两份是同一批输出汇的，排序后与 `rerun.sh` 末尾比的那两份同一个生成法。
- 用户动作没放开扫：被判那一判发生在 M2 写行根落盘之前，在任何用户动作之前；M1、M2 都是挂载自己。
- K3–K8 全是读码，没跑代码；K7 只抽查了几处改注释，没逐条核几十处。
- 只抽了一次样：K1 是「没打中」，按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 要再抽一次同立场腿才能记「没打中」。

## 九、没做什么

- 不判正推那几格（K2 的用例与变异行没看）；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑任何门禁阶段、checker 档测试、崩溃与变异的轻型两条；没起 GPU。
- 没造 K4、K6 相邻那两形的盘面（第三、五节，推的）。

## 要主 agent 跑的命令

无必须的。要复核 K1 的数：上面「复跑命令与模型文件」那一条，工作目录随意，环境不用加前缀（`rerun.sh` 自己给编译那一步带 `BINDGEN_EXTRA_CLANG_ARGS`），内存上限 24G，线程 31（`RERUN_THREADS` 可改），末行看 `name=rerun_compare all_same`；挂钟按本轮约 30–45 分钟，超过 2 小时线的可能小，若要拆，按 `tools/pieces.txt` 的批号（0、1、10）逐批跑 `tools/run-batch.sh`。

## 十、草稿与产物去向

草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-b44-code-r1-opus/`：`tree/`（冻结副本的拷贝，装了记录层）、`model/`（原型）、`target/`（编译目录）、`runs/`（各件原始输出与汇总）、`stage/`。模型与汇总已拷进 `research/prompts/m3-b44-code-r1-opus-model/`；原始逐件输出（`runs/enum/b*.out`）没入库：体积大，汇总脚本从它们算出的两份已入库，复跑能重出。交回前删 `tree/`、`target/`、`stage/`，`runs/` 留给主 agent 核到交回之后。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-b44-code-r1-opus-output.md` 末行（core 下被引的文件与冻结副本 sha256 相同，`sha256sum -c research/prompts/m3-b44-code-r1-snapshot/crates-sha256.txt` 在主工作区只有 checker 档的 6 份对不上）：

```
  ✓ 核了 27 处引文、0 处裸行号，对上 27 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```
