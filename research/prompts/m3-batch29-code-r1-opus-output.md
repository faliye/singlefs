# m3-batch29-code-r1 攻方腿（Opus）报告：K2，重点 L1、L2，另做 K2 末条第二次抽样（2026-10-06）

立场：攻方，在冻结副本的拷贝上造反例；只跑 harness 档，GPU 不跑（GPU 一侧读 wgsl、按 CPU 算期望）。
被判对象：冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/`（6 份被判文件 `sha256sum -c` 逐份 OK）。
引代码写冻结副本全路径:行号；引 kb 写 kb 文件自己的行号（`.claude/kb/invariants.md`）。

## 复跑

```
SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree SINGLEFS_TOOLS=/home/fy5090/code/singlefs \
  bash research/prompts/m3-batch29-code-r1-opus-model/rerun.sh <空工作目录> all
bash research/prompts/m3-batch29-code-r1-opus-model/rerun.sh compare <工作目录>
```

模型各文件 sha256：见本报告「模型文件 sha256」一节（交回前现算，写在报告末尾那一节）。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据（节） |
|---|---|---|---|
| L1 第三支：墓碑树 / 谱系树层级 1 节点里类型段 0 的条目 | **打中（漏判 I-2.1，第 29 批之前就有、第 29 批没覆盖）** | 这一支同样不往下走孩子，两份位置条目校验和都坏时 I-2.1 报成立；CPU 与 GPU 同漏 | 一 |
| L1 长度口径：头里类标签是登记码、与写进去的长度不符 | **口径不一（不进误红口径：镜像本来就坏）** | 同一个单元、同一对位置条目校验和，类型段登记时 I-2.1 成立，类型段不认得 / 孩子是另一类时 I-2.1 红并连带 I-7.2、I-4.8、I-7.4；类标签改成 9 不红、改成 2 红 | 二 |
| L1 合法状态上多红 I-2.1 | 没打中（一次抽样） | 两支要类型段不认得或孩子自证成另一类，合法写者造不出；harness 全部测试二进制上探针计数见第六节 | 二、六 |
| L1 一份读不出、一份对得上 | 没打中（一次抽样） | 三棵树 × 两支 × 层级 1（墓碑树另加层级 2）：红集合 = 登记那一支的红集合 ∪ 身份那一条 | 二 |
| L1 CPU 与 GPU | **打中（推的，CPU 一侧量过）**，但不分辨新旧：两边共用的前提 | 位置条目指着从没写过的槽、校验和恰是全零 16 KiB / 32 KiB 的 CRC：CPU 读出全零、I-2.1 成立；GPU 那个槽不是落点、读不出、I-2.1 红并连带 I-4.8、I-7.4。登记那一支同样分叉 | 三 |
| L2 链长 255 / 256 / 257、根在候选集里 / 外、中途断开 | 没打中（读码模型，一次抽样） | 第 29 批的代码下 GPU 要么与 CPU 读到同样的行、要么判不了交 CPU；432 + 160 + 32 形里没有一形 GPU 判了而读到的行与 CPU 不同 | 四 |
| L2 扫描内核两处站点 37、38 走不走得到 | **反例**（对实现员与主 agent 两句话都是） | 候选集外的根走读内核各起一份走读走全链，先记 61；走得到 37 / 38 的是候选根：链途经一片已在上下文 0 走过的片（坏镜像）。现有 257 片那条对拍流落在 61，删掉那两句它照样绿（推的） | 四 |
| 第二次抽样：m3-batch28-code-r1 的 H-V 合法状态误红 | 没打中（第二次） | harness 162 个测试目标加探针，「孩子是另一类」那一支只在坏镜像用例里走到；合法状态上这一格仍是空的 | 六 |
| 第二次抽样：m3-batch28-code-r1 的 H-B 合法状态误红 | 没打中（第二次） | 同上，H-B 两支只在用例名就写着「孩子读不出」的坏镜像用例里走到；层 0 崩溃状态因每段 > 10⁶ 没跑 | 六 |
| 第二次抽样：m3-batch28-code-r1 的 CPU 与 GPU 第 28 批改的几支 | 没打中（读码，第二次） | H-V 两份都看、H-B 只比父条目两段，CPU 与 GPU 逐支同判 | 五 |

## 一、L1 第三支：墓碑树 / 谱系树层级 1 节点里类型段 0 的条目（打中，漏判 I-2.1）

**判据**：`.claude/kb/invariants.md` 第 130 行 I-2.1「任一被引用的块，其校验和与内容匹配」。第 29 批给「走读不往下读的孩子」补了每一份副本照判 I-2.1（类型段不认得、孩子是另一类两支）；墓碑树、谱系树里还有第三支同样不往下走：层级 1 节点里类型段 0 的条目。

**历史与每一步许可它的那一句**（坏镜像，从干净镜像出发改一处、重封、补校验和到根槽）：
1. 起点：墓碑树用「三个单元的文件发完意图」，谱系树用「回退到 A 之后」（`checker_known_bad_images.rs` 现成的两份镜像），树根是层级 1、直接管容器。
2. 根第 0 条条目类型段改成 0（type0；type0z 再把身份三段清零），子指针两条位置条目的整单元校验和各翻一字节，孩子本身不动。
3. 走读：类型段 0 先问孩子是不是另一类，两份按自己那一类的长度都对不上校验和，交回 None（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4205` 那个函数）；落到 `InternalNode` 那一支，进 `walk_the_internal_node_below_a_container_tree_entry`（同文件第 2242 行调用）。
4. 层级 ≤ 1 时只记走读失败就返回，不读孩子、不判 I-2.1、不登记引用：同文件第 2353–2358 行，说明原文第 2355 行：
   `"{name}层级 {} 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点，这两棵树里没有条款定的形状",`
   调用方的注释说读不出归 I-2.1（同文件第 2229 行）：`// （第二节 T7）：读不出的孩子这一小句不判，读不出归 I-2.1 与走读失败。`——层级 1 上没有谁判这一份 I-2.1。
5. GPU 同形：`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3105` `if (n.level <= 1u) {` 里只 `w_failures` 加一就 `continue`，不调 `w_judge_every_copy_of_a_child_the_walk_does_not_go_into`（推的，读码；CPU 与 GPU 同漏，不分叉）。

**原样输出**（攻方仓副本，`rerun.sh` 的 kbi-new 段，`logs/kbi-new.summary`；`i2_1_verdict` 是 I-2.1 那一格的判定）：
```
RESULT stage=kbi-new kbi form=tomb-L1T-sum01bad-type0z red=[I-4.8,I-7.2,I-7.4] i2_1=- i2_1_verdict=holds i7_2=elsewhere
RESULT stage=kbi-new kbi form=lin-L1T-sum01bad-type0z red=[I-4.8,I-7.2,I-7.4] i2_1=- i2_1_verdict=holds i7_2=elsewhere
RESULT stage=kbi-new kbi form=tomb-L1T-sum01bad-type0 red=[I-4.8,I-7.2,I-7.4,I-9.23] i2_1=- i2_1_verdict=holds i7_2=elsewhere
RESULT stage=kbi-new kbi form=tomb-L1T-sum01bad-type7 red=[I-2.1,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=unwalked-child i2_1_verdict=violated i7_2=unwalked-child-both
RESULT stage=kbi-new kbi form=tomb-L1T-sum01bad-typeR red=[I-2.1,I-4.8,I-7.2,I-7.4] i2_1=elsewhere i2_1_verdict=violated i7_2=elsewhere
```
同一对坏校验和：类型段 7（第 29 批那一支）与类型段原样（登记那一支）I-2.1 红，类型段 0 的 I-2.1 报成立。只坏一条时孩子另一份自证成码 3、走「孩子是另一类」那一支，照判（`tomb-L1T-sum0bad-type0z` 红 I-2.1，`i2_1=unwalked-child`）；漏的只是两份都自证不过的那一形。inode 树没有这一支（类型段 0 照 16 KiB 读、判 I-2.1：`inode-L1T-sum01bad-type0z` 红 I-2.1）。

**四句**：
- 分不分辨臂：分辨。被判的是第 29 批「不往下走的孩子每一份照判 I-2.1」这条改法覆盖了几支；类型段 7 与登记那一支在同一张镜像上都红 I-2.1，只有这一支不红。
- 系统当时看不看得到：看得到。两条位置条目的校验和与盘上两份就在手里，第 29 批的共用函数拿同样的输入就判得出。
- 满足判据字面的哪一句：I-2.1「任一被引用的块，其校验和与内容匹配」——条目的子指针引用了这个块，两份都对不上，checker 报成立；按 m3-batch22-code-r1 正文第五节的口径是**漏判**（镜像违反那一行字面而 checker 报成立）。镜像本来就红（I-7.2 等），这一形的害处是 I-2.1 这一格说错了。
- 改法在打中的那几格上还中不中：攻方改法 F-L0（零轮，只在我的模型上量过、被攻过零轮）：那一支照调第 29 批的共用函数。副本上量过（`logs/kbi-fix.summary` 与 kbi-new 逐行比）：只变 6 行——上面 4 个 type0 / type0z 形（墓碑、谱系各两）转成 `i2_1=unwalked-child i2_1_verdict=violated`，另 2 行是第二节那一形（`tomb-L1-tag2-type0`、`lin-L1-tag2-type0`，类标签改成 2 之后同样落进这一支，按 16 KiB 比、红，口径见第二节）。现有 `checker_known_bad_images`（156）、`container_trees_grow_a_level_when_the_root_is_full`（7）、`pool_checker_judges_the_deletion_state_of_every_walked_version`（45）在 F-L0 下全过（副本上量过，`test result: ok. 156 passed` / `7 passed` / `45 passed`）。GPU 那一处同改是推的（没写、没跑）。

**测试尖度**：harness 全部测试二进制上加只计数的探针（第六节），这一支（探针 `l1_type0`）只在攻方自己的用例里走到，现有用例一条都没走到——把第 2353 行那一支整段删掉或改成往下读，现有用例照样绿（推的，按探针计数）。

## 三、L1 的 CPU 与 GPU：位置条目指着从没写过的槽（打中，推的；不分辨新旧）

**形**：根第 0 条条目的第二条位置条目改成同一块盘上一个从没写过的槽（两块盘上那 32 KiB 全零，用例里先断言），校验和改成 16 KiB 全零的 CRC（zero16）或 32 KiB 全零的 CRC（zero32）；第一条位置条目不动。类型段 7 / 0 走第 29 批的共用函数，类型段原样走登记那一支。

**CPU 一侧（量过，`logs/kbi-new.summary`、`logs/ctg-new.summary`）**：
```
RESULT stage=kbi-new kbi form=tomb-L1-loc1zero16-type7 red=[I-3.1,I-3.11,I-9.23] i2_1=- i2_1_verdict=holds i7_2=-
RESULT stage=kbi-new kbi form=tomb-L1-loc1zero32-type7 red=[I-3.1,I-3.11,I-9.23] i2_1=- i2_1_verdict=holds i7_2=-
RESULT stage=kbi-new kbi form=tomb-L1-loc1zero32-typeR red=[I-3.1,I-3.11] i2_1=- i2_1_verdict=holds i7_2=-
RESULT stage=kbi-new kbi form=tomb-L1-loc1zerowrong-type7 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-7.4,I-9.23] i2_1=unwalked-child i2_1_verdict=violated i7_2=-
RESULT stage=ctg-new ctg form=tomb-L2-loc1zero16-type0 red=[I-3.1,I-3.11] i2_1=- i2_1_verdict=holds i7_2=-
```
CPU 的读者对没写过的扇区读出全 0（`SparseDevice` 的读法），头里类标签 0 不登记，第 29 批的函数 16 KiB、32 KiB 两种都比（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4250` `None => vec![node_bytes(), data_unit_bytes()],`），全零的 CRC 对得上，这一份 I-2.1 成立。登记那一支按条目那一类的长度读（叶容器 32 KiB），zero32 同样成立。三棵树、层级 1 与层级 2（墓碑树）都是这个样子（zero16 / zero32 各 3 + 3 行，见 summary）。

**GPU 一侧（推的，读 wgsl，没跑）**：
1. GPU 只认「落点」：`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:406` 的说明：`/// 全部落点（还没填写与版本）：系统配置每盘两槽、根环全部槽、journal 环里基线或枚举写碰过的记录槽、单元区里基线或枚举写碰过的 16 KiB 槽。`——从没写过的槽不是落点。
2. `read_at`（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:470`）第 471 行 `let location = find_location(device, offset);` 找不到就交回 `no_slice()`，不记判不了。
3. 第 29 批的 GPU 函数（`crash_judge_checker.wgsl:3014`）第一句 `copy.version == NONE` 就交回对不上：`w_mismatches` 加一、I-2.1 红；最新根那一版 `crash_judge_checker.wgsl:4169` `let newest_walked_into_reused_or_erased_unit = w_failures > 0u || w_mismatches > 0u;` 再红 I-4.8、I-7.4。
4. 期望：zero16 / zero32 那几形 GPU 比 CPU 至少多红 I-2.1、I-4.8、I-7.4（I-3.1、I-3.11 两格 GPU 怎么判没推）。登记那一支（`w_read_referenced_unit`，同文件第 1286 行起）同样先 `read_at`，`tomb-L1-loc1zero32-typeR` 那一形 CPU 不红 I-2.1、GPU 红——分叉不是第 29 批引进的。

**四句**：
- 分不分辨臂：不分辨第 29 批的改法与不改；是 GPU 落点模型与 CPU 读者共用的前提（「位置条目只会指着写过的槽」）的缺口，第 29 批的函数两种长度都比，让它在类型段不认得那一支上多了一种对得上的长度。按 evidence-discipline「打中不分辨臂」另立一笔账。
- 系统当时看不看得到：CPU 看得到全零；GPU 表里没有这个槽的字节，判的那一刻看不到——GPU 一侧是「判别子观测不到」。
- 判据字面：I-2.1 字面「校验和与内容匹配」，全零内容与全零的 CRC 匹配，CPU 的「成立」合字面，GPU 的「红」不合；镜像本来就坏（引用了没分配的槽，CPU 红 I-3.1），不进 m3-batch22 正文第五节的误红口径，按本轮正文第五节「CPU 与 GPU 判得不一样」算打中。
- 改法（零轮，推的，没实现）：F-零槽——`read_at` 找不到落点、偏移在单元区里且整段在盘内时，交回一份全零的切片（检查字取全零 16 KiB / 32 KiB 的 CRC），与 CPU 的稀疏读者同一个答案；或者更保守，引用指着不是落点的单元槽时记判不了交 CPU。两样都只改 GPU 的读法，第 29 批的函数不用动。对拍补一条流：一条位置条目指着没写过的槽、校验和是全零的 CRC。

## 四、L2：实例表链片数上限（没打中；扫描内核两处站点走得到的形与两句话的反例）

**读码模型**（`l2_chain_model.py`，推的：把三处链循环的控制流照抄成 Python，在 624 形上逐形算；GPU 不跑）。抄的是：
- 走读内核 `w_walk_instance_table_chain`：`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1611` `if (!w_visit(pointer.device0, pointer.slot0)) {`（走过的单元合并、`break`），读不出 / 身份不对 / 末片都在加一之前 `break`，第 1686 行 `if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED) {` 才记 61。
- 哪几条根各起一份走读：最新根与候选根在上下文 0 里接着走、共用走过的单元（第 3795 行那一阶段不调 `w_start`）；被抛弃的根每条 `w_start(1u, false)` 另起一份（第 3916 行那一阶段，第 3928 行）；候选集外的根收割那一遍共用一份（第 4065 行）；F 之下的根每条另起一份（第 4093 行）。全内核只在第 4155 行 `w_walk_one(walk_kind, walk_base, walk_is_newest, fsid_low);` 走，每次先走实例表链。
- 扫描内核：`crash_judge_checker_scan.wgsl` 第 523 行、第 1157 行两句（第 29 批加的），调用点第 696 行（候选根引用集合）与第 1926 行（I-3.12 每条根）。
- CPU：`walk.rs` 读链不设上限（`instance_table_rows_of_root_without_judging`）。

取样：最新根链长 {1, 200, 255, 256, 257}，另一条根链长 {255, 256, 257, 300}、角色 {候选根、被抛弃的根、F 之下的根}、中途第 {无, 100, 255, 256} 片读不出、最新根第 1 片的链指针指到那条根的第 {无, 100, 250} 片（那一片在最新根那条链上身份不对，是坏镜像），共 624 形（跳过无意义的组合）。原样输出（`logs/l2.summary`）：
```
RESULT stage=l2 l2-summary batch29=decided mutant=decided shapes=432
RESULT stage=l2 l2-summary batch29=undecidable-37 mutant=decided shapes=16
RESULT stage=l2 l2-summary batch29=undecidable-37 mutant=decided-but-rows-truncated:candidate shapes=16
RESULT stage=l2 l2-summary batch29=undecidable-61 mutant=undecidable-61 shapes=160
RESULT stage=l2 l2-killers count=16
```
- **第 29 批的代码下没打中**：432 形 GPU 判了，其中没有一形 CPU 读到的行比 GPU 多（模型里有这一判，摘要里没有 `decided-cpu-reads-more`）；其余 192 形 GPU 判不了交 CPU。链长 255 / 256 两边都读到末片；257 及以上 GPU 判不了；中途断开两边都当读不出、都不判。
- **两句话的反例**：实现员写在扫描内核头上 `crash_judge_checker_scan.wgsl:12` 的 `// 走满先记它自己的判不了（站点号 61），扫描内核不跑；这里两处是走读内核哪天不再走某一类根时的接手（读码推的，没有一条流落到这里）`；主 agent 在正文第二节说候选集外的根只有扫描内核 I-3.12 走到。读码两句都不对：
  - 候选集外的根（被抛弃的根、F 之下的根）各起一份走读走全链，链 257 片先记 61（模型里 `role=abandoned` / `below-floor` 的 257、300 片形全是 `undecidable-61`）。对拍流「被抛弃根 C 自己那张实例表 257 片」落在 61，用例说明 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7190` 也这么推。
  - 走得到 37 / 38 的是**候选根**：候选根与最新根共用上下文 0，候选根的链途经一片已被最新根那一遍走过（标成走过）的片时，走读内核在那一片合并、`break`，不记 61；扫描内核从头读这条链、读满 256 片记 37（有已释放记录时先到引用集合那一处）或 38。模型里 16 + 16 形是这个样子，例：
    `RESULT stage=l2 l2 newest=200 other=257 role=candidate break=None share=(1, 100) batch29=undecidable-37 mutant_without_the_two_lines=decided-but-rows-truncated:candidate`
    这一形是坏镜像（最新根第 1 片身份不对，I-1.1 红），由流单独走得到（对拍流照「改造实例表链」那几个现成函数造得出，推的，没造）。
- **删掉那两句（变异 M）**：上面 16 形在 M 下 GPU 判了而候选根那张表只读了前 256 片（I-3.12 / I-3.9 少判，CPU 读到末片照判）。现有对拍流落在 61，删掉两句它照样绿（推的）；要钉住这两句得补上面那一形的流，且「只删第 1157 行那一句」在有已释放记录时被第 523 行那一句先接住，要另一形（没有已释放记录）才钉得住（推的）。归 K1 / 测试那一格，不在这条腿判。

**四句**（对「两句话的反例」）：分不分辨臂——不涉及臂，是两份说明与代码不符；系统看不看得到——走读内核在那一片看得到「走过」，就是因为看到了才停；字面——对的是实现员注释与正文第二节的陈述；改法——注释改成「候选根的链途经上下文 0 里已走过的片时走读内核不记 61，扫描内核这两处接手」，对拍补一形（零轮，推的）。

## 五、第二次抽样：m3-batch28-code-r1 的 CPU 与 GPU 第 28 批改的几支（没打中，读码，第二次）

第一次抽样（m3-batch28-code-r1 攻方腿）没读；这一次独立逐支对，冻结副本（第 29 批之后）：

| 支 | CPU（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs`） | GPU（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`） | 对 |
|---|---|---|---|
| H-V 两份都看 | 第 4205 行起逐条位置条目看，第 4228 行 `&& class_other_than_the_entry_type_names.is_none()`：交回位置条目在前的那一份另一类 | 第 2988 行起先第一份、再第二份，第 3003 行 `if (class_in_the_second_header != class_the_entry_type_names && class_other_than_the_entry_type_names == NONE) {` | 同：自证的判法（头 16 KiB 读、类标签 1–3、按那一类的长度比校验和）、取哪一份的次序都一样 |
| H-B inode 类型段 2 | 第 4279 行 `birth_tree_in_the_child_pointer_head == identity_birth_tree,` | 第 2667 行 `w_judge(INVARIANT_I_9_2, eq64(child.birth_tree, identity_birth_tree));` | 同 |
| H-B inode 类型段 0 | 第 4289 行 `let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;` | 第 2664 行三段为 0 | 同 |
| H-B 墓碑 / 谱系容器读不出 | 第 2418 行 `child.birth_tree == identity_birth_tree,` | 第 3153 行 `w_judge(identity_invariant, eq64(child.birth_tree, rd64(n.base, o + 8u)));` | 同 |

逐支同判；读不出之后两边都记走读失败（CPU `walk_failures`、GPU `w_failures`）。这一格第二次也没打中。限度：读码，GPU 没跑；第三节那一形（指着没写过的槽）在这几支上同样分叉，但它不是第 28 批引进的，归第三节。

## 二、L1 的其余几格：长度口径、合法状态、一份读不出

### 2.1 长度口径：头里的类标签是登记码、与写进去的长度不符（口径不一，不进误红口径）

**形**：孩子单元（层级 1：码 3 容器 32 KiB；层级 2：码 2 节点 16 KiB）头里的类标签改成 `tag`，不重封，新的整单元校验和沿引用链补到根槽——两份副本按写进去的长度都与位置条目对得上，只是头里的类标签说的长度不同。再分类型段：7（不认得）、原样（登记那一支，对照）。

**原样输出**（`logs/kbi-new.summary`、`logs/ctg-new.summary`，墓碑树各取一行，inode 树、谱系树同形）：
```
RESULT stage=kbi-new kbi form=tomb-L1-tag2-type7 red=[I-1.11,I-2.1,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=unwalked-child i2_1_verdict=violated i7_2=unwalked-child-both
RESULT stage=kbi-new kbi form=tomb-L1-tag2-typeR red=[I-1.11,I-1.2,I-1.6,I-2.4,I-4.2,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=- i2_1_verdict=holds i7_2=elsewhere
RESULT stage=kbi-new kbi form=tomb-L1-tag9-type7 red=[I-1.11,I-9.23] i2_1=- i2_1_verdict=holds i7_2=-
RESULT stage=kbi-new kbi form=tomb-L1-tag1-type7 red=[I-1.11,I-9.23] i2_1=- i2_1_verdict=holds i7_2=-
RESULT stage=ctg-new ctg form=tomb-L2-tag3-type7 red=[I-1.11,I-1.6,I-2.1,I-2.4,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=unwalked-child i2_1_verdict=violated i7_2=unwalked-child-both
RESULT stage=ctg-new ctg form=tomb-L2-tag3-type0 red=[I-1.11,I-1.6,I-2.4,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=- i2_1_verdict=holds i7_2=elsewhere
```
同一个单元、同一对位置条目校验和：登记那一支按条目那一类的长度读、I-2.1 成立；第 29 批那两支按头里的类标签取长度（`walk.rs:4249` `Some(unit_bytes) => vec![unit_bytes],`），类标签 2 说 16 KiB、内容按 32 KiB 写，对不上，I-2.1 红、两份都红再记走读失败（I-7.2），连带 I-4.8、I-7.4；类标签改成 9（不登记，两种长度都比）或 1（32 KiB）就不红。GPU 同口径（`crash_judge_checker.wgsl:3014` 起，类标签 1–3 只按那一类比），CPU 与 GPU 不分叉。

**四句**：分辨臂——分辨第 29 批判决选的 F-α（登记码按那一类的长度）与对照臂「F-任一长」（16 KiB 与 32 KiB 有一种对得上就算对得上，攻方起的名，零轮）；系统看得到——两种长度的 CRC 都在手里；字面——I-2.1「任一被引用的块，其校验和与内容匹配」没说按什么长度，按写进去的长度这一块是匹配的，F-α 报红；镜像本来就坏（I-1.11、身份那一条红），不进 m3-batch22 正文第五节的误红口径，是「同一块、同一个校验和，I-2.1 随父条目类型段与头里类标签翻面」的口径不一；改法——F-任一长，副本上量过：`logs/kbi-anylen.summary`、`logs/ctg-anylen.summary` 与 new 逐行比只变 5 行（`inode/tomb/lin-L1-tag2-type7`、`tomb-L2-tag1-type7`、`tomb-L2-tag3-type7`，都转成 `i2_1_verdict=holds`），别的形一行不动；现有三个测试目标在它下面跑的结果见第七节「existing-anylen」。F-任一长 会不会放过真的坏副本：一份坏了的副本要在另一种长度上恰好与校验和相等才放过，概率是 CRC 碰撞那一档（推的）。交主 agent 判要不要换口径。

### 2.2 合法状态上多红 I-2.1（没打中，一次抽样）

读码：第 29 批的函数只在两支里调——类型段不认得（`walk.rs:2272`、第 2606 行）与孩子自证成类型段之外另一类（第 2299 行、第 2575 行）。合法写者不写不认得的类型段；孩子要「自证成另一类」，得有一份按另一类的长度算出的整单元 CRC 恰好等于位置条目里那个校验和，合法状态上是 CRC 碰撞那一档。「孩子没落盘的一份」走不到这两支：没落盘的副本自证不过，交回 None，照类型段那一支读（读不出归原来的 I-2.1），不经第 29 批的函数。
探针（第六节）：harness 全部 162 个测试目标上，第 29 批的函数（`unwalked`）只在 `checker_known_bad_images` 的坏镜像用例与攻方自己的用例里走到。

### 2.3 一份读不出、一份对得上（没打中，一次抽样）

第二条位置条目改成池里没有的盘 7（头读不出）或盘外的槽，第一条不动；三棵树 × {类型段 7、类型段 0（孩子是另一类）、原样} × 层级 1，墓碑树另加层级 2（类型段 7、1、0）。每一组里第 29 批那两支的红集合 = 登记那一支的红集合 ∪ 身份那一条，例（`logs/kbi-new.summary`）：
```
RESULT stage=kbi-new kbi form=lin-L1-loc1dev7-type7 red=[I-2.1,I-4.8,I-7.4,I-9.17] i2_1=unwalked-child i2_1_verdict=violated i7_2=-
RESULT stage=kbi-new kbi form=lin-L1-loc1dev7-type0 red=[I-2.1,I-4.8,I-7.4,I-9.17] i2_1=unwalked-child i2_1_verdict=violated i7_2=-
RESULT stage=kbi-new kbi form=lin-L1-loc1dev7-typeR red=[I-2.1,I-4.8,I-7.4] i2_1=elsewhere i2_1_verdict=violated i7_2=-
```
读不出的那一份 I-2.1 红、另一份对得上不记走读失败，与登记那一支的 `read_referenced_unit`（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/image.rs:1491` 起，每份都判 I-2.1）同判。GPU 同形（`crash_judge_checker.wgsl:3014` 第一句 `copy.version == NONE` 交回对不上，盘不在池里时 `read_at` 找不到落点）。

## 六、第二次抽样：m3-batch28-code-r1 的 H-V、H-B 合法状态误红（没打中，第二次；这两格在合法状态上仍是空的）

**做法**（与第一次不同的取样）：攻方仓副本的 walk.rs 打只计数的探针（`make_variants.py` 的 probe，不改任何判定），六格：`hv`（孩子是另一类那一支）、`hb_inode`、`hb_container`（孩子两份都读不出，H-B 那几小句）、`unwalked`、`unwalked_mismatch`（第 29 批的函数与它判出的对不上）、`l1_type0`（第一节那一支）；设 `OPUS_B29_PROBE_LOG` 时每走到一次写一行「格名 用例名」。singlefs-harness 的 162 个测试目标逐个编、逐个跑（经内存包装 4G，线程先 15、收到主 agent 消息后 31），统计每个用例走到哪几格。原样（`logs/harness.summary` 末行）：
```
RESULT harness-total targets=162 passed=1214 failed=2 targets_with_probe_hits=3
```
- 走到任一格的只有 3 个目标：`checker_known_bad_images`（坏镜像语料，加攻方两条用例）、`container_trees_grow_a_level_when_the_root_is_full`（只攻方那条用例）、`pool_checker_judges_the_deletion_state_of_every_walked_version`（两条坏镜像用例：`an_intermediate_version_whose_inode_leaf_does_not_read_back_is_not_judged_rather_than_red`、`a_tombstone_container_that_does_not_read_back_leaves_the_watermark_invariant_not_judged`，用例名就说孩子读不出）。其余 159 个目标（合法镜像、崩溃状态、回退、被抛弃的根、层级 2 / 3 的墓碑树、谱系树）六格一次都没走到。
- 失败的 2 个用例与探针无关：冻结副本没有 `litmus/` 目录（`a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error` 与 `publish_order_matches_litmus` 读 `../../litmus/*.litmus` 读不到）。
- 结论：两格第二次也没打中；与第一次抽样一样，合法流碰不到这两支，「没打中」是空的（只说明走不到，不说明走到了判得对）。

**层 0 崩溃状态没跑**（照派发「一段历史自己就超过约 10⁶ 的，那段不跑」）：原型 `crash/` 在内存稀疏盘上建起点（mkfs → 新池新建文件 → 建 4 个空 inode），放开扫 24 段用户动作（删哪个号、墓碑容器压不压小、删完回不回退、回退到倒数第几版、建几个号，长度 1–3），只按 `closed_form_state_count` 数（`logs/crash-count.summary`）：一次 unlink 意图那一段 26 次写，单段 2^26 级；最小的 `N1`（建 1 个号）也有 1048583 个状态。原样两行：
```
RESULT stage=crash-count crash U2 writes=56 segments=8 longest_segment=26 closed_form_states=68157453
RESULT stage=crash-count crash N1 writes=25 segments=4 longest_segment=20 closed_form_states=1048583
```
24 段全在 10⁶ 以上，一段都没枚举；全量合起来远过 10⁷，也不交主 agent 跑（要跑就得先缩成单段 ≤ 20 次写的历史，删除批的发布做不到）。这条腿原型跑的崩溃状态总数：0。

## 七、没打中的形状、限度、交接

### 没打中的形状（试过哪些、取样多大）
- L1 合法状态多红：读码 + 探针（第二、六节），harness 162 个测试目标，第 29 批的函数只在坏镜像用例里走到。
- L1 一份读不出一份对得上：三棵树 × 三种类型段 × {盘 7、盘外槽、只坏一条校验和} × 层级 1，墓碑树层级 2 另 3 × 3 形；红集合全与登记那一支一致（第 2.3 节）。
- L1 CPU 与 GPU 除第三节那一形外：类标签判法（偏移 6、登记码 1–3）、头读不出、盘不在池里、盘尾只剩 16 KiB 的槽（GPU 落点那一槽长 16 KiB、`c2` 没有 32 KiB 那一位；CPU 32 KiB 读不出），读码逐支同判。
- L2：624 形（第四节）；r_instance_rows 与 CPU 读法的拒收条件（魔数、flags、类 3、版本、头 / 载荷 CRC、预留位、身份、宽 88、计数与声明长度、链指针记录之前全是行）逐条对过，没找到 GPU 收而 CPU 拒或反过来的一形。
- 第二次抽样两格：第五、六节。
- 用户决定的那几步：第一、二、三节是坏镜像，没有用户动作；第六节原型放开扫 24 段用户动作（删哪个号、压不压容器、回不回退、退几版、建几个号），只固定起点，因状态数超限一段都没枚举。

### 这条腿自己的限度
- GPU 一侧全是读 wgsl 推的，没跑卡；第三节、第四节的 GPU 结论与「删掉那两句现有对拍照样绿」都是推的。
- 第一、二、三节的坏镜像沿用 `checker_known_bad_images.rs` 与 `container_trees_grow_a_level_when_the_root_is_full.rs` 的现成搭法：它们的 `build_pool` 在临时目录建两个 4 GiB 稀疏镜像文件、用完删掉，不是内存稀疏盘；只有第六节的原型照派发用内存稀疏盘。
- inode 树层级 2 没造（要上万个 inode）；谱系树层级 2 没造（墓碑树与谱系树共用 `walk_the_entries_of_a_container_tree_node`）。
- 攻方改法 F-L0、F-任一长、F-零槽都只在我的模型上量过或只推过，被攻过零轮：F-L0、F-任一长在副本上量过（量过：`logs/kbi-fix.summary`、`logs/kbi-anylen.summary`，现有三个目标 `RESULT stage=existing-fix test result: ok. 156 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` 与 `RESULT stage=existing-anylen test result: ok. 156 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` 等六行全绿）；GPU 同改是推的；F-零槽是推的。

| 改法 | 修哪一格 | CPU | GPU |
|---|---|---|---|
| F-L0 | 第一节第三支漏判 I-2.1 | 量过（6 行变、208 条现有用例绿） | 推的（第 3105 行那一支照调 `w_judge_every_copy_of_a_child_the_walk_does_not_go_into`） |
| F-任一长 | 第 2.1 节口径不一 | 量过（5 行变、208 条绿） | 推的（`crash_judge_checker.wgsl` 第 3018–3022 行那一段不看类标签、两种长度都比） |
| F-零槽 | 第三节 CPU 与 GPU 分叉 | 不用改 | 推的（`read_at` 找不到落点时交全零切片或记判不了） |

- 收到主 agent 消息「线程上限改成 31」时（第五节写完之前、harness-probe 第一遍跑到第 51 个目标），从下一条命令起线程取 31，`rerun.sh` 默认值随之改成 31。
- 机器负载在跑的时候到过 100 上下（别的会话），两次前台命令过了 240 秒被转去后台跑完（一次 harness-probe 一段、一次 existing-fix / existing-anylen），都是我起的、等它跑完才往下走，没停、没换写法。

### 要主 agent 跑的命令
没有要另跑的长活。复跑整份（串行，约 40 分钟，harness 两段按 `OPUS_STAGE_BUDGET` 反复调到 DONE）：
```
cd /home/fy5090/code/singlefs
SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree SINGLEFS_TOOLS=/home/fy5090/code/singlefs OPUS_THREADS=31 \
  bash research/prompts/m3-batch29-code-r1-opus-model/rerun.sh <空工作目录> all
bash research/prompts/m3-batch29-code-r1-opus-model/rerun.sh compare <工作目录>
```
内存：编译 6G、跑 4G（脚本里写死）。判结果看末行 `SUMMARY_MATCHES 1385 行`。我在空目录里从头跑过一遍（逐段调），`compare` 原样：`SUMMARY_MATCHES 1385 行`；更早一遍（setup 之后才加 anylen 变体）除 setup 那一行 sha256 外逐段相同。

### 没做什么
- 不判 K1、K3（正推腿的格）；不替主 agent 采纳；副本上的数不算入库装置上的数。
- GPU 不跑；重型测试不跑（checker 档、层 0 入库用例、变异表都没跑）；第六节层 0 枚举一段都没跑（状态数超限）。
- 没读禁读清单里的文件；第 28 批攻方腿报告没读（第五、六节是独立的第二次抽样，只读了 m3-batch28 判决里那几格的结论与它的模型目录的 `rerun.sh` 头与原型的调用写法）。

### 模型文件 sha256
```
53b028803e32b2528bb7e03b2ede7469aca127a1048abe781cd4cb188eaa5431  ./crash/Cargo.toml.in
b84959ed6752581797c1e50cfc420d60575b6da575f74aaa1ddcf2aa70f57c50  ./crash/src/main.rs
4d6493e5c47d2b82804e9c5eeb41ad6969e916c266757452ad07bcf7ec49b343  ./expected-summary.txt
1518ca13c718582346c68b15157fd3a10f3d51d0b6e42114eebc0940a7c54df4  ./harness_probe_summary.py
386c8ba86f9aac7874772b29737b474e0c1a7f020b003fd10337ebb3ba6b7691  ./l2_chain_model.py
c6d3f4c988891f62addc064f3f23ed586d413d36d7a69875cc4f1e8479f62349  ./make_variants.py
da1736e128a6604e6c70595d39170aa8bb2c06b31b3a5f25939e393324b13136  ./opus_b29_ctg_append.rs
1699fc1ef353d451d8bac230199cd8328c3e2b564d4c68c9b78f40a52ed98132  ./opus_b29_kbi_append.rs
fee456d031ad20b850deba42384ec398b58953a428fa63a5369c35380bd8c1a8  ./rerun.sh
```
（在 `research/prompts/m3-batch29-code-r1-opus-model/` 里 `find . -type f | sort | xargs sha256sum` 现算。）

## 附：正文引到的代码行（cite-check 认的写法，路径相对冻结副本根；kb 相对仓根）

- 「任一被引用的块，其校验和与内容匹配」（`.claude/kb/invariants.md:130`）
- 「"{name}层级 {} 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点，这两棵树里没有条款定的形状",」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2355`）
- 「if node.level <= 1 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2353`）
- 「// （第二节 T7）：读不出的孩子这一小句不判，读不出归 I-2.1 与走读失败。」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2229`）
- 「self.walk_the_internal_node_below_a_container_tree_entry(」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2242`）
- 「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(&child, &what);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2272`）
- 「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(child, &what);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2299`）
- 「fn judge_every_copy_of_a_child_the_walk_does_not_go_into(」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2312`）
- 「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2575`）
- 「self.judge_every_copy_of_a_child_the_walk_does_not_go_into(」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2606`）
- 「fn child_unit_class_other_than_the_entry_type_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4205`）
- 「&& class_other_than_the_entry_type_names.is_none()」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4228`）
- 「Some(unit_bytes) => vec![unit_bytes],」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4249`）
- 「None => vec![node_bytes(), data_unit_bytes()],」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4250`）
- 「birth_tree_in_the_child_pointer_head == identity_birth_tree,」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4279`）
- 「let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:4289`）
- 「child.birth_tree == identity_birth_tree,」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/walk.rs:2418`）
- 「pub fn read_referenced_unit(」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker/src/image.rs:1491`）
- 「if (n.level <= 1u) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3105`）
- 「fn w_copy_matches_its_location_checksum_at_the_length_its_header_names(copy: Slice, checksum: u32) -> bool {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3014`）
- 「return slice_crc_matches(copy, NODE_BYTES, checksum) || slice_crc_matches(copy, DATA_UNIT_BYTES, checksum);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3022`）
- 「fn w_child_unit_class_other_than_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> u32 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2988`）
- 「if (class_in_the_second_header != class_the_entry_type_names && class_other_than_the_entry_type_names == NONE) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3003`）
- 「w_judge(INVARIANT_I_9_2, eq64(child.birth_tree, identity_birth_tree));」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2667`）
- 「w_judge(INVARIANT_I_9_2, is_zero64(identity_birth_tree) && is_zero64(identity_container) && is_zero64(identity_birth));」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2664`）
- 「w_judge(identity_invariant, eq64(child.birth_tree, rd64(n.base, o + 8u)));」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3153`）
- 「let newest_walked_into_reused_or_erased_unit = w_failures > 0u || w_mismatches > 0u;」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4169`）
- 「fn w_read_referenced_unit(p: Ptr, unit_bytes: u32) -> Slice {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1286`）
- 「if (!w_visit(pointer.device0, pointer.slot0)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1611`）
- 「if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1686`）
- 「} else if (phase == WALK_PHASE_CANDIDATE_ROOTS) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3795`）
- 「} else if (phase == WALK_PHASE_ABANDONED_ROOTS) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3916`）
- 「w_start(1u, false);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3928`）
- 「w_start(1u, false);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4065`）
- 「w_start(1u, false);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4093`）
- 「w_walk_one(walk_kind, walk_base, walk_is_newest, fsid_low);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4155`）
- 「走满先记它自己的判不了（站点号 61），扫描内核不跑；这里两处是走读内核哪天不再走某一类根时的接手（读码推的，没有一条流落到这里）」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:12`）
- 「undecidable = UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED;」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:523`）
- 「undecidable = UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED;」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1157`）
- 「r_references_of_root(root.base, every || index == c_newest_index, c);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:696`）
- 「if (!r_instance_rows(root.base)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1926`）
- 「fn read_at(device: u32, offset: vec2<u32>, length: u32) -> Slice {」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:470`）
- 「let location = find_location(device, offset);」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:471`）
- 「全部落点（还没填写与版本）：系统配置每盘两槽、根环全部槽、journal 环里基线或枚举写碰过的记录槽、单元区里基线或枚举写碰过的 16 KiB 槽。」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:406`）
- 「记哪一个原因字是推的（读码，没跑过）：走读内核把根环里每条自证过的根都另起一份走读走一遍」（`/home/fy5090/.local/share/singlefs/work/m3-batch29-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7190`）
