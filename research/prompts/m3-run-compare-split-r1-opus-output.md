# m3-run-compare-split-r1 攻方腿（Opus）报告（2026-10-10）

立场：假设第三节那份两段形态是错的，造可达的历史、输入或改动序列打穿它。只攻派发给的五个面（漏、错配、落不下、分界越位、第 7 与第 9 问）。
代码引用两种：「主工作区」= `/home/fy5090/code/singlefs` 此刻（HEAD 5d0b7e7c）；「副本」= 主工作区 rsync（带 `.git`）后依次打 v、e、e2、e3、e4、e5 六份补丁（六次 apply-writer-patch 退出码 1、0、0、0、0、0，v 那一次同事实表的说法），副本交回前已删。没被补丁动过的文件（harness 的 `model.rs`、`history.rs`、`mount_transparency.rs`、`reference_filesystem_translation.rs`，RefFS 的 `lib.rs`，全部 `*.wgsl`，`crash_judge_dispatch.rs`）两边逐字节相同（sha256 前 12 位比过），这些写主工作区路径；补丁动过或新建的只写「副本 路径:行号」、不带原文引号（cite-check 在主工作区上核不了它们）。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-run-compare-split-r1-opus-model/`。先建副本：`rsync -a --exclude target --exclude 'target-*' <主工作区>/ <副本>/`，依次 `python3 scripts/gate/apply-writer-patch.py ~/.local/share/singlefs/work/m3-merge-impl/<名>/patch --root <副本>`（v e e2 e3 e4 e5）。

```
# 体量与 CPU 的算术（纯 Python，不读库）
python3 -I research/prompts/m3-run-compare-split-r1-opus-model/volume_and_cpu.py
# 内核展开量（只依赖 naga 30.0.1 的小程序；先量三个现成内核、与钉值比上才信探针）
bash scripts/hosts/run-with-memory-cap.sh 6G nice -n 19 bash scripts/hosts/capped.sh 8 \
  bash research/prompts/m3-run-compare-split-r1-opus-model/kernel_measure.sh <副本> <草稿目录>
# 判法闭包探针（照 build.rs 同一套，#[path] 引副本的 judging_closure.rs；探针树建在草稿目录）
bash scripts/hosts/run-with-memory-cap.sh 6G nice -n 19 bash scripts/hosts/capped.sh 8 \
  bash research/prompts/m3-run-compare-split-r1-opus-model/closure_probe_trees.sh <副本> <草稿目录>
```

三条的输出原样在 `outputs/` 下（`volume-and-cpu.out`、`kernel-measure.out`、`closure-probe.out`）。每个文件的 sha256：

```
908d1dd1504267428ab799cdbb781e4d9578ca5002de7c5f633c9795198abc57  ./closure-probe/Cargo.toml
e90a5045e204636566154fefd803e5bb18dce10b7db99df5a29d6c0c6881ed21  ./closure-probe/src/main.rs.template
5265d23e98137f451ebdc1c7d186996595a6bf48c775295f26adba641b9f59bf  ./closure_probe_trees.sh
d5592e85c545e4e6a658e21ac5f7e7dac64136601a265bb341d4ee2b9a8f6194  ./kernel_measure.sh
ac9f72e0fc8626cc15b4e54992734d54e61c08c90aaa88d73795fbf83f324524  ./kernel-size/Cargo.lock
7ec49d2e9a16bb133671e72186b2aae6a59559576dfa543b84a23fdd431a53c0  ./kernel-size/Cargo.toml
4fdfe1b0906c0db4baf3b8a2c2bb8d8e1142dcbd8d570d726cfab11bf151b5e2  ./kernel-size/src/main.rs
d1e8ba4842e7eef2458112721ab49d8e66b9f8f1ae19e2a589bd238bcbccfd88  ./outputs/closure-probe.out
fdb47589c8dd96f153fe441bd0f2ceebff22e16588d306e0d4caecc3319a4d9b  ./outputs/kernel-measure.out
5539769b900ba7ffa2bc60f8b1a39e01e68828394364600e0853137deaf409bb  ./outputs/volume-and-cpu.out
50949c4455dbf92ab010832c219e199499e308e4f033f0899808b618ae19f91a  ./p14_merged_probe_entry.wgsl
7e1407c0caadc8d83f17353fc1d53834f50951ff82f835bdcb17b3f7ef0999a7  ./p14_probe_entry.wgsl
44ca3194b68159f23439cc5744310e6ed2421aae6098fff6e4ae8cf7ab90dd01  ./stub_prelude.py
68772e0c4b10ada460d6a71addfdceaa9545c411a57745cbd729901920e3a4aa  ./volume_and_cpu.py
```

内核量法的自证：同一个小程序在副本的三份内核上量出 334042 / 395566 / 352481，与 `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1502`「("recovery", recovery_kernel_source(), 334_042, 65_958),」及后两行钉值逐个相同（`outputs/kernel-measure.out` 前三行）。生成常量那一段用全 `u32 = 1u` 的替身（`stub_prelude.py`），替身不改表达式个数由这三格相同来证。

## 各格判定一览

| 格 | 攻击面 | 判定 | 数怎么来的 | 节 |
|---|---|---|---|---|
| A1 | 漏 | 打中：RefFS 翻译里历史自己的冷启动恢复判（读回在不在允许集合）住在跑数据那一段，P1–P18 没有它，后面的允许集合靠它的结果 | 读代码 | 一 |
| A2 | 漏 | 打中：历史执行里的 harness 判（分配记录的代、冷启动读回失败）判红就停历史，P8 只列 `judge_by_model` 19 处 | 读代码 | 一 |
| A3 | 漏 | 打中：新加的「录入复用」一层没有剪枝自证；`import_from` 的同块判定比对不在表里 | 读代码 | 一 |
| A4 | 漏（簇） | 打中：调旧引擎 `enumerate_layer0_*` 的测试文件 17 份（3.4 写 14 份），另有 e161 装置与 `layer0_progress` | 命令数的 | 一、六 |
| A5 | 射程判据 | 给判据，不算打中 | 读代码 | 一 |
| B1 | 重建 | 打中：崩溃态「注入过读故障」那一标（拒绝合法性、挂载透明都按它分支）由历史步种与录制流记号现算，3.2 哪张表都没存 | 读代码 | 二 |
| B2 | 重建 | 打中：`14` 的键「一段历史的块键」今天不存在；摆不出崩溃状态的历史没有任何给定状态表键，每步跑数据无处落 | 读代码 | 二 |
| B3 | 重建 | 打中（与 A1 同根）：RefFS 允许集合是有限集合，拆得开；但第二次崩溃那一份要先以第一次读回重起头，跑数据那一段要先做一次「判」或改 RefFS 的口 | 读代码 | 二 |
| B4 | 重建 | 没打中：`01` 那 72 字节罩得住今天消费者读的恢复报告字段 | 读代码 | 没打中 |
| C1 | 错配 | 打中，假绿：丙簇新内核不在 J8 三把结论版本任何一把的闭包里，改了内核、结论照旧复用 | 量过（探针树） | 三 |
| C2 | 错配 | 打中，假绿（推的）：录入版本只罩 core、录制器、模型、RefFS，跑数据那一段自己的提取与序列化代码不在里面 | 读代码 | 三 |
| C3 | 错配 | 打中，代价（不是假绿）：录入版本一族合一，改理想模型就作废全部 `01`，26 亿态 Rust 恢复白跑 | 算的 | 三 |
| D1 | 体量 | 打中：原始 389.8 GiB 只放得下一份；双机每跑一项整份拷库两次、留在 `runs/`，压缩比 5 时 3.3 项占满盘；另两处事实更正（盘余 514G、库 272M） | 算的 + 现查 | 四 |
| D2 | CPU | 打中：每态两遍 Rust 恢复按 E161 参考数 160–723 µs，26.16 亿态 116–525 线程时；小镜像项 GPU 每线程预算 39.7 µs，CPU 成瓶颈 | 算的（参考数是旧树） | 四 |
| D3 | GPU 展开 | 部分打中：P14 并进恢复内核的最少形态 387189（余 12811），判据本体一加就到顶；单独成内核 62660；P1–P3 加的那一截够；扫描内核共用合成路径每处至多 +58 | 量过（模型） | 四 |
| D4 | GPU：P8 | 打中：卡在 `judge_and_advance` 把「比」与「模型往前走」焊在一起（同 E1） | 读代码 | 四、五 |
| E1 | 分界 | 打中：3.1 左列「真跑理想模型、RefFS」不比就跑不下去（P8、P11），左右列在这两处交叉 | 读代码 | 五 |
| E2 | 分界 | 打中：右列 P14、P17 的 Rust 本体是真跑 core 的择配置、择根，要写明 GPU 一侧是 WGSL 重写、Rust 那份降为参照 | 读代码 | 五 |
| E3 | 分界 | 不分辨臂：J6 跳过时判链上没有一处 CPU 读 GPU 产物，今天就是这样，P7 不新增 | 读代码 | 五 |
| F1 | 次序 | 打中：甲先做，但录入版本的入口要等丁一、戊定跑数据代码住哪个文件——环 | 读代码 | 六 |
| F2 | 簇 | 打中：丙与甲（闭包种类）、乙（展开预算闸）都交；戊缺 `model.rs` 与 RefFS `lib.rs`；己没有会红的检查 | 读代码 | 六 |
| F3 | 改名先做 | 没打中（代价写明：改名就作废全部复用） | 读代码 | 没打中 |
| G1 | 第 9 问 | 打中：红步之后，只读镜像那几样与 RefFS 判仍然有效，只有与理想模型比的与「已知红」归类失义；整段统一标「红步之后」会误导 | 读代码 | 七 |
| H1 | 第 8 问 | 没打中：轻型库是几百到几千态，放哪都放得下 | 现查 | 没打中 |

## 一、漏（第 1 问）

### A1 RefFS 翻译里的历史内恢复判，住在跑数据那一段

被判对象许可它的那一句：3.1 左列「真跑理想模型、RefFS（出各版与允许集合）」；P11 只列 B2、B10（崩溃镜像那两票）。

历史（可达，种子任取）：崩溃注入一段历史里抽到一步 `ColdStartRecover`（生成权重里有它，`crates/singlefs-harness/src/history.rs:714`「(HistoryOperationKind::ColdStartRecover, 8),」）。
1. 跑数据那一段跑完历史，照 3.1 左列「出允许集合」，要把 `TranslatedHistory` 翻出来（副本 `crates/singlefs-checker-tier/src/crash_injection.rs:1460` 调 `translate`）。
2. 翻译到那一步，事件是冷启动恢复：`crates/singlefs-harness/src/reference_filesystem_translation.rs:416`「HistoryOperation::ColdStartRecover => match &seen.ending {」交出 `RecoverWithEveryOperationReturned(读回)`。
3. 交给 RefFS 时当场判读回在不在允许集合：同一文件 `:532`「match model.crash_and_recover(crash, read_out.clone()) {」；不在就记一句红、翻译停下（`:568`「"历史里一次恢复读出 {}，允许集合 {}",」），这一步之后的每个崩溃时刻都拿不到允许集合，B2、B10 只能交回这一句红。

所以：这是一次「读回 ∈ 允许集合」的对比，跑在 3.1 划给 CPU 的那一段里；而且这一比的结果决定跑数据那一段后面还出不出允许集合——不先比就录不下去。P1–P18 没有一格罩它（事实表第 2 节的 117 处 grep 词表里有 `.judge_a_crash(`，没有 `crash_and_recover`）。
复现：`grep -n 'crash_and_recover' crates/singlefs-harness/src/reference_filesystem_translation.rs`（命中 `:532` 那一处）。
四句：分辨臂——分（「左列照 3.1 写」与「左列只录事件、判挪到对比段」两条臂在这一格上不同）；系统看得到——跑数据那一段手里有读回与允许集合，看得到；分句——第 1 问「有没有对比不在 P1–P18 里」；跑前改法——正文没给改法。
改法（只在我的读码上推过、被攻过零轮）：RefFS 加一个「不判、以读回重起头」的口（`restart_from_the_recovered_state` 今天是私有的，`crates/singlefs-reference-filesystem/src/lib.rs:451`「fn restart_from_the_recovered_state(」），跑数据那一段一律照读回重起头、把（读回、允许集合）录进 `12`，对比段判属于，并照今天「第一处红之后那一段历史的 RefFS 票都交这一句」的语义在对比段补回停的规矩。代价：RefFS 包今天不在任何一簇的文件清单里（见 F2）。

### A2 历史执行里的 harness 判，判红就停历史

P8 只点名 `judge_by_model` 19 处。同一个循环里另有两样判，判红同样停历史：`crates/singlefs-harness/src/history.rs:5897`「if !violations.is_empty() || harness_judgement.is_some() || model_disagreement.is_some()」。`harness_judgement` 的来源：分配记录改写的代要落在这几次发布的 txg 里（`:3242`「fn allocation_generation_judgement(」，同文件 9 处调用），冷启动读回失败（`:1926`「pub fn harness_judgement_of_outcome(outcome: &StepOutcome) -> Option<HarnessJudgement> {」）。这两样比的是跑出来的分配记录与 txg、读回结局，是对生成物的比较，3.1 两列都没写、P8 没罩；「历史跑到实现自己停」之后它们一样要从库里读、一样要定红步之后怎么办，`14` 的值表（实现交回值与模型的答）装不下分配记录的前后两份。
四句：分辨臂——分；系统看得到——看得到；分句——第 1 问；改法——正文无。

### A3 新加的复用层没有剪枝自证；导入时的同块比对

- `test-script-discipline.md`（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:63`「- 剪枝要自证：在小规模输入上关掉剪枝与复用跑一遍，判定与开着时逐项相同。剪枝自证进快档。」）。3.2 让 `10`–`14` 按「块键 ‖ 录入种类 ‖ 录入版本」存，就是一层新的复用（录过的不再跑）。今天的剪枝自证两份（`crash_amplification_two_flows_share_their_prefix`、`crash_amplification_pipeline_records_checks_and_reuses`，入口 `BOTH_TIERS_GROUPS` 的 `pruning-self-proof` 一行）只罩结论的复用。P1–P18 没有一格给录入复用写自证，C1、C2 那种假绿正是这一层的自证要抓的。
- `import_from`（副本 `crates/singlefs-checker-tier/src/verdict_store.rs:1566`，文档在 `:1561`：库里已有同块同版本而判定不同就报错）是两台结论之间的比对，在 CPU 上、不在表里。3.2 写它「要把 `0F`–`14` 一起导」，那么两台各录同一块（续跑、第二台失败后本机补跑）时录入行也要比，行有 10 MiB 大（D1）。归 J6 一类（参照）还是归对比，要写一句。

### A4 旧引擎的用户不止 14 份

命令：`grep -rlc 'enumerate_layer0' crates/singlefs-checker-tier/tests/ | wc -l` 在副本上得 17；`ls crates/singlefs-checker-tier/tests/crash_enumeration_*.rs | wc -l` 得 14。多出的三份：`crash_points_of_the_instance_table_sweep.rs`、`record_checker_judges_absence_by_the_persisted_set.rs`（事实表丁二列了这两份）、`crash_segments_per_device_and_torn_in_place_overwrites.rs`（事实表与 3.4 都没列，`:405`、`:725` 调 `enumerate_layer0_selecting`、`enumerate_layer0_in_state_slices`）。另有两处非测试的用户：实验装置 `src/bin/e161_crash_state_dedup_and_time_split.rs:3023`，与断点续跑、分片账本 `src/layer0_progress.rs`（文件头第 12 行写写片行、合并前核片数都在 `crash::enumerate_layer0_in_state_slices`）。这几份里的判（`check_records`、观察者闭包）都是 CPU 当场判，丁二不迁它们，「开了 GPU 所有对比都上 GPU」在它们身上不成立；迁了，e161 装置的产物与 `layer0_progress` 的续跑语义跟着变（第 7 问）。

### A5 射程判据（第 1 问后半，给判据，不算打中）

读码得出的判据：被 `scripts/crash/crash-point-replay.sh` 起到的测试目标（合表第 ②③④ 列点到的与 `BOTH_TIERS_GROUPS` 的）连同它们经模块引用够到的代码，在射程里；`crates/singlefs-harness/tests/` 是 harness 档，不经这个入口，不在；`src/bin/` 实验装置归实验复跑，不经这个入口，不在——但 e161 装置调的是同一个旧引擎（A4），丁二若删旧引擎它就编不过，所以它在「改动的射程」里、不在「两段形态的射程」里。`crash_segments_per_device_and_torn_in_place_overwrites.rs` 不在合表、不在 `stage-inputs.tsv`（`grep` 命中 0），只在 checker 档快档里跑：按这条判据它不在两段形态的射程里，但会被丁二的改动打到。推翻条件：入口或门禁 `checker-tier-layer0-replay` 的快档起到了这份文件之外别的 checker 档目标而它有对比。

## 二、对比那一段只读库重建不出的输入（第 2 问）

### B1 崩溃态「注入过读故障」那一标

拒绝合法性的判法要两样读故障的输入：`crates/singlefs-harness/src/mount_transparency.rs:3031`「read_faults: ReadFaultsInTheCrashState,」与 `:3032`「read_faults_under_the_refused_mount: ReadFaultsUnderTheRefusedMount,」。C670、G4、Q3 三条只在 `NoneInjected` 时判，被拒那次挂载自己带故障层时判据复算为假不算不合法（同函数文档）。今天这一标在跑的那一刻现算：副本 `crash_injection.rs:1098` 的 `read_faults_in_a_crash_state`，读的是历史每一步的步种（哪几类步在注入的读故障下读盘）与每一步之后录制流长到多少；挂载透明在副本 `crash_injection.rs:1737` 起按它分两支。
3.2 的六张表里：`0F` 是基线、写表、段、被判根、版本表；`10`/`03` 是各截读回；`11` 是挂载交回的；`12` 是模型各版、允许集合、挂载写出的根——步种序列与「每步之后流长」一样都没有。对比那一段从库里算不出这一标。
造的输入：一段历史里有一步「每块盘最新那一槽系统配置读回全 0 的可写挂载」（这一类步在读故障下读盘），崩溃窗口在它发过写之后；第一截再挂以判据 N-配置 拒。对比段不知道这一态注入过读故障：默认按 `NoneInjected` 判，C670 那条照判（判据在无故障镜像上为真）——报一个不该报的红；默认按 `Injected` 判，所有不带故障的态都跳过 C670、G4、Q3——漏红，假绿。两种默认都错，只有把这一标（或步种与流长）录进库才对。
四句：分辨臂——分（「3.2 照写」与「`12` 多存步种与流长」）；系统看得到——对比段看不到，这就是缺口；分句——第 2 问「哪一样照 3.2 的表装不下或重建不出」；改法——正文无。

### B2 `14` 的键与摆不出崩溃状态的历史

3.2 写 `14` 的键是「块键（一段历史）」，`12` 是「块键（给定状态表那一把，块起点 0）」。今天给定状态表的块键是一张表一把（副本 `crash_amplification.rs:8765` 的 `block_key_of_the_table`：项、种子与规模、表名、基线、写表、接缝、每个持久集合），一段历史按掉电接缝分组有几张表、第三截另有表，「一段历史的块键」这个东西不存在；第二截挂载、第三截二次崩溃的表又依赖跑出来的挂载写，跑之前定不了。
更直接：有的历史一个崩溃状态都摆不出（副本 `crash_injection.rs:1480` 那一处把 `histories_without_any_crash_point` 记 1），它没有任何一张给定状态表、没有任何块键，但它照样每一步都要和理想模型比（P8）。照 3.2 它的 `14` 行无处落。
四句：分辨臂——分；系统看得到——看得到；分句——第 2 问；改法——正文无。改法（零轮）：`14` 与 `12` 改用「项 ‖ 种子与规模 ‖ 历史序号 ‖ 录入版本」这一把历史键，给定状态表的块键另记它属于哪一把历史键。

### B3 RefFS 允许集合：是集合，但第二次崩溃那一份要先「判」才出得来

事实表 P11 问「拆不拆得开」。读码：`allowed_set_for_a_crash_in` 交回的是有限集合（`AllowedSet`，每个元素是「没有文件」或「一个文件的内容」），第一截那一票拆得开：跑数据那一段录集合的摘要清单，对比段判属于。第三截那一票（`judge_a_second_crash_after_the_restart`）要的是「第一次崩溃判绿、以它的读回重起头」的那个 RefFS：今天只有 `crash_and_recover` 判过属于之后才重起头（`crates/singlefs-reference-filesystem/src/lib.rs:419`「pub fn crash_and_recover(」），判不属于就没有重起头的那一份、第三截弃权。跑数据那一段要出第三截的允许集合，就得先判第一截——与 A1 同根。拆法同 A1：照读回无条件重起头、录下第二份允许集合，对比段先判第一截，红了就把第三截那一票改成今天的弃权。这要改 RefFS 包的口。

## 三、键与版本的错配（第 3 问）

### C1 丙簇新内核不在任何一把结论版本里：改了内核，旧结论照旧复用（假绿，量过）

被判对象许可它的几句：3.2「块键照旧只由流程定」；J8 三把结论版本（GPU 判器闭包、CPU 判器闭包 ∪ 设点文件、观察者版本）；3.3 P5 的四条续做断言、P10、P11、P14、P15、P16 上丙簇新写的「录入比对」内核。
今天三把版本各从哪出发：副本 `crates/singlefs-checker-tier/src/judging_closure.rs:637` 起的 `JUDGE_ENTRY_FILES` 只有 `crash.rs` 与 `crash_judge_gpu.rs` 两份，观察者那一把只从观察者代码文件出发（同文件 `:1122`），闭包顺模块引用收，遇到流水线五份（`:678` 起，`crash_amplification.rs`、`crash_judge_dispatch.rs`、`gpu_unit_checks.rs`、`verdict_store.rs`、`layer0_progress.rs`）就停。复用的判法：一块要 `needed` 里每一种结论都在库里才算判过（副本 `crash_amplification.rs:1129`），键是（块键，结论种类，版本）。

历史：
1. 照 P5 把观察者四条断言写进丙簇内核 K（新文件，由流水线派活那一边引——今天派活都在 `crash_amplification.rs` 与 `crash_judge_dispatch.rs`，后者判器不许引）。K 有一处宽了（例：续做步数只核不超过、该核相等）。
2. 跑一趟崩溃点重放（剪枝跑法，带观察者的两项 `multi-record-publish`、`unlink-after-the-multi-record-publish`），观察者结论按（块键，观察者种类，观察者版本）入库，全绿。
3. 修 K（收严成相等）。观察者版本从观察者代码文件出发、不经 K，不变；GPU、CPU 两把也不经 K，不变；块键只由流程定，不变。
4. 再跑同一条命令：`record_crash_points_needing` 见每块三种结论都在，整块算复用，K 的新判法一个状态都没判——上一趟宽判出的绿照旧记绿。

量过（探针树，副本之外另建，`outputs/closure-probe.out`）：照 `build.rs` 同一套在副本上算整张结论代码摘要表（GPU 一行，48 份候选文件各两行，共 97 行），三棵树——原样、加一份内核文件只由 `crash_amplification.rs` 引、再把内核 WGSL 改一个字——整张表的 sha256 都是 `ef5ca944…f430d`，GPU 判器那一把都是 `2b46a29c…10b6`。正对照：同一份内核改由 `crash_judge_gpu.rs` 引，GPU 那一把变成 `b8f2650d…0e66a`，再改那一个字变成 `bb737269…3287`——探针看得见闭包里的改动，看不见闭包外的。
用户动作放开扫（只固定「先跑一趟、再改 K」）：剪枝跑法——中；逐条跑法（`--item` 点到这两项）——中（逐条也读库复用，入口文件头只写 `--full` 清空复用）；全量（`--full` 换新库）——不中；J6 开或用户授权跳过——都中（CPU 那一把也不经 K）。
四句：分辨臂——分（「J8 照旧三把」与「结论版本的入口加上丙簇内核」两条臂在这一格上一中一不中，探针正对照就是后一条）；系统看得到——复用判定那一刻只看块键、种类、版本，K 的改动三样都不进，看不到，这就是病根；分句——第 3 问「复用会不会把旧录入配给新判法（或反过来）判出错的绿」末一分句（这里是旧结论配新判法）；跑前改法——正文没给。
改法（只在我的模型上量过、被攻过零轮）：每一种结论的入口文件清单里加上它用到的录入比对内核（或丙簇内核另立第四种结论、自带闭包）；另加一条会红的自证：`src/` 下凡 `include_str!` 读 `*.wgsl`、或自己 `create_compute_pipeline` 的文件，要么在某一把闭包里、要么在「不判对错」表里写明为什么，否则红。正对照（探针树 kernel3/kernel4）只证了「进了闭包就会换版本」，没证这条自证。

### C2 录入版本没罩跑数据那一段自己的代码（假绿，推的）

被判对象许可它的那一句：`research/prompts/_m3-run-compare-split-r1-body.md:39` 里「录入版本另起一族（录入那一段跑的代码闭包摘要：core、录制器、理想模型、RefFS），块键照旧只由流程定」。
跑数据那一段除了这四件，还要有一段代码把 Rust 交回的东西提取成表行：P2 写录入段在 `record_crash_points_needing`（流水线那份文件）里逐状态跑恢复、写 `10`/`01`；`01` 的读回 SHA-256、txg 宽度、journal 三个计数都是这段代码算的、编的；`11` 的挂载写索引、`12` 的允许集合摘要同理。这段代码住在流水线五份里，闭包按停止表停在它门外；它也不在 core、录制器、模型、RefFS 任何一个的闭包里（harness 不依赖 checker 档）。
历史：提取代码有一处错（例：读回摘要取了空区间，每态都等于「没有文件」那一版的摘要）→ 跑一趟，`10` 入库，对比段拿这份摘要比版本表，「没有文件」那几版对上、别的报红或——若对比只核「摘要在版本表里」——全绿 → 修提取代码 → 录入版本不变（四件都没改）→ 再跑：录入行键相同、照旧复用，旧的错摘要接着被新判法判。
四句：分辨臂——分；系统看得到——看不到；分句——第 3 问「旧录入配给新判法」；改法——正文无。改法（零轮）：录入版本的入口取「写 `10`–`14` 的每个函数所在的文件」，用一条自证钉住（写这几张表的调用方都在闭包里），不按 crate 列四件。

### C3 录入版本一族合一：改模型就作废全部 Rust 恢复录入（代价，不是假绿）

3.2 的录入版本是一把，罩四件。改 `crates/singlefs-harness/src/model.rs`（流水线一个状态都不读它）也换这一把，`10`/`01` 二十六亿态的行全部按新版本重录：按 D2 的参考数是 116–525 线程时白跑。反过来不出错，只是贵。改法（零轮）：按录入种类各一把（`01` 只罩 core 的读路径与提取代码，`12` 罩模型与 RefFS），结论版本再按它读了哪几种录入把那几把并进去——这也就是 C1、C2 的同一个改法。

### 录入版本与 J8 三把怎么并存（第 3 问的正答，推的）

| 改了什么 | 该换的 | 照 3.2 + J8 今天的写法换不换 |
|---|---|---|
| core 写路径 | 块键（流变了） | 换（块键跟流） |
| core 只读恢复路径 | `10`/`01` 录入、读它的三把结论 | 换：录入版本罩 core；GPU 一把经 `crash.rs` 碰巧也罩（`crash_judge_gpu.rs` 引 `crate::crash`，副本第 28 行） |
| 提取与序列化代码 | 对应录入行 | 不换（C2） |
| 丙簇内核 | 用它的那几种结论 | 不换（C1，量过） |
| 理想模型、RefFS | `12`、`14` 与读它们的结论 | 录入换（且连带 `01`，C3）；GPU 那一把不换（探针树上 GPU 闭包 54 份文件里没有 `model.rs` 与 RefFS，有 harness 的 `lib.rs`、`memory_pool.rs` 与 core 的 `recovery.rs`、`mount.rs`）；CPU 那一把对崩溃注入两项经设点文件、`crash_injection.rs`、`history.rs` 够到 `model.rs`、换（读码推的，探针没算这一把的文件清单），对流水线各项不换；新写的模型比对、RefFS 属于两个内核同 C1，不换 |

## 四、落不下（第 4、5、6 问）

### 先更正两处事实（现查）

- 盘余：正文写 569G。这一轮 `df -h ~/.local/share/singlefs` 现查两次，先得 514G、后得 1.8T（同一天里别的会话在删东西）。盘余不是常数，下面按 514 GiB 与 1843 GiB 两档算。
- 库大小：正文写「判定库今天 4.4G（`du -sh ~/.local/share/singlefs/crash-amplification`）」。那是整个家目录；分开量是 `library` 272M、`runs` 4.1G（67 个运行目录）。库本身装着 26 亿态量级的结论只占 272M，大头是双机每跑一项留下的库拷贝（下面 D1）。

### D1 体量：原始只放得下一份，双机的拷库办法让它落不下（算的）

模型 `volume_and_cpu.py`（`outputs/volume-and-cpu.out` 原样）：

```
VOLUME_RAW basis=states_full_body states=2616000000 blocks=39917 raw_144_gib=350.8 raw_160_gib=389.8 fits_once_in_free=True
VOLUME_LZ4 basis=states_full_body assumed_ratio=5.0 on_disk_gib=78.0
VOLUME_LZ4 basis=states_full_body assumed_ratio=20.0 on_disk_gib=19.5
VOLUME_LZ4 basis=states_full_body assumed_ratio=94.0 on_disk_gib=4.1
VOLUME_DICTIONARY basis=states_full_body on_disk_gib_before_lz4=5.56
WRITE_VOLUME basis=states_full_body assumed_ratio=5.0 wal_plus_compaction_gib=1271
WRITE_VOLUME basis=states_full_body assumed_ratio=94.0 wal_plus_compaction_gib=437
TWO_HOST_COPIES basis=states_full_body assumed_ratio=5.0 library_gib=78.0 copies_per_item_run_gib=155.9 item_runs_until_disk_full=3.3 item_runs_until_disk_full_at_1843_gib=11.8
TWO_HOST_COPIES basis=states_full_body assumed_ratio=20.0 library_gib=19.5 copies_per_item_run_gib=39.0 item_runs_until_disk_full=13.2 item_runs_until_disk_full_at_1843_gib=47.3
TWO_HOST_COPIES basis=states_full_body assumed_ratio=94.0 library_gib=4.1 copies_per_item_run_gib=8.3 item_runs_until_disk_full=62.0 item_runs_until_disk_full_at_1843_gib=222.2
TWO_HOST_COPIES_UNCOMPRESSED basis=states_full_body copies_per_item_run_gib=779.6 item_runs_until_disk_full=0.66 item_runs_until_disk_full_at_1843_gib=2.36
```

算法：`01` 每态 160 字节（事实表按字段推的数）× 26.16 亿态 = 389.8 GiB（十进制 418.6 GB，正文的「约 420 GB」）。压缩比没有现成的数，取一个实测参照：库的 `LOG` 末次统计（`~/.local/share/singlefs/crash-amplification/library/LOG` 第 5888 行 `ingest: 17.56 GB`、第 5901 行 Sum 一行 `186.79 MB`、W-Amp 11.3）给出今天结论表 LZ4 之后约 94 倍——但结论表每态只是一个字节的判定向量编号与八个小整数，`01` 里有 32 字节的读回摘要，压缩比只会更低；94 当上界，5 与 20 是我取的两档（推的）。

打中的那一格不是「一份放不放得下」，是双机的拷库办法：`scripts/crash/crash-amplification-two-hosts.sh:1040`「cp -a -- "$local_library" "$library_for_peer" || two_hosts_break_is ignore-snapshot-failure || { echo "  ✗ 拷不了库的快照给第二台" >&2; echo "  → 怎么办：看 $run_directory 可不可写" >&2; exit 2; }」——每跑一项，先把整份本机库拷一份送第二台，再把第二台整份库取回（`fetched/`），两份都留在 `runs/<运行目录>/` 下不删（今天 67 个目录，其中 `crash-amplification-20261008T074510Z-2688179-two-hosts` 那一个 `library-for-peer` 206M、`fetched` 203M）。一份库装全部分项（`.claude/rules/verification.md` 崩溃放量全量那一行），所以每一项都拷整份。录入进了同一份库之后：压缩比 5 时每跑一项拷 155.9 GiB，一趟全量七项拷 1091 GiB，514 GiB 的盘 3.3 项就满、1843 GiB 的 11.8 项就满；不压缩时 514 GiB 一项都撑不完。今天的 `runs/` 从不清，所以压缩比再高也只是推迟：到 94 倍是 62 项（514）或 222 项（1843），按今天「一趟全量七项」算是 9 趟到 32 趟全量。
用户动作放开扫：配置写 `ENABLE_ACROSS_MACHINES=1`（今天 `multi-host.env` 就这样）——中；单机——不拷，不中；`--full` 换新库——新库同样每项拷两份，中。
四句：分辨臂——分（「录入与结论同库」对「录入另起一份不随双机拷的库」）；系统看得到——驱动看得见库多大，今天不看；分句——第 4 问「算出数，给出能落得下的形态」；跑前改法——正文给的三个例（只存摘要、按块滚动删、压缩）都不碰拷库这一步：压缩只把满的时点往后推、只存摘要同理、按块滚动删删的是录入不是 `runs/` 下的拷贝，三个在这一格上都还中。
落得下的形态（推的、零轮）与它丢什么：
1. 录入另起一份库（或同库另一个列族），双机只拷结论表：不丢判别力；代价是第二台对比要读本机录入时得另送（两台各录各比则不用送）。
2. 每块一张去重字典 + 每态每遍一个字节下标（`VOLUME_DICTIONARY` 5.56 GiB 未压缩）：同一块里落到同一版的状态读回摘要相同，无损；块里不同的恢复结局超过 256 种时要退回宽下标（没量过一块有几种）。
3. 只存对比要用的摘要（不存读回全文）：今天 `01` 已经只存摘要，再收就丢「判红之后从库里重建读回内容」那一步，要重跑 Rust 恢复取回现场。
4. 写盘总量另算：WAL 不压缩，入库一遍就是 389.8 GiB 的写，再加压缩后字节 × W-Amp 11.3（`WRITE_VOLUME` 437–1271 GiB 一趟全量），盘放得下，写入量是 SSD 的寿命账，不是这一轮的闸。
三档：轻型（「甲二」∪ 展开 6，入口日志里展开 6 的块是 13 到 261 个状态一类的量）KB 级；重型剪枝只补输入变了的项，量随改动；全量就是上面的 389.8 GiB 原始。

### D2 CPU 代价：Rust 恢复两遍按参考数在小镜像项上比 GPU 慢一个量级（算的，参考数是旧树）

今天的树上没有每态 Rust 恢复两遍的速率。能找到的参考：E161 第二轮单线程那份产物（`research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-threads1-2026-10-05.out` 第 128 行「cell=first_segment_head states=4096 state_elapsed_ns=7343368818 recovery_consult_elapsed_ns=384412448」，第 173 行是 second_quick 那一格），五格的两遍恢复合计 160–723 µs/态。这是 2026-10-05 的树、E142 那两条流的数，只是参考（`.claude/agent-common.md` 找历史数据那一条：读到的是当时的数）。

```
CPU_KEEP_UP gpu_states_per_second_two_hosts=116900 per_thread_budget_ns_to_keep_up=530368
CPU_RECORD cell=first_segment_head ns_per_state=159583 thread_hours=116 hours_on_31=3.74 hours_on_62=1.87 vs_gpu_full_run=0.25 keeps_up_with_gpu=True
CPU_RECORD cell=first_small ns_per_state=357353 thread_hours=260 hours_on_31=8.38 hours_on_62=4.19 vs_gpu_full_run=0.55 keeps_up_with_gpu=True
CPU_RECORD cell=second_quick ns_per_state=722596 thread_hours=525 hours_on_31=16.94 hours_on_62=8.47 vs_gpu_full_run=1.12 keeps_up_with_gpu=False
CPU_KEEP_UP_SMALL_IMAGE_ITEM gpu_states_per_second_local=780738 per_thread_budget_ns_to_keep_up=39706
```

算法：26.16 亿 × 每态纳秒 / 线程数；两台各 32 核（这一轮 host-resources 报的），按 full-run 日志的 `threads=31` 取 31 与 62。GPU 那一边取 `~/.local/share/singlefs/crash-amplification/full-run-20260930-i.log` 第 266 行「CRASH_AMPLIFICATION_TOTAL items=7 failed=0 states=2563727926 red=0 seconds=27288」与同一份日志里各项的 `gpu_states_per_second`。结论：整趟总量上 CPU 录入是 GPU 判的 0.25–1.12 倍；但小镜像项（固定脚本那一项本机两卡 780738 态/秒）要每线程每态 39.7 µs 才跟得上，参考数是它的 4–18 倍，那几项上 GPU 等 CPU。今天开 GPU 而 J6 跳过时这一块一秒都不花，P3 是净加的。
量它的办法（要主 agent 跑，重型）：在打齐 V…E6 的树上，拿流水线最大的两项（删除线那一项与 `multi-record-publish`），没开 GPU、单机、`--expand-up-to` 取两档（6 与一档能在 10 分钟里跑完的），日志里 `checking_seconds` 与现判态数相除得每态 CPU 判的用时；P3 落地之前它含池级 checker（E161 里占 85%），要的是录入段单独的数，就得等丁一把录入段拆出来、让入口打录入段自己的 `recording_seconds`。量到哪算够：七项里镜像最大与最小的各一项、两档展开，四个点的每态纳秒相差在两倍以内就拿最大值外推；相差更大就七项各量。不许拿这个数去缩枚举域（`.claude/main-agent.md`「禁止」第一条），只拿它定分片：录入段按块分给两台全部线程、与对比段经库按块流水（录完一块就放行那一块的对比），GPU 那一侧按块等。

### D3 GPU 展开预算：P1–P3 够，P14 并进恢复内核就到顶（量过：模型）

`outputs/kernel-measure.out` 原样（节选，全文在文件里）：

```
kernel=recovery KERNEL_INLINED entry=main expressions=334042 margin_to_400000=65958
kernel=checker-scan KERNEL_INLINED entry=main expressions=395566 margin_to_400000=4434
kernel=checker-walk KERNEL_INLINED entry=main expressions=352481 margin_to_400000=47519
probe=p14-alone KERNEL_INLINED entry=p14_probe expressions=62660 margin_to_400000=337340
probe=p14-merged-into-recovery KERNEL_INLINED entry=merged_probe expressions=387189 margin_to_400000=12811
FUNCTION name=recover own=116 inlined=154704 copies_under_entry=2
FUNCTION name=choose_system_configuration own=273 inlined=15670 copies_under_entry=1
FUNCTION name=check_records own=156 inlined=7073 copies_under_entry=1
FUNCTION name=parse_system_configuration own=271 inlined=4335 copies_under_entry=3
scan FUNCTION name=version_of_location own=76 inlined=180 copies_under_entry=76
scan FUNCTION name=location_word own=7 inlined=16 copies_under_entry=771
```

- 预算：`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:247`「pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;」。恢复内核的大头是两遍 `recover`（每份 154704，`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:605`「let consulted = recover(true, cfg_choice);」与下一行）。
- P1–P3（读 Rust 录入、与自己的落点比、按录下的落点再跑一遍记录核对器）：多一个 `check_records` 调用点 7073，比十来个字几百——余量 65958 够。若改成「按录下的落点把两遍 oracle 也再跑」，`classify_oracle` 每份 394，也够；若要「按录下的落点重走到文件」，那是再一份 `walk_to_file`（98647），超。
- P14 判据复算：探针 `p14_probe_entry.wgsl` 只摆判据里调到的几段解析各一个调用点（择系统配置、择根、扫 journal、见证那一版往回找根、实例表链两样、命名单元读一份），判据本身的比较一行没写，单独成内核 62660；并进恢复内核（择系统配置复用 main 那一次，另加 P1 那个 `check_records`）是 387189，余 12811。Rust 判据按盘逐槽直读系统配置（`mount_transparency.rs` 第 2352 行那个函数），WGSL 里每一个这样的调用点是一份 `parse_system_configuration`（4335）：三个调用点就把 12811 吃光。所以 P14 只能单独成内核（这与事实表第 4 节的推断相同，这里给了数）。
- P12「每状态两份掩码」若改在三个内核共用的镜像合成路径上（`version_of_location`、`location_word`），扫描内核里它们各有 76 与 771 份：余量 4434 折成 `version_of_location` 每份至多 +58 个表达式、`location_word` 每份至多 +5。改在恢复内核独有的那一截（记录核对器读第二份掩码）不碰扫描内核。
- 判表形态上限（推的，没量）：P5 观察者把挂载写并进给定状态表，卡在 `MAXIMUM_WRITES_PER_LOCATION`（副本 `crash_judge_tables.rs:51`，64）与 `MAXIMUM_ENUMERATED_WRITES`（副本 `crash_facts.rs:287`，2048）。系统配置 0 号槽在错位复用那条流上已经被写 34 次（同文件第 50 行注释），观察者的两项不是那条流，每次挂载各加一次，单看一组到不了 64；组数才是闸——挂载要重放的 journal 记录随崩溃态里落了哪几条而变，组数的上界是「流里 journal 写的子集数」量级，最坏退化成一态一表（事实表第 7 节的推翻条件），没量。

### D4 P8 每步模型比搬上 GPU：卡在「比」与「模型往前走」是同一个函数

`crates/singlefs-harness/src/model.rs:2870`「/// 拿实现的结局与模型的答案比；对得上就按模型自己算的结果往前走（不按实现交回的），对不上交回哪一格。」——`judge_and_advance` 里，实现拒了就只写拒之前做完的那几次发布的根，做成了就 `advance_by_success`，分哪一支看的是实现交回的结局；对不上直接交回错、模型不动。`history.rs:1452` 照它判（`judge_by_model`）。
所以每一步模型的答由上一步的「比 + 走」定：跑数据那一段要录下第 k+1 步的模型答，第 k 步就得先走过 `judge_and_advance`，而它对不上时没有定义怎么走。P8「历史跑到实现自己停」要先补一条规矩：对不上的那一步模型怎么往前走（不走、照实现走、照模型自己走），三种给出三套不同的后续模型答，`14` 存哪一套由它定。GPU 那一半：比的那一半（`capacity_wall_is_permitted` 等要读模型状态，`model.rs:2835` 起）要的是每步的模型状态快照，`14` 里没有；崩溃注入快档一共 24 段历史 × 24 步（`tests/crash_injection_campaign.rs` 的 `FAST_TIER_SEEDS`、`FAST_TIER_OPERATIONS_PER_HISTORY`），GPU 上并行度只有几百，WGSL 里重写一份模型判法等于给项目唯一的功能对照物（`.claude/rules/verification.md@功能正确性靠模型对拍`「这是这个项目唯一的功能对照物，没有现成实现可以拿输出当标准答案。」）造第二份规范。量没法从代码推，同事实表。

## 五、分界越位（3.1 与用户原话逐字对）

用户原话（不攻，只对形态）：记录第 996 行「cpu跑 录入数据 gpu跑对比运算数据」；第 1003 行判据复算那一问的答「初期的环境检测 指纹检测让cpu做。生成物的对比可以交给gpu的都交给gpu」（主 agent 读作通用分界，同一行后半）。主 agent 第 996 行据此写的是「CPU 只录入（跑实现、跑模型、录镜像与读回与允许集合）」。

### E1 左列「真跑理想模型、RefFS」做不成「只跑不比」

- 理想模型：D4 已证，模型的每一步由 `judge_and_advance` 的「比 + 走」定（`crates/singlefs-harness/src/model.rs:2870`），「跑模型」不比就跑不到第二步。3.1 把「真跑理想模型」划进左列、把「每步模型比」划进右列，两样在今天的代码里是一个函数。
- RefFS：A1、B3 已证，RefFS 的「出允许集合」在冷启动那一步与第三截都要先判一次属于。
所以 3.1 的两列在 P8、P11 上不是「改一下调用次序」就分得开，要先改 harness 的模型与 RefFS 包的口（F2）。这不碰用户原话：原话说的是「跑」归 CPU、「比」归 GPU，攻的是 3.1 以为今天的代码已经把这两样分成了两件。
四句：分辨臂——分（「3.1 照写」对「3.1 加一行：模型与 RefFS 先拆口」）；系统看得到——看得到；分句——第 1 问后半「有没有该归 CPU 的步被划进了对比、或反过来」；改法——正文无。

### E2 右列 P14、P17 的 Rust 本体是「真跑 core」

- P14 判据复算：`crates/singlefs-harness/src/mount_transparency.rs:36`「choose_root, choose_system_configuration, chosen_system_configuration_slot_bytes,」——判据函数调的是 core 自己的择配置、择根、扫 journal（同文件 `:2352` 起那几个函数）。
- P17 判红之后归类：`crates/singlefs-harness/src/history.rs:6058`「let system_configuration = choose_system_configuration(image_before_raising).ok()?;」，崩溃注入那一份（副本 `crash_injection.rs:4562`）同样先择配置、择根。它决定一个红记成已知红还是新发现（副本 `crash_injection.rs` 第 471 行起的计数），是改红绿账的一步，不是纯展示。
按 3.1 左列「真跑 core」，这两样的 Rust 本体落在左列；按用户第 1003 行那一答，它们是对生成物（崩溃镜像）的判，落在右列。两种读法都说得通，3.1 只写了右列、没写 GPU 那一份是什么。按右列做，GPU 一侧只能是 WGSL 重写的一份 core 读路径（恢复内核里已有 `choose_system_configuration` 15670、`choose_root` 9001，D3），Rust 那份就成了参照（J6 一类）。要写明的有两句：GPU 那一份与恢复内核共用同一段 WGSL 择根时，「已知红」归类与恢复判共享一个缺陷就一起错（今天 Rust 那边同样是归类与实现共用 core 的择根，独立性不变，但要写出来）；Rust 参照在开 GPU 时跑不跑（J6 默认跑、用户可授权跳过）。
四句：分辨臂——分；系统看得到——看得到；分句——同 E1；改法——正文无。

### E3 不分辨臂：J6 跳过时判链上没有一处 CPU 读 GPU 产物

想打的形状：P7 把第 ③ 段与 J7 挪上 GPU，J6 又由用户授权跳过（`research/prompts/m3-crash-enumeration-merge-r3-main-verification.md:51` 里「跳过由用户当场授权，弹窗写明空真守门一起跳过」），GPU 静默不跑、结论字全 0 时没有一处 CPU 会看出来。核了今天：GPU 判的块的计数向量由 CPU 从结论字解（副本 `crash_amplification.rs` 第 2518 行一带 `count_the_gpu_bits`），全 0 的字解出「判到 0、不适用 = 状态数」，J7 核的判到加不适用等于状态数（副本 `crash_amplification.rs` 第 10295 行那个函数）照样对上；第 ③ 段两边都来自 GPU。今天 J6 跳过时就抓不到，P7 不新增。按证据纪律「打中不分辨臂」，不拿它判 3.1，另记：J6 跳过时要一处 CPU 环境检测（开跑前在卡上判一个已知红的小输入，红不出来就退 78），这正落在用户「初期的环境检测…让cpu做」那一类。

## 六、次序与簇（第 7 问）

### F1 甲先做，但录入版本的入口要等丁一、戊——环

3.4：甲（库布局与录入版本）先做，别簇用它的接口。录入版本是「录入那一段跑的代码」的闭包摘要，入口是哪几份文件，取决于丁一把流水线的录入段放在哪（P2 写在 `record_crash_points_needing`，流水线那份文件、闭包停止表里）、戊把崩溃注入的「跑一段历史、录 `0F`–`14`」放在哪。甲先定入口，丁一、戊后放代码，就是 C2；甲等它们，就不是「先做」。拆法（零轮）：甲只做机制（入口清单是一张表 + 一条自证：写 `10`–`14` 的调用方都在闭包里），清单由丁一、戊各自填，自证在最后一簇合进来时判。

### F2 簇的文件清单与交集漏了几处

| 漏处 | 依据 | 后果 |
|---|---|---|
| 丙「不交」，实际与甲交 | C1：丙的内核要进某一把结论版本，入口清单在 `judging_closure.rs`（甲） | 不交就是 C1 的假绿 |
| 丙「不交」，实际与乙交 | 只有经 `compute_pipeline` 编的判器内核过展开预算闸（事实表第 4 节；`crash_verify_gpu.rs`、`gpu_unit_checks.rs` 自己建管线、不过闸） | 丙自己建管线就绕开 400000 的闸，走闸就要改乙的 `crash_judge_dispatch.rs` |
| 戊缺 `crates/singlefs-harness/src/model.rs` | D4：要加「不判、照规矩往前走」的口 | P8 做不成 |
| 没有一簇管 `crates/singlefs-reference-filesystem/src/lib.rs` | A1、B3：重起头的口今天是私有的 | P11 第三截与冷启动那一判做不成；RefFS 包有自己的依赖闸（只依赖 std） |
| 丁二「14 份」 | A4：17 份测试文件 + e161 装置 + `layer0_progress.rs` | 漏迁的那几份照旧在 CPU 边跑边比；实现员一次至多 8 份文件，要按 17 份切 |
| 己只删标记与表行 | 记录第 996 行：门禁「只核标记与表行对得上，判不出「开了 GPU 仍在 CPU 上的对比」」，要另立会红的检查 | 今天就没标记的几处（挂载透明、I-8.4、每步模型比、第二截记录核对器，事实表第 5 节末）与 A1、A2 留在 CPU 上，己照样能报「零行」；会红的检查不在任何一簇里 |

### F3 改名放在两段之前（没打中，写代价）

改名碰设点文件名：给定状态表的节点名由设点文件、项名、表名三段拼成（副本 `crash_amplification.rs` 第 8765 行那个函数），CPU 判器那一把的闭包文本里有设点文件的路径，所以改名作废全部复用，下一趟是全量。两段改造本来也要抬布局号 2 → 3、换一份库，改名先做只多一趟全量，换来实现员不在别人改名的文件上撞车；我造不出改名先做让结果变错的序列。要写的只有一句：事实表的文件名与行号都是改名之前的，改名之后各簇开工前要重新现查。另：副本里 E5 已把给定状态表那一段挪进 `crash_given_states.rs`，但 `GivenStatesEngine` 仍在 `crash_amplification.rs`（副本第 8361 行），3.4「先把那一节挪成单独文件」照旧要做；事实表 `crash_injection.rs` 的行号是 V…E4 的，与 V…E5 对不上。

## 七、红步之后照判（第 9 问）

被判对象：用户弹窗「照判，报告里标在红步之后」，3.3 P8「历史跑到实现自己停、红步之后照判单列」。
读码分两类（第 k 步每步模型比判红之后，k 之后的崩溃点）：
1. 只读一张镜像的那几样——两遍恢复、七类 oracle（对的是录制流里被判根的版本表，不是理想模型）、池级 checker、记录核对器（B3、B8、P12、P13）：输入只有写表、持久集合与实现自己的恢复，与理想模型无关。历史跑过第 k 步之后写出来的流是实现真写的，这些判定照样是有效判定，而且是今天拿不到的新覆盖（今天第 k 步就停、之后没有崩溃点）。
2. RefFS 那一票（B2、B10）：RefFS 收的是实现交回的结局（`submit(操作, 结局)`），不读理想模型；只要 RefFS 自己的翻译没停（A1），它在第 k 步之后照样有含义。
3. 与理想模型比的（B1、B9 与 P8 自己的后续步）：模型在第 k 步没往前走或照某条规矩走（D4），之后的答是对一段实现没走过的历史给的。这些判绿不证明任何东西，判红多半是第 k 步的连锁。
4. 「已知红」归类（P17）：今天一段历史第一处红就收尾（`HistoryEnding::KnownRed`，副本 `crash_injection.rs` 第 1439 行一带），清单的每一形都按「这段历史的第一处红」写；第 k 步之后的红再拿去对清单，对上对不上都没有清单作者给的含义。
所以「整段标在红步之后」会误导两头：把第 1、2 类有效的判降成存疑，又让第 3 类的绿混进「判了 N 态」的总数。单列的写法（零轮）：按「依不依赖理想模型」分开报——第 1、2 类照常进现判数与红绿；第 3 类只报条数、不进现判数、绿不算通过、红不钉回快档（钉表要的是「在出红的那份代码上同样红」，连锁红换一条模型规矩就变）；第 4 类只给第一处红做。推翻条件：读到第 1 类里有一样实际读了理想模型的状态（例如 oracle 的版本表来自模型而不是录制流），那一样就该挪进第 3 类。
四句：分辨臂——分（「统一单列」对「按依不依赖模型分」）；系统看得到——判那一刻知道这一判读没读模型（看它的输入是哪几张表），看得到；分句——第 9 问「这些判定的含义是什么、单列怎么写才不误导」；改法——正文无。

## 没打中的形状

- B4 恢复报告字段：今天的消费者读 `effective_root`、结局、`journal.prefix_applied`、`verification_passed`、`verification_failed`（副本 `crash.rs` 第 1666、1669 行与几份层 0 测试），事实表 `01` 的字段（结局、实例、全宽 txg、失败点、读回长度与摘要、journal 三个计数）罩得住，条件是三个计数取这三个；`mapping_fallbacks`、`devices_without_a_mountable_system_configuration_slot` 在 checker 档里没有消费者（`grep` 命中 0）。
- 第 ② 段事实核对、第 ③ 段、J7 上 GPU：比的是小整数表，展开量与显存都不碰顶；J7 的期望值（按段现算的态数）是从计划算出的参照，在 CPU 上算、交 GPU 比，不越界。
- 块键「只由流程定」本身：流水线的块键带写表与设点文件身份，core 写路径一变块键就变；给定状态表的块键带项名、种子与规模、写表与每个持久集合（副本 `crash_amplification.rs:8765`），读故障那一项与快档项名字不同、不撞键。造不出两段不同的流程撞同一把块键。
- 第 8 问两档：轻型的库是这一趟新建、跑完删的（入口文件头），展开 6 的块是十几到几百态，放哪都放得下；「轻型 ⊂ 重型」由合表第四列取自同一项的重型部分保证，两段改造不改这一层。只留一处：丁二若把层 0 部分改走 RocksDB，重型层 0 那一半的复用就从「全绿标记 + 进度文件」换成「判定库」，`layer0_progress.rs` 的断点续跑与分片账要么跟着迁、要么留给没迁的那几份（A4），两套复用并存时轻型那几个测试目标走哪一套要写明。
- 改名先做（F3）。
- 试过、没成形的：拿 `13` 的 SHA-256 去重造撞键（同内容两次写在不同偏移）——偏移在 `0F` 的写表里，内容去重不丢偏移；拿两台各录同一块造不一致——两台分块不交（同一个按块号轮转），只有续跑补块才重录，那时走 `import_from` 的比对（A3）。
- 取样范围：内核展开量只量了三个现成内核与两个探针入口，判据本体没写；闭包探针只造了「内核由流水线引」「由 GPU 判器引」两种接法、各改一个字；体量与 CPU 是算术，没跑任何崩溃枚举（这一轮没跑一段历史、一个崩溃状态，`closed_form_state_count` 累计 0 态）。

## 这条腿自己的限度

- 只读代码与算术，加三个小模型；没编 checker 档、没跑任何 checker 档测试、轻型也没跑。闭包探针与内核量法是我另写的两个小程序：前者 `#[path]` 引副本的 `judging_closure.rs` 原文（与 `build.rs` 同一套引法），后者照 `kernel_inlined_size` 另写一份数法、以三个钉值相同自证；两者都只在副本与探针树上跑过，数不进 kb，要在入库装置上重做才能引。
- CPU 速率是 2026-10-05 的旧树、E142 两条流、单线程的参考数；压缩比 94 是结论表的，`01` 的没有实测。D1、D2 里凡写「推的」都是按代码数字段或按一个参照外推。
- 我提的改法（C1–C3、A1、B2、F1、第七节单列写法）都只在我的模型上量过或只是推的、被攻过零轮。改法表：

| 改法 | 修哪一格 | 标记 |
|---|---|---|
| 结论版本的入口加上它用到的录入比对内核 | C1 | 量过（探针树 kernel3/kernel4：进了闭包，WGSL 改一个字 GPU 那一把就换） |
| 录入版本按「写 `10`–`14` 的调用方所在文件」取入口、配自证 | C2、F1 | 推的 |
| 录入版本按录入种类各一把，结论版本并进它读的那几把 | C3、C1 | 推的 |
| 录入另起一份库、双机只拷结论 | D1 | 推的（拷库那一步的代码读过，改法没实现） |
| RefFS 加「不判、照读回重起头」的口 | A1、B3、E1 | 推的 |
| 模型加「不判、照规矩往前走」的口，并先定对不上那一步怎么走 | D4、E1、第 9 问 | 推的 |
| 历史键「项 ‖ 种子与规模 ‖ 历史序号 ‖ 录入版本」 | B2 | 推的 |
| `12` 多存步种序列与每步之后的流长 | B1 | 推的 |
| P14 单独成内核 | D3 | 量过（模型：单独 62660，并进 387189） |
| 按依不依赖理想模型分开单列 | G1 | 推的 |

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳；副本与探针树上的数不算入库装置上的数。
- 没读禁读清单里的两处。
- 没量：今天树上的每态 Rust 恢复速率（要编 checker 档跑流水线，见 D2 的量法，归主 agent）；观察者挂载写的分组数；RefFS 每个崩溃时刻的允许集合大小；`01` 的真实压缩比；P8 每步的数据量。
- 跑前看了负载：别的会话在跑 `cargo test --release -p singlefs-checker-tier …`（重型，不是我起的），没等锁（我只编了两个不依赖 checker 档的小程序）。`apply-writer-patch.py` 的打补丁计数写在副本自己的 `.git` 里，没写到主工作区。
- 草稿目录 `~/.local/share/singlefs/work/m3-run-compare-split-r1-attack/` 里的副本、探针树与编译目录交回前删掉，留打补丁日志与模型输出的草稿。

## 要主 agent 跑的命令

没有必须由主 agent 跑才能交回的；D2 的量法是给下一步的（重型，主 agent 带前缀、照 `.claude/skills/crash-test/commit-stages.md` 的起法跑），这一轮不需要它也能判这份报告。
