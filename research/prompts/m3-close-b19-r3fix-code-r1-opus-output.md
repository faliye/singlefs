# m3-close-b19-r3fix-code-r1 攻方腿报告（2026-11-05）

立场：攻——找 `crates/singlefs-core/src/mount.rs` 抬 F 那一遍三处按槽认盘、认槽会错的那一格。代码按此刻主工作区（`mount.rs` sha256 cb00d920…，与开工快照 `research/prompts/m3-close-b19-r3fix-code-r1-snapshot/crates-sha256.txt` 三份逐个对上，`sha256sum -c` 退 0）。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-b19-r3fix-code-r1-opus-model/`（函数级等价模型，Rust 测试模块，接在副本的 `mount.rs` 末尾编）：

| 文件 | sha256 |
|---|---|
| `ring_slot_identification_equivalence_model.rs` | 9b6c7db62096ba88aabe40b54fb3327f0d8cc35bd83efcac57263ee50c6fe4c4 |
| `tools/rerun.sh`（照 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`） | 949c2d301c66b5faa59764f55685428008c3a8b0be3da2bbd700d3145ba2b0a2 |

冻结副本：`/home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-opus/frozen-tree`（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`prune-server`（工作区要读它的清单）、`.claude/heavy-thread-default`，不含 `target`；其中 `mount.rs`、`root_ring.rs`、`walk.rs` 的 sha256 与开工快照相同）。复跑整条见末尾「要主 agent 跑的命令」。模型一次都没跑过，模型目录里没有冻结输出，下文凡引模型的数一律标「推的，待主 agent 跑」。

## 各格判定一览

| 格 | 判定 | 标 |
|---|---|---|
| Q2.1 写者（按槽之后）与 checker 在准入或卸载上结局不同的合法历史 | **造不出**。三处改动在 `read_root_ring_slot` 交得出的每一种输入上与改前按 txg 的算法输出逐个相同（见 Q2.3 的三条前提），所以按槽认盘不可能新造出写者与 checker 的分歧；改前若有分歧，改后原样还在，与认法无关 | 读码坐实；模型上的全量核对待主 agent 跑 |
| Q2.2 补读那几槽带的槽与真正住的槽不同 | **不会不同**：重读读出的根经落点公式核过才交出；读坏不是全零与读回全零两种都顶上根环表里记着的那条根，它的键就是重建那一遍读它的槽、也核过公式。即使带错，也只在第 0 代根上（输出照样相同）；非第 0 代根带错才会算错，那一形被 `read_root_ring_slot` 与 I-7.17 挡在外面 | 读码坐实 |
| Q2.3 「真写者上两种认法算出的数相同」 | **结论站得住，实现员给的前提不是撑住它的那一条**。撑住的是 P1（读出的非第 0 代根都住在公式那一槽）、P2（第 0 代根按任何实例表都不被抛弃）、P4（没有释放代为 0 的已释放记录）三条，与暖机无关。暖机那条前提在「被吞的根槽写」下不成立（给了历史），而两种认法仍相等 | P1、P2 读码坐实；P4 推的；全量核对待主 agent 跑 |
| Q2.4 三条单测与三行变异 | 三行变异各被点名的单测抓到（推的，与任务 6526 的结论同向）；**但三行在整池层面都是等价变异或近乎等价**：按 txg 认盘、认槽两行在 P1、P2、P4 下输出不变，「不从钉里滤掉」只在 E2 重读那一下读坏时才看得出。只能在函数这一级测，整池用例测不到；按 txg 认槽那一行的槽数取 `AT_MAKE_FILESYSTEM`（8），不是改前那几行取的池的 S。第 4994、4997 行带的槽没有变异罩着 | 读码坐实；变异各红在哪推的 |
| Q3 mount.rs 里别处按 txg 推区域认盘或认槽 | 有九处，全部只拿 txg ≥ 1 的根、记录或要发的新 txg（txg ≥ 1 时公式就是真的落点），或是公式的逆（钉值，第 0 代另认）；**没有一处该改按槽**。唯一能碰到 txg 0 的是落后支缺根那一处（第 6982 行），而「落后于第 0 代那一版」的盘造不出 | 读码坐实四处（第 2953/2961、4790、5187、6219 行）；五处的「txg ≥ 1」靠实例代号排序推出，推的 |
| Q1（辅） | 与条款的对照只在 Q2.3 涉及的两句上做了：「环里最旧有效根」「每块盘上最新有效根」两句按槽认与字面一致；条款没写按槽还是按 txg，两种认法的数相同 | 读码坐实 |

这一条腿没打中。按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 这一节，「没打中」一次不算。

## Q2.3 两种认法为什么算出同一个数（这一条腿的主体）

改前改后的差在实现员补丁 `~/.local/share/singlefs/work/m3-close-b19-r3-impl-core/patch/crates.patch`：改前环里最旧有效根那一处按 `target_for_publish(root.checkpoint_txg, slots_per_region)` 推「这一遍在哪几槽读出了根」，卸载那一格按 `device_of_txg(root.checkpoint_txg)` 推盘；改后两处都取读出它（或顶上它）的那一槽。同一条根两种认法认出不同的槽，只有一种可能：它不住在自己 txg 的公式槽里。

**P1：读出的根都住在公式槽里，第 0 代根例外（读码坐实）。**
- 抬 F 那一遍第一读与第 4986 行的补读都经 `read_root_ring_slot`：`crates/singlefs-core/src/recovery.rs:1867`：「if !root_sits_in_the_ring_slot_the_formula_gives(」，不合公式的照「自证不过」交回，不进 `roots_with_ring_slots`。
- 第 4997 行顶上的那条根来自重建那一遍，同样核过公式：`crates/singlefs-core/src/mount.rs:2808`：「if !root_sits_in_the_ring_slot_the_formula_gives(」。
- 例外只有第 0 代根：`crates/singlefs-core/src/root_ring.rs:188`：「if checkpoint_txg == crate::make_filesystem::GENESIS_CHECKPOINT_TXG {」，三个区域的槽 0 都认；按 txg 推时一律落区域 0（`crates/singlefs-core/src/root_ring.rs:172`：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」）。
- 所以两种认法只在 txg 0 的根上不同，与函数注释第 5068 行那一句相符。

**P2：第 0 代根按任何实例表都有效（读码坐实）。** `crates/singlefs-core/src/recovery.rs:2230`：「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」——txg 0 大于某个 Ti 恒假。

**环里最旧有效根（第 5041、5069 行）**：两种认法的钉表不同 ⇔ 这一遍在区域 1 或 2 的槽 0 读出了第 0 代根（按 txg 会留着那一槽的钉，或把区域 0 槽 0 的钉拿掉）。读出了第 0 代根，由 P2 它进有效根（`crates/singlefs-core/src/mount.rs:5038`：「.filter(|(_, root)| !abandoned_by_table(root, table))」），有效根的最小值就是 0、钉值都 ≥ 0，`oldest_txg_in_the_ring_rereading_slots_bad_at_mount` 交回 0：最小钉值也是 0 时 `crates/singlefs-core/src/allocator.rs:1791`：「if oldest_valid_root.is_some_and(|oldest| oldest < pinned_txg) {」不成立、去重读那一槽，重读读坏交钉值 0、读出有效根换进去、读出被抛弃根拿掉钉，有效根里那个 0 都还在。两边交回值都是 0。重读的读数不留（第 5060 行只取 `.0`），没有副作用。这一处连「槽 0 的钉值是 0」（第 3001 行）都用不上：钉值多大，最小值都是有效根里那个 0。

**卸载那一格（第 5088、5311 行）**：每块盘取有效根的最大 txg，第 0 代根只贡献 0；两种认法只差那个 0 记在哪块盘。那块盘上另有有效根时最大值不变；只有某块盘除第 0 代根外一条有效根都没有时，一边那块盘是 `Some(0)`、另一边是 `None`，`crates/singlefs-core/src/mount.rs:4284`：「.collect::<Option<Vec<CheckpointTxg>>>()?」把整体变成 `None`。所以两边要么相同，要么一边 `None`、一边 `Some(0)`。`Some(0)` 时当场放开的门槛是 `crates/singlefs-core/src/mount.rs:5324`：「floor_of_the_reclaim.min(newest_valid_root_on_every_device),」= 0；`None` 时 `crates/singlefs-core/src/mount.rs:5328`：「None => Vec::new(),」一个都不放。

**P4：没有释放代为 0 的已释放记录（推的）。** `crates/singlefs-core/src/allocator.rs:2283`：「record.generation = release_generation;」释放代取释放它的那次发布的 txg；mkfs 那一版不释放，之后每次发布 txg ≥ 1。盘上读回的已释放记录（重建那一遍）的释放代没逐条核。P4 在，`Some(0)` 与 `None` 当场放开的都是空集。

**结论**：P1、P2、P4 下，两处改动对每一种输入交出同一个回收门槛、同一截当场放开的槽。这句比实现员的「真写者上相同」更宽：不靠暖机，函数这一级的任意输入都成立。推翻它的观测：模型 `premises_hold` 两行里 `oldest_differs` 或 `unmount_released_up_to_differs` 不是 0。

**实现员给的前提（暖机已在每块盘落了本实例的根）不满足的历史（推的，没在实现上跑）**：
1. mkfs，S = 8，第一版几何（区域 0、2 在盘 0，区域 1 在盘 1），三个区域的槽 0 各一份第 0 代根。
2. 可写挂载：写行那次发布 txg 1 落区域 1 槽 0（盘 1），盘 1 吞掉这次根槽写、FUA 照常返回（故障注入「被吞的根槽写」那一类），那一槽仍是第 0 代根；暖机 txg 2 落区域 2 槽 0（盘 0）。
3. 写一个文件，发布 txg 3（区域 0 槽 1，盘 0）。
4. 正常卸载：这一遍读出区域 0 槽 0 与区域 1 槽 0 的第 0 代根、txg 2、txg 3。区域 1 槽 0 是这个进程知道住着根的槽，第一读自证得过就收（`crates/singlefs-core/src/recovery.rs:2050`：「roots.push((ring_slot, root));」），不比是不是自己写的那一条。盘 1 上一条本实例的根都没有，前提不成立。
5. 按槽：盘 0 最新 3、盘 1 最新 0，交 `Some(0)`，当场放开 ≤ 0，空；按 txg：三份第 0 代根都归盘 0，盘 1 是 `None`，空。两边一样。

这段历史属于故障注入、不是合法写者，用它只为说明暖机那条不是撑住结论的前提；撑住的是 P1、P2、P4。

## Q2.2 补读那几槽带的槽

第 4980–4998 行只对「重建两遍里只有一遍自证过」、这一遍第一读没读出根的槽补读，三种读数：

| 补读读到 | 交进 `roots_with_ring_slots` 的 | 带的槽与真正住的槽 | 标 |
|---|---|---|---|
| 自证过的根（重读） | 读出来的那一条，槽是被读的那一槽 | 相同：`read_root_ring_slot` 核过公式（`crates/singlefs-core/src/recovery.rs:1867`），非第 0 代根只住公式槽 | 读码坐实 |
| 读坏、不是全零 | 根环表里记着的那一条，槽是表的键 | 相同：表的键是重建那一遍读它的槽，记之前核过公式（`crates/singlefs-core/src/mount.rs:2808`）。这个进程写掉那一槽之后它就不在这一类（`crates/singlefs-core/src/allocator.rs:1566`：「这个进程写掉那一槽之后就不在其内。」） | 读码坐实 |
| 读回全零 | 同上一行（全零在 `read_root_ring_slot` 里解不出根，走 `Bad`） | 同上一行 | 读码坐实 |

写失败（FUA 没返回）的那一槽（推的：没逐行查写失败时根环表换不换类）：按条款表里照旧是重建时那一条（`.claude/kb/decisions/16-发布语义.md:36`：「写失败的槽按旧内容算」），盘上可能已是新根；第一读读出新根就按新根算，读坏就顶上旧根——旧根确实住过那一槽，txg 更低，回收门槛只会更低、卸载每块盘最大值不升，方向安全（推的）。

假如带错了会怎样（推的，供判「会不会放开被引用的单元」）：带错只能在第 0 代根上，输出不变（Q2.3）；非第 0 代根带错时，卸载把它记到别的盘上会抬高那块盘的「最新有效根」、多放开一截，属于放开被引用单元的方向。这一形被 P1 挡在外面；镜像上出现时 checker 判 I-7.17 红（`crates/singlefs-checker/src/image.rs:474`：「pub const ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES: &str = "I-7.17";」）。模型 `root_read_off_its_formula_slot` 那一臂就是这一形，预期分得开（推的，待主 agent 跑）。

## Q2.1 写者与 checker 在准入或卸载上结局不同

**造不出（读码坐实这一层；全量核对推的，待主 agent 跑模型）。** 依据：Q2.3 证的是按槽与改前按 txg 在两处的输出逐个相同，这一批合入没改写者在准入、卸载上的任何一个数，所以不会因为按槽认盘新造出与 checker 的分歧。checker 一侧同样按读出那一槽的区域认盘：`crates/singlefs-checker/src/walk.rs:21115`：「let device = geometry.region_devices[usize::try_from(*region).expect("区域号")];」；镜像上不住在公式槽的自证根由 I-7.17 判红，合法写者的镜像上没有这一形。两边都按槽、写者的非第 0 代根又只住公式槽，口径一致这一句在 P1 之下成立（这一句是推的：我没逐个读 walk.rs 每一处按槽认盘的调用者，只读了第 21115 行与 I-7.17 的常量）。

扫过的历史形状（每一形都是先找按槽与按 txg 会不会分开，分开了再看输出）：
- 卸载时盘 1 上只剩区域 1 槽 0 的第 0 代根：要么这个进程在盘 1 写过的根这一遍读坏（知道住着根的槽，重读仍坏就拒，卸载报错、不走到算这一截），要么根槽写被吞（Q2.3 那段历史）。分开的只有 `None` 对 `Some(0)`，放开的都是空集。
- 挂载时区域 0 槽 0 读坏或读回全零、这一遍又读坏，区域 1、2 槽 0 读出第 0 代根：按 txg 把区域 0 槽 0 的钉拿掉、按槽留着，最小值两边都是 0（有效根里有 0）。
- 挂载时区域 1 或 2 槽 0 读坏、这一遍读出第 0 代根：按 txg 留着钉、按槽拿掉，最小值两边都是 0。
- 「两遍里只有一遍自证过」的槽正是某个区域的槽 0、记着第 0 代根，这一遍补读读坏，顶上记着的那一条：同上一形。
- 实例表把实例 0 抛弃在 Ti = 0：第 0 代根照样有效（P2），这一形不存在。
- 非第 0 代根被读成别的槽（错位读）：两条读者都照「自证不过」交，进不来；镜像上出现就判 I-7.17 红。

## Q2.4 三条单测与三行变异

| 变异行 | 改成 | 单测（第 11206 行） | 单测（第 11247 行） | 单测（第 11276 行） |
|---|---|---|---|---|
| `crates/mutations.tsv:6742`：「抬 F 那一遍按 txg 认盘」 | 区域号取 `root.checkpoint_txg.0 % 3` | 红：三份第 0 代根都归盘 0，盘 1 是 `None`，断言要 `Some(0)` | 绿：不调 `roots_on_the_devices_of_their_ring_slots` | 绿：同左 |
| `crates/mutations.tsv:6743`：「抬 F 那一遍按 txg 认读出根的槽」 | 读出的槽取 `target_for_publish(root.checkpoint_txg, AT_MAKE_FILESYSTEM)` | 绿：不调 | 红：两份第 0 代根都推成区域 0 槽 0，区域 1 槽 0 的钉留着，交回非空 | 红：区域 1、2 的第 0 代根推成区域 0 槽 0，区域 0 槽 0 的钉被拿掉，交回空表 |
| `crates/mutations.tsv:6744`：「抬 F 那一遍读出根的槽不从钉里滤掉」 | 过滤恒真 | 绿 | 红：区域 1 槽 0 的钉留着 | 绿：本来就交回原表 |

各红在哪是读码推的；背景材料说副本上 `prove-red.sh` 三行都抓到（任务 6526），我没读那份日志。

三处要交主 agent 的观察：
1. **三行在整池层面是等价变异或近乎等价（读码坐实；全量核对待主 agent 跑）**。6742、6743 在 P1、P2、P4 下与按槽的输出逐个相同（Q2.3），整池用例分不开，只有函数级单测分得开。6744 留下读出了根的槽的钉：那一槽的钉值 ≤ 它读出的根的 txg，成了最小值就走 E2 重读那一槽，重读读出同一条有效根就换回去、输出不变；只有第一读读得出、E2 那一下重读读坏时才交回更低的钉值，回收得更少，方向安全。所以这三条单测测的是「认法」本身，不是「认法改变了哪一个数」——后者今天不存在。
2. **6743 的替换串不是改前那几行**：替换串写死 `AT_MAKE_FILESYSTEM`（S = 8），改前取池的 S（实现员补丁 `~/.local/share/singlefs/work/m3-close-b19-r3-impl-core/patch/crates.patch` 第 57 行删掉的「let slots_per_region = parameters_of_the_pool.geometry.root_ring_slots_per_region;」）。函数签名里没有 S，只能写死；三条单测的根都是第 0 代、槽号 0 与 S 无关，照样红。S ≠ 8 的池上这一行等于「按 txg 认槽、还用错 S」，比改前更坏，不影响判红，只影响「变异 = 改回改前」这句话。
3. **没罩住的改动**：第 4994、4997 行推进 `roots_with_ring_slots` 的槽 `*ring_slot` 没有变异行。换成按 txg 推槽是等价变异（Q2.3）；换成一个错槽（例如区域号写死 0）才算错，而三条单测都不经过 `readable_roots_for_raising_the_floor`，抓不到；整池里要一个「两遍里只有一遍自证过」的槽落在区域 1、卸载那一遍补读它，才看得出（推的，没查现有整池用例有没有这一形）。

## Q3 mount.rs 里别处按 txg 推区域或槽

`grep -n 'ROOT_RING_REGIONS\|target_for_publish\|device_of_txg\|root_sits_in_the_ring_slot_the_formula_gives'`（第 11000 行之前，测试模块不算）。没有一处写 `% ROOT_RING_REGIONS` 求区域（第 2360、2943 行的 `ROOT_RING_REGIONS` 是环长 3S）；按 txg 求槽的九处：

| 行 | 做什么 | txg 落在哪 | 该不该按槽 | 标 |
|---|---|---|---|---|
| 2033 | 水位之上没施加的记录点名的落点，按记录的 txg 归槽（C 那一圈） | (实例代号, txg) 大于所选那一版；第 0 代那一份的实例代号最小，txg 0 进不来 | 不该：那次发布的根就写在公式槽 | 推的 |
| 2111 | 没施加的发布的账按 txg 归槽 | 同上 | 不该 | 推的 |
| 2251 | 较新可读根的账按根的 txg 归槽 | 比所选那一版新，同上 | 不该：读出的根经 P1 | 推的 |
| 2363、2369 | 较新根与中间那几次发布的槽 | 同上 | 不该 | 推的 |
| 2953、2961 | 读坏槽能住哪些 txg（公式的逆） | 全部 | 不该：槽 0 另由第 2994–3005 行认成第 0 代的 0 | 读码坐实 |
| 4790 | 准入推一次抬 F 那一串要发的新 txg 落哪块盘 | 现行 txg 之后，≥ 1 | 不该：新根就写公式槽 | 读码坐实 |
| 5187 | 抬 F 那一串要发的新 txg 落哪块盘 | 同上 | 不该 | 读码坐实 |
| 6219 | 暖机要发的新 txg 落哪块盘 | 同上 | 不该 | 读码坐实 |
| 6982 | 落后支缺根：那一版的根该在哪一槽 | 那一版的 txg；第 0 代那一版只核区域 0 槽 0（盘 0） | 不该：落后于第 0 代那一版的盘造不出（mkfs 每块盘都写了），盘 1 那一份不必核 | 推的 |

mount.rs 之外（没问，只顺手记）：`crates/singlefs-core/src/allocator.rs` 第 1386、1431、2794、2891 行按 txg 求槽，取的是被抛弃根或这个进程自己写的根，txg ≥ 1（推的）。

## Q1（辅）

D16 已定项 1「环里最旧有效根」一行（`.claude/kb/decisions/16-发布语义.md:36`）与「正常卸载」一行（`.claude/kb/decisions/16-发布语义.md:43`）都写「盘上」「每块盘上」，没写按槽还是按 txg 归盘；按槽与字面一致，两种认法的数又相同，这两句对得上（读码坐实）。别的句子没逐句对，留给正推腿。

## 模型（推的，待主 agent 跑）

建它的时候我已经从读码推出「输出相同」，所以它不是先于倾向的；它的用处是把读码那一步换成调实现里的真函数逐个比，并给出一臂会红的对照。按槽那一臂调实现里的 `ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass`、`roots_on_the_devices_of_their_ring_slots`、`newest_valid_root_txg_on_every_device`、`oldest_txg_in_the_ring_rereading_slots_bad_at_mount`、`oldest_txg_a_ring_slot_bad_at_mount_can_have_held`、`abandoned_by_table`；按 txg 那一臂照补丁删掉的那几行写（槽数取池的 S）。

- 取样：三个区域槽 0 与三个区域槽 1 共 6 个槽，每槽读坏、读回全零、合公式的实例 1 / 2 的根（槽 0 另加第 0 代根与下一圈的 txg）；挂载时钉着的槽取区域 0 / 1 / 2 槽 0 与区域 0 槽 1 四个槽的全部 16 个子集；实例表 6 种（含把实例 0、1、2 都抛弃在 Ti = 0 那一张）；E2 重读口径 3 种；S = 4、8。每个 S 是 5 × 7 × 7 × 4 × 4 × 4 × 16 × 6 × 3 = 4515840 格；破前提那一臂（区域 0 槽 1 另可住 txg 4 的根）S = 8，5644800 格；合计 14676480 格（算术，推的）。
- 没有崩溃状态：这是函数级的输入枚举，不造池、不录写流，崩溃点枚举那一条用不上。
- 估时（推的）：debug 构建每格估 1–10 µs，比较段 15 秒到 2.5 分钟，另加编 singlefs-core 的测试（几分钟）；远低于 40 分钟，没缩取样。
- 跑前写下的预期（模型里同样以断言写着，断言在全部打印之后）：`premises_hold` 两行 `oldest_differs=0`、`unmount_released_up_to_differs=0`、`differences_without_a_genesis_root_read=0`、`unmount_newest_raw_differs` 等于两类 `None`/`Some(0)` 之和、`pin_maps_differ` 与 `unmount_newest_raw_differs` 都大于 0；`root_read_off_its_formula_slot` 那一行 `unmount_released_up_to_differs` 大于 0。结果反过来时：`premises_hold` 里任何一个 differs 不是 0 就是这一条腿打中，翻成「按槽认盘改了写者的数」，再看分开的那几格是哪一臂放得多；破前提那一臂分不开说明模型判别力不够，`premises_hold` 的 0 不作数。

## 没打中的形状

- 按槽与按 txg 在两处的输出：读码证到 P1、P2、P4 下逐个相同；模型枚举的范围见上一节（待跑）。
- 补读那几槽带错槽：三种读数都带对（Q2.2）。
- 第 0 代根在卸载那一格被记到盘 1 而多放开：放开门槛只会是 0，P4 下放开空集。
- 第 0 代根被抛弃造成「按 txg 拿掉钉、按槽留着」的分歧：P2 让它不存在。
- 实现员前提失效（根槽写被吞）：两边仍相同（Q2.3 那段历史）。
- 不重复前三轮的角度：准入算上限那一处按槽、全零并、推空都没碰。

## 这条腿自己的限度

- 一次抽样；「没打中」照规则不算观测，要第二样本。
- 模型没跑，报告里模型的数全是推的；模型是函数级的，不是整池历史，没有造写者走到卸载的真历史，也没对拍 checker。
- P4 没逐条核盘上读回的已释放记录；walk.rs 只读了第 21115 行附近与 I-7.17 常量，没逐个读 checker 每一处按槽认盘的调用者。
- Q3 里五处的「txg ≥ 1」靠实例代号排序推出，没逐个读调用方怎么滤。
- 写失败那一槽根环表换不换类没逐行查（Q2.2 写明了推的）。

## 要主 agent 跑的命令

1. 模型复跑（不是重型；`singlefs-core` 的 `--lib`，在冻结副本的拷贝里编，脚本里那一条 cargo 自带范围声明）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-r3fix-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-opus/frozen-tree --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-opus/rerun-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-opus`（冻结副本已在那一目录的 `.deliverables` 写成「交接」）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里三行 `MODELRESULT`（`name=premises_hold S=4`、`name=premises_hold S=8`、`name=root_read_off_its_formula_slot S=8`）与末行 `name=rerun_roots … cargo_test_exit=0`。`exit` 非 0 而日志里有 `panicked` 是断言打中（预期被推翻，按上一节「结果反过来时」办）；有 `error[E` 是模型编不过，交回攻方腿改。整份输出在 `<--work>/model.out`，三行另在 `<--work>/model-results.txt`。
2. 复跑完照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」那一段第 3 步删副本与编译目录。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，模型一次没跑，判决要引的数要主 agent 复跑。
- 没建整池历史、没跑崩溃点枚举；没对拍 checker；没读任务 6526 的日志。
- 冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-opus/frozen-tree`（48M，`du -sh` 量的）没删：交主 agent 跑上一节的命令，已在 `.deliverables` 登「交接」。草稿目录里另有 `mount-diff.txt`（`git diff HEAD -- crates/singlefs-core/src/mount.rs` 的输出，读码用），不是副本。
