# m3-b44 代码轮攻方腿第二次抽样（Opus）报告（2026-10-16）

攻击面：正文「三、格」K1、K3、K4、K5、K6，余力做了 K7 的一小部分。冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r1/repo/`（`sha256sum -c research/prompts/m3-b44-code-r1-snapshot/crates-sha256.txt` 在副本里跑，退 0）。下面的数全是**副本上的数**：冻结副本原样拷进本腿草稿目录（不打补丁），加本腿原型；不是入库装置上的数。引的代码行号按冻结副本，被引的那几份 `crates/` 文件与主工作区此刻逐字节相同（`cmp` 量过）。没读禁读清单里的任何一份。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-b44-code-r1-opus-s2-model/`。复跑（整条经内存包装；`BINDGEN_EXTRA_CLANG_ARGS` 由模型目录 `.cargo/config.toml` 的 `[env]` 给，与冻结副本那一份逐字相同）：

```
bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/m3-b44-code-r1-opus-s2-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-b44-code-r1/repo \
  --work <新的草稿目录>
```

末行判结果：`name=rerun_compare all_same`（`out/survey.txt`、`out/agg.txt` 排序后逐行相同）。挂钟（本轮实测，本机同时有别的会话在编 `cargo build --all-targets`、跑 checker 档）：编译 1 分 04 秒，换几何取样 12 秒，k=5 全量两批 7 分 23 秒、7 分 14 秒，`oiww` 全量一批 8 分 23 秒（都按件输出文件的建立时刻与这一批最后一个 `.rc` 的时刻相减）。这条腿自己没有把 `rerun.sh` 从头跑一遍（见「这条腿自己的限度」）。

| 文件 | sha256 |
|---|---|
| `.cargo/config.toml` | 891b14e98b472f513f13dea65ff5b2a7cf1429b97a29b2c488f06e90ee095fc5 |
| `Cargo.lock` | a9082012f2579acbb52332ebef23be7088eae22f5063320a494c45462eb420a7 |
| `Cargo.toml` | f4680a654dde32edd3ebd9a7ceedd8a30f392012eba4ae0491a73b30a0bb44ef |
| `out/agg.txt` | 8d55d92b49361bf81654b35b73e24808c7d556aca86d9c03fec793e3b54c3478 |
| `out/survey.txt` | 689280734b2a39fee2836d7a3fe67e0842981623090c2154cc5dc17b393384d9 |
| `src/main.rs` | 1c54809a0c88ac5e874733ce18e661fc7d48dae9b6e6b0303acbb98a65b5b9f1 |
| `src/prefix.rs` | f3539334eb345a29a2baaa30c652d6d31cdb0c872dca58f38bedbd44578d10c3 |
| `tools/aggregate.py` | b4b173923d4ac2e1f1b126803fa60f8545b5583f6a68fd8753e63bc1d8a1f358 |
| `tools/faultsets.txt` | 65a85720a02875a40fc05ebdcfbbe7205b0e3ba74acf2330f2484d0561584bfb |
| `tools/rerun.sh` | 62f0e64eae5c5b2c3e9f53bbf467d0b9929e3be71900810feed1840191f055d5 |
| `tools/run-batch.sh` | ad68102d953d2f2ee670b433ac45e6cc1fe03eab303c3a1df485e491ec321f0d |
| `tools/summary.py` | 6e9b14dfafc1e699e1a879dc16ffcfccd20a08d96c293d0cc1a0b6b69d42c9f0 |
| `tools/survey-batch.sh` | 2271fce67162f500437e04302f363211b75ebf0579fff798487b4b0b63b31506 |
| `tools/survey-histories.txt` | 83c59ba4c8adcdad5bd3b6d9983bec39ebb3e55fb8ac181780286b9f229762dc |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps` 那一段（第 1 到 73 行与「# 自证：模板拷进」起的尾段与模板逐字相同，`diff` 量过）。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 |
|---|---|---|
| K1 R 走得完而放过并写掉 | 没打中：232 段几何 × 六组读故障 × 两序（2784 次挂载）、入库 `urJ` 那一段 M1 写流 20516 个崩溃状态 × 七组读故障 × 两序、三条记录前缀 `oiww` 那一段 24633 个崩溃状态 × 三组读故障 × 两序，判错 0 | 量过（副本） |
| K1 入库 `urJ` 那一形它拒 | 没打中：k=5 那一形六组读故障、两序都放过、落到 P；没有读故障时 232 段里这一判一次都没拒 | 量过（副本） |
| K1 它拒的地方 | 全在「R 已有单元两份没了 + 读故障藏住证据」上，三类几何（第一节第 2 小节）；k=5 全量里 `tt1+mr0+mr1+rec` 下拒、R 已没 11964 个，与 c834 第三轮判决算的代价数相同 | 量过（副本） |
| K1 实现与定案逐句 | 对得上（第一节第 1 小节）；单元头不核 fsid、两个指针都空时拒，定案没写这两格，都推不出错放 | 推的（读码） |
| K3 误拒合法槽 / 照择坏槽 | 没打中；节点大小那一判排在 S 区间判之前，两样都坏的造出来的槽改前整池拒、改后换槽，条款没写 | 推的（读码） |
| K4 合法写路径写出同盘指针 / 全零例外认错 | 没打中 | 推的（读码） |
| K4 相邻：坏镜像走到新断言 | 打中一形（推的）：墓碑、谱系、退役三棵容器树的指针不在读者的 I-2.5 那一道里，照抄写回时走到写者断言；同盘那一形改前照写、改后 panic | 推的（读码，没造镜像） |
| K5 产品调用方走到新断言 | 没打中 | 推的（读码） |
| K6 那一次发布在写之前交回、三处 `match` | 没打中 | 推的（读码） |
| K6 相邻：整次挂载在写之后才报这个错 | 打中一形（推的）：挂载续做里的删除步报 C841 的错时取号、写行、暖机已落盘，每次重挂再烧一个号；条款只管「那一次发布」 | 推的（读码，没造镜像） |
| K7 | 只查了 C842 与几个删掉的名字零引用；改过的注释没核 | 量过（grep） |
| K8 | 没做 | — |

## 一、K1：C834 新判法 `one_pointer_has_both_copies_overwritten_by_a_later_instance`

结论：两条验收都没打中。造不出「R 走得完而它放过并写掉」的状态，入库 `urJ` 那一形它放过；它拒的地方全落在「R 已有单元两份都没了、且有读故障」上，与定案字面一致，是定案时已经算进去的可用性代价。

### 1. 实现与定案逐句对（读码）

定案原文（`research/prompts/c834-r3-main-verification.md@四、用户定案（2026-10-15）`）：「用户在弹窗里选「同一指针两份都更新」」。

| 定案的一句 | 实现 | 对不对得上 |
|---|---|---|
| 树表与中央映射树根两个指针中有一个 | `[&root.tree_table, &root.mapping_root]` 取 `any` | 对 |
| 两条位置条目指着的两个槽都 | `pointer.locations.iter().all(…)` | 对 |
| 读出自证过的单元头 | 三类解析都核头校验和与整单元载荷 CRC；不核单元头里的 fsid（见下面第 3 小节） | 对（「自证」照实现员的读法，条款没说要不要比 fsid） |
| 实例代号比 R 大 | `crates/singlefs-core/src/mount.rs:2168`「.is_some_and(|instance| instance > root.instance)」，索引节点取头里的实例代号、数据单元与打包记录单元取写序里的 | 对 |
| 索引节点读 16 KiB，数据单元与打包记录单元读 32 KiB | 先 16 KiB 解索引节点（`crates/singlefs-core/src/mount.rs:2190`「.and_then(|bytes| crate::unit::parse_index_node(&bytes).ok())」），解不开再读 32 KiB 依次解两类 | 对 |
| 全零空指针 | 先滤掉（`crates/singlefs-core/src/mount.rs:2164`「.filter(|pointer| **pointer != crate::pointer::NodePointer::empty_root())」），两个都空时 `any` 交 `false`、拒 | 对（定案没写这一格；R 是较新可读根，映射根不会空） |
| 否则照「记录不齐」那一支办 | 凭据为假才进 `placements_of_an_unapplied_publish_whose_account_is_unreadable`（`crates/singlefs-core/src/mount.rs:2078`「if !one_pointer_has_both_copies_overwritten_by_a_later_instance(」），凭据为真 `continue` | 对 |
| 不读系统配置里的号 | 参数 `journal_instance_in_the_system_configuration` 与调用处那一句都删了 | 对 |

### 2. 量过：换几何取样（M1 写行根之前全落的那一个盘面）

装置：冻结副本原样（不打补丁）拷进草稿目录，原型 `src/main.rs` 经 `#[path]` 引入库搭建 `common_unwitnessed_publish` 录 R、做 M1（入库 `urJ` 那一形：R 的根槽、R 写的全部单元、R 的全部记录读回全零，M1 取号 2、写行）。232 段历史（R 是覆盖写或三条记录的顺序写，前缀是「覆盖写 k 次」k = 0–10、12、14 或 `o`/`i`/`w` 动作串，同 c834 第三轮攻方的取样表），每段在 M1 写行根之前全落的那一个盘面上挂 M2：六组读故障（`none`、`mr0+mr1`、`tt0+tt1`、`rec`、`mr0+mr1+rec`、`tt0+tt1+rec`）× 两序 = 2784 次挂载（`grep -c '^name=k1_landed'` 量过）。

结果（`out/survey.txt`）：

- 判错 0：2784 次里做成的，写行根之前新写掉 R 单元的份数全是 0（`new=0`）。
- 入库 `urJ` 那一形（覆盖写、k=5）六组读故障、两序都放过、落到 P，原样一行：

```
name=k1_landed history=urj_overwrite_k5 faults=none order=forward clause_credential=true outcome=mounted:eff=(1,8):inst=3:new=0:gone=7:red_before_row_root=I-2.1+I-4.8+I-7.2+I-7.4:red_after=I-7.4
```

- 没有读故障时这一判一次都没拒；没有读故障的拒全是三条记录那一族的「记录有缺口」那一支（`PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`，c834 第三轮攻方第一节第 4 小节那一类，不归 K1）。
- 走到这一判而拒的（`AccountOfAnUnappliedPublishWithIncompleteReadableRecords`）只有三类，都是 R 已有单元两份没了、再加一处读故障：

| 几何类（几段） | R 的四个指针槽 | 拒在哪组读故障 | 为什么 |
|---|---|---|---|
| 映射根被 M1 盖住（这一类 11 段，走到这一判的 5 段，如覆盖写、前缀 `oiiiw`；另 6 段是三条记录，先拒在记录有缺口那一支） | `tt=idx:1`、`mr=idx:2` | `mr0+mr1` | 盖住映射根的 M1 单元被读故障藏住，树表还是实例 1 的 |
| M1 盖了 R 别的单元、两个指针槽没盖（5 段，三条记录、如前缀 `iiiiw`） | 四槽都是 `idx:1` | `mr0+mr1+rec` | R 已没了 6 个单元，但证据只看两个指针，指针槽上没有更新的单元 |
| 映射根被 M1 打包记录单元的后半截盖住（3 段，如三条记录、前缀 `oiww`） | `tt=idx:2`、`mr=zero` | `tt0+tt1+rec` | 树表那一对被读故障藏住，映射根那一对读回全零 |

两条原样（survey 里各取一行）：

```
name=k1_landed history=urj_overwrite_poiiiw faults=mr0+mr1 order=forward clause_credential=false outcome=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0
name=k1_landed history=urj_seq3_poiww faults=tt0+tt1+rec order=forward clause_credential=false outcome=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0
```

这三类拒都在任何写之前（`writes=0`），R 在那一刻已走不完（`landed_red` 含 I-7.2）。定案的字面要「两个槽都读出」，读故障藏住的那一对按字面不算，拒是对的。读故障消失之后：第一类同一个盘面放过（`none` 那两行做成）；后两类是三条记录，`none` 下先拒在记录有缺口那一支，只藏记录（`rec`）时做成（映射现算得出，或树表那一对放过）。中间那一类（指针槽没被盖、R 却已没了）条款对「R 已没而证据只看指针」没有另一句，只在有读故障时出现，记作可用性代价，不算打中。

### 3. 量过：入库 `urJ` 那一段（覆盖写、k=5）M1 写流全量崩溃状态

M1 写流 38 次写、9 段，`crash_state_count_with_torn_in_place_overwrites` 算出 20516 个状态（原型 `count` 模式那一行：`name=k1_history history=urj_overwrite_k5 r=(1,9) p=(1,8) r_records=1 m1_instance=2 writes=38 segments=9 state_count=20516`）。每个状态照崩溃点重放的枚举域展开（`crash_state_enumeration_tables` + `extract_flow_facts`，每段 `full_expansion`），先跑池级 checker，再挂 M2：七组读故障（`none`、`rec`、`tt1+mr1+rec`、`tt1+mr0+mr1+rec`、`tt0+tt1+mr1+rec`、`mr0+mr1+rec`、`tt0+tt1+rec`）× 两序；M2 做成的在写行根之前那一刻与全部写落盘之后各跑一次 checker。合 20516 × 14 = 287224 次挂载，每组每序 20516 个状态序号 0–20515 一个不缺（`out/agg.txt` 的 `name=k1_coverage` 行）。

判错（R 的根在根环里、R 每个单元至少剩一份，M2 做成而写行根之前新写掉 R 的单元）：14 组 × 序全是 0（`tools/summary.py out/agg.txt` 的 `wrong=` 列）。按结局分（正向；反向逐格同数）。「拒在这一判」指拒成 `AccountOfAnUnappliedPublishWithIncompleteReadableRecords`：这几组读故障都含 `rec`，R 一条记录都读不出，走的就是 K1 那一判；「凭据」是原型照定案另写的一份（`clause_credential`），做成而凭据为真的不一定走到了这一判（映射现算得出时走不到）：

| 读故障 | 做成、凭据真 | 做成、凭据假 | 拒在这一判、R 已没 | 拒在这一判、R 走得完 | 拒在相邻一支（下文） |
|---|---|---|---|---|---|
| `none` | 11292 | 9224 | 0 | 0 | 0 |
| `rec` | 7192 | 9224 | 0 | 0 | 4100 |
| `tt1+mr1+rec` | 0 | 9944 | 6148 | 324 | 4100 |
| `tt1+mr0+mr1+rec` | 0 | 3480 | 11964 | 972 | 4100 |
| `tt0+tt1+mr1+rec` | 0 | 8216 | 7228 | 972 | 4100 |
| `mr0+mr1+rec` | 4120 | 5192 | 7104 | 0 | 4100 |
| `tt0+tt1+rec` | 4120 | 12296 | 0 | 0 | 4100 |

每行合 20516（python 按 `out/agg.txt` 的 `name=k1_cell` 行加的）。

「拒、R 走得完」那两格就是 c834 第三轮打中 `pointer_per_kind` 的两形，例子序号也对得上（第三轮攻方序号 1032、1288）：

```
name=k1_cell history=urj_overwrite_k5 faults=tt1+mr0+mr1+rec order=forward clause_credential=false r_units_gone=0 outcome=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords wrong=false refused_with_r_gone=false states=972 example_ordinal=1032
name=k1_cell history=urj_overwrite_k5 faults=tt1+mr1+rec order=forward clause_credential=false r_units_gone=0 outcome=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords wrong=false refused_with_r_gone=false states=324 example_ordinal=1288
```

`tt1+mr0+mr1+rec` 下「拒、R 已没」11964 个，与 c834 第三轮判决写的这一判的代价数相同（判决「三、处置」第一条「11964 个状态拒」）。所以在入库那一段上，实现给出的放过与拒逐格落在定案时算过的那几格里。

另有一种拒不归 K1：读故障组含 `rec` 时 M1 写行那次发布已经把自己的记录写进 R 的记录槽，`rec` 按字节区间藏住的是 M1 的记录；M1 写行根与轮换落了之后（序号 16402 起）择到的系统配置见证了 M1 那次发布而它的记录读不出，拒成 `PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion`（4100 个，各组相同）。读故障组是攻方点的、藏的是 R 那几段字节，这一拒是相邻那一支照它自己的条款办，记在这里不算打中。

三条记录、前缀 `oiww` 那一段（映射根被 M1 打包记录单元的后半截盖住、树表被 M1 的索引节点盖住那一类）：M1 写流 55 次写、13 段，24633 个崩溃状态（`name=k1_history history=urj_seq3_poiww r=(1,8) p=(1,7) r_records=3 m1_instance=2 writes=55 segments=13 state_count=24633`），三组读故障 × 两序全量，判错 0。正向（反向逐格同数）：

| 读故障 | 做成、凭据真 | 做成、凭据假 | 拒在这一判、R 已没 | 拒在这一判、R 走得完 | 拒在相邻一支 |
|---|---|---|---|---|---|
| `none` | 12333 | 12296 | 0 | 0 | 4（记录有缺口那一支） |
| `rec` | 4108 | 12296 | 0 | 0 | 8229（见证那一支） |
| `tt0+tt1+rec` | 0 | 12300 | 4104 | 0 | 8229（见证那一支） |

这一段上这一判只在 `tt0+tt1+rec`（藏住盖着树表的 M1 索引节点）下拒，拒的 4104 个 R 都已没了。

### 4. 为什么造不出「放过而 R 走得完」（推的，读码）

放过要某个指针两个槽都解出实例代号比 R 大的单元头。读故障只能把字节读成全零、读不出更新的单元头；所以那两个槽上真有别的实例写的、解得开的单元，R 的那个单元两份都已被盖掉（两份同槽，盘 0、盘 1 各一份），R 那一刻就走不完。剩下三条路逐条看：

- 实例代号倒退（更新的单元的号反而比 R 小，或 R 引用了比自己号大的单元）：取号取「全部自证过的系统配置槽与根环里全部根」的最大值加一（`crates/singlefs-core/src/transaction.rs:890`「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」），有槽读错而没有一块盘两槽都自证过就拒（R2）。要让号倒退得两块盘上带最大号的槽都读不出而不报读错（撕裂、位翻转），而带最大号那次取号之后已经过了屏障、两盘都持久了；本轮没造出来。
- 单元头不核 fsid：同一块盘上更早一次 mkfs 留下的单元，号可以比 R 大；但 R 的单元先于 R 的根落盘，那一槽在 R 之后只会被这个池后来的写盖掉，旧池的单元回不到这一槽。
- 撕裂写：崩溃枚举域里单元写不取撕裂态（c834 第三轮第一节第 6 小节已记）；撕裂的槽三类都解不开，只会把凭据推向假、推向拒。

推翻条件：造出一段历史，被判的那次挂载里某个指针两个槽都解出比 R 大的实例代号，而池级 checker 在那一刻不红 I-7.2、R 在根环里。

### 5. 没打中的形状与取样范围（K1）

- 232 段几何 × 六组读故障 × 两序，M1 写行根之前全落的那一个盘面：2784 次挂载。
- 全量两段：入库那一段 20516 个崩溃状态 × 七组读故障 × 两序；三条记录前缀 `oiww` 那一段 24633 个崩溃状态 × 三组读故障 × 两序。
- 读故障只按 R 的每一份分开点（`tt0`/`tt1`/`mr0`/`mr1`/`rec`）；没点 R 的别的单元、没点 M1 写的别处；读故障在 M2 里不随重读消失（只在被写过的那一段解除）。
- 只有一次 M1；没造两次互相看不见的挂载接连盖 R 的两份（推的：两份都被盖时 R 那个单元没了，与第 4 小节同一条）。
- 没造盘上残留旧池单元、没造实例代号倒退（推的，见第 4 小节）。

## 二、K3：`parse_slot` 判节点大小（C840）

没打中。试过的形全是读码（推的），没造镜像：

- 合法写路径写出、而它误拒的槽：写者在偏移 301 写节点大小之前断言位置（`crates/singlefs-core/src/system_configuration.rs:478`「writer.assert_position(NODE_BYTES_OFFSET, "节点大小");」），写的是 `self.mutable.node_bytes()`，读者判的是同一个常量（`crates/singlefs-core/src/system_configuration.rs:579`「if u64::from(recorded_node_bytes) != NODE_BYTES {」）。整槽校验和罩着这 4 字节，撕裂的槽先拒在校验和那一关。找不到写别的值的写者。
- 该拒而照择：读系统配置槽的三处都只收 `Ok`。择系统配置把新成员并进自证不过那一路（`crates/singlefs-core/src/recovery.rs:963`「| SystemConfigurationSlotRefusal::NodeBytesNotTheFormatConstant { .. },」）；取号的 max 与见证读走同一个逐盘读法，`Err` 一律丢掉（`crates/singlefs-core/src/recovery.rs:3458`「if let Ok(system_configuration) = SystemConfiguration::parse_slot(&bytes) {」）；池级 checker 自己判一份同一个值（`crates/singlefs-checker/src/lib.rs:286`「if u64::from(read_u32(slot, SYSTEM_CONFIGURATION_NODE_BYTES_OFFSET)) != NODE_BYTES {」）。三处同判，找不到一处择得到坏槽。
- 次序上的一处变化（只在造出来的坏槽上）：节点大小那一判排在每区槽数 S 的区间判之前（S 的切法在 `crates/singlefs-core/src/system_configuration.rs:586`「let root_ring_slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(」）。一槽节点大小与 S 都坏时，改前整池拒（`RootRingSlotsPerRegionOutOfRange`），改后这一槽当自证不过、换一槽再择。合法写路径写不出这样的槽；条款（C840「当自证不过」）对「两样都坏」没有一句话，记在这里，不算打中。

## 三、K4：`pointer.rs` 写者断言（I-2.5 严格升序、全零豁免）

K4 验收的两形都没打中；相邻一形打中（推的，读码，没造镜像）。

没打中的两形：

- 合法写路径写出同盘两条位置条目：写者的位置条目一律由 `location_entries` 给，它把盘表身份排序后取前两个（`crates/singlefs-core/src/make_filesystem.rs:274`「sorted.sort();」、`crates/singlefs-core/src/make_filesystem.rs:282`「device: sorted[1],」）。mkfs 断言两块盘（`crates/singlefs-core/src/make_filesystem.rs:478`「"第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"」），区域归属写死 0 / 1 / 0，盘表里必有 0 与 1 两个身份；挂载拒重复身份（`DeviceIdentitiesHandedInMoreThanOnce`）。合法盘表上排序后前两个一定不同。
- 全零例外认错：豁免只认整条等于 `empty_root()`（`crates/singlefs-core/src/pointer.rs:316`「if *self == Self::empty_root() {」）。非空指针要整条全零，得两条位置条目都是盘 0，上一条说了合法写者写不出；全零以外带盘 0 / 盘 0 的一律断言，没有放宽。

相邻一形（读者那一道没罩到、坏镜像走到新断言）：

- 读进来的指针在发布前按 I-2.5 判的那一处列了根记录、树表条目、extent 树、inode 叶容器、分配记录树、记账树、中央映射树与数据指针（`crates/singlefs-core/src/recovery.rs:5121`「fn pointers_of_the_rebuilt_version_ascend_by_device(」到 `crates/singlefs-core/src/recovery.rs:5157`「.chain(output.central_mapping_tree.pointers.iter());」），没有墓碑树、谱系树、退役树的容器指针与它们的码 2 节点指针（`TransactionOutput` 里另有这几项，`crates/singlefs-core/src/transaction.rs:2884`「pub tombstone_containers: Vec<TombstoneContainerVersion>,」）。
- 下一次往这三棵树开新容器的发布照抄没重写的那几片的指针（`crates/singlefs-core/src/transaction.rs:7692`「pointer: (to_build.carried_container_pointer)(position),」），装进父节点条目时经 `NodePointer::write_to`（`crates/singlefs-core/src/records.rs:271`「child.write_to(&mut writer);」），走到新断言（`crates/singlefs-core/src/pointer.rs:326`「self.location_entries_ascend_by_device().is_ok(),」）。
- 历史：墓碑树有一片装满的容器，坏镜像把根条目里那片容器的子指针第二条位置条目的设备号改成 0（两条都是盘 0、同槽），重封根节点、校验和沿引用链补到根槽（入库用例 `crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs` 的改法，只换改的字段）；可写挂载照常做成（容器按提示读盘 0 那一份，读得出）；下一次 unlink 意图要开新容器。改前断言是 `<=`，0 ≤ 0 照写不误、把坏指针写回盘上；改后在断言上 panic。设备号逆序（1, 0）那一形改前就 panic，这一处缺口改前就在，这次只把「同盘」并了进去。
- 四句：不涉及臂（这一格只有一个改法）；被判的写者在断言那一刻看得到判别它的东西（两条位置条目），漏的是读者那一道；满足的是 K4 条款字面以外的形（坏镜像，不是合法写路径），按跑前条款第三行「反例落在条款没写的地方」交用户；改法是把三棵容器树的容器指针与节点指针并进 `pointers_of_the_rebuilt_version_ascend_by_device`，只在读码上推的、被攻过零轮，没实现、没跑。
- 推翻条件：照上面的改法造出那份坏镜像，可写挂载之前就拒成 I-2.5（读者在别处已经判了），或 unlink 意图那次发布不经 `NodePointer::write_to` 写照抄的容器指针。

## 四、K5：外部审核 ①② 两处前提断言

没打中（推的，读码）：

- `allocation_records_are_one_per_device` 的产品调用方只有一个包装函数，两处都传读者的盘表（`crates/singlefs-core/src/recovery.rs:2894`「let device_identities = reader.device_identities();」、`crates/singlefs-core/src/recovery.rs:5400`「let device_identities = reader.device_identities();」），之前已经经读者读完分配记录树。零块盘的读者在择系统配置那一步就交错，可写挂载另有可写设备数下限。「有记录、盘表空」那一处行为变化（改前交 `false`、改后 panic）在产品路径上走不到：盘表空时连分配记录树都读不出来。
- `index_node_entry_capacity` 的调用方全部交格式常量（inode 树、容器树、`deleted_inodes`、多层码 2 树按树种取的宽、`build_index_node` 自己的断言），盘上读来的 key 宽不进这里。断言位置见 `crates/singlefs-core/src/unit.rs:159`「"条目宽是 0：index_node_entry_capacity 只收格式常量里的条目宽"」。
- 试过的形：逐个 grep 两个函数的调用方（`grep -rn 'allocation_records_are_one_per_device\|index_node_entry_capacity(' crates`），没跑镜像。

## 五、K6：C841 开新容器 checked 加法与三棵树交 `Result`

条款字面那一格没打中；相邻一形打中（推的，读码，没造镜像）。

没打中的：

- 一次发布之内，错在任何写之前交回：墓碑树、退役树的追加在 `ordinary_allocation_part` 里，它排在空间准入与动分配器之前（`crates/singlefs-core/src/transaction.rs:6605`「.ordinary_allocation_part(previous, &trees, deletion_and_lineage)?」，退役树那一处 `crates/singlefs-core/src/transaction.rs:5340`「.map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?;」）；谱系树在 `resolve` 里，同样在落盘之前那一段。可写挂载取号之前的预演只发写行与暖机，两种发布都是 `NEITHER_A_DELETION_NOR_A_ROLLBACK`，三棵树照抄，走不到开新容器。
- 三处穷举 `match`：会话判「是不是空间不够」交 `false`（`crates/singlefs-core/src/mounted_session.rs:934`「| PublishError::NextContainerNumberPastTheTopOfItsRange(_)」，不推抬 F，对：推抬 F 不会让号变小）；模型对拍归成没有理由（`crates/singlefs-harness/src/model_comparison.rs:725`「| PublishError::NextContainerNumberPastTheTopOfItsRange(_)」）；随机历史按名字记（`crates/singlefs-harness/src/history.rs:2906`「PublishError::NextContainerNumberPastTheTopOfItsRange(_) => {」）。三处处置都对得上「坏镜像才有」。
- 空间推抬 F 之后重试那一路：推之前的那一次已经跑过 `ordinary_allocation_part`，推的空发布照抄三棵树，重试时同一份容器、同一个结论，不会推完抬 F 才报这个错。

相邻一形（条款「那一次发布盘不变」之外，整次操作盘已变）：

- 挂载续做：可写挂载在取号、写行、暖机之后把 `deleted_inodes` 走空（`crates/singlefs-core/src/mount.rs:6453`「} = continue_the_deletion_queue_until_it_is_empty(」），走空里一步报了不是空间不够的错就整次挂载报 `MountError::Publish`（`crates/singlefs-core/src/mount.rs:6624`「return Err(MountError::Publish(Box::new(PublishSequenceFailed {」），那时取号写、写行与暖机的根都已落盘。删空一片 inode 叶容器的删除步要往退役树追加（推的：`retirement_records_of_this_publish` 收「删空摘掉、被左邻吸收的叶容器身份」），退役树最右那片已满而号是 `u64::MAX` 的坏镜像上，这一步报 C841 的错。下一次可写挂载照样取一个新号、写一行、走到同一步再报，每试一次烧一个实例代号、实例表多一行，可写挂载永远做不成。
- 会话里的用户改动同理：一次改动先把队列走空（每步落盘），后面那一步报这个错时前面几步已落盘。
- 改前这几处是加一溢出 panic，同样在那些写之后；所以这不是这一批改坏的，是 C841 的条款（`crates/singlefs-core/src/container_tree.rs:458`「开新容器那一次发布报这个错、不 panic，读盘不变）。容器号从 1 起逐片加一，健康的树走不到顶，」）只管「那一次发布」，整次挂载怎么处置没有一句话。按跑前条款第三行交用户：续做里报这个错时，挂载是照 `StillShortAfterTheFloorRaises` 那样做成可写、队列留着，还是整次报错（今天的样子）。
- 四句：不涉及臂；被判的那一步看得到判别它的东西（最右那片的号）；满足的是条款没写的那一格（整次挂载），不是字面「那一次发布盘不变」那一句；改法没提，交用户定。
- 推翻条件：挂载续做里的删除步（`Batch` / `FinalBatch`）在这个仓里从不追加退役记录，或续做报错之前有一道把整次挂载回卷的处置。

## 六、K7、K8（有余力，只做了一小部分）

- K7 只查了 C842：`txg_is_inside_the_one_revolution_window` 在冻结副本 `crates/` 与仓里 `crates/mutations.tsv`、`scripts/`、`.claude/gate.d`、`.claude/hooks` 里零命中（`grep -rn` 量过）；C842 那一行要连带改的 `scripts/experiments/e139_tightened_floor.rs` 那句注释已改（同一个 grep 在 `scripts/` 零命中，`git status` 里它是改过的）。别的删函数（`rightmost_path`、`PublishShape`、`ExtentTreePlan::roles`、挪进测试的 `every_unlink_step_from`）在 `crates/mutations.tsv`、`scripts/`、门禁表里零命中；`PublishShape` 只剩 `.claude/kb/milestone/02-second-txn.md` 第 374 行那条历史记录里的提法（说的是 2026-09-17 那一次，不改）。改过的注释一处没核。
- K8 没做。

## 七、没打中的形状（汇总）

- K1：见第一节第 5 小节；换几何 232 段（前缀长 1–9 的 `o`/`i`/`w` 串与 26 种「覆盖写 k 次」，R 是覆盖写或三条记录的顺序写），只看 M1 写行根之前全落那一个盘面；全量只跑了两段（入库那一段、`oiww` 那一段）。
- K3：读码逐个看了读系统配置槽的三处与写者；没造坏槽镜像跑挂载。
- K4：读码看了写者取位置条目的那一处、mkfs 与挂载对盘表的判、全零豁免的判法；相邻那一形没造镜像。
- K5：grep 全部调用方；没跑。
- K6：读码看了三棵树的追加点相对空间准入与落盘的次序、取号之前的预演、三处 `match`、挂载续做与会话走空；相邻那一形没造镜像。

## 八、这条腿自己的限度

- 「放过而 R 走得完」只在两段历史上做了全量，别的 230 段只看了一个盘面；读故障组只点 R 的每一份，没点 M1 写在别处的单元。第一节第 4 小节的论证是推的。
- 原型里的「凭据」是照定案另写的一份（`clause_credential`），没在 core 里加探针；判「走到这一判」靠拒因成员与「读故障组含 `rec`」推，不是读 core 的执行路径。
- K3–K6 全是读码，相邻那两形（K4、K6）没造镜像、没跑；改法都没实现。
- 这条腿没把 `tools/rerun.sh` 从头跑一遍：`out/` 里两份是本腿在草稿目录里跑出来的（命令与 `rerun_steps` 里的逐条相同，见下一小节），`rerun.sh` 只过了 `bash -n`。

### 取样、估时与分批

- 先跑一小段：k=5 序号 10000–10019 这 20 个状态 × 14 组读故障 × 两序，单进程 22.312 秒（`real 0m22.312s`），约 40 毫秒一次挂载（含 checker）。按它估：五段历史各 14 组全量是 5 × 20516 × 28 × 0.04 ÷ 31 ≈ 3700 秒，超过 40 分钟，所以缩：全量只跑 k=5（读故障从 14 组缩到 7 组）与 `oiww`（缩到 3 组），估 6 分钟加 4 分钟。实际并行时一次挂载慢到约 0.12 秒（本机同时有别的会话在编与跑），k=5 两批 7 分 23 秒、7 分 14 秒，`oiww` 一批 8 分 23 秒。
- 崩溃状态没有缩：留下的两段，每段的崩溃状态逐个展开、一个不落。状态数：k=5 20516、`oiww` 24633，合 45149 个，都在约 10⁶ 之下，全腿合计远在约 10⁷ 之下。
- 分批：k=5 两批，每批 31 件、每件 330 / 332 个状态；`oiww` 一批 31 件、每件 795 个状态；换几何一批 31 段、共 8 批。每件退出码写进带批号与件号的 `.rc`，每批开跑前删掉这一批的 `.rc`，一个不带参数的 `wait` 等齐后核件数再按次序读（`tools/run-batch.sh`、`tools/survey-batch.sh`）；全部件退 0。批不设限时。
- 本腿起的批有两次超过前台 4 分钟、被工具挪成后台任务，等到完成通知才往下走；没有中途停。

## 九、没做什么

- 不判正推那几格；不替主 agent 采纳；副本上的数不算入库装置上的数。
- K2 不在本腿攻击面；K7 只做了一小部分，K8 没做。
- 没跑任何门禁阶段、checker 档测试与重型测试；没起 GPU。
- K4、K6 相邻那两形只读码，要坐实得在副本上造镜像：K4 照 `crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs` 的造法，把墓碑树根条目里容器子指针第二条位置条目的设备号改成 0，挂可写、发一次要开新容器的 unlink 意图，看是不是在 `crates/singlefs-core/src/pointer.rs:326` 那句断言上 panic；K6 要一片满的退役容器号是 `u64::MAX`、队列里有一个删除中的 inode、它的删除步会删空一片叶容器，再挂可写看报错之前录制流里有没有取号与写行的写。两样都是 harness 档，本腿没写。

## 十、草稿与产物去向

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-b44-code-r1-opus-s2/`：`repo/`（冻结副本的拷贝，编译用）、`target/`（编译目录）、`model/`（原型）、`runs/`（逐件原始输出）、`part-*.md`（报告草稿）。交回前删 `repo/` 与 `target/`；`runs/` 的原始输出留着（汇总已进模型目录 `out/`），主 agent 不用可删。

## 十一、引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-b44-code-r1-opus-s2-output.md` 末行（写完这一节之前跑的；这一节本身不含引文）：

```
  ✓ 核了 32 处引文、1 处裸行号，对上 32 处，没判 1 处（其中裸行号没有引文可比 1 处；逐处列在上面）
```

没判的那一处是第九节里的 `crates/singlefs-core/src/pointer.rs:326`（裸行号，同一行在第三节带引文引过、对上了）。
