# m2-witness-r1 云端攻方腿（Opus）：W2 放处、W3 写序

时刻：2026-09-24（UTC，JST 同日晚）。格：W2、W3。开工快照 `research/prompts/m2-witness-r1-snapshot/sha256sums.txt` 13 行全部 `OK`（`sha256sum -c` 现核）。
派发没给禁读清单，按空处理；这一轮没读别的腿的 output、没读岔路单。
**本报告里所有数都是副本上量的**（`/tmp/claude-1000/m2-witness-r1-opus/repo`，入库仓一个字节没动），只在我的模型上量过、被攻过零轮；引用前要在入库装置上重做。

## 复跑

```bash
# 1. 仓拷一份（排除 target、.git），把原型补丁打上
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ /tmp/x/repo/
cd /tmp/x/repo && patch -p1 < /home/fy5090/code/singlefs/research/prompts/m2-witness-r1-opus-model/src/prototype.diff
bash /home/fy5090/code/singlefs/research/scripts/capped.sh 4 cargo build --release -p singlefs-harness --bin e158_root_choice_repair
# 2. 一臂一进程；放处 none|a|b|c，写序 pre|post|late，物理块 512|4096
SINGLEFS_WITNESS_PLACEMENT=a SINGLEFS_WITNESS_ORDER=pre E158_PBS=512 target/release/e158_root_choice_repair w-family 2   # H3 G0 L=3，|F|≤2
SINGLEFS_WITNESS_PLACEMENT=a E158_PBS=512 target/release/e158_root_choice_repair w-min                                # 构造 F* = Hi ∪ 见证副本
SINGLEFS_WITNESS_PLACEMENT=a SINGLEFS_WITNESS_ORDER=pre E158_PBS=512 target/release/e158_root_choice_repair w-targeted   # 挑的两条回退边
SINGLEFS_WITNESS_PLACEMENT=b E158_PBS=512 target/release/e158_root_choice_repair w-stale                              # 旧 fsid 的见证片；加 SINGLEFS_WITNESS_SKIP_FSID=1 是不核 fsid 那一臂
# 3. 成批：model 目录里 run-batch.sh <批名> batch4.txt（xargs -P 4，每进程单线程）；归并：python3 matrix.py out/batch4
```

模型目录 `research/prompts/m2-witness-r1-opus-model/` 91 个文件，逐个 sha256 在同目录 `sha256sums.txt`（它自己的 sha256 `c47dcb2f7239ac0e91fe8ca374a1f45b123cb175f76de4bc1b39e8d964419da2`）。源码与驱动：

| 文件 | sha256 |
|---|---|
| `src/prototype.diff`（副本相对入库仓的全部改动，1716 行） | `7fef042e95f35f20b06eb0c9970a0d950663443117cb15c01e717d7072209d83` |
| `src/witness.rs`（见证模块） | `ef7beb1f3be007f0646b23e629c90667011690cf7647600819bac45c945d11ca` |
| `src/w_attack_include.rs`（接在 E158 bin 上的驱动） | `f34a6a2342326cd6ed75a0d75688fdaec874801ea2ef15042892a8c32dfda714` |
| `run-batch.sh` | `67b392c71d510d215f2eb0a165907b48be6f674e1928f8e116de2fcc01a12d06` |
| `minseg.py`（三.2 的表） | `768d6b26d37849d66728d3d82f35664ef7828a078d52f8c64db41ca4af8a765d` |
| `matrix.py` / `summarize.py` | `5c4e605a…2d09593` / `98444022…853f804a4`（全值见 sha256sums.txt） |

⚠️ `out/batch1` 是在最终二进制之前一版上跑的；之后的三处改动只在 (c) 模式与崩溃模式里生效（按构造），`out/recheck` 在最终二进制上重跑了 none / (a) 两臂的全族，逐字段与 batch1 相同（见第二节）。

## 哪些是全量、哪些是挑的历史

- **全量（在主 agent 转用户令之前跑完的）**：`w-family`——E158 的 H3 族（G0，σ 长 ≤3，覆盖写次数 1/2/3，1822 个节点）× 故障集合 |F|≤2，故障全集 = 可读根槽 ∪ 这一臂的见证副本。它罩住 E158 那 213 个出错组合。`w-min`——同一族每个带可读被抛弃根的节点构造一次 F*。
- **挑的历史（用户令之后）**：`w-targeted` 两条回退边 E1、E2（第三节）上回退那一次挂载的每个崩溃点（含撕裂）× |F|≤3；T6 续接历史；T5 旧 fsid 片。
- **被停掉的全量崩溃枚举**：batch3（深度 1 前缀全部回退边 × 全部崩溃点），用户令到时按进程号停掉（931783 931791 931815 931816 932582 953681，`proc.py stop` 原样输出 6 行 `terminated`）。停之前跑完的两臂（none-512、a-late-512）只留了汇总行，**不进本报告任何结论**。全量崩溃枚举留给最后统一的层 0。

## 各格判定一览

| 格 | 判定 | 依据（节） |
|---|---|---|
| W2 三个放处在 E158 那 213 个出错组合上 | **三个都 0 个还错**；512、4096 两种物理块都 0；|F|≤2 全族里新开的错也是 0 | 二.2（全量） |
| W2 回退做完之后（稳态）最少几个故障能静默撤销 | (a)(c)：**静默撤销造不出来**——见证副本读不出就是系统配置槽读不出，全读不出时挂载报错（1702/1702 个构造集合都是「恢复失败」，不是违例）；(b)：Hi + 2（最少 4），因为 (b) 每次回退每盘只写一份 | 二.3（全量） |
| W2 撕裂（512 物理块） | (a) 越过 512 的槽撕了＝那一槽作废，与「没持久」同类，没开新格；**(c) 开了一格新错**：只持久了见证扇区、没持久系统配置扇区时，见证已生效而取号没生效（H3） | 二.4、三.3（挑的） |
| W2 4096 物理块 | 三个放处都没有撕裂维；各格最少故障数与 512 同形，只少了撕裂那几个状态 | 三.2（挑的） |
| W2 块头身份字段 | fsid 承重（不核 fsid 时一份旧见证片把择根拉回 (0,0)，核时不动）；世代号只用在写的轮换上；盘号在我的历史里不承重 | 二.5（挑的） |
| W3 写序 pre（第一个新根之前） | **打中 H1**：见证写到一半崩（只落了一份）⇒ 回退在冷启动里已生效、新实例一条根都没有；那一份读不出（1 个故障）⇒ 择根回到被抛弃的尾。见证落齐、新根没落时 2 个故障。三个放处同形，不分辨放处 | 三.3 |
| W3 写序 post（第一个新根 FUA 之后） | **打中 H2**：新根落盘、见证没落盘那一段，1 个故障撤销（与今天同）；见证落齐之后 ≥3 | 三.3 |
| W3 写序 late（暖机之后） | 在整个暖机窗口里与今天逐格相同（1 个、2 个故障那几段原样留着），late 对崩溃窗口不起作用 | 三.3 |
| W3 续接历史（T6） | pre 下 1 个故障的那一格：可写挂载把新实例建在被抛弃的尾上，撤故障后回退永久消失；**(b) 另开一格新错 H4**：见证片不随系统配置轮换被盖掉，留在盘上与新实例的时间线矛盾，之后新实例三条根读不出时择根跳到 R_old，而不是新实例的父根 | 三.4 |
| W3 层 0 要加的流 | 五条（三.5） | 三.5 |

四句（分不分辨臂、判别子看不看得到、满足判据哪一分句、改法在打中那几格还中不中）逐个写在三.3、三.4 每个打中后面。

## 一、原型：三个放处与三种写序各是什么（副本上的定义，只在我的模型上量过、被攻过零轮）

一条见证 16 字节 `(N 新实例, r 目标实例, T 目标 txg)`，意思是：按 (实例, txg) 字典序落在 (r, T) 与 (N, 0) 之间（两端不含）的根与记录都在被这次回退抛弃的时间线上。表 = 1 字节条数 + 4 条（N_w 取 4），65 字节。
读法三处一样：盘上**全部**自证过的见证副本取并集；择根跳过被并集抛弃的根（`choose_root` 那一处，入库文件的位置见 `crates/singlefs-core/src/recovery.rs:625` 那行文档注释「/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的。」）；重放遇到被抛弃的记录即停；回退候选集排除被抛弃的根。写的时候把盘上并集加上这次那一条一起写（不从内存带）。
这几处咬合（择根、重放、候选集）是我为让原型自洽补的前提，属于 W4，不归我判，照实写在这里。

| 放处 | 盘上位置与形状 | 什么时候写 |
|---|---|---|
| (a) 系统配置槽 | 槽内 481 + 8（C331 乙占位）之后 65 字节，[489, 554)，**越过 512**；被已有的整槽校验和罩着 | 每一次系统配置写（取号、每次发布末尾的轮换）都带上并集 |
| (b) 独立自证单元 | 每盘两槽，盘上偏移 512 KiB 与 512 KiB+4096（系统配置两槽之后、根环 1 MiB 之前空着的那段）；一片 = 一个物理块（512 或 4096），内容 97 字节：magic 4 + fsid 16 + 盘号 4 + 世代号 8 + 表 65 + CRC32C 4；世代号 mod 2 选槽 | 只在回退时写（每盘一次 + 一道屏障） |
| (c) 分片自证 | 系统配置槽第 2 个 512 扇区放一片（与 (b) 同形，magic 不同）；系统配置自己的整槽校验和把这一扇区按 0 算，系统配置仍 ≤512；表空时不写片 | 同 (a)：每次系统配置写都带上 |

| 写序 | (a)(c) | (b) |
|---|---|---|
| pre | 取号那次系统配置写本身（取号之后本来就有一道屏障） | 取号屏障之后、写行那次发布的单元之前：两盘各一片 + 屏障 |
| post | 写行那次发布的根 FUA 之后那次轮换（D16 已定项 7 的次序不动：根 → 系统配置） | 写行那次发布（根 FUA、轮换）之后：两盘各一片 + 屏障 |
| late | 暖机全做完之后多一次轮换 + 屏障 | 暖机之后两盘各一片 + 屏障 |

故障模型：点名一个范围，读落在范围里就失败（与 E158 的「点名根槽读不出」同义，推广到见证副本）。(a) 的一份见证副本 = 一个系统配置槽 4096；(c) 的一份 = 系统配置槽第 2 扇区 512——但读系统配置是整槽 4096 一次读，所以这一扇区读不出时那一槽的系统配置也读不出；(b) 的一份 = 那一片。

## 二、W2 放处

### 二.1 锚：今天那一臂在副本上复现 E158

`out/recheck/w-family-none-512.out` 与 `out/batch1/w-family-none-pre-512-2.out`、`-4096-2.out` 三份逐字段相同（只差 pbs 字段），整行抄其一：

```text
E7RESULT name=witness_family_summary placement=None order=BeforeFirstRoot pbs=512 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) max_weight=2 nodes_visited=1822 pairs=170719 pairs_with_any_violation=213 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=0 skipped_by_the_reduction_rule=145145
E7RESULT name=witness_family_by_checkpoint placement=None order=BeforeFirstRoot pbs=512 a_V1=213 a_V2=175 b_V1=0 b_V2=175 c_V1=0 c_V2=175
```

与 E158 实验页的 `fault_set_violation_summary`（1822 / 170719 / 213 / 145145）与三处 V1/V2 分布逐数相同。我的判定函数换成了「按范围点名」、冷启动走 `MemoryPool`（带 journal 提示）而不是 E158 的稀疏设备切片，这个锚说明换了之后判的还是同一件事。4096 物理块上也是 213。

### 二.2 那 213 个组合：三个放处都 0 个还错（全量）

故障全集 = 可读根槽 ∪ 这一臂的 4 份见证副本，所以对数从 170719 变成 283947；那 213 个只点名根槽的组合是它的子集。整行抄（放处写序 pre；pre/post/late 在「回退做完」的节点上盘面相同——取号、写行、暖机三次轮换都带上见证，所以全族只跑 pre，这一句是推的，没跑 post/late 的全族）：

```text
batch1/w-family-a-pre-512-2.out:
E7RESULT name=witness_family_summary placement=SystemConfigurationSlot order=BeforeFirstRoot pbs=512 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) max_weight=2 nodes_visited=1822 pairs=283947 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=0 skipped_by_the_reduction_rule=251085
batch1/w-family-b-pre-512-2.out:
E7RESULT name=witness_family_summary placement=IndependentUnit order=BeforeFirstRoot pbs=512 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) max_weight=2 nodes_visited=1822 pairs=283947 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=0 skipped_by_the_reduction_rule=251085
batch2/w-family-c-pre-512-2.out:
E7RESULT name=witness_family_summary placement=SectorPiece order=BeforeFirstRoot pbs=512 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) max_weight=2 nodes_visited=1822 pairs=283947 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=0 skipped_by_the_reduction_rule=251085
batch1/w-family-a-pre-4096-2.out:
E7RESULT name=witness_family_summary placement=SystemConfigurationSlot order=BeforeFirstRoot pbs=4096 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) max_weight=2 nodes_visited=1822 pairs=283947 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=0 skipped_by_the_reduction_rule=251085
batch1/w-family-b-pre-4096-2.out:
E7RESULT name=witness_family_summary placement=IndependentUnit order=BeforeFirstRoot pbs=4096 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) max_weight=2 nodes_visited=1822 pairs=283947 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=0 skipped_by_the_reduction_rule=251085
batch2/w-family-c-pre-4096-2.out:
E7RESULT name=witness_family_summary placement=SectorPiece order=BeforeFirstRoot pbs=4096 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) max_weight=2 nodes_visited=1822 pairs=283947 pairs_with_any_violation=0 cold_recover_with_fault_failed=0 mount_writable_with_fault_failed=0 skipped_by_the_reduction_rule=251085
```

（`grep -c '^VKEY'` 在 none 两份上是 213，在 a/b/c 六份上是 0。）

**读法**：三个放处在 |F|≤2 上都修掉了 C332，也没新开 |F|≤2 的错。**这一格不分辨放处。**分辨放处的是下面二.3、三.3、三.4。

### 二.3 回退做完之后，静默撤销最少要几个故障（全量，`w-min`）

每个「有可读被抛弃根」的节点（1702 个）构造 F* = Hi ∪ W_eff：Hi = 当前时间线上 (txg, 实例) 高于最高那条可读被抛弃根的可读根；W_eff = 单独留下时读得到见证的那几份副本。真跑一次冷启动：

| 放处 | 构造集合打中 | 没打中 | 撤掉见证那一半还打中 | |F*| 分布 |
|---|---|---|---|---|
| 今天（none） | 1702 | 0 | — | {2: 213, 3: 1086, 4: 58, …, 9: 16} |
| (a) | **0** | 1702 | 0 | {6: 213, 7: 1086, …, 13: 16} |
| (b) | 1702 | 0 | 0 | {4: 77, 5: 94, 6: 166, 7: 1019, …, 13: 12} |
| (c) | **0** | 1702 | 0 | {6: 213, 7: 1086, …, 13: 16} |

（整行在 `out/batch1/w-min-*-pre-512.out`、`out/batch2/w-min-c-pre-512.out`。）

- (a)(c) 的 0：见证副本全读不出 ⇒ 两盘四个系统配置槽全读不出 ⇒ `choose_system_configuration` 报 `NoValidSystemConfiguration`，恢复失败——**响亮的失败，不是静默撤销**。见证与系统配置同一个失效域，这是 (a)(c) 相对 (b) 买到的东西。
- (b) 的 4：(b) 每次回退每盘只写一片，另一槽是旧表（或空），W_eff = 2；Hi 最少 2。见证与系统配置分属两个失效域，所以见证全坏时系统照样挂得上、照样选错。
- 撤掉见证那一半（只打 Hi）三臂都 0 打中：择根确实是靠见证挡住的。

`w-min` 四行整行抄：

```text
E7RESULT name=witness_minimum_fault_summary placement=SectorPiece order=BeforeFirstRoot pbs=512 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) sigma_length_limit=3 nodes_with_live_abandoned_root=1702 constructed_set_hits=0 constructed_set_misses=1702 without_witness_half_hits=0 histogram_of_fault_count={6: 213, 7: 1086, 8: 58, 9: 123, 10: 160, 11: 26, 12: 20, 13: 16}
E7RESULT name=witness_minimum_fault_summary placement=None order=BeforeFirstRoot pbs=512 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) sigma_length_limit=3 nodes_with_live_abandoned_root=1702 constructed_set_hits=1702 constructed_set_misses=0 without_witness_half_hits=0 histogram_of_fault_count={2: 213, 3: 1086, 4: 58, 5: 123, 6: 160, 7: 26, 8: 20, 9: 16}
E7RESULT name=witness_minimum_fault_summary placement=SystemConfigurationSlot order=BeforeFirstRoot pbs=512 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) sigma_length_limit=3 nodes_with_live_abandoned_root=1702 constructed_set_hits=0 constructed_set_misses=1702 without_witness_half_hits=0 histogram_of_fault_count={6: 213, 7: 1086, 8: 58, 9: 123, 10: 160, 11: 26, 12: 20, 13: 16}
E7RESULT name=witness_minimum_fault_summary placement=IndependentUnit order=BeforeFirstRoot pbs=512 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) sigma_length_limit=3 nodes_with_live_abandoned_root=1702 constructed_set_hits=1702 constructed_set_misses=0 without_witness_half_hits=0 histogram_of_fault_count={4: 77, 5: 94, 6: 166, 7: 1019, 8: 56, 9: 103, 10: 137, 11: 23, 12: 15, 13: 12}
```

### 二.4 撕裂（512 物理块；挑的历史，三.2 的两条边上）

撕裂的取法：长于物理块的写，只持久它与底图不同的那几个物理块的非空真子集（与层 0「撕裂并进没持久」不同，这里单独列出来，因为越过 512 的问题就在这里）。

- (a)：见证越过 512 时，一次系统配置写与底图不同的是第 0、1 两个扇区；撕了之后整槽校验和不过，**那一槽作废**。作废的是这次要盖掉的那一槽（世代号较老的那份），另一槽原样在，所以撕裂与「这次写没持久」落进同一类，E2（表 2 条、字节延伸到 522）上 (a)-pre 撕裂那几个状态的最少故障数与不撕的同格状态逐个相同（`out/batch4/w-targeted-a-pre-512.out` 里 `CRASH E2 seg=0/11` 那 11 行，`grep -c` 现数）。越过 512 在这两条边上没有开出新格。
- (c)：两个扇区各自自证，撕裂会留下**系统配置是旧的、见证片是新的**。取号那次写（pre）撕成这样时：见证 (N, r, T) 已经在盘上，N 的取号却没落盘。E1 上这一格整行抄：

```text
CRASH E1 seg=0/11 [SC0.1,SC1.1] persisted=[] torn=SC0.1:blocks[1]of[0, 1] rb0=true chosen0=(1, 3) new_root_durable=false witness_entries=1 copies_with_new_entry=1 flip_to_rb_under_fault=false minF=1 example=[W0@4608]@a
```

  两盘都没「持久」取号写、只有盘 0 那次写的第 2 扇区落了，冷启动已经按回退选 R_old=(1,3)，那一扇区读不出（1 个故障）就回到被抛弃的尾。这一格 (a)(b) 没有（(a) 撕了就作废；(b) 的见证在取号屏障之后才写）。记作 **H3**，只在 512 上有。
- 4096 物理块：系统配置槽、见证片、独立单元都是一个物理块，撕裂维消失；(a)(c) 在 4096 上逐格相同（三.2 表）。

### 二.5 块头身份字段换来什么（挑的历史，`w-stale`）

盘上留一份 fsid = 0xEE×16、表里一条 (u32::MAX, 0, 0)（抛弃 (0,0) 之上的一切）的旧见证，整行抄：

```text
E7RESULT name=witness_stale_foreign_piece placement=IndependentUnit order=BeforeFirstRoot pbs=512 checks_fsid=true chosen_without_stale=Some((1, 4)) chosen_with_stale=Some((1, 4))
E7RESULT name=witness_stale_foreign_piece placement=IndependentUnit order=BeforeFirstRoot pbs=512 checks_fsid=false chosen_without_stale=Some((1, 4)) chosen_with_stale=Some((0, 0))
E7RESULT name=witness_stale_foreign_piece placement=SectorPiece order=BeforeFirstRoot pbs=512 checks_fsid=true chosen_without_stale=Some((1, 4)) chosen_with_stale=Some((1, 4))
E7RESULT name=witness_stale_foreign_piece placement=SectorPiece order=BeforeFirstRoot pbs=512 checks_fsid=false chosen_without_stale=Some((1, 4)) chosen_with_stale=Some((0, 0))
```

| 字段 | 在哪个放处 | 换来什么（本轮量到的） |
|---|---|---|
| fsid | (a) 继承系统配置自己的；(b)(c) 片头 16 字节 | **承重**：不核时一份别的池留下的片把择根拉回 (0,0)。(b) 的位置在 mkfs 不写的那段，重格式化不清它，fsid 是唯一挡它的东西（推的：原型的 mkfs 没清那段，真 mkfs 清不清是另一条条款） |
| 世代号 | (a) 继承系统配置的；(b)(c) 片头 8 字节 | 只用在写：选下一次盖哪一槽。读取并集，读的时候不用它 |
| 盘号 | (b)(c) 片头 4 字节 | 本轮没有一段历史让它承重（没攻「片被搬到另一块盘」那一格） |
| 条目里的 N | 三处都有 | 三.3、三.4 的打中都出在「见证条目在盘上，但 N 这个实例并没有按条目说的那样成立」——**条目没绑到任何能核 N 成立的东西上**，这是三个放处共同缺的一样身份 |

### 二.6 与 C331 乙（8 字节）同放

(a)：481 + 8 + 65 = 554，越过 512 共 42 字节，槽内还剩 4096 − 554 = 3542；(c)：系统配置 489 ≤ 512（余 23），见证片 97 在第 2 扇区；(b)：系统配置 489，见证在别处。这三个数是算术，W5 归本地攻方，这里只为二.4 交代 (a) 为什么撕得开。

## 三、W3 写序（挑的历史）

### 三.1 两条边与判法

两条边都是 G0、覆盖写次数 1（`EDGE` 行整行抄自 `out/batch4/w-targeted-none-pre-512.out`）：

```text
EDGE E1 placement=None order=BeforeFirstRoot pbs=512 path=[Overwrite] target=(1, 3) pre_tip=(1, 4) abandoned={(1, 4)} new_instance_roots=[(2, 5), (2, 6), (2, 7)]
EDGE E2 placement=None order=BeforeFirstRoot pbs=512 path=[Overwrite, Overwrite, RollbackTo(1, 4)] target=(1, 3) pre_tip=(2, 7) abandoned={(1, 4), (2, 6), (2, 7)} new_instance_roots=[(3, 8), (3, 9), (3, 10)]
```

E1 是一张空表上的第一次回退；E2 前面已有一次回退，表里是第二条（(a) 的表因此延伸到 522，越过 512）。两条都挑「退一步、目标带文件、目标与尾内容不同」的那种，V2 看得见。
每条边：录下 `mount_rollback` 那一次挂载的全部写与屏障，按屏障与 FUA 切段；崩溃状态 = 更早的段全持久 + 当前段任意子集（段里有单元时单元一个不持久、其余任意子集）+ 二.4 的撕裂 + 全做完。
每个崩溃状态先不带故障冷启动：所选根不在被抛弃集合里记「回退已成立」（rb0）。rb0 的状态上对 U = {新实例已落盘的根槽} ∪ 4 份见证副本 ∪ {R_old 的槽, 回退前尾的槽} 取 |F|≤3 全部子集，跑 P332 三处（冷启动带故障；带故障可写挂载后再冷启动；撤故障后冷启动），记最早打中的 |F|（minF）。
**这个「rb0」是我的操作化**：C332 条目（`.claude/kb/checks-owed.md:286`）「怎么拦」那一格（转述，那一行很长不整抄）讲的是回退之后注入根读不出、逐状态判挂载根，没说回退那一次挂载崩在半路算不算「之后」。三.3 的打中都落在这个缝上，四句里写明。

### 三.2 每段最早打中的故障数（`minseg.py`，E1；E2 逐段同形）

段名：SC 系统配置写，WU 独立见证片，UNIT 单元，REC 记录，ROOT 根 FUA。格里是这一段内回退已成立的状态中最小的 minF，「-」= 这一段里回退还没成立。

| 臂 | 取号 | 单元 | 记录 | 行根 | 轮换 | 记录 | 暖1根 | 轮换 | 记录 | 暖2根 | 轮换 | 做完 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 今天（512/4096 同） | - | - | - | - | **1** | **1** | **1** | 2 | 2 | 2 | 3 | 3 |
| (a)/(c)-pre（512/4096 同） | **1** | 2 | 2 | 2 | 3 | >3 | >3 | >3 | >3 | >3 | >3 | >3 |
| (a)/(c)-post（512/4096 同） | - | - | - | - | **1** | 3 | 3 | >3 | >3 | >3 | >3 | >3 |
| (a)/(b)/(c)-late（只跑了 512） | - | - | - | - | **1** | **1** | **1** | 2 | 2 | 2 | 3 | >3 |

(b) 多一段见证片（WU）：(b)-pre `0:SC=- 1:WU=1 2:UNIT=2 3:REC=2 4:ROOT=2 5:SC+UNIT=3 6:REC=3 7:ROOT=3 8…11,done=>3`；(b)-post `4:SC+WU=1 5:UNIT=3 6:REC=3 7:ROOT=3 8…done=>3`（原样见 `python3 minseg.py out/batch4 E1`，minseg.py sha256 `768d6b26d37849d66728d3d82f35664ef7828a078d52f8c64db41ca4af8a765d`）。
「行根」那一格对今天与 post 是「-」：那一段子集为空时根还没落，落了就算进下一段的起点。

读法：
- 今天的 1 个故障窗口从写行那次的根落盘起，到暖机第一次的根落盘止（新实例只有一条根、只在盘 0）；这是 C332「与暖机只防单故障同一类」那一格，今天就有。
- **post**：把 1 个故障的窗口缩到「根落了、带见证的那次轮换还没落」这一段；轮换落了之后 ≥3。
- **pre**：1 个故障的窗口挪到见证那一次写的中间（只落了一盘），之后到新实例第一条根之前是 2 个故障；**这一段里新实例一条根都没有，回退却已在冷启动里成立**（E1 上 (a)-pre 512 这类状态 35 个，`rolled_back_without_new_root`）。
- **late**：与今天逐格相同直到暖机做完，只把「做完」那一格从 3 抬到 >3。

### 三.3 打中

**H1（pre，三个放处同形）**：崩在见证那一次写的中间，只落了一份。(a)-pre E1 整行抄（`out/batch4/w-targeted-a-pre-512.out`）：

```text
CRASH E1 seg=0/11 [SC0.1,SC1.1] persisted=[0] rb0=true chosen0=(1, 3) new_root_durable=false witness_entries=1 copies_with_new_entry=1 flip_to_rb_under_fault=false minF=1 example=[W0@4096]@a
```

历史：覆盖写到 (1,4) → 管理员回退到 (1,3) → 取号那次系统配置写只落了盘 0 就崩 → 冷启动选 (1,3)（回退已成立）→ 盘 0 那一槽读不出（1 个故障）→ 冷启动选 (1,4)，被抛弃的尾。每一步许可它的那一句：取号先于本实例一切非系统配置写（`crates/singlefs-core/src/transaction.rs:389` 整行：「/// 实例代号) + 1；逐盘写一次系统配置槽（世代号逐盘 +1）；两写之后一道屏障，屏障完成才把新号交出去，于是本实例的第一个」；调用处 `crates/singlefs-core/src/mount.rs:1527`）；原型的择根按见证并集过滤；D20 已定项 4 自证单元那一行（`.claude/kb/decisions/20-承重面单元的原子性与自包含.md:68` 整行：「| **自证单元**（根槽、journal 记录头、**系统配置槽**） | **等于运行时探测到的 `physical_block_size`，不许硬编码。** 这一句说的是**撕裂判定的分辨率**，不是结构尺寸。见 D23（journal 的角色与格式） 已定项 4 的明文禁令 | 核心层合成（整单元校验和 + 代号 + 槽轮换，见 D22（单元原子性怎么合成） 已定项 21）；设备声明只做上界参数 |」）——一次取号写落一盘、另一盘没落，是两块盘各一次写的持久子集，层 0 的枚举域本来就含它。
- 分不分辨臂：**不分辨放处**（(a)(b)(c) 在 pre 下同形），分辨写序（post、late 在这一段没有「回退已成立」的状态）。
- 判别子看得到吗：看不到。冷启动时「见证只写了一份」与「写了两份、另一份读不出」逐字节相同；任何写序都有一个「只有一份承诺证据」的时刻（今天是新实例第一条根，只在一盘上），所以「每个崩溃状态都要 ≥2 个故障才撤销」这条判据**任何写序都满足不了**——属于「判别子观测不到」那一形（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:183` 整行：「| **判别子观测不到** | 判据要被判的系统区分两种情形，而那个系统在做决定的那一刻看得到的东西，两种情形逐条相同 | 要求这两种情形判得不同的两条判据互相矛盾，任何做法都只能满足其中一条。改判据，改完是新一轮 |」），该改的是判据：崩在回退那一次挂载半路的状态算不算「回退之后」。
- 满足判据哪一分句：C332「怎么拦」那一格（`.claude/kb/checks-owed.md:286`，转述）判挂载根落不落在被抛弃的时间线上，这一分句满足；「回退之后」那一分句按我的 rb0 操作化才成立——**可能归错判据**，按字面它更像 C340（`.claude/kb/checks-owed.md` 里 C340 那一行，转述）那边「崩在回退生效之前，恢复要与没发起回退时相同」管的事。
- 改法在这一格还中不中：三个放处都还中；post 在这一格不中（回退还没成立），但它在「行根之后、轮换之前」那一格中（H2）。

**H2（post 与 late）**：新实例第一条根落了、带见证的写还没落，1 个故障撤销。与今天同一格，post 只是把它缩成一段，late 一格没缩。分不分辨臂：分辨写序（pre 在这一段已经 3），不分辨放处。判别子：同 H1。判据分句：这一格回退已按新根成立，属于 C332 字面的「回退之后」更近一些，但 mount_rollback 仍没返回。改法：pre 在这一格不中；三个放处之间没有差别。

**H3（(c)，只在 512）**：见二.4。撕裂留下「见证新、取号旧」。分辨放处：只有 (c)；(a) 撕了就作废，(b) 的见证在取号屏障之后。判别子：同 H1。改法：(c) 把见证片放进取号屏障之后单独写（等于变成 (b) 的写序）就没有这一格（推的，没实现）。

### 三.4 续接历史 T6 与 H4（(b) 的孤儿见证）

T6 = 在 H1 那一格（见证只落一份、回退已成立、新实例没有根）上接着走：那一份读不出时可写挂载 → 撤故障冷启动 → 让新实例 M 的全部根读不出再冷启动。E1、512，整行抄（`out/batch4/w-targeted-a-pre-512.out`、`-b-pre-512.out`）：

```text
T6 E1 placement=SystemConfigurationSlot order=BeforeFirstRoot pbs=512 seg=0/11 [SC0.1,SC1.1] persisted=[0] faulted_witness=[W0@4096] before=(1, 3) content=R_old m_built_on=(1, 4) m_roots=[(2, 5), (2, 6), (2, 7)] after_fault_removed=(2, 7) content=abandoned-timeline witness_after={} m_roots_faulted=[R2.1,R0.2,R1.2] recover_with_m_roots_faulted=(1, 4) content=abandoned-timeline
T6 E1 placement=IndependentUnit order=BeforeFirstRoot pbs=512 seg=1/12 [WU0.1,WU1.1] persisted=[0] faulted_witness=[W0@528384] before=(1, 3) content=R_old m_built_on=(1, 4) m_roots=[(3, 5), (3, 6), (3, 7)] after_fault_removed=(3, 7) content=abandoned-timeline witness_after={WitnessEntry { new_instance: 2, target_instance: 1, target_txg: 3 }} m_roots_faulted=[R2.1,R0.2,R1.2] recover_with_m_roots_faulted=(1, 3) content=R_old
```

（`content=` 的标签是相对回退之后那条新时间线记的：`abandoned-timeline` = 读回的是回退前尾 (1,4) 的内容。）

- 两臂共同的：冷启动看到过「已回退」，一个读故障下的可写挂载把 M 建在 (1,4) 上，撤故障后回退**永久**消失。这是 H1 的后果，三个放处同形（(c) 的 T6 行与 (a) 同形，见 `-c-pre-*.out`）。
- **(a)**：那一份见证被 M 的下一次系统配置写盖掉（`witness_after={}`）——读不出的那一槽被当成没有，下一次写正好落在它上面。M 与见证之后不再矛盾。另注意 M 的实例号是 2，与没做成的回退 N 同号：N 的取号只落在那一槽里，槽读不出，号被重取（推的机制，原型没改取号）。
- **(b) H4**：见证片只在回退时写，M 不盖它；撤故障后盘上留着 (2, 1, 3)，它抛弃 (1,4)——而 (1,4) 正是 M 的父根。之后 M 的 3 条根读不出时，今天的择根回到 M 的父根 (1,4)，(b) 跳过 (1,4) 回到 (1,3)，**(1,4) 那一版在 M 的时间线上、却被一条孤儿见证挡掉**：见证与 M 的实例表（M 按 `mount_writable` 的上一实例行会写 (1, 4, W)——按代码推的，没从盘上读出来核）说反话。
  - 分不分辨臂：分辨 (b) 与 (a)(c)。但 (a)(c) 「不中」是按轮换机制推的：本段只固定了「崩溃后立刻带故障可写挂载」这一个用户动作，**没按规则把后续用户动作放开扫**（例如崩溃后先只读挂载、或撤故障后先建对象），所以 (a)(c) 在这格上「不中」只能算推的。
  - 判别子：M 挂载时见证读不出，看不到它；撤故障之后看得到，但原型的择根没有「这条见证之后又有人在被它抛弃的根上建了新实例」这一判。改法（推的、没实现）：条目生效要求新实例 N 在盘上有证据（N 的根或记录可读），或者在 M 写行时把与 M 父链矛盾的条目作废——两者都属于 W4。
  - 满足判据哪一分句：不是 P332（M 的所选根不在任何被抛弃集合里）；它是「已确认的写在之后每次挂载选的根上读得回」那一类（C331 的判据形态），按门槛算 1 个早先的读故障 + 3 个后来的根故障。

### 三.5 层 0 要加哪几条流（对表：录到的写流，`STREAM` 行）

| 候选×写序 | 回退那一次挂载的写流（段以屏障、FUA 切） | 与今天的流比，新在哪 |
|---|---|---|
| 今天 | SC×2 ‖ UNIT… ‖ REC×2 ‖ ROOT ‖ SC×2+暖机 UNIT… ‖ REC×2 ‖ ROOT ‖ SC×2+UNIT… ‖ REC×2 ‖ ROOT ‖ SC×2 | — |
| (a)(c)-pre | 同形，取号那段 SC×2 带见证 | 形不变、内容变；要一条「前面已有回退」的回退流（表越过 512），(c) 要按扇区撕 |
| (a)(c)-post | 同形，第一次轮换带见证 | 同上 |
| (a)(c)-late | 末段变 SC×4（轮换 + 多出来的那次） | 末段多两个写 |
| (b)-pre | SC×2 ‖ **WU×2** ‖ UNIT… ‖ … | 取号与第一批单元之间多一段 |
| (b)-post | … ROOT ‖ SC×2+**WU×2** ‖ UNIT… ‖ … | 行根之后那段多两个写、多一道屏障 |
| (b)-late | … ROOT ‖ SC×2+**WU×2** | 末段多两个写 |

五条要进层 0：(b)-pre、(b)-post、(b)-late 三种新写序各一条；(a)(c) 各一条「第二次回退」的流（表越过 512 / 片与系统配置分扇区），并且层 0 的撕裂要能按物理块拆开——层 0 今天把撕裂并进「没持久」，(c) 的 H3 在那个枚举域里看不见。另要一条跨挂载的流：回退挂载崩在半路 → 带读故障的可写挂载 → 撤故障的挂载（T6 / H4 的形状）；层 0 两条写死的流里没有它。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 回退做完之后 |F|≤2（根槽 ∪ 见证副本） | H3 G0 全族 1822 节点、每臂 283947 对，(a)(b)(c) × 512/4096 | 0 违例（全量，确定性枚举，不是抽样） |
| (a) 表越过 512 的撕裂 | E2 上取号段 11 个状态、各轮换段的撕裂状态 | 没开出新格（撕了＝那一槽作废＝没持久） |
| 故障让一个「还没回退」的状态看起来回退了 | 两条边上全部未成立状态 × |F|≤3 | 0（`flip_to_rolled_back_under_fault=0`，每臂每边） |
| 回退做完、见证落齐之后 |F|≤3 | 两条边，每臂 | 见证三臂都 >3（今天 3） |
| (a)(c) 见证全坏时静默选错 | 全族 1702 个构造集合 | 0，全部响亮失败 |

## 改法各修哪一格（全部只在我的模型上、被攻过零轮）

| 改法 | H1（pre 半路 1 份） | H2（行根后、见证前） | H3（(c) 撕成见证新取号旧） | H4（(b) 孤儿见证） | 213 | 标注 |
|---|---|---|---|---|---|---|
| 放处 (a)/(b)/(c) 任一，写序 pre | 中 | 不中 | (c) 中 | (b) 中 | 修掉 | 量过（二.2、三.2、三.4） |
| 写序 post | 不中（回退未成立） | 中 | (c)-post 下取号段不带见证，不中（量过：`c-post` 取号段全「-」） | 没跑（T6 只在 pre 下有起点） | 修掉 | 量过 |
| 写序 late | 不中 | 中（且窗口与今天一样长） | 不中 | 没跑 | 修掉 | 量过 |
| 把「回退已成立」定成 mount_rollback 返回之后（改判据，不改实现） | 不再是打中 | 不再是打中 | 不再是打中 | 仍中（它发生在回退返回之后的另一次挂载） | — | 推的 |
| 条目要求 N 在盘上有证据才生效 / M 写行时作废与父链矛盾的条目 | 推的：H1 那一格 N 没有根没有记录 ⇒ 条目不生效 ⇒ 回退未成立 | 不影响 | 推的：同 H1 | 推的：修 | 未知 | 推的，没实现 |
| (c) 的片挪到取号屏障之后单写 | — | — | 推的：修 | — | — | 推的 |

## 这条腿自己的限度

- 崩溃只在两条挑的回退边（G0、覆盖写 1 次）上跑；全量崩溃枚举按用户令留给最后统一的层 0。E1、E2 各段最少故障数逐段同形，但这是两条边，不是一族。
- 段里有单元时单元一律不持久；撕裂只取与底图不同的物理块数在 2–4 之间的写（系统配置槽、多数记录落在里面，单元多半不在）。
- 崩溃状态上的故障全集 U 只含新实例的根、4 份见证、R_old 与回退前尾的槽，|F|≤3；U 之外的根槽故障没扫。
- 故障只有「整段读失败、挂载期间一直失败」一种；没有写错、没有读回改坏但自证通过的字节、没有瞬时（第一次读失败第二次读成）。
- 择根、重放、回退候选集怎么读见证是我补的（W4），换一种读法，三.3、三.4 的数可能变。
- rb0（不带故障冷启动显示已回退）是我的操作化，H1–H3 是否算 C332 的打中取决于它（三.3 四句）。
- 表满的处置（W1）没攻：G0 L=3 至多 3 次回退，容量 4 没满过；S4 L=4 没跑见证臂。
- T6 / H4 只固定了一种后续用户动作（崩溃后立刻带故障可写挂载），没按规则放开扫；(a)(c) 在 H4 上「不中」是推的。
- `out/batch1` 在最终二进制之前一版上跑；差异按构造只在 (c) 模式与崩溃模式，none / (a) 两臂已在最终二进制上重跑逐字段相同（`out/recheck`）。

## 没做什么

- 没判 W1、W4、W5（不归这条腿）；W2「与乙同放的余量」只写了支撑二.4 的那三个数。
- 没改入库仓任何文件；副本上的数没有在入库装置上重做（要引，按 `.claude/rules/three-way-inference.md` 做成入库装置的新一次跑）。
- 没跑门禁阶段（`.claude/gate.d/stage-owners.tsv` 没给这条腿登记，派发也没要）。
- 全量崩溃枚举（batch3）按用户令停掉，停前跑完的 none-512、a-late-512 两臂只留汇总（`out/batch3/completed-before-stop-E7RESULT.txt`），不进结论。
- 进度记录在 `research/prompts/m2-witness-r1-opus-progress.md`（拷进模型目录）。草稿目录 `/tmp/claude-1000/m2-witness-r1-opus/` 里还有副本仓与 batch2 的 panic 输出、batch3 的全量输出（15 MB），没入库：前者是修好之前的崩溃，后者是用户令停掉的全量。

**什么会推翻这些结论**：在入库装置上照复跑命令重做，(a)(b)(c) 在 213 个组合上有任何一个还错；或者 (a)(c) 的构造集合出现「恢复成功而选错」；或者换一种见证读法（W4 定案）后 pre 的取号段不再出现「回退已成立而新实例没有根」的状态。
