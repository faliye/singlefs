# m3-close-c656-r1 正推腿报告（云端正推，Sonnet，2026-10-20）

轮名 m3-close-c656-r1，格 V1–V4。立场：从 `crates/singlefs-core/src/make_filesystem.rs` 今天的写序出发，正推「甲（旧池原样可挂）」与「乙（先作废旧池再写）」各自的写序与崩溃态结局，量出代价。方案按 `crates/` 今天的实现来谈。

数的来源标记：【量】= 本腿这一次的命令跑出、贴了原样输出；【副本】= 调查员副本上的数（报告 `~/.local/share/singlefs/work/m3-close-w8-c758-c656-measure/report.md`，没在入库装置上重做，本腿没重跑，引用时写明）；【推】= 从代码或条款推出，没有跑过。

模型目录 `research/prompts/m3-close-c656-r1-sonnet-model/`（纯 Python）。复跑命令（工作目录随意）：

```
bash scripts/hosts/run-with-memory-cap.sh 1G bash research/prompts/m3-close-c656-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <任一目录> --work <新的草稿目录>
```

末行 `name=rerun_compare same=model-run.txt` 即与 `out/model-run.txt` 逐行相同。三个文件的 sha256：

```
1cc9988b3744f3f59e48756a2622af79afec82fa754e2edff8a24e38dce44643  states_model.py
d16d59c2fe2d64d7a38d9e0c11d9d29ec2c34e211a55f6b328eb9ca82b469d0c  tools/rerun.sh
200a069957db30b68be8cb1f2dc007163007c57c92cc552c78269cf5236612a9  out/model-run.txt
```

## 判定一览（开头版；末尾另有一张同内容的紧凑版）

| 格 | 判定 | 依据 |
|---|---|---|
| V1 甲做不做得到 | **字面的甲做不到**：给不出一个「提交点之前旧池读得到的字节一个都不改」的写序。能做到的最强形是把作废拆成四步单槽清零（本报告叫「甲拆槽」），但它改了系统配置槽，落在乙那一族里，在结局类上比乙没有多出一格 | 新池必写的四类固定结构（系统配置槽、根环槽 0、journal 环、单元区起点）在偏移上与旧池的同类完全重合：读者只收第一版常量（`recovery.rs` 1109、1118、1148 行），所以非覆盖不可；其中只有系统配置槽能让「旧池不可挂」而不留一个坏掉的旧池，它有两盘四槽、没有原子写 |
| V2 乙撑不撑得住 | **撑得住，条件是写序必须是「两块盘各清 [0, 根环基址)、两块盘各过屏障、之后才写别的」两趟**；逐盘做完的写序会重新造出旧池读回没有保证的态 | 作废段只写 [0, 根环基址) 的零，旧池的根、journal、单元一个字节不动；【推】恢复只用系统配置的不可变字段，幸存槽之间这些字段相同，所以挂得上的态读回与最后确认逐字节相同；作废持久之后的 31 186 个态里挂上旧池的 0 个、新旧混合的 0 个（【量】模型枚举，按段模型的构造） |
| V3 代价 | 乙：写 21 次不变，屏障操作 4 步→6 步，操作 25→27，段序列 `14+1+1+1+4`→`2+12+1+1+1+4`，两态闭式 16402→4117，B 型基镜像全量 280 019→31 195、快档 35 075→3 979；`crates/` 里写死这些数的断言 7 处（3 个文件）。甲拆槽：写 23、操作 31、闭式 4118、枚举态数与乙相同。严格的甲没有写序，没有代价可算 | 全部【量】（模型，已对上四个已登记或已量的总数）；断言处数是 grep 数出来的 |
| V4 没量的几何 | 三样（旧池 io_min 不同、fsid 相同、u 取别的值）**都不让乙翻面**，也不让甲变得做得到；它们只改今天写序下的结局分布与各写序的态数 | 【推】作废段只碰 [0, 根环基址)，与槽距、fsid、历史长度无关；态数的敏感度【量】在模型第八节 |

## 先说清楚的三件事

1. 甲与乙的差别，在这份代码里归结为「第一个改旧池读得到的字节的写落在哪一类字节上」。落在系统配置槽上（乙、甲拆槽），部分落地的态只会让旧池「少几个系统配置槽」；落在根环、journal 环、单元区上（今天的写序），部分落地的态会让旧池「丢字节」。V1–V2 的结论都从这一条出。
2. 题面 V2 第一问问的是「今天的 248 832 态里哪些落进作废段」。作废段在今天的写序里不单独存在（[0, 根环基址) 清零与根环、journal 环清零、单元写同在一个 14 次写的段里），所以我把它读成：这 14 个写里只有 [0, 根环基址) 那两次清零落了（一盘或两盘、整落或撕裂）的态。
3. 严格的甲要的是「旧池读得到的任何字节不变」，而作废旧池本身就必须改系统配置槽，所以 V1 的判定落在「甲要么做不到，要么退化成乙的细化」上；我把退化的那一支单列出来比较，不替主 agent 采纳。

## V1　甲做不做得到

### 1.1 旧池恢复读哪些字节（读集合）

恢复一个旧池读四类字节，入口是 `recover`：

- 系统配置槽：每盘读槽 0，再按槽 0 自述的槽距读槽 1，「let slot_zero = mountable_slot_or_refusal(」（`crates/singlefs-core/src/recovery.rs:1398`）；
- 根环全部槽：「let Some(root) = choose_root(reader, &system_configuration) else {」（`crates/singlefs-core/src/recovery.rs:7054`）；
- journal 环：「let records = scan_journal(reader, &system_configuration);」（`crates/singlefs-core/src/recovery.rs:7069`）；
- 所选根指着的单元：「let (outcome, effective_root) = match walk_to_file_judging_the_truncation_fields_as(」（`crates/singlefs-core/src/recovery.rs:7100`）。

### 1.2 新池必写的字节，与旧池读集合的重合

| 类 | 新池今天写在哪 | 位置能不能挪 | 与旧池的关系 |
|---|---|---|---|
| 系统配置槽 | 每盘两槽：「device.write_at(offset, &slot, WriteDurability::Plain)?;」（`crates/singlefs-core/src/make_filesystem.rs:660`）；槽 0 恒在偏移 0：「槽 0 在设备内偏移 0」（`.claude/kb/layout/01-first-txn.md:162`） | 槽 0 不能；槽 1 随槽距 | 同一偏移 0 住着旧池的槽 0；择槽取每盘世代号最大的：`.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位`：「择槽取校验和过且世代号最大的」，新池槽世代号是 1，旧池槽世代号更大，所以旧槽不死，新槽就不生效 |
| 根环槽 0 | 三个区域各自的槽 0 一条第 0 代根：「device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;」（`crates/singlefs-core/src/make_filesystem.rs:629`） | 不能：读者只收第一版根环起点：「if recorded_root_ring_base_slot != ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION {」（`crates/singlefs-core/src/recovery.rs:1109`） | 区域内槽位是`.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位`：「区域内槽位 = (txg div R) mod S」，所以旧池 txg 模 3S 落在 0、1、2 的那条根就住在某个区域的槽 0；那条根若正是旧池最新的一条，写第 0 代根就盖掉它（【推】，公式推出，没造过这个历史） |
| journal 环 | 整环清零：「device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;」（`crates/singlefs-core/src/make_filesystem.rs:563`） | 不能：「if recorded_journal_ring_start_slot != JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION {」（`crates/singlefs-core/src/recovery.rs:1118`） | 旧池的记录就住在这一段；不清则旧池留下的合法记录在同 fsid 下照样自证得过（mkfs 注释写的理由），所以非清不可 |
| 单元区起点三个槽 | 实例表单元与树表单元第 0 版两盘各一份，落在环末尾的下一个槽起 | 起点随环长走：「if recorded_unit_area_start_slot != slot_after_the_ring {」（`crates/singlefs-core/src/recovery.rs:1148`） | 环长相同则起点相同，覆盖的是旧池单元区最前面三个槽；旧池分配器会重用释放过的槽，所以那三个槽是否活着取决于旧池的历史。mkfs 自己没有读盘调用，无法绕开：【量】`make_filesystem.rs` 里读盘调用点数 0（模型第九节，见下） |

【副本】同 io_min 的重用历史里，新 mkfs 的 4 次单元写落在旧池单元区同样的槽上，盖掉旧池还在用的单元（报告第二节「池级 checker 红的几项」那一段）；这一条本腿没重跑。

### 1.3 为什么写不出严格的甲

甲要的是：提交点之前，旧池读集合里的字节一个都不改。把 mkfs 的写序里第一个落在读集合内的写叫「首改写」，它之前的写都落在旧池读不到的字节上，它本身就是提交点。

- 1.2 的四类里，前三类（系统配置槽、根环槽 0、journal 环）的位置是格式常量，读者只收这些常量，新池结构只能落在与旧池同一偏移上，没有「旧池不读的地方」可以先写；第四类（单元区）只在旧池的这三个槽恰好空闲时才不碰旧池，mkfs 不读旧池就不知道，读旧池要把旧池的分配器搬进 mkfs。
- 所以首改写必然落在 1.2 的某一类里。首改写落在根环、journal 环或单元区上，而系统配置槽还完好时，旧池照样挂得上，读回的是被改坏的内容：这就是今天的写序，【副本】两块盘都重用的历史里 248 832 个择到旧池系统配置的态中只有 173 568 个读回与最后确认的 (1, 6) 逐字节相同，6 144 个悄悄读回更早一版，69 120 个恢复失败。
- 首改写落在系统配置槽上，则可以做到「部分落地的态只让旧池少几个系统配置槽」（1.4）。但这一步已经改了旧池读得到的字节，不满足甲字面的「不改动任何字节」。

结论：严格的甲没有写序；卡住它的是系统配置槽（新旧同偏移、新槽世代号 1 压不过旧槽，必须先作废旧槽）与根环槽 0、journal 环（位置是读者只收的常量）。

### 1.4 能做到的最强形：甲拆槽（把作废拆成四步单槽清零）

写序（逐步、逐道屏障；盘 1 先于盘 0，槽 1 先于槽 0；每道屏障只发给刚写的那块盘）：

| 步 | 写 | 屏障 |
|---|---|---|
| 作废 1 | 盘 1 [4096, 根环基址) 清零（盖住槽距 ≥ 4096 的槽 1） | 盘 1 |
| 作废 2 | 盘 1 [0, 4096) 清零（槽 0） | 盘 1 |
| 作废 3 | 盘 0 [4096, 根环基址) 清零 | 盘 0 |
| 作废 4 | 盘 0 [0, 4096) 清零 | 盘 0 |
| 之后 | 与乙的「清零与单元段」起相同：根环三区域与 journal 环清零、单元写 → 屏障 → 三个根 FUA → 四次系统配置写 → 屏障 | |

槽 1 在 [4096, 根环基址) 内：槽距下界与上界是`.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位`：「不小于 4096、512 的整数倍、不越过同一读者收的根环起点减 4096」。

四步各只写零、只落在系统配置槽上，旧池的根、journal、单元不动；一槽被清零或撕裂只有「自证过 / 不过」两种后果。【量】模型第六节枚举（原样）：

```
## 六、甲拆槽 作废段（四步单槽清零）的结局
  步序0 N 槽={0: ('old', 'old'), 1: ('old', 'old')} -> 读=old 可写=old
  步序0 T 槽={0: ('old', 'old'), 1: ('old', 'dead')} -> 读=old 可写=old
  步序1 N 槽={0: ('old', 'old'), 1: ('old', 'dead')} -> 读=old 可写=old
  步序1 T 槽={0: ('old', 'old'), 1: ('dead', 'dead')} -> 读=old 可写=refused:SystemConfigurationUnreadableAfterTheReadStage(old)
  步序2 N 槽={0: ('old', 'old'), 1: ('dead', 'dead')} -> 读=old 可写=refused:SystemConfigurationUnreadableAfterTheReadStage(old)
  步序2 T 槽={0: ('old', 'dead'), 1: ('dead', 'dead')} -> 读=old 可写=refused:SystemConfigurationUnreadableAfterTheReadStage(old)
  步序3 N 槽={0: ('old', 'dead'), 1: ('dead', 'dead')} -> 读=old 可写=refused:SystemConfigurationUnreadableAfterTheReadStage(old)
  步序3 T 槽={0: ('dead', 'dead'), 1: ('dead', 'dead')} -> 读=refused:NoValidSystemConfiguration 可写=refused
  汇总(读挂载结局): {'old': 7, 'refused:NoValidSystemConfiguration': 1}
  最后一个槽的清零取 T 的盘面 = 四槽全死 = 整池作废（这一步就是提交点），其余每一格都挂上旧池

```

每一步之后（清到最后一个槽之前），盘 0 的两槽里至少有一个旧槽，读挂载择到的系统配置是盘 0 幸存的那一槽（可能是较旧的一槽，见 1.5）；最后一个槽的清零一落（含撕裂），四槽全死，整池拒绝：这一步就是提交点，提交点之后与乙同问（V2）。
任意扇区子集（不限于崩溃点重放的撕裂模型）也一样：两槽各活或死共 16 种盘面，15 种读挂载挂上旧池、1 种（四槽全死）整池拒，没有第三种。【量】模型第六之二节：

```
## 六之二、任意扇区子集的作废盘面（两槽各活/死，16 种）
  汇总(读挂载结局): {'old': 15, 'refused:NoValidSystemConfiguration': 1}
  汇总(可写挂载结局): {'old': 9, 'refused': 1, 'refused:SystemConfigurationUnreadableAfterTheReadStage': 6}

```

### 1.5 甲拆槽比乙多了什么、少了什么

- 结局类上没有多：作废段的所有态里，甲拆槽 8 个（四步各 2 个）、乙 8 个（两盘各 N / T / F 的 3 × 3 去掉全落），读挂载全部挂上旧池，只差在哪几槽还活着（乙的模型第四节、甲拆槽的第六节，上面和 V2 里原样贴出）。
- 幸存槽不一定是该盘较新的一槽：甲拆槽固定「槽 1 先于槽 0」，旧池较新的一槽若恰是槽 1，清到盘 0 的槽 1 之后盘 0 只剩较旧的槽 0。要保证「幸存槽是最新的」，mkfs 得先读旧槽比世代号；mkfs 今天一次读盘都没有（【量】模型第九节：`make_filesystem.rs` 里读盘调用点数 0），加读还要处理槽 0 读不出时的逐档读（`recovery.rs` 里 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`），为的是一个只在崩溃态出现的性质。
- 这个性质对读回不起作用（【推】）：恢复择好系统配置之后，根、journal、单元的读法只用不可变字段（几何、fsid）：「let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);」（`crates/singlefs-core/src/recovery.rs:7096`）；运行量（journal tail、回退下界）只在所选根走读失败时才进来：「// 走读失败才再扫一遍环拿记录给退回的判法（只读、不写盘；走读做成的恢复不多扫）。」（`crates/singlefs-core/src/recovery.rs:7115`）。作废段不碰根与单元，走读不会失败；【副本】同一类现象已有一份对照：旧池系统配置世代号 7 与 8 两种择法，27 648 态各自读回 (1, 6)（报告第二节「先复现 55 296 / 27 647」一段，旧池 io_min 不同、根环没被清到的那一份历史，不是同 io_min 的）。
- 多出的代价：写 23（乙 21）、屏障操作 8 步（乙 6 步）、操作 31（乙 27）。

### V1 判定

**字面的甲：做不到。** 卡在系统配置槽（新旧同在偏移 0，新槽世代号 1 压不过旧槽，必须先作废旧槽，而旧池的两盘四槽没有原子改写），以及根环槽 0 与 journal 环（位置是读者只收的第一版常量，新池结构只能与旧池同偏移）；单元区看旧池分配，mkfs 不读盘，无从避开。
**退化的甲（甲拆槽）：能做到**，但它改系统配置槽，属乙那一族；在结局类上不比乙多一格，只多 2 次写、2 步屏障操作，条件是同样的两趟写序。

什么现象会推翻它：
- 推翻「字面的甲做不到」：给出一个写序，它的首改写落在旧池读不到的字节上，例如证明新旧池的根环起点或 journal 起点可以错开（要先改 `recovery.rs` 1109、1118 行把它们封死的判，属于格式改动）；
- 推翻「甲拆槽在结局类上不比乙多」：单态复跑今天写序下只落了清零的那 9 个态（序号 0–2、162–164、324–326，命令在末尾「要主 agent 跑的命令」）读回不是 (1, 6)，或读挂载没有挂上旧池。

## V2　乙撑不撑得住

### 2.1 乙的写序要求（对 `make_filesystem.rs` 的改动，草稿，未改仓）

今天（`make_filesystem.rs` 里）：「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」（`crates/singlefs-core/src/make_filesystem.rs:559`）与根环、journal 环清零在同一个逐盘循环里。乙要把它提出来单独成一趟，再加一趟屏障，之后才进原来的循环：

```
// 草稿：mkfs 第一步，作废旧池的系统配置
for (_, device) in devices.iter_mut() {
    device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;   // 两块盘各清 [0, 根环基址)
}
for (_, device) in devices.iter_mut() {
    device.barrier()?;                                                   // 两块盘各过屏障，之后才写别的
}
for (_, device) in devices.iter_mut() {                                  // 原来的循环，去掉 [0, 根环基址) 那一句
    for region_to_clear in 0..ROOT_RING_REGIONS { /* 根环三区域清零 */ }
    /* journal 环清零 */
}
/* 其余与今天相同：单元写 → 屏障 → 三个根 FUA → 四次系统配置写 → 屏障 */
```

**循环结构不是可选项**：写序必须是「所有盘的作废写 → 所有盘的屏障 → 别的写」。若写成一块盘一块盘做完（盘 0：作废、屏障、其余清零；再盘 1：同样），段模型把它切成四段，盘 0 的根环清零所在的段早于盘 1 的作废所在的段，于是存在「盘 1 的旧系统配置还活着、盘 0 的根环与 journal 镜像已清」的态：旧池挂得上，读回只能靠盘 1 上的根与 journal 镜像补，没有保证完好（今天那 6 144 类悄悄回退是两盘都清了的同类现象）；这是【推】，没有造过这个态。【量】模型第十节（原样）：

```
## 十、写序的循环结构：先两块盘各清零、再两块盘各过屏障，与逐盘做完的差别
  两趟写序的段： [['Z0@d0', 'Z0@d1'], ['ZR@d0', 'ZR@d1']]
  逐盘写序的段： [['Z0@d0'], ['ZR@d0'], ['Z0@d1'], ['ZR@d1']]
ok   两趟写序：两块盘的作废都严格早于别的清零: got=True want=True
ok   逐盘写序：盘 0 的根环清零早于盘 1 的作废（盘 1 的旧系统配置还活着时盘 0 的根环已经清了）: got=False want=False

```

### 2.2 作废那一步之前崩溃：今天的 248 832 态里哪些落进这一段，结局是什么

今天的写序里，14 个写（每盘 5 次清零加 2 次单元写）在同一个段里，段内取任意组合；每次清零在 B 型基镜像（两块盘都重用、同槽距、journal 环有旧记录）上取三态，【量】对上已量的总数（模型第一节 280 019、第二节今天的行）。「只有 [0, 根环基址) 那两次清零落了」的态：两盘各 N / T / F 共 9 个，序号按混合进制拆（模型第七节，原样）：

```
## 七、今天写序下"只落了 [0,根环基址) 清零"的 9 个态的序号（B 型基镜像，全量混合进制，写表里最前那次写是最低位）
   [0, 1, 2, 162, 163, 164, 324, 325, 326] (数位 0 没持久 1 撕裂 2 持久；盘 0 清零下标 0 盘 1 清零下标 5 )
ok   9 个序号: got=[0, 1, 2, 162, 163, 164, 324, 325, 326] want=[0, 1, 2, 162, 163, 164, 324, 325, 326]

```

9 个里，两盘都整落（序号 326）的一个没有任何系统配置槽，整池拒；其余 8 个（含序号 0 的起点）读挂载挂上旧池，它们就是乙的作废段全部的态。这 8 个在今天 248 832 个「择到旧池系统配置」的态里占 8 个（模型的构造，没有逐个对 `runs/` 里的聚合行，那些行按结局类聚合、不带序号，所以对不出来）。【量】模型第四节（原样）：

```
## 四、乙作废段（作废那一步没持久完）的结局，B 型基镜像
  盘0=N 盘1=N 槽={0: ('old', 'old'), 1: ('old', 'old')} -> 读挂载=old 可写挂载=old
  盘0=N 盘1=T 槽={0: ('old', 'old'), 1: ('dead', 'old')} -> 读挂载=old 可写挂载=old
  盘0=N 盘1=F 槽={0: ('old', 'old'), 1: ('dead', 'dead')} -> 读挂载=old 可写挂载=refused:SystemConfigurationUnreadableAfterTheReadStage(old)
  盘0=T 盘1=N 槽={0: ('dead', 'old'), 1: ('old', 'old')} -> 读挂载=old 可写挂载=old
  盘0=T 盘1=T 槽={0: ('dead', 'old'), 1: ('dead', 'old')} -> 读挂载=old 可写挂载=old
  盘0=T 盘1=F 槽={0: ('dead', 'old'), 1: ('dead', 'dead')} -> 读挂载=old 可写挂载=refused:SystemConfigurationUnreadableAfterTheReadStage(old)
  盘0=F 盘1=N 槽={0: ('dead', 'dead'), 1: ('old', 'old')} -> 读挂载=old 可写挂载=refused:SystemConfigurationUnreadableAfterTheReadStage(old)
  盘0=F 盘1=T 槽={0: ('dead', 'dead'), 1: ('dead', 'old')} -> 读挂载=old 可写挂载=refused:SystemConfigurationUnreadableAfterTheReadStage(old)
  汇总(读挂载结局): {'old': 8}
  汇总(可写挂载结局): {'old': 4, 'refused:SystemConfigurationUnreadableAfterTheReadStage': 4}
ok   乙作废段态数 = 3x3-1: got=8 want=8

```

（N = 这次写没落，T = 撕裂，F = 整落。T 的读法：「let first_old_byte = first_differing_byte + differing_span_in_bytes / 2;」（`crates/singlefs-checker-tier/src/crash.rs:2272`），新旧不同的那一截前一半清零、后一半留旧，槽 0 整个落在前一半、槽 1 在后一半，所以模型里 T = 槽 0 清了、槽 1 留着。这只是崩溃点重放的撕法；真设备可以落任意扇区子集，两槽各活或死共 16 种，15 种挂旧池、1 种整池拒，见 1.4 末尾，结论不变。）

- **读回**【推】：作废段只写 [0, 根环基址) 的零：`.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 8：固定结构的放置`：「恰盖住系统配置槽 0 与逐档找槽 1 的全部 2 033 档」，旧池的根环、journal 环、单元一个字节不动。恢复择到任何一个幸存的旧槽之后，读法只用不可变字段（几何、fsid；见 1.5 引的 `recovery.rs` 7096、7115 行），所以读回与最后确认的版本逐字节相同。【副本】旁证：起点镜像（一个写都没落）`read_only=Ok(i1t6)`、`writable=Ok(i1t6)`（报告 `runs/mkfs-baselines.log`）。
- **可写挂载**：8 个里 4 个做成（每盘至少剩一槽），4 个拒（某盘一份自证过的槽都没有）：「return Err(device);」（`crates/singlefs-core/src/mount.rs:5731`）。【副本】旁证：第二块盘本来就是空盘的起点，`writable_with_old_parameters=Err:SystemConfigurationUnreadableAfterTheReadStage`（同一份日志）。拒的是可写，读挂载照挂。
- **R2（判不出）与按盘读**：零和撕裂的槽设备不报读错，R2 只在「有槽设备报读错」时判不出：「撕裂的系统配置轮换（读得出、自证不过，设备没报错）不算判不出；一块盘报读错而另一块盘两槽都自证过也不算。」（`crates/singlefs-core/src/recovery.rs:4304`）。这 8 个态没有读错，R2 不参与。
- **幸存槽可能是较旧的一槽**（例如盘 0 槽 0 被撕、槽 1 是较旧世代）：这是平时「每盘一槽坏」的单故障形，可写挂载的落后判里有这一格：「或只剩一槽自证、那一槽停在上一次轮换）；记下了至少一个才交回 (那份的 tail, 缺的」（`crates/singlefs-core/src/mount.rs:5787`），只落后、单元都在则放行。

### 2.3 作废持久之后：一盘作废一盘没作废，会不会择到旧池系统配置

- **一盘作废、一盘没作废的态只存在于作废段内**（上面 4 个只读态和 T 的几个态）。段模型的构造：当前段每块有写的盘都被自己的屏障放行时这一段才关上，之后的段里前面的段全部持久（D13 已定项 4：`13-验证路线.md@已定项 4：崩溃点重放只枚举整写子集`）。两盘的作废写同在作废段，两道屏障都发完它才关；所以从「其余清零与单元段」起，每个态里两盘的旧系统配置槽都已死。
- 因此作废持久之后没有任何态能择到旧池系统配置：清零与单元段 31 103 个态、三个根 FUA 的 3 个态，挂载都是 `NoValidSystemConfiguration`（【量】模型第三节段拆开的数；结局【推】：[0, 根环基址) 全零，槽 0 与逐档找的槽 1 都读不出自证过的槽：「(None, Some(device)) => Err(RecoveryFailure::NoValidSystemConfiguration {」（`crates/singlefs-core/src/recovery.rs:1472`））。
- **fsid 筛（S3）不参与**：新旧两种活槽在任何一个态里都不并存（新系统配置只在最后一段写，那时旧槽全死）。模型的 `mount_outcome` 对每个枚举出来的态断言「活槽只有一种 fsid」，这是写序的构造，不是量出来的。今天没有这一筛，两盘择到的 fsid 对不上才报 `SystemConfigurationsDisagree`：「return Err(RecoveryFailure::SystemConfigurationsDisagree);」（`crates/singlefs-core/src/recovery.rs:1448`），这一格走不到；S3 落地与否不改 V2。
- **新系统配置段 80 个态**（作废已持久，四次新槽写各 N / T / F）【量】模型第五节（原样）：

```
## 五、乙与今天共有的新系统配置段（四次新系统配置写，作废已持久）
  汇总(读挂载结局): {'new': 64, 'refused:NoValidSystemConfiguration': 16}
  汇总(可写挂载结局): {'new': 24, 'refused': 16, 'refused:SystemConfigurationUnreadableAfterTheReadStage': 40}
ok   新系统配置段态数 3^4-1: got=80 want=80

```

读法：16 个态四个槽都没落成自证过的新槽，整池拒绝；64 个态至少一个新槽落成，读挂载挂上**新池**，其中 24 个每盘都有自证过的槽、可写挂载也做成，40 个有一盘一个新槽都没有、可写挂载拒（读挂载照挂）。旧池、新旧混合：0。
「完整的新池」在这里指内容完整（根、journal、单元在这一段之前都已持久：三个根 FUA 各自成段，单元由前面的屏障隔开），系统配置冗余可能缺。乙的题面写「整池拒（没有自证得过的系统配置）或挂上完整的新池」；那 40 个态是「新池内容完整、冗余缺一盘」，算不算「完整」，条款没写，交主 agent。

### 2.4 整池拒之后用户看到什么、能不能直接再跑 mkfs

- 错误：挂载（读和可写）报 `MountError::Recovery(RecoveryFailure::NoValidSystemConfiguration { first_device_with_no_valid_system_configuration_slot })`，带次序里第一块盘（盘 0）：「MountError::Recovery(failure)」（`crates/singlefs-core/src/mount.rs:895`）。【副本】今天写序下同一类结局的聚合行：`choice=Err:NoValidSystemConfiguration recover=Failed:NoValidSystemConfiguration raw_red=["I-8.5"] read_only=Err:Recovery`（报告 `runs/mkfs-full-B-0.log`）。这条错误对用户的意思是「盘上没有系统配置」，没有「mkfs 没做完」的字样；要不要加专名，条款没写。
- 直接再跑 mkfs：可以。`make_filesystem` 不读盘（【量】读盘调用点数 0），几何判只看盘的字节数：「let unit_area_start = check_geometry(parameters, &sizes)?;」（`crates/singlefs-core/src/make_filesystem.rs:485`）；所以对作废态、对半写完的新池，重跑都从头清零开始。旧池的根、journal、单元留在盘上直到被重跑清掉，单元区除开头三个槽外不清（旧内容不可达，不影响读法）。

### 2.5 要带着的条件与旁注

- 两趟写序（2.1）。
- 作废态里旧池的根、journal、单元都在，只缺系统配置这道闸。在飞的 α' 离线出路（批 8 系统配置落地，`records/2026-10-18-里程碑三收尾与近期欠账.md` 批 8 那一条）若落地、带扫描，它在这些态上读到的会是旧池。`crates/` 今天没有：
```
$ grep -rniE 'offline|离线' crates/singlefs-core/src/*.rs | head -8
(退出码 1，零命中)
```
- 用户 2026-10-20 的前提是「不接受挂上丢了已确认写的旧池」。乙在作废段内挂上的旧池没有丢写（上面 2.2），作废持久之后不挂旧池。乙付出的是：作废一落地，旧池就不能再由挂载读回——想要旧池的人，作废之后只能靠离线工具（今天没有）。

### V2 判定

**撑得住**，条件：两趟写序。作废段 8 个态全部挂上不丢写的旧池（可写 4、只读 4）；作废持久之后的 31 186 个态挂上旧池 0、新旧混合 0，整池拒 31 122（31 103 + 3 + 16），挂新池 64。「新池内容完整、冗余缺一盘」的 40 个态算不算条款里的「完整的新池」，规则没说。

什么现象会推翻它：
- 作废段内有一个态读回的不是 (1, 6)（序号 0–2、162–164、324–326 单态复跑；命令见末尾）；
- 作废持久之后有一个态读挂载挂上旧池（需要一个态里旧槽还活着而别的清零已落，等于两趟写序被破坏，或真设备的屏障没把作废写落盘，那是 C6 的范围，不在这个模型里）；
- 恢复读回依赖幸存槽是哪一槽（即 `recover` 的读回会随世代号变）。【副本】27 648 态世代号 7 与 8 都读回 (1, 6) 是这条推论唯一的实测旁证，且那一份历史的根环没被清到。

## V3　代价

### 3.1 写序与录制流（B 型基镜像）【量】模型第二节，原样

```
## 二、B 型基镜像（两盘重用、同槽距、journal 环有旧记录）三种写序的写表
今天: 段序列=14+1+1+1+4 写=21 操作=25 全量态数=280019 快档甲二态数=35075
乙: 段序列=2+12+1+1+1+4 写=21 操作=27 全量态数=31195 快档甲二态数=3979
甲拆槽: 段序列=1+1+1+1+12+1+1+1+4 写=23 操作=31 全量态数=31195 快档甲二态数=3979
ok   今天 段序列: got=14+1+1+1+4 want=14+1+1+1+4
ok   乙 段序列: got=2+12+1+1+1+4 want=2+12+1+1+1+4
ok   甲拆槽 段序列: got=1+1+1+1+12+1+1+1+4 want=1+1+1+1+12+1+1+1+4
两态闭式：今天=16402 乙=4117 甲拆槽=4118

```

| 项 | 今天 | 乙 | 甲拆槽 | 严格的甲 |
|---|---|---|---|---|
| 写次数 | 21 | 21 | 23（作废拆成 4 次） | 没有写序 |
| 屏障操作步数（录制器按设备记，池屏障每道两盘各一步） | 4 | 6 | 8 | |
| 操作数 | 25 | 27 | 31 | |
| 段序列 | `14+1+1+1+4` | `2+12+1+1+1+4` | `1+1+1+1+12+1+1+1+4` | |
| 两态闭式（登记表口径） | 16402 | 4117 | 4118 | |
| 崩溃点重放全量态数 | 280 019 | 31 195 | 31 195 | |
| 快档（甲二）态数 | 35 075 | 3 979 | 3 979 | |
| 「第一道屏障之前」的态数 | 279 935 | 8（作废段） | 8（四步各 2） | |

乙的种类串（【推】由段序列推出，没有跑录制流）：`[zero_fill×2,barrier×2]|[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]`。

### 3.2 `make_filesystem.rs` 的改动

- 乙：把「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」（`crates/singlefs-core/src/make_filesystem.rs:559`）提成单独一趟，后面加一趟 `device.barrier()?`，原来的循环去掉这一句（草稿在 2.1）。写次数不变，池屏障趟数 2 → 3。
- 甲拆槽：作废拆成 4 次写、4 道单盘屏障，写次数 21 → 23；
- 注释要跟着改：文件头「//! 写序（字节表零 m1..m4；段序列 `12+1+1+1+4`）：」（`crates/singlefs-core/src/make_filesystem.rs:3`）本来就落后于已登记的 `14+1+1+1+4`（那一行写 12，写序里已有每盘 [0, 根环基址) 那一次清零）；「// 次序与屏障：清零按设备内偏移升序发（[0, 1 MiB)，根环 1 / 4 / 7 MiB，journal 环 16 MiB），几段之间不另加屏障——」（`crates/singlefs-core/src/make_filesystem.rs:540`）与「不为一个没有条款的语义加屏障」（`crates/singlefs-core/src/make_filesystem.rs:544`）两处，在有了崩溃条款之后都不再成立。

### 3.3 `crates/` 里写死 mkfs 段数或步数的处数

【量】grep（命令与原样输出；长行截到 170 字符）：

```
$ grep -nF -e '"14+1+1+1+4"' -e '"[zero_fill×10,' crates/singlefs-harness/tests/new_pool_file_creation_publish.rs crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs | cut -c1-170
crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:364:        (25, "14+1+1+1+4".to_string(), 16402)
crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:368:        "[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua
crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:280:        "14+1+1+1+4",
crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:285:        "[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_f
$ grep -nE '^ +25,$|operations: 25|vec!\[25, 2, 20, 35, 57\]' crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs | cut -c1-170
crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:289:        25,
crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:294:        StreamIntegrity::Consistent { operations: 25 }
crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2679:            vec![25, 2, 20, 35, 57],
$ grep -nF '(25, "14+1+1+1+4".to_string(), 16402)' crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
364:        (25, "14+1+1+1+4".to_string(), 16402)
```

7 处断言、3 个文件：
- 「(25, "14+1+1+1+4".to_string(), 16402)」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:364`）：操作数、段序列、闭式三元组；
- 「[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:368`）：种类串；
- 「"14+1+1+1+4",」（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:280`）：段序列；
- 「[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]」（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:285`）：种类串；
- 「25,」（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:289`）：操作数；
- 「StreamIntegrity::Consistent { operations: 25 }」（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:294`）：流完整性的操作数；
- 「vec![25, 2, 20, 35, 57],」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2679`）：五段操作数的第一个 25。

乙要改成 27、`2+12+1+1+1+4`、4117 与新种类串；甲拆槽要改成 31、`1+1+1+1+12+1+1+1+4`、4118。这些是 grep 数出来的，不含写成算式、或用别的形态钉 mkfs 步数的地方；我翻过的几份崩溃点重放用例（`crash_segments_per_device_and_torn_in_place_overwrites.rs`、`in_place_overwrite_torn_state_count.rs`）都从 mkfs 之后的基镜像起、`mkfs_operation_count` 现取，不钉 mkfs 本身；没有逐份翻完全部几十份用例。

`crates/` 之外还钉着同一组数的地方（【量】grep，命令与原样输出）：

```
$ grep -rl -e 'zero_fill×10' -e '14+1+1+1+4' scripts .claude
scripts/experiments/e142_new_pool_file_creation_dry_run.rs
scripts/crash/check-segment-registry-product-sample.txt
.claude/gate.d/checker-tier-qemu-device-streams.sh
.claude/gate.d/fixtures/checker-tier-qemu-device-streams.sh/mkfs-segments-red/.claude/kb/layout/01-first-txn.md
.claude/gate.d/fixtures/checker-tier-qemu-device-streams.sh/PROVENANCE.txt
.claude/gate.d/fixtures/checker-tier-qemu-device-streams.sh/mkfs-segments-red/expect
.claude/kb/experiments/142-新池新建文件的干跑.md
.claude/kb/checks-owed.md
.claude/kb/checks-owed.ja.md
.claude/kb/experiments-history.md
.claude/kb/checks-owed.en.md
.claude/kb/layout/01-first-txn.md
.claude/kb/milestone/02-second-txn.md
.claude/kb/layout/01-first-txn.ja.md
.claude/kb/layout/01-first-txn.en.md
.claude/kb/milestone/02-second-txn.en.md
.claude/kb/milestone/02-second-txn.ja.md
```

其中有登记表本身（`.claude/kb/layout/01-first-txn.md` 八，三种语言各一份）、E142 的独立手写装置（`scripts/experiments/e142_new_pool_file_creation_dry_run.rs`，写死这组数的行有 2280、5662、8712、8730、8731、8851、8868 等处）、门禁 doc-registries 的产物样本与门禁 checker-tier-qemu-device-streams 的样本。装置是独立模型，按「装置的数与实现对不上时两边都查」处理（`.claude/rules/implementation-first.md`），改完要重出产物；门禁 checker-tier-qemu-device-streams 拿虚机录到的 mkfs 那一行与登记表比，虚机要重录（重型，归主 agent）。

### 3.4 D22 已定项 8 第 3 条要改的句子

现行条款的相关句（整句，原文在 `22-单元原子性怎么合成.md@已定项 8：固定结构的放置` 第 3 条）：

- `.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 8：固定结构的放置`：「清零几段之间不加屏障」：乙要改成「[0, 根环基址) 那一段清零之后每块盘过一道屏障、两块盘都过了才写别的；其余清零与单元写之间不加屏障」，并把崩溃结局写进去：作废段之前挂上旧池（旧池读回与最后确认的版本相同）、作废持久之后整池拒或挂上新池（草稿，措辞待主 agent 定）。
- 「每块盘 5 步、两块盘共 10 步」（同一节）：乙不变（作废仍是每盘一次调用）；甲拆槽改成每盘 6 步、两块盘共 12 步。
- 「mkfs 中途崩溃（第一道屏障之前）该挂上旧池、整池拒还是别的，没有条款」（同一节「射程」）：替换成上一条的结局条款；「欠」里的 C656 一项随之收掉。
- 同样的句子在布局表里：「mkfs 写元数据之前把每块盘 [0, 根环基址) 整段写 0」（`.claude/kb/layout/01-first-txn.md:162`）要加「并过一道屏障」；登记表那一行：「mkfs 种根」（`.claude/kb/layout/01-first-txn.md:410`）按 3.1 的数改，三种语言各一份。
- `.claude/kb/checks-owed.md` 里 C656 那一行：「mkfs 中途崩溃的语义没有条款」（`.claude/kb/checks-owed.md:429`）的「条款写下 mkfs 中途崩溃该怎样之后」那一格，由 3.5 的验收格还。

严格的甲：没有写序，没有句子可改。

### 3.5 C656 验收那一格（在用过的盘上枚举 mkfs 屏障之前的崩溃态，结局与条款不同判红）各要枚举多少态

`crates/` 里没有这一格的用例：【量】

```
$ awk -F'\t' '$1 ~ /mkfs|make-filesystem|make_filesystem/' .claude/gate.d/crash-point-replay-items.tsv | wc -l
0
```

登记的崩溃点重放项里没有 mkfs 的项；现有两条重做 mkfs 的用例只判做完之后的终态（`remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader`、`remaking_the_pool_with_the_same_filesystem_identifier_leaves_no_root_of_the_old_pool`）。要加这一格，得新写一个「旧池历史 → 在同一套盘上第二次 mkfs 的录制流 → 以旧池镜像为基镜像枚举」的装置；调查员副本里打过这样的补丁（`m3-close-w8-c758-c656-measure/tools/patch_copy.py`），入库的没有。

两支下这一格要枚举的态数（B 型基镜像，【量】模型第二、三、八节）：

| 范围 | 今天 | 乙 | 甲拆槽 |
|---|---|---|---|
| 只枚举「第一道屏障之前」 | 279 935（全量）/ 34 991（快档） | 8 / 8 | 8 / 8 |
| 枚举整条 mkfs 到末尾（含作废持久之后的拒与新池） | 280 019 / 35 075 | 31 195 / 3 979 | 31 195 / 3 979 |
| 旧池只做过 mkfs（journal 环全 0），整条 | 124 499 / 15 635 | 13 915 / 1 819 | 13 915 / 1 819 |

乙下这一格要判的结局类（【推】，来自 V2）：作废段 8 个态读回 (1, 6) 且挂旧池（可写 4、只读 4）；之后 31 122 个整池拒；64 个挂新池。结局与条款不同判红。快档与全量在作废段上相同（两个原地写、没有单元写，3 × 3 − 1 = 8），所以平时快档就能把这一格判全。

### V3 判定

量出了代价，没有哪一支被代价排除：乙写次数不变、多一趟屏障，崩溃态数从 28 万降到 3.1 万；甲拆槽多 2 次写 2 步屏障、态数同乙；严格的甲无写序。7 处断言、`make_filesystem.rs` 三处注释、D22 已定项 8 与布局表三语、E142 装置与两处门禁样本要跟着改。

什么现象会推翻它：模型的态数公式对不上入库枚举器的实际输出（公式已对上 16402、82 970、280 019、279 974 四个已登记或已量的总数；对不上拆法的情形是计数器分不出哪几次写取三态，但这四个总数里三态写的个数各不相同，拆法被它们逐一约束住了）；单态复跑的序号落点与模型的混合进制不符（模型对上了【副本】最小复现的序号 11780）。

## V4　没量的几何会不会翻面

【量】的只有态数（模型第八节，原样）；结局都是【推】。

```
## 八、敏感度：基镜像变了，乙与今天的态数怎么动（全量 / 快档）
B 型（journal 环有旧记录）: 今天 280019 / 35075；乙 31195 / 3979；甲' 31195 / 3979
旧池只做过 mkfs（journal 环全 0）: 今天 124499 / 15635；乙 13915 / 1819；甲' 13915 / 1819
旧池 io_min 65536、两盘都重用: 今天 279974 / 35030；乙 31150 / 3934；甲' 31148 / 3932
只重用盘 0（旧池 io_min 65536）: 今天 82970 / 10394；乙 13855 / 1759；甲' 13855 / 1759

```

（基镜像的写法是我设的：「旧池 io_min 65536、两盘都重用」把槽 1 放在 65536 处、新槽距 4096 处槽 1 下没有旧内容；「只做过 mkfs」把 journal 环当全 0。这些是计数假设，不是量出的盘面。）

| 没量的 | 乙 | 严格的甲 / 甲拆槽 | 今天的写序 |
|---|---|---|---|
| 旧池 io_min 与新池不同，两盘都重用 | **不翻面**。作废段只写 [0, 根环基址) 的零，槽 1 在旧槽距处，而槽距的上界就是`.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位`：「槽 1 整槽落在同一槽自述的根环起点之前」，旧槽 1 一定在被清的范围里。旧槽距比新槽距大时新 mkfs 的根环清零（长 S × 新槽距）够不到旧池在远处的根，但那时旧池已不可挂，新池按新槽距只读 S 个槽，读不到它们 | 不翻面：槽 0 恒在偏移 0、根环与 journal 起点是常量，与 io_min 无关，1.2 的四类还是重合；甲拆槽同乙 | 结局分布会变：【副本】只重用盘 0、旧槽距 65536 那份，55 296 个择到旧池的态读回全是 (1, 6)（根没被新槽距的清零够到）；两盘都重用的这一格没量，不能外推 |
| fsid 相同，两盘都重用 | **不翻面**。作废持久之前，盘上没有新池的任何槽，fsid 相同与否进不了任何判；持久之后旧根在同 fsid 下仍自证得过，但没有系统配置就挂不上，且根环清零在根 FUA 之前隔着原有那道屏障（C484 的理由，`make_filesystem.rs` 的注释），新系统配置出现时旧根已清 | 不翻面；相反，同 fsid 说明「先写新系统配置再清别的」这条路更不行：新配置一落，旧根在新 fsid 下仍验得过，得到新旧混合 | 屏障之前新池只落了单元写，没有带 fsid 的槽或根，结局类应与 fsid 不同时相同（【推】；【副本】A 历史 `fsid=same` 与 `fsid=diff` 两行的 `states_with_torn` 都是 82970，`runs/mkfs-count-full.log` 前两行） |
| u 取别的值（旧池的历史长度） | **不翻面**：读回与 u 无关（作废段不碰根与 journal）；u 只改态数——journal 环从没写过记录时那两次清零不取三态（上表第二行） | 不翻面；u 大到根环转过一圈（txg ≥ 3S）时，旧池最新的根可能恰在某区域的槽 0，1.2 的根环那一类更肯定非覆盖不可 | 结局分布会变（日志环里有没有可补的记录、根在不在被清的槽里），没量 |

题面没列、紧挨着的一样：旧池 journal 环长与新池不同。环更长时新池的 journal 环清零会伸进旧池的单元区，环更短时新池的两个单元写会落进旧池 journal 环里。乙：这些清零和单元写都在作废持久之后，旧池已不可挂，不翻面。严格的甲：单元区起点随环长走（「if recorded_unit_area_start_slot != slot_after_the_ring {」（`crates/singlefs-core/src/recovery.rs:1148`）），环长不同只让单元区那一类的重合与否变成取决于环长，其余三类照旧重合，不翻面。

### V4 判定

**三样都不翻面。** 乙的结论只用到「作废段只写 [0, 根环基址) 的零」这一点，它不随槽距、fsid、历史长度变；严格的甲的障碍是格式常量，同样不随它们变。会变的是今天写序下的结局分布（这三样都没量，不能外推）和各写序的态数（上表）。

什么现象会推翻它：一个旧池，它的系统配置槽 1 落在 [0, 根环基址) 之外（读者不收这样的槽，上界由 `recovery.rs` 的 `largest_fixed_structure_slot_spacing_under_the_root_ring_base` 定，但更早版本写出的池没查）；或一个态里作废段的零落进了 [0, 根环基址) 之外。

## 要主 agent 跑的命令（本腿没编译 Rust，下面都没跑）

1. 单态复跑今天写序下「只落了作废写」的 9 个态（B 型历史），验证 2.2 的 8 + 1 与读回 (1, 6)。先在新草稿目录里建出探针（调查员的 `rerun.sh` 前半：拷树、补丁、编；它按件跑，没有只编的件，用最小的 c656-a 顺带建）：
   `bash ~/.local/share/singlefs/work/m3-close-w8-c758-c656-measure/tools/rerun.sh /home/fy5090/code/singlefs <新草稿目录> c656-a`（工作目录仓根；线程 31；c656-a 本身 82 970 态，约几分钟，调查员原话「本机负载低时全跑约 25 分钟」是五件加起来的）。
   再逐段跑：
   `PROBE_EXPANSION=full bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 2G <新草稿目录>/target/release/c758-c656-probe mkfs 'W:x=512:y=512:u=5:reuse=both:fsid=diff' <起> <止>`，`<起> <止>` 取 `0 3`、`162 165`、`324 327`。
   看 `name=sig` 行：序号 0–2、162–164、324–325 应 `choice=old@…`、`recover=FileRead(i1t6):old_last_same_root`、`read_only=Ok(i1t6)`；序号 326 应 `choice=Err:NoValidSystemConfiguration`。
2. 乙整条流在入库装置上的枚举：在上面那棵树的 `crates/singlefs-core/src/make_filesystem.rs` 里照 2.1 的草稿改写序，重编探针，跑 `mkfs 'W:x=512:y=512:u=5:reuse=both:fsid=diff' 0 31195`。应得：作废段 8 态全挂旧池且读回 (1, 6)；31 122 态 `Err:NoValidSystemConfiguration`；64 态 `new@…`；挂旧池的态在作废持久之后 0 个。探针按「择到旧池 / 新池 / 拒」分类，不带「新池内容完整」这一档，40 个缺一盘配置的新池态要自己看 `writable` 一列。

## 模型复跑与哈希

模型目录 `research/prompts/m3-close-c656-r1-sonnet-model/`，纯 Python，不编译、不读盘镜像；`--repo` 只用来核 `make_filesystem.rs` 的三处源码事实。

```
bash scripts/hosts/run-with-memory-cap.sh 1G bash research/prompts/m3-close-c656-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本或任一目录> --work <新的草稿目录>
```

末行应为 `name=rerun_compare same=model-run.txt`（与 `out/model-run.txt` 逐行相同）。本腿跑过一次，退出码 0。
模型会红的证明：把 journal 环当成两态（少两个三态）后，B 型总数变成 124 499，对 280 019 红；把乙写成逐盘做完的次序，第十节的检查红（`False`）。两处都是本腿用 Python 在草稿里改参数现跑的，输出：`broken(journal 环当两态) 124499 want 280019 -> False`、`逐盘写序判安全: False`。

```
1cc9988b3744f3f59e48756a2622af79afec82fa754e2edff8a24e38dce44643  states_model.py
d16d59c2fe2d64d7a38d9e0c11d9d29ec2c34e211a55f6b328eb9ca82b469d0c  tools/rerun.sh
200a069957db30b68be8cb1f2dc007163007c57c92cc552c78269cf5236612a9  out/model-run.txt
```

## 没做什么

- 没编译 Rust、没跑 mkfs 探针、没跑任何重型测试（派发提示「重型测试：不跑」，共用约束也不让子 agent 编译）。标【推】的结局没有在入库装置上跑过；单态复跑与乙整条流的枚举写在「要主 agent 跑的命令」里。
- 没重跑【副本】的数（248 832、173 568、6 144 等）。我只用公式对上了它的总数（280 019、82 970、279 974）和一个序号（11780），没有对它的结局分布；那些聚合行不带序号，也对不到具体的态上。
- 没判采纳哪一支；D22 已定项 8 的句子、布局表、验收格都只写了「要改哪里」与草稿措辞，没动 kb，没动 `crates/`。
- 没判：乙的作废段 8 个态上池级 checker 红不红（系统配置槽少了几个，I-7.x 一类不变量会不会报）；可写挂载做成的那 4 个态做成之后再读回什么。这两样是 C656 验收格要不要比对的内容，条款没写。
- 没判：真设备上 `write_zeroes_at` 是不是真映射成 WRITE ZEROES、屏障对它是否生效（C6 的范围）；模型的撕裂只有崩溃点重放那一种，我用「任意扇区子集 16 种盘面」补了一道，但没有真设备数据。
- 没判：mkfs 运行中途块设备报错返回（`BlockDeviceError`）这一路——它的盘面与崩溃同形，结论照搬，但没有单独推。
- 没逐份翻 `crates/` 里全部几十份用例，7 处断言是 grep 数出来的下限；没查 α' 离线出路的实现（批 8 在飞，`crates/` 里今天没有）。
- 没碰禁读清单里的文件；没用 `~/.local/share/singlefs/work/` 以外的地方放中间产物，草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-c656-r1-sonnet` 里只有模型复跑的输出（`rerun1/`），没有编译目录与仓副本。

## 判定一览（末尾版）

| 格 | 判定 | 一句话 |
|---|---|---|
| V1 | 字面的甲做不到；甲拆槽能做但属乙一族 | 新池四类固定结构与旧池同偏移，只有系统配置槽能当闸，它没有原子写 |
| V2 | 乙撑得住（条件：两趟写序） | 作废段 8 态挂旧池不丢写；作废持久之后挂旧池 0、混合 0、拒 31 122、挂新池 64 |
| V3 | 量出代价，两支都不被代价排除 | 乙：操作 25→27、段 `2+12+1+1+1+4`、全量 280 019→31 195；7 处断言要改 |
| V4 | 三样都不翻面 | 乙只用到「作废段只写 [0, 根环基址) 的零」，不随槽距、fsid、历史长度变 |

## cite-check 末行

```
$ python3 scripts/agents/cite-check.py research/prompts/m3-close-c656-r1-sonnet-output.md | tail -1
  ✓ 核了 43 处引文、7 处裸行号，对上 43 处，没判 7 处（其中裸行号没有引文可比 7 处；逐处列在上面）
```
