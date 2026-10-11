# m3-run-compare-split-r2 攻方腿（Opus）报告（2026-10-10）

立场：假设第一轮判决第三节七条修订形态是错的，造可达的改动序列、输入或跑法去打穿它。只攻分到的三问：第 3 问修补本身、第 4 问产生结论的方法、第 5 问没被看过的地方；不重复第一轮的漏、错配、落不下、分界越位、次序、第 9 问。
代码引用三种：「副本」= `~/.local/share/singlefs/work/m3-merge-impl/e5/repo`（主工作区 + V…E5，只读），带原文的副本引文写绝对路径（cite-check 在主工作区上核不了副本行）；harness、RefFS、`*.wgsl` 与脚本没被补丁动过的写主工作区路径（`crates/singlefs-harness/src/history.rs` 两边 `cmp` 相同，已核）；第一轮的报告与判决写 `research/prompts/` 路径。
每条结论标「读代码」「量过（我的模型）」「算的」「推的」之一。我的模型上的数只是我的模型上的数，不进 kb。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-run-compare-split-r2-opus-model/`。四样都是只用 std（或只依赖 harness 与 core）的小程序或纯 Python，不编 checker 档、不跑任何 checker 档测试；探针树建在草稿目录。

```
# ① 闭包探针：七棵树（原样、内核由 GPU 判器引、再改内核一个字、内核只由流水线引、读故障态测试改每步 checker、crash.rs 纯搬家、crash.rs 对调两边）
bash scripts/hosts/run-with-memory-cap.sh 2G nice -n 19 bash scripts/hosts/capped.sh 6 \
  bash research/prompts/m3-run-compare-split-r2-opus-model/run_probe.sh <副本根> <草稿目录>
# ② 双机按块亲和：两趟权重之间块换归属的比例（纯 Python，自带两组钉值自证）
python3 -I research/prompts/m3-run-compare-split-r2-opus-model/weighted_shares_drift.py 136700,45275 140427,62390 642805,291558
python3 -I research/prompts/m3-run-compare-split-r2-opus-model/weighted_shares_drift.py 136700,45275 136701,45275 273400,90550
# ③ 随机历史写表长度：BROAD 比重、两块 4 GiB 盘、144 步，种子偏移 0–35（我分 8 件并行跑，每件 3 或 6 个种子）
bash scripts/hosts/run-with-memory-cap.sh 4G nice -n 19 bash scripts/hosts/capped.sh 6 \
  bash research/prompts/m3-run-compare-split-r2-opus-model/run_history_writes.sh <副本根> <草稿目录> 0 36 144
# ④ 层 0 切片与判定库块对不对得齐（纯 Python）
python3 -I research/prompts/m3-run-compare-split-r2-opus-model/layer0_slices.py multi-record-layer0-domain=1358954634 delete-30=1448083638 fixed-unmount-28=1610875015
```

自证：① 原样那棵树算出的整张结论代码摘要表 sha256 `ef5ca944…f430d`，与第一轮攻方 `outputs/closure-probe.out` 第 1 行 probe-base 的数相同；k-gpu、k-gpu-edit 两棵与第一轮 probe-kernel3、probe-kernel4 的整表摘要也逐个相同（`8ed2546f…`、`b8fd8c21…`）。② 对上副本单测 `weighted_shares_hand_out_blocks_in_proportion_and_never_skip_a_block` 的两组钉值（`SELFTEST … ok=True`）。③ 不摆崩溃状态，`closed_form_state_count` 合计 0 态。

```
626c1d0de499aeee0989b0d937fa647f7cbbbe6eef2e5ad5b77fe2118e57b151  ./build_trees.sh
0dfdf88874047f529c07fa2ef4475456cc417658f94575068621123923d07895  ./closure-probe2/Cargo.toml
5910ecbcf176e4a6eb10cafe0e23656c8c60fedfb748af3a5b908b38b4c05e3d  ./closure-probe2/src/main.rs.template
82e147c8edb1d15c680903bbbc575f3260d46fa7391e3c5490e755b225df30c6  ./history-writes/Cargo.toml.template
4b6e96bdeff7bc7a72c27ac0eddf6c9a93fabdf49efa6301bcace8c9bb238652  ./history-writes/src/main.rs
5374e881eb1a987932530a8a2004523def7a04309d31352e2c0391afbe4592a7  ./outputs/closure-files-crash_amplification.out
bf15fdec4b24b4c53742f8783cc631a04148723062b78f32597c9f9939cfbb69  ./outputs/cluster0-acceptance.out
dfbe3cd2d282a5270c02ebee6185a3ecd2945ef8fe1a9abc58a5a1f363d1f395  ./outputs/cluster0-move-token-delta.out
762e6b564a4ac21172010c2ba21bd4e39f123609f9179f66784cf38fc8dee3d5  ./outputs/history-writes-144-steps.out
ed550a17e44f615c254a5ed598edbc7f55b1f969d1d75e4b18c9e48e5e56e186  ./outputs/history-writes-small.out
4bfed656dce77ee2933c58dd23a28d58708a8a569aea2e0e3b0d6c0f36f8fd68  ./outputs/owner-drift.out
0efa256d8b2f317470528502ac5ea0c62243e143934bbbc29ebb8160789dde23  ./outputs/owner-drift-small.out
4a16e5e8ada4968c061baaefa948b0622d52fd10d891cc3ab0787fca47565c10  ./outputs/record-entry-closures.out
80b6edc7f5b344819242d01b1e241cdfde6289cd347c79741b4c3baba875a5cc  ./outputs/selfproof2.out
ef5ca944c4faf1f90ca0f48484b8fc8ed1e8a9b3e00afbfc43ee8b56450f430d  ./outputs/table-base.tsv
21ff096b2831f1c03eb1849a253fe5babe9ba76a3830003fd19bae46f835026b  ./outputs/table-diff.out
b8fd8c217f162c784fd6c2074f1694da30d5b2075c359504b120071b1b28b0d9  ./outputs/table-k-gpu-edit.tsv
8ed2546f4bb5bfcd2948c9a3c803fad0b52f6e2c353acb281ea64dc9c16ece39  ./outputs/table-k-gpu.tsv
0577294d9a003e089cf7d57d3bfd3fd8498923df5ee081f8714ade09e2d51e4a  ./outputs/table-t-readfault.tsv
aaea09ce9e876ca6fb886346fd2f3deaa8711dba657208cb3f8a9069b283b551  ./run_history_writes.sh
23141ffceae38768bce96aaaed42d57b21d4b7b33549c1f4e6c8ea25a46895a9  ./run_probe.sh
ce3f6faf40392afe0eabf36f5b84718d71abcd6859716ee72eca8cbbe3b5e054  ./weighted_shares_drift.py
10b526a043e303f8d5685ba6dd6c6bcfc1511ffc6af5528721f0c48ac2d5fb55  ./layer0_slices.py
ff977b8d4e68d79b9e814e4ef0b1dbf2167af7048a48052bf17b494afa10a78d  ./outputs/layer0-slices.out
```

取样与挂钟：① 七棵树各 24M 源码，整条 1 分钟内跑完；② 每对权重算一整个轮转周期（至多 93 万位），秒级；③ 先跑 2 段 24 步估时（每段 2.0–3.6 秒），144 步每段实测 6.6–16.4 秒，36 段分两批、每批 4 件并行（3G 上限），每批 1 分钟内跑完，没缩取样。

## 各格判定一览

| 格 | 问 | 被打的条 | 判定 | 数怎么来的 | 节 |
|---|---|---|---|---|---|
| R1 | 3 | 第 2、3 条：历史键 × 录入版本 | 打中，假绿：历史键是调用方手写的「种子与规模」串，录入版本从写表函数出发、不含设点测试文件；合并判决 H8 让测试文件出发的整份闭包进版本，这一条把它丢了。读故障态测试改一处执行参数，历史键、三把候选录入闭包都不变，CPU 结论版本变——旧 `12`/`14` 配新判法 | 量过（探针 t-readfault） | 一.1 |
| R2 | 3 | 第 3 条自证之二 | 打中，假绿：「要么在某一把闭包里」量词太弱。内核宿主挂在 GPU 判器闭包里，自证绿；观察者那一把版本照旧不动，P5 观察者断言搬进这个内核后改内核、观察者结论照旧复用 | 量过（探针 k-gpu、k-gpu-edit） | 一.2 |
| R3 | 3 | 第 3 条「入口取写各表的函数所在文件」与分族 | 打中，代价与自相矛盾：今天写录入的函数住的三份文件，闭包分别 61、60、49 份；流水线那份闭包含 5 份 WGSL 与 CPU、GPU 两个判器——改内核一个字 `10`/`11` 录入版本就换；崩溃注入那份 `12`、`14` 同一把闭包（含 CPU 判器、RefFS、模型、历史执行器），分族落空 | 量过 | 一.3 |
| R4 | 3 | 第 3 条自证之一 | 打中，假绿（推的）：只往上看一层调用方；测试文件经 trait 调引擎、引擎再写表，测试文件不是直接调用方 | 读代码 + R1 的探针 | 一.4 |
| R5 | 3 | 第 6 条簇 0 验收 | 打中，两头错：词法单元多重集不看次序——真正的纯搬家多了 `use`/`pub(crate)` 几个词，多重集变（假红）；把 `a < b + c` 对调成 `b < a + c`，多重集不变（假绿，语义变了） | 量过（探针 move、swap） | 一.5 |
| R6 | 3 | 第 6 条己「开 GPU 时 CPU 对比入口被调就红」 | 打中，做不成：开 GPU 时 J6 对照与 GPU 判不了时的退回今天就调 CPU 判器入口，这条检查照字面每趟都红；给 J6 开口子之后它分不出 J6 与越位的 CPU 对比 | 读代码 | 一.6 |
| R7 | 3 | 第 7 条（连同第 1 条「RefFS 只录」） | 打中，判错：历史内恢复读出不在允许集合时，今天翻译停、之后的 RefFS 票全交这一句；拆开以后录入段照读出无条件重起头，还把这份读出推进 RefFS 的版本表。第 7 条按「依不依赖理想模型」分，RefFS 自己红之后的票照常记、进总数 | 读代码 | 一.7 |
| R8 | 3、5 | 第 4 条按块亲和、只拷结论 | 打中，做不成（代价）：双机权重每趟重量，同一块两趟之间换归属 40–43%（真权重三组），权重改 1 就有 38% 的给定状态表换主；只改判器之后这一截要在新主那台重录。「或列族」那一读法撞上今天整份 `cp -a` 库给第二台 | 量过（Python 模型） | 一.8 |
| R9 | 3 | 第 5 条 | 打中，缺行：P12（第二截记录核对器两份掩码）与 P17（判红后归类，用户定上 GPU）没有落处；P12 落在三内核共用的镜像合成路径上就碰扫描内核 4434 的余量 | 读代码 + 第一轮的量 | 一.9 |
| R10 | 3 | 第 2 条「给定状态表块键记它属于哪一把历史键」 | 部分打中（结构，推的）：块键按内容取、跨历史去重，一把块键可以属于几段历史；单值的「属于」装不下，读 `12` 的结论按块键复用会跨历史复用 | 读代码，可达性没量 | 一.10 |
| Q1 | 4 | 第一轮闭包探针 | 方法偏：只造两种接法、只报 GPU 一行，没看观察者与 CPU 行，第 3 条那条自证由它推出，太弱（R2） | 量过 | 二.1 |
| Q2 | 4 | 正推腿「约 60 步封顶」 | 打中（数偏紧）：36 段 144 步历史，写表第一次过 2048 在第 71–136 步，2 段到 144 步都没过；第 24 步时 76–736 次写（正推腿推约 900） | 量过 | 二.2 |
| Q3 | 4 | 体量与 CPU 的态数基数 | 方法偏：两腿的基数（26.16 亿、30 深下界 37.4 亿）之外，9/30 那趟另有一条流 `exhaustive=false`；基数还是下界 | 现查日志 | 二.3 |
| Q4 | 4 | 第 5 条的新内核与 P14 展开量 | 偏向已知：P14 两个数是下界（判据本体没写），偏差只会更支持「单独成内核」；六合一新内核「≲ 2 万」是推的、没量 | 读两腿报告 | 二.4 |
| Q5 | 4 | 翻面表 | 给出：哪个数翻了哪一条翻面 | 推的 | 二.5 |
| N1 | 5 | `layer0_progress` 双机分片 | 打中，做不成：层 0 按切片序号 `% n` 分台，片长 ≈ 态数 ÷ 65536，与 65536 态一块对不齐；30 深多记录流一块横跨 3–4 片、两台交替，按块亲和对迁过来的层 0 用例无从做起 | 算的 | 三.1 |
| N2 | 5 | 判红钉回快档 | 打中（推的）：按历史键钉的红例在生成器或执行参数变了之后指到另一段历史；只在 GPU 上红的 P1–P3 比不上，在不开 GPU 的轻型里复判不出「同一条断言」 | 读代码 | 三.2 |
| N3 | 5 | 双机 × `12`/`14` | 部分打中（代价）：历史键的行没有归属规则，两台各跑整批历史，`12`/`14` 两台各录各比一遍 | 读代码 | 三.3 |
| N4 | 5 | 续跑、`import_from`、变异表、广谱变异、e161 | 没打中新的（续跑落到 R1 那一格；`import_from` 落到 R8 那一格；见「没打中的形状」） | 读代码 | 三.4 |

## 一、第 3 问：修补本身

### 一.1 R1 历史键与录入版本丢了 H8 的「测试文件出发的整份闭包」（假绿，量过）

被判对象许可它的两句（第一轮判决第三节，原行）：

```
research/prompts/m3-run-compare-split-r1-main-verification.md:55
2. **表**：`12`、`14` 用历史键（项 ‖ 种子与规模 ‖ 历史序号），给定状态表块键记它属于哪一把历史键；`12` 多存步种序列与每步之后的流长（或直接存每个崩溃态的读故障标）；`14` 存模型派生常量；`10` 定长字段收到比较要用的那几样。
research/prompts/m3-run-compare-split-r1-main-verification.md:56
3. **版本**：录入版本按录入种类分族（`10`/`11` 罩 core 读路径与录制器、`12` 罩模型与 RefFS 与翻译、`14` 罩 core 与模型），入口取写各表的函数所在文件，配一条「写这几张表的调用方都在闭包里」的自证；结论版本的 `extra_inputs` 绑它读的录入版本；每种结论的入口罩它用到的内核，配一条「`src/` 下 `include_str!` 读 `*.wgsl` 或自建管线的文件要么在某一把闭包里、要么登记不判对错」的自证。
```

今天为什么不出事：合并第三轮正文 H8（`research/prompts/_m3-crash-enumeration-merge-r3-body.md:38`，第三轮判决 J8 采了）末两句是「随机历史两项的键 = 种子 + 规模参数 + 测试文件出发的整份闭包，判定按段存。不做等价剪枝。」（`research/prompts/_m3-crash-enumeration-merge-r3-body.md:38`），副本里落成 `crash_amplification.rs:8256`「/// （H8「随机历史两项的键 = 种子 + 规模参数 + 测试文件出发的整份闭包，判定按段存」：种子与规模进块键，闭包进版本）。」——种子与规模串是调用方手写的，罩不全输入，靠两样兜：给定状态表的块键按内容取（基线、写表、接缝、每个持久集合，`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:8776`「"given-states-input-v1\nitem={}\nseeds_and_scale={}\ntable={}\nbase={}\n",」），CPU 结论的版本从设点测试文件出发。第 2 条把 `12`/`14` 改挂在历史键上（不按内容），第 3 条的录入版本从「写各表的函数所在文件」出发（不含测试文件）——两样兜底在 `12`/`14` 上一样都没有了。

改动序列（可达，只改一个测试文件的一行）：
1. 读故障态那份测试的种子与规模串是手写的：`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs:371`「"seed_base={SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE} seeds_searched={SEEDS_SEARCHED} histories={histories} operations_per_history={OPERATIONS_PER_HISTORY} weights=NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS width={WIDTH:?}"」——比重写的是名字不是值，执行参数里只有宽度；每步 checker、空间准入、取哪几段历史（`continue` 掉一步没发写的种子）、崩溃态取哪几个前缀都不在里面。
2. 照两段形态跑一趟：录入段跑每段历史、录 `12`/`14`（键 = 项 ‖ 这串 ‖ 历史序号），对比段判，全绿入库。
3. 改这份测试里历史执行的一行：`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs:188`「per_step_checker: PerStepChecker::Skipped,」改成 `Run`（`crates/singlefs-harness/src/history.rs:5281` 那一支「Run,」：每一步写过盘之后都跑池级 checker，判红就停）。同类的还有换比重常量而忘改串里那个名字、改取历史的判据。
4. 再跑同一条命令：历史键（串没变、序号没变）不变；`12`/`14` 的录入版本不变（量过，下面）；结论版本变（CPU 结论从设点文件出发，量过）。录过的不再跑：`14` 照旧是 `Skipped` 那一趟录的，对比段的新判法读它判——`Run` 那一趟每一步的 checker 判一次都没发生，结论照旧绿。

量过（探针，`outputs/table-diff.out` 与 `outputs/record-entry-closures.out` 原样节选）：

```
TABLE_DIFF from=base to=t-readfault kind=cpu_judge rows=48 changed=1
TABLE_DIFF from=base to=t-readfault kind=gpu_judge rows=1 changed=0
TABLE_DIFF from=base to=t-readfault kind=observer_code rows=48 changed=1
ROW_CHANGED from=base to=t-readfault kind=cpu_judge file=crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs
tree=base CLOSURE entries=crates/singlefs-checker-tier/src/crash_injection.rs files=60 digest=7d1ed58ae13da543955f95fed60c0a0f03ce334592ca6469363e7530a0d822da wgsl=0 …
tree=t-readfault CLOSURE entries=crates/singlefs-checker-tier/src/crash_injection.rs files=60 digest=7d1ed58ae13da543955f95fed60c0a0f03ce334592ca6469363e7530a0d822da wgsl=0 …
tree=base CLOSURE entries=crates/singlefs-checker-tier/src/crash_given_states.rs files=49 digest=3743e97ca5effeca6ee2d486cd5752c938a7ba12e9a349a1b7648d1c8955f4a5 …
tree=t-readfault CLOSURE entries=crates/singlefs-checker-tier/src/crash_given_states.rs files=49 digest=3743e97ca5effeca6ee2d486cd5752c938a7ba12e9a349a1b7648d1c8955f4a5 …
```

（`…` 处是我截掉的同一行后半：几个是否含某文件的标记与停在门外的清单，原行在 `outputs/record-entry-closures.out`。）三份候选写表文件（流水线 `crash_amplification.rs`、崩溃注入 `crash_injection.rs`、给定状态表 `crash_given_states.rs`）出发的闭包，改测试那一行前后摘要逐个相同；CPU 结论那一行（设点文件那一把）变了。崩溃注入战役那份测试的串写 `{:?}` 把执行参数整个带上（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:81`「"first_seed={} seed_count={} operations_per_history={} draw={:?} weights={:?} execution={:?}",」），同一处改动在它那里会换键；所以缺口是「键靠调用方自觉」，读故障态那份今天就没罩全。

用户动作放开扫（只固定「先跑一趟、再改测试文件一行」）：剪枝跑法——中；`--item` 点这一项——中（逐条也读库复用）；`--full` 换新库——不中；轻型（每趟新库）——不中；双机——中（两台各自复用自己那份 `12`/`14`）。
四句：分辨臂——分（「历史键 + 写表函数入口」对「历史键按内容取，或录入版本并上设点文件出发的闭包」一中一不中）；系统看得到——复用判定那一刻只看历史键、录入种类、录入版本，测试文件那一行三样都不进，看不到，这就是病根；分句——第 3 问「历史键与给定状态表块键之间的对应会不会错配」与「旧录入配新判法」；跑前改法——第 3 条给的两条自证都不中（自证之二只管 WGSL；自证之一见 R4，测试文件不是写表函数的直接调用方）。
改法（推的，被攻过零轮）：甲、历史键按内容取——生成好的操作序列、执行参数（`HistoryExecution` 整个 `{:?}`）、几何、取历史与取崩溃态的判据所在文件摘要，一起取 SHA-256；乙、`12`/`14` 的录入版本照 H8 并上设点文件出发的闭包（与 CPU 结论那一把同一个入口集），代价是改测试文件就重录这一项的 `12`/`14`（随机历史两项的量小，见三.3）。乙更接近 H8 原文，甲不依赖调用方。

### 一.2 R2 自证之二的量词：「在某一把闭包里」放过了「在错的那一把里」（假绿，量过）

被判对象许可它的那一句：第 3 条末尾（原行见一.1 第二段代码块）「每种结论的入口罩它用到的内核」是规则，配的自证只要求读 `*.wgsl` 的文件「在某一把闭包里」。第 5 条把 P5 观察者断言与 P7、P10、P11、P15、P16 放进同一个新内核，观察者是单独一种结论，它的版本只从观察者代码文件出发：`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/judging_closure.rs:1122`「Self::ObserverCode { observer_code_file } => vec![observer_code_file.as_str()],」。

改动序列：
1. 丙簇写新内核 K，宿主文件 `crash_record_compare_gpu.rs` 用 `include_str!` 读 `crash_record_compare.wgsl`，由 GPU 判器那份文件引（今天三个内核的源码就住在那里：`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:64`「pub const GPU_JUDGE_COMMON_SOURCE: &str = include_str!("crash_judge_common.wgsl");」）。管线由派活那份建（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:415`「device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {」，主工作区与副本相同），派活在停止表里、登记不判对错。
2. 照第 3 条跑自证之二：绿（下面）。
3. 跑一趟，观察者结论按（块键，观察者种类，观察者版本）入库。
4. 修 K 一处（例：续做步数只核不超过、该核相等），再跑：观察者版本不变，整块复用，K 的新判法一个状态都没判。

量过（`outputs/selfproof2.out` 与 `outputs/table-diff.out` 原样节选；自证照第 3 条字面仿写：`src/` 下 `include_str!` 读 `.wgsl` 或出现 `create_compute_pipeline` 的文件，在整张表任一把闭包里或在登记表里就绿；登记表放今天自建管线的三份）：

```
== tree=k-gpu
SELFPROOF2_FILE file=crates/singlefs-checker-tier/src/crash_record_compare_gpu.rs reads_wgsl=true builds_pipeline=false in_some_closure=true registered=false gpu_closures_holding_it=1 observer_closures_holding_it=4 verdict=green
SELFPROOF2 checked=5 red=0 verdict=green
== tree=k-amp
SELFPROOF2_FILE file=crates/singlefs-checker-tier/src/crash_record_compare_gpu.rs reads_wgsl=true builds_pipeline=false in_some_closure=false registered=false gpu_closures_holding_it=0 observer_closures_holding_it=0 verdict=red
SELFPROOF2 checked=5 red=1 verdict=red
TABLE_DIFF from=k-gpu to=k-gpu-edit kind=cpu_judge rows=48 changed=4
TABLE_DIFF from=k-gpu to=k-gpu-edit kind=gpu_judge rows=1 changed=1
TABLE_DIFF from=k-gpu to=k-gpu-edit kind=observer_code rows=48 changed=4
```

变了的 4 行观察者行全是 GPU 对拍那几份测试自己（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 等，它们 `use` 了 GPU 判器）；真正的观察者代码文件那一行（`tests/common_crash_point_observers/mod.rs`，副本 `:398` 把它填成 `observer_code_file`）三棵树都是 `9d9e25f37f6ff074b23c70cb88a46e147fc1271e16296e020b8edf9fd1e003d3`（`outputs/table-base.tsv`、`table-k-gpu.tsv`、`table-k-gpu-edit.tsv` 里 `observer_code	crates/singlefs-checker-tier/tests/common_crash_point_observers/mod.rs` 那一行）。k-amp 是负对照：宿主只由流水线引（第一轮 C1 那种接法），自证红——它抓得住第一轮那一格，抓不住这一格。
用户动作放开扫：观察者挂在哪几项都一样（版本只看观察者代码文件）；J6 开或跳过——都中（J6 比的是 GPU 与 CPU 判器的恢复字与红项，副本 `crash_amplification.rs:9271` 那一处，观察者不在里面）；`--full`——不中。
四句：分辨臂——分（「在某一把里」对「在用它的那一种结论的那一把里」）；系统看得到——自证那一刻看得到每一把的文件清单（我的探针就是这么数的），但看不到「哪种结论派了哪个内核」：派活住在停止表里的流水线与派活文件，这层对应今天没有任何登记；分句——第 3 问「两条自证能不能被绕开」；跑前改法——第 3 条的规则句本身对，配的自证碰不到这一格。
改法（推的，被攻过零轮）：甲、把内核宿主按结论种类拆文件，每种结论的入口清单写死它的宿主文件，自证改成「派了内核的每一处，所在结论种类的闭包含那个内核的宿主」——要先有一张「派活处 → 结论种类」的登记，这张表本身又是手写的；乙、跑时绑：派活那一刻把实际编进管线的源码摘要（派活文件里已有现成的一份，`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:163`「fn kernel_sources_digest(kernel_sources: &JudgeKernelSources) -> [u8; DIGEST_BYTES] {」，今天只当编译缓存的键）拼进那一种结论的 `extra_inputs`。乙的判别子在做决定那一刻看得到，甲看不到。

### 一.3 R3 「入口取写各表的函数所在文件」量出来的闭包：分族落空，改内核就重录（代价，量过）

今天写录入的函数住三份文件：流水线的录入段 `record_crash_points_needing`（副本 `crash_amplification.rs:1113`，事实表 P2 就写在这里）、崩溃注入的录写（`crash_injection.rs`）、给定状态表（`crash_given_states.rs`）。照第 3 条「入口取写各表的函数所在文件」，从这三份出发、照今天的停止表收（`outputs/record-entry-closures.out` 原样节选）：

```
tree=base CLOSURE entries=crates/singlefs-checker-tier/src/crash_amplification.rs files=61 digest=56a6b9c123581c513dadf62141acebbf51c73e3b4d085a3a1c3c0432ac5ef2f4 wgsl=5 crash_rs=true crash_judge_gpu_rs=true harness_model_rs=false harness_history_rs=false reffs_lib_rs=false core_recovery_rs=true …
tree=base CLOSURE entries=crates/singlefs-checker-tier/src/crash_injection.rs files=60 digest=7d1ed58ae13da543955f95fed60c0a0f03ce334592ca6469363e7530a0d822da wgsl=0 crash_rs=true crash_judge_gpu_rs=false harness_model_rs=true harness_history_rs=true reffs_lib_rs=true core_recovery_rs=true …
tree=base CLOSURE entries=crates/singlefs-checker-tier/src/crash_given_states.rs files=49 digest=3743e97ca5effeca6ee2d486cd5752c938a7ba12e9a349a1b7648d1c8955f4a5 wgsl=0 crash_rs=true crash_judge_gpu_rs=false …
tree=k-gpu CLOSURE entries=crates/singlefs-checker-tier/src/crash_amplification.rs files=63 digest=55d4a945b751593c87054f4bbbb850c58bf2ea0e4fdcbe06adffc727de71d978 wgsl=6 …
tree=k-gpu-edit CLOSURE entries=crates/singlefs-checker-tier/src/crash_amplification.rs files=63 digest=7ce5e0bf3dedff778ce4bfbbb6d43832a101fbfac12325f0dff3b8ff10a7c90c wgsl=6 …
```

（入口自己即使在停止表里也照收：副本 `judging_closure.rs` 的 `closure_members` 只对「被引到的」判停。）三处与第 3 条前半句的分族对不上：
- `10`/`11` 写在流水线那份：闭包 61 份，含 5 份 WGSL、CPU 判器 `crash.rs`、GPU 判器 `crash_judge_gpu.rs`。k-gpu → k-gpu-edit 只改内核一个字，这一把就换（`55d4a945…` → `7ce5e0bf…`）：改一个判器内核，26 亿态的 Rust 恢复两遍全部重录。拆两段最值钱的那一样（「判器改了只重比、不重录」，正推腿 6.3 节的表）没了。
- `12`、`14` 都写在崩溃注入那份：两族是同一把闭包（60 份，含 CPU 判器、RefFS、模型、历史执行器），「`12` 罩模型与 RefFS 与翻译、`14` 罩 core 与模型」没分开；改 CPU 判器 `crash.rs` 一处也换这两族。
- 第 6 条簇 0 只拆 `crash.rs`、`crash_injection.rs`、挪 `GivenStatesEngine`，流水线那份不拆；写 `10` 的函数留在那里，上面第一条就照旧。

四句：分辨臂——分（「入口 = 写表函数今天所在的文件」对「写表函数先搬进只引 core 读路径与录制器的叶子文件，再取入口」）；系统看得到——看得到（闭包清单）；分句——第 3 问「编不出或做不成」（分族这一句照入口那一句做不出来）；跑前改法——第 1 轮 C3 给的「按录入种类各一把」正是这一条，入口那一句把它抵消了。
改法（推的，被攻过零轮）：入口句改成「写表函数搬进各族自己的一份叶子文件，入口取那份」，另配一条会红的检查：每族闭包里不许出现 `*.wgsl`、CPU 判器入口文件与另一族的叶子文件（我的探针按这三样数就能判）。簇 0 的清单里加「流水线录入段搬出 `crash_amplification.rs`」。

### 一.4 R4 自证之一只往上看一层（假绿，推的）

「写这几张表的调用方都在闭包里」判的是写表函数的直接调用方。录进 `12`/`14` 的东西是谁定的，要再往上走：读故障态那份测试自己跑历史（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs:212`「let run = execute_history_with(&history, execution, &stream, &mut |observation| {」），再经 trait 对象把表交给引擎（同文件 `:159`「judge: &dyn GivenStatesJudge,」、`:165`「.judge_the_given_states(table)」；trait 在副本 `crash_given_states.rs:122`，实现在副本 `crash_amplification.rs:9211`）。录入段若照今天的形状写——测试调引擎的一个方法、引擎再调写表函数——直接调用方是引擎那份 src 文件，测试文件在两层之上，这条自证照字面判绿，R1 那一行的改动照样漏。
四句：分辨臂——分；系统看得到——自证那一刻能看到调用图的一层，看不到「这一行值从哪来」；分句——第 3 问「两条自证能不能被绕开」；跑前改法——R1 的改法乙（录入版本并上设点文件出发的闭包）在这一格上中，自证之一本身不中。没造探针（要先有写表函数才数得出调用方），所以标推的。
改法（推的，零轮）：不判调用方，判输入：录入行的键或版本要盖住它的全部输入来源，随机历史两项就是 H8 那句「测试文件出发的整份闭包」。

### 一.5 R5 簇 0 的验收「判法摘要的词法单元多重集不变」两头错（量过）

被判对象许可它的那一句：第 6 条「纯搬家、判法摘要的词法单元多重集不变当验收」（原行见下）。

```
research/prompts/m3-run-compare-split-r1-main-verification.md:59
6. **簇**：簇 0（按文件拆 `crash.rs`、`crash_injection.rs`、挪 `GivenStatesEngine`，纯搬家、判法摘要的词法单元多重集不变当验收）→ 甲（只做通用表框架与版本机制）→ 乙、丁一、戊（戊加 `model.rs` 与三份集成测试）→ 丙（排在丁一、戊的 CPU 参照之后）→ 丁二（17 份，三份测旧引擎功能的定迁或留；旧引擎留到 e161 与这三份有去处）→ 己（删豁免表，同批立「开 GPU 时 CPU 对比入口被调就红」的检查）；脚本侧一件工具员活。
```

两棵树（照 `crates/singlefs-checker-tier/src/judging_closure.rs` 的 `lexical_tokens` 切词，闭包照 CPU 判器入口 `crash.rs` 与 GPU 判器入口两把收）：
- move：把副本 `crash.rs` 里根记录回退下界那个常量与 `rollback_floor_written_by` 原样挪进新文件 `crash_rollback_floor.rs`（函数加 `pub(crate)`，新文件补两行 `use`，`crash.rs` 补一行 `use`，`lib.rs` 加 `pub mod`）。没编译，形状照真搬家写。
- swap：副本 `/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash.rs:231`「&& later_start < sector_offset + SECTOR_BYTES」对调成 `&& sector_offset < later_start + SECTOR_BYTES`。这是记录核对器「这一扇区被后来一次合法写盖过」的区间相交判定，对调之后判的是另一个区间，语义变了。

`outputs/cluster0-acceptance.out` 原样：

```
tree=base TOKENS entries=crates/singlefs-checker-tier/src/crash.rs files=47 token_count=396110 multiset=0e9f18e91f6205d5761925f5508db3eb7ac21c52ad9b70ca793b96dfdf1b95cf sequence=9a973494bf57a1dc406cb5e5a2c81453eab211c2a1718a967afc1db0a2fe7eb8 item_multiset=f22e6b07b188900dbfd8b68046d4e2c3a87c1f739871813a8dcf58bc76de84d3 items=2744
tree=base TOKENS entries=crates/singlefs-checker-tier/src/crash_judge_gpu.rs files=54 token_count=414522 multiset=b245e3fc1d30518daa8b72c3fb9143747027805e3aa05745c683140fd42a7ae6 sequence=94ce5cbf16cbe7fe0f2b6b0b19978925d0097cd2e8b65d4848c754f85acf209a item_multiset=587a00a3f414a2099d8cf34512b9431145ef72e27bb11f0db7b6afdd4d9eeb31 items=2897
tree=move TOKENS entries=crates/singlefs-checker-tier/src/crash.rs files=48 token_count=396138 multiset=3757a961d7546ba34ff77a8dc1ea4d886fed781b9752659a59f8c4e64f5e3096 sequence=250ff69aebcc7cc7fa0c08da6e8cc8782dfa7e5b8a805678553eb2bea8b47eda item_multiset=f22e6b07b188900dbfd8b68046d4e2c3a87c1f739871813a8dcf58bc76de84d3 items=2744
tree=move TOKENS entries=crates/singlefs-checker-tier/src/crash_judge_gpu.rs files=55 token_count=414550 multiset=7e9a4e72cebb54ed4f7b1a2070b137985fbe58c5f7056365f1555f719a544032 sequence=7fc6608599e741a6b317b1c7156d2025b1e036af7fdeb62e27ec3745d5fc7384 item_multiset=587a00a3f414a2099d8cf34512b9431145ef72e27bb11f0db7b6afdd4d9eeb31 items=2897
tree=swap TOKENS entries=crates/singlefs-checker-tier/src/crash.rs files=47 token_count=396110 multiset=0e9f18e91f6205d5761925f5508db3eb7ac21c52ad9b70ca793b96dfdf1b95cf sequence=f59ff30f0cad0973ee55d5a20722850e2c9cd615723c10db29cdf7ffd5540ea7 item_multiset=469d0c2e69a2874b0187ff616ce390b4af2fecbd0f0962b09193df5e803e1c22 items=2744
tree=swap TOKENS entries=crates/singlefs-checker-tier/src/crash_judge_gpu.rs files=54 token_count=414522 multiset=b245e3fc1d30518daa8b72c3fb9143747027805e3aa05745c683140fd42a7ae6 sequence=262605b27b0da8e9a959cf169bcd2def5a42c449c2edbb2272ebead6fc684994 item_multiset=1a319915f6bd6f26c9141bc616a219c23ddb4074296b859f088974d1b7682d81 items=2897
```

- 纯搬家：多重集变了（两把都变，`0e9f18e9…` → `3757a961…`；多出 28 个词，`outputs/cluster0-move-token-delta.out` 逐词列：`use`、`crate`、`crash_rollback_floor`、`pub`、`(`、`)`、`:`、`;`……）。照字面验收，簇 0 每一次真搬家都判不过——要么验收改松（给 `use`/`mod`/可见性开口子），要么验收形同虚设。
- 对调：多重集逐字相同（`0e9f18e9…` = `0e9f18e9…`，GPU 那一把 `b245e3fc…` = `b245e3fc…`），语义变了。多重集不看次序，任何换位、换操作数、把 `if` 两支互换都过得去。
四句：分辨臂——分（「多重集」对「按项的多重集」，下面）；系统看得到——验收那一刻手里有两棵树的全部词序，看得到，是摘要选错了；分句——第 3 问「判错、假绿」；跑前改法——第 6 条只给了这一个验收。
改法（只在我的模型上量过、被攻过零轮）：按项比——每份文件按顶层项切（花括号深度回到 0 或深度 0 的 `;`），丢掉 `use` 与 `mod x;` 两种项、去掉项头的 `pub`/`pub(…)`，取「项的词序列」的多重集。上面两棵树上它分得开：move 不变（`f22e6b07…` = `f22e6b07…`，`587a00a3…` = `587a00a3…`），swap 变（`469d0c2e…`、`1a319915…`）。它的漏处（推的）：同名项在两份文件之间互换、`use … as …` 改名把调用接到另一个函数——要另加一条「每个被引名字解析到的项不变」。

### 一.6 R6 己的检查「开 GPU 时 CPU 对比入口被调就红」与 J6 撞（做不成，读代码）

第 6 条末尾（原行见一.5）要己同批立「开 GPU 时 CPU 对比入口被调就红」。开 GPU 时 CPU 判器今天在两处被正当地调：
- J6 对照：`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:9271`「let cpu = cross_check_runs.then(|| self.judged_on_the_cpu_here(table));」，判完还按 CPU 种类入库（同文件 `:9142`「self.store_the_conclusions(key, JudgedBy::CpuJudge, cpu, JudgingDevice::Cpu)?;」）。第 1 条自己也写「判器自证与手摆镜像用例是参照，留 CPU」，正文 3.1 写「J6 对照：开 GPU 时 CPU 判器在同一批数据上也判，从库里读」。
- GPU 判不了的状态退回 CPU 判（同文件 `:9148`「None => self.judged_on_the_cpu_here(table),」，这一趟已判红）。
所以照字面，这条检查在每一趟开 GPU、带对照的跑法上都红。要它不红只能给 J6 那一路开口子（另一个入口、或一个「这是对照」的标），开了口子它就分不出「对照」与「把某一样对比偷偷留在 CPU 上、走同一个口子」：两者调的是同一个函数、读的是同一批录入。
四句：分辨臂——不分辨三列形态与旧形态，打中的是这一条检查自己；系统看得到——入口被调的那一刻看得到「谁调的」，看不到「这一份结果最后是不是当结论用」——这是「判别子观测不到」那一形，要改判据；分句——第 3 问「做不成」；跑前改法——没有别的改法。
改法（推的，零轮）：判「结论从哪来」而不是「入口被调」：开 GPU 的一趟收尾时，每一块被当结论用的那一份必须是 GPU 种类、块属性 `JudgingDevice` 是 GPU（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/verdict_store.rs:404`「/// 这一块在哪判的（块属性，第三轮判决 J8、M2-5）：CPU，或 GPU 连同卡型与驱动版本。不进版本：两台驱动不同不分版本，靠对照与轻型的小输入对拍抓差。」——这一样今天就入库）；CPU 那一份只许以对照种类出现。对还没进库的对比（P8、P10、P11、P14–P16 之前的样子），按种类数「这一趟交 GPU 判的块数 = 这一种结论现判的块数」。

### 一.7 R7 RefFS 自己红之后的票：第 7 条照常记，判错（读代码）

被判对象许可它的几句：第 1 条「RefFS 只录」（`research/prompts/m3-run-compare-split-r1-main-verification.md:54` 那一行的左列），第 7 条「按依不依赖理想模型分开报」，第二节两行（原行）：

```
research/prompts/m3-run-compare-split-r1-main-verification.md:29
| 历史内 RefFS 恢复一票 | G4 漏 | A1 漏 | **两腿一致，坐实**。拆法用 `lib.rs:440` 现成的口，不用新开接口（攻方「要改 RefFS 的口」说重了） |
research/prompts/m3-run-compare-split-r1-main-verification.md:46
| 红步之后照判 | 按模型时间线分开报，不进总数 | G1 按依不依赖模型分开报 | **一致**：只读镜像那几样与 RefFS 照常记，依赖模型的单列不进总数、不登记新发现 |
```

今天：历史里一次冷启动恢复读出不在允许集合，翻译当场停（`crates/singlefs-harness/src/reference_filesystem_translation.rs:532`「match model.crash_and_recover(crash, read_out.clone()) {」，红的说明串在 `:568`「"历史里一次恢复读出 {}，允许集合 {}",」）；之后每一个崩溃时刻从头重放到这一步就停，交回的就是这一句（同文件 `:994`「StoppedVerdict::Red { reason } => ReferenceFileSystemVote::Red {」）——后面的 RefFS 票一张都不另判。
拆开以后：录入段用第一轮判决点名的现成口 `crates/singlefs-reference-filesystem/src/lib.rs:440`「pub fn restart_after_a_recovery_that_abandoned_the_newest_root(」，它「// 只核 hi 判不判得了；读出的内容在不在允许集合里不判」（`:445`），然后照读出重起头，并且把读出推进版本表（`:463`「// 恢复读出、以它重新起头的那一份也是一个版本：之后可写挂载写的根映到它（r6 用户定案 H1-收严）」）。这个口原本是给「恢复抛弃最新根（带读故障）」那一种的；拿它当所有历史内恢复的「只录」口，读出越界那一次也照样重起头。

历史（可达）：一段历史第 k 步冷启动恢复，读出 V 不在允许集合（实现的 bug）；之后第 m 步崩溃。
1. 录入段：第 k 步照 V 重起头、V 进版本表；第 m 步的允许集合从 V 起算，录进 `12`。
2. 对比段：第 k 步那一票判红（读出 ∉ 集合），对；第 m 步拿实现的读出比「从 V 起算」的集合——实现照它自己的错状态往下走，读出多半就在这个集合里，判绿。
3. 第 7 条：RefFS 不依赖理想模型，第 m 步这一票「照常记」，进现判数与红绿总数；它若红，还登记成新发现。
所以 RefFS 在 k 之后的票是「以一个已知越界的状态为真」判的：绿不证明什么、红多半是 k 的连锁——正是第 7 条为理想模型那一类定的「不进总数、不登记新发现」的理由，第 7 条的分法没罩到。版本表那一句还让后面可写挂载写的根能映到 V 这一版，挂载那几票也一起受连累（推的）。
用户动作放开扫（固定「第 k 步读出越界」）：k 之后摆几个崩溃点、建不建文件——RefFS 那几票都照上面走，中；k 之后再无崩溃点——不中。
四句：分辨臂——分（「按依不依赖理想模型」对「按依不依赖任何一条已判红的参照的时间线：理想模型或 RefFS」）；系统看得到——对比段判第 m 步时手里有第 k 步那一票的结果，看得到；分句——第 3 问「判错」，连同第 1 条「RefFS 只录」那一句；跑前改法——第 7 条只有这一种分法，第二节「第二次在对比段由「第一次判绿」与「第二次读出 == R1」判」那一条管的是第三截，不管历史内的恢复。
改法（推的，零轮）：第 7 条改成「按依不依赖已判红的参照分」：RefFS 在它自己第一处红之后的票与理想模型那一类同样单列；或者录入段在读出越界时不重起头、照今天停（第一次判就在录入段做，这一判是 RefFS 自己的只读函数，归「只录」可以说得通，要用户点头）。

### 一.8 R8 按块亲和、只拷结论：权重每趟重量，四成的块换主（代价，量过；「或列族」撞整份拷库，读代码）

被判对象许可它的那一句：`research/prompts/m3-run-compare-split-r1-main-verification.md:57`「4. **库与双机**：录入另起一份库（或列族），双机只拷结论表；按块亲和（哪台录就在哪台比）。`runs/` 的拷贝清理另记一条（不归这一轮）。」

块归哪台由权重与块号定：流水线按块序号（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:7692`「.filter(|block| host_shares.owner_of_block(*block) == share.index)」），给定状态表按块键输入指纹头 8 字节（同文件 `:8737`「self.host_shares.owner_of_block(block_number) == self.share.index」），轮转里先取模（同文件 `:984`「let position = block_sequence % total;」）。权重不是常数：双机驱动每趟先量两台的速度再分（日志里的 `CRASH_AMPLIFICATION_HOST_WEIGHTS` 行，`runs/` 下现有三趟，三组值各不相同）。

量过（`weighted_shares_drift.py`，Python 照副本逐行改写，单测钉值自证过；`outputs/owner-drift.out`、`outputs/owner-drift-small.out` 原样节选）：

```
OWNER_DRIFT kind=pipeline before=136700,45275 after=140427,62390 blocks=39917 moved=16094 moved_fraction=0.4032
OWNER_DRIFT kind=given_states before=136700,45275 after=140427,62390 blocks=20000 moved=8053 moved_fraction=0.4027
OWNER_DRIFT kind=pipeline before=140427,62390 after=642805,291558 blocks=39917 moved=17007 moved_fraction=0.4261
OWNER_DRIFT kind=pipeline before=136700,45275 after=136701,45275 blocks=39917 moved=2178 moved_fraction=0.0546
OWNER_DRIFT kind=given_states before=136700,45275 after=136701,45275 blocks=20000 moved=7629 moved_fraction=0.3815
OWNER_DRIFT kind=pipeline before=136700,45275 after=273400,90550 blocks=39917 moved=0 moved_fraction=0.0000
```

真权重三组两两之间 40–43% 的块换主；一台的权重只差 1，给定状态表就有 38% 换主（指纹取模，总和一变整片重排），流水线 5.5%；比例相同、总和翻倍的一组 0%。
改动序列：跑一趟全量（两台各录各比、只拷结论）→ 改一个判器内核（结论版本换、录入版本不换，这是拆两段要买的那一样）→ 再跑：权重重量过，四成的块落到另一台，那一台没有它们的录入（只拷了结论），只能重跑 Rust 恢复两遍。按第一轮攻方 D2 的参考数（每态 160–723 µs、26.16 亿态 116–525 线程时），四成就是 46–210 线程时的重录（算的，参考数是旧树）。
「或列族」那一读法另撞一处：双机驱动每趟把本机整份库目录拷给第二台（`scripts/crash/crash-amplification-two-hosts.sh:1040`「cp -a -- "$local_library" "$library_for_peer" || two_hosts_break_is ignore-snapshot-failure || { echo "  ✗ 拷不了库的快照给第二台" >&2; echo "  → 怎么办：看 $run_directory 可不可写" >&2; exit 2; }」；副本同一行在 `:1031`）。录入放同一个 RocksDB 的另一个列族，目录一拷就整份带走——第一轮 D1 那一格（每项拷整份库）原样回来。只有「另起一份库、放在这个目录之外」那一读法过得去。
用户动作放开扫：单机——不中；双机、权重写死（`SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS` 由调用方给定）——不中；双机、照驱动每趟重量——中；只改结论种类不碰内核的提交——同样中（结论版本换了就要重比）。
四句：分辨臂——分（「按块亲和 + 权重每趟重量」对「归属钉在录入那一刻、记进库」或「录入随块走」）；系统看得到——分块那一刻看得到这一块的录入在不在本机库里，今天不看；分句——第 5 问「双机分片在新形态下怎么走」，连同第 3 问「做不成」；跑前改法——第 4 条只有这一种亲和法。
改法（推的，零轮）：录入那一刻把「这一块归哪台」写进库（块属性），之后的对比照这张表分，不再按当趟权重重分；权重只管还没录的块。或者权重量一次写进多机配置、改了才重排（代价：两台速度变了分得不匀）。

### 一.9 R9 第 5 条没给 P12、P17 落处（缺行，读代码 + 第一轮的量）

`research/prompts/m3-run-compare-split-r1-main-verification.md:58`「5. **GPU**：P1–P3 并进恢复内核；P14 单独成内核；P5、P7、P10、P11、P15、P16 一个新内核；新内核各一条 CPU 对 GPU 对拍与塞故障的自证进快档。」列了十二处，事实表要新写 GPU 一半的还有两处没落：
- P12 第二截记录核对器：事实表写「现成恢复内核的记录核对器只认一份镜像：`given_persisted_sets` 要改成每状态两份掩码」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/gpu-split-survey/report.md:155`）。第一轮攻方 D3 量过（`research/prompts/m3-run-compare-split-r1-opus-output.md:236`）：改在三个内核共用的镜像合成路径上，扫描内核余量 4434 折成 `version_of_location` 每份至多 +58、`location_word` 每份至多 +5；改在恢复内核独有的那一截不碰扫描内核。第 5 条没写走哪一条，走前一条就编不出（撞 400000 的闸）。
- P17 判红后归类：用户定「三处都上 GPU」，第一轮 E2 坐实它的 Rust 本体是 core 的择配置、择根；WGSL 里一份择配置 15670、择根 9001 个表达式（第一轮攻方 D3），六合一那个新内核按正推腿估「≲ 2 万」装不下它，要与 P14 或恢复内核同住。
四句：分辨臂——不分辨（缺的是落处，不是臂）；系统看得到——看得到；分句——第 3 问「编不出」；跑前改法——无。改法（推的，零轮）：第 5 条补两句「P12 只改恢复内核里记录核对器那一截」「P17 并进 P14 那个内核」，乙簇的展开预算闸照量。

### 一.10 R10 「块键记它属于哪一把历史键」是单值，块键却跨历史去重（结构，推的）

给定状态表的块键只看内容（一.1 引的 `:8776`），项名与种子规模串在里面，历史序号不在。同一批里两段历史若在某个崩溃点写出逐字相同的表（例：都在 mkfs 之后第一步就崩、那一步一个写都没落；或第三截在逐字相同的崩溃镜像上挂载），今天就是同一块、第二段照库复用——对今天是对的，因为今天给定状态表的结论只读镜像。第 2 条之后读 `12`（模型各版、允许集合、读故障标）的那几样结论，输入多了一段历史专有的东西，块键却没多：一把块键挂两段历史，「属于哪一把历史键」装不下第二把；结论按块键复用，第二段历史就拿第一段的 `12` 判过的结论。我只证得出结构上会这样，没造出两段历史读 `12` 判出不同结论的实例（读故障标今天按「盘面与没走这一步一样就不算注入」定义，副本 `crash_injection.rs:1098` 那个函数的文档，几乎是盘面的函数；允许集合在逐字相同的镜像上，读出相同，要分出不同结论得一边越界一边不越界）。
四句：分辨臂——分（单值「属于」对「块键 ‖ 历史键」一起当读 `12` 那几种结论的键）；系统看得到——复用那一刻看得到块键，看不到它这一次属于哪段历史（键里没有）；分句——第 3 问「历史键与给定状态表块键之间的对应会不会错配」；跑前改法——无。改法（推的，零轮）：读 `12`/`14` 的结论种类，键用（块键，历史键），不读的照旧只用块键。

## 二、第 4 问：产生结论的方法

### 二.1 Q1 第一轮闭包探针只看了 GPU 一行（量法偏，量过）

第一轮攻方的闭包探针造了两种接法（内核宿主由流水线引、由 GPU 判器引），每棵树只报两样：整表摘要与 GPU 那一行（`research/prompts/m3-run-compare-split-r1-opus-model/outputs/closure-probe.out` 每行 `table_without_gpu_file_list_sha256=… gpu_judge_digest=…`）。第 3 条那条自证之二就是从「进了 GPU 闭包就会换版本」这一个正对照推出来的。我在同三棵树上逐行比（整表摘要与第一轮逐字相同，见开头自证）：进了 GPU 闭包，GPU 一行与 4 份 GPU 对拍测试的 CPU、观察者行换，真正的观察者那一行与其余 44 份设点文件的 CPU 行都不换（`outputs/table-diff.out`）。取样点只在「GPU 那一把」上，结论被外推到「每种结论」。翻了的是第 3 条自证之二（R2）。

### 二.2 Q2 「随机历史开 GPU 约 60 步封顶」：量出来是 71–136 步（数偏紧，量过）

正推腿那个数是这样推的：「2048 ÷ 30 ≈ 68 次发布」（`research/prompts/m3-run-compare-split-r1-sonnet-output.md:310`），快档那一句「≈ 900，够；历史长过约 60 步的大档，开 GPU 时判表建不出」（`research/prompts/m3-run-compare-split-r1-sonnet-output.md:310`）。每次发布约 30 写取自多记录流 A、B、C 三段，不是随机历史量的。
我照快档的生成与执行（BROAD 比重、两块 4 GiB 盘、不跑每步 checker、空间准入按公式）跑了 36 段 144 步的历史，数「崩溃窗口在第 k 步末尾时」给定状态表里有几次写（表从 mkfs 第一个不是清零的写起，照副本 `crash_injection.rs:1914` 那个构造）。`outputs/history-writes-144-steps.out` 36 行，按 `first_step_over_2048=` 数：

```
71 75 77 78 78 80 80 81 82 82 84 86 87 89 90 92 92 93 93 94 94 95 97 98 103 107 107 117 119 120 125 134 135 136 none none
```

（上面这一行由 `grep -o 'first_step_over_2048=[a-z0-9]*' outputs/history-writes-144-steps.out | cut -d= -f2 | sort -n` 出，两个 none 我挪到了末尾；原行形如 `HISTORY_WRITES steps=144 seed_offset=27 steps_run=144 ending=Discriminant(0) mkfs_data_writes=11 table_writes_at_step_24=671 table_writes_at_end=4102 first_step_over_2048=71 seconds=…`。）第 24 步时表里 76–736 次写（正推腿推约 900）。
口径与限度：我数的是写表里全部的写（含清零、journal、系统配置槽写），GPU 掩码按 `layer0_enumeration_tables` 的枚举写表数，不会比我数的多——所以过 2048 的那一步只会比我量的晚；第三截（崩溃镜像上挂载、发布）的表还要加挂载写，没量，那一类会早几步。36 个种子、一组比重、一种盘宽。
翻面：这个数没让第三节哪一条翻面（快档 24 步离 2048 远）；它改的是第二节那一行零轮数的口径：「约 60 步」改成「BROAD 比重下 71–136 步（36 段，2 段到 144 步没过），第三截更早、没量」。

### 二.3 Q3 体量与 CPU 的态数基数还是下界（现查日志）

两腿算体量、CPU 代价都用的 26.16 亿（9/30 展开 28）与 37.4 亿（30 深下界，正推腿 7.1 节）。正推腿的下界只把多记录流、删除流两项换成了 30 深的数，别的五项写「没核」。9/30 那份日志里另有一条流同样不穷举：`~/.local/share/singlefs/crash-amplification/full-run-20260930-i.log` 第 225 行 `CRASH_AMPLIFICATION flow=position_addressed_ExtentLowerSegmentGrowsToTwoLevels mode=two_hosts_cpu_with_gpus states_checked_here=10 …` 那一组行末是 `exhaustive=false`（同一条流三行都是，`grep -n` 命中第 207、225、237 行）。它在 28 深下只判了 10 个态；不穷举说明有一段超过 28 写，30 深下它至少是 2²⁹ 量级还是仍不穷举，我没量。所以「一份放得下、闸在双机拷库」（第一轮判决第二节体量那一行）与 CPU 外推都建在一个下界上。翻面条件写在二.5。

### 二.4 Q4 展开量：两个数偏向已知的一边，一个数是推的

- P14 单独 62660、并进恢复内核 387189（第一轮攻方 D3）：探针只摆了判据要调的解析各一个调用点，判据本体的比较一行没写，两个数都是下界。偏差只会让「并进去」更装不下，第 5 条「P14 单独成内核」不会因此翻面；会翻面的只有「单独成内核也过 400000」，那要判据本体自己超过 33.7 万个表达式（推的，不像）。
- 六合一新内核「≲ 2 万」（正推腿 9.2 节）：推的，按每样几千估，没量；第 5 条采了它。R9 的 P17 若并进来就不成立。
- E6 不改这几个数：E6 的交回报告写「展开量：三个内核改前、改后一样」（`~/.local/share/singlefs/work/m3-merge-impl/e6/patch/report.md` 第 11 行），我没重量。

### 二.5 Q5 哪个数翻了，第三节哪一条翻面（推的）

| 数 | 现在的值与来路 | 翻到多少 | 翻的是哪一条、怎么翻 |
|---|---|---|---|
| 两趟之间块换主的比例 | 40–43%（我的模型，真权重三组） | 接近 0（权重写死或入库） | 第 4 条按块亲和的代价消失，R8 不成立 |
| 每态 Rust 恢复两遍的用时（M1） | 160–723 µs（E161 旧树参考） | 远小于 GPU 每态预算（39.7 µs，第一轮 D2） | R3、R8 的重录只是小钱；第 3 条入口那一句、第 4 条亲和都可以照写 |
| 录入行的真实压缩比（M3） | 没量；第一轮按 5/20/94 三档 | 每块字典一类形态能到几 GiB | 「双机只拷结论」没有必要，录入随块拷，R8 与「或列族」那一格一起消失；第 4 条改成「录入随结论走」 |
| 30 深全量态数 | 下界 37.4 亿（正推腿），另有一条流不穷举（二.3） | 盘余（1.7T 量级，会变）÷ 每态字节 | 第一轮判决「体量的闸是双机拷库，不是一份放不放得下」翻成「一份也放不下」，第 2 条「`10` 定长字段收到比较要用的那几样」要收得更狠或按块滚动删 |
| 历史写表过 2048 的步数 | 71–136（我的模型） | 小于快档每段步数 24 | 第 5 条「新内核各一条对拍进快档」在随机历史两项上建不出判表；今天的 736 离得远 |
| 观察者每态挂载成本与写分组数（M2） | 没量 | 分组数接近态数 | 第 5 条 P5 放进新内核那一格要先退回「一态一表」，展开与显存另算 |

## 三、第 5 问：没被看过的地方

### 三.1 N1 `layer0_progress` 的双机分片按片、判定库按块，对不齐（做不成，算的）

层 0 那一路的双机是按切片交错分：`crates/singlefs-checker-tier/src/layer0_progress.rs:15`「//! `slice_index % n == i` 的那些片，跑完把这些片的片行写成这一片的账本（与进度文件同一种片行，另加文件头、片的线程行与收尾行）；」（主工作区与副本相同），片长只看态数（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash.rs:2537`「const LAYER0_RESUMABLE_SLICE_COUNT: u64 = 65_536;」，片长 = ⌈态数 ÷ 65536⌉，至少 16）；入口也照这条走（`scripts/crash/crash-point-replay.sh:6` 的长活声明「登记了 shard=across-machines 的用例按多机配置两台各跑一片」）。判定库一块是 `/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:153`「pub const STATES_PER_BLOCK: u64 = 1 << 16;」。
算的（`layer0_slices.py`，`outputs/layer0-slices.out` 原样）：

```
LAYER0_SLICE item=multi-record-layer0-domain states=1358954634 states_per_slice=20737 blocks=20737 slices_per_block=3.16 blocks_with_states_on_both_hosts=20736
LAYER0_SLICE item=delete-30 states=1448083638 states_per_slice=22097 blocks=22097 slices_per_block=2.97 blocks_with_states_on_both_hosts=22096
LAYER0_SLICE item=fixed-unmount-28 states=1610875015 states_per_slice=24581 blocks=24581 slices_per_block=2.67 blocks_with_states_on_both_hosts=24580
```

态数小于 2³² 时片长总小于一块，交错分两台，每一块（除最后一块）的态都落在两台上。丁二把层 0 用例迁进两段形态时（第 6 条「17 份，三份测旧引擎功能的定迁或留」，迁的是 14 份），两条路必选一条，第三节一条都没写：
- 照旧按片分：第 4 条「哪台录就在哪台比」对这些用例无从做起，每块的录入一半在另一台，「只拷结论」就比不了。
- 改按块分：`layer0_progress` 的分片账本、`merge/<n>` 核片数、断点续跑的片行、门禁 checker-tier-layer0-replay 里登记 `shard=across-machines` 的那几行全部要换成库的复用；这不在任何一簇的文件清单里。
四句：分辨臂——分（「按片」对「按块」）；系统看得到——看得到；分句——第 5 问「`layer0_progress` 断点续跑、双机分片在新形态下怎么走」；跑前改法——无。改法（推的，零轮）：第 6 条丁二写明「迁过来的层 0 用例改按块分、续跑走库」，`layer0_progress.rs` 与门禁那几行进丁二的文件清单；或层 0 用例的片长改成块长的整数倍、片号按块号归台（两套并存时对得齐）。

### 三.2 N2 判红钉回快档：按历史键钉会漂，只在 GPU 上红的复判不出（推的）

今天还一条都没钉（`scripts/crash/crash-point-replay.sh:169`「PINNED_RED_EXAMPLES_NOT_RUN_REASON="崩溃点重放还没有钉表：全量出的红要钉回快档（test-script-discipline.md「全量的红，钉回快档」），今天一条都没钉"」），两段形态是第一次定钉法的地方。规则要「钉完核一次：在出红的那份代码上，钉住的用例同样红，红在同一条断言上」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:87`）。两处会断：
1. 随机历史两项的红，顺手的钉法是钉历史键（项、种子与规模、序号）。R1 已说明这把键不按内容：生成器、比重、执行参数、取历史的判据一变，同一把键指到另一段历史，钉住的「回归」悄悄换了输入。要钉内容（基线摘要、写表、持久集合，即 `0F`/`13` 的那几行），而 `13` 只在全量那份库里，快档用的是一趟一建、跑完就删的空库（`crash-point-replay.sh` 文件头「两个域的写法」）——钉表要自带写内容，体量没算。
2. P1–P3 那一类红（GPU 恢复与 Rust 录入比不上）是内核与 Rust 之间的分歧，不开 GPU 的轻型只有 CPU 判那一路，复判时根本不走那一条比较：在出红的那份代码上照样绿，「红在同一条断言上」核不了。要写明这一类钉进「开 GPU 才判、没开报本次未跑」的那一组（今天 `BOTH_TIERS_GROUPS` 有 GPU 对拍一类）。
四句：分辨臂——分（「钉历史键」对「钉内容」）；系统看得到——钉的那一刻看得到写表；分句——第 5 问「判红钉回快档」；跑前改法——无。

### 三.3 N3 双机 × `12`/`14`：历史键的行没有归属规则（代价，读代码）

今天双机时给定状态表一张表一块、按块键归台，但两台都把整批历史跑一遍（不归自己的表交占位结论，导入之后再跑一趟，副本 `crash_amplification.rs` 的 `tables_left_to_the_other_hosts` 那一段）。`12`/`14` 挂在历史键上、没有块，第 4 条的按块亲和对它们没有定义：两台各录各比一遍整批 `12`/`14`，结论按同键同版本互导。随机历史两项量小（快档 24 段），代价不大；要写一句归属（例：历史键的行归第 0 号历史所在那台，或两台都做、导入时逐行比相同），不然 `import_from` 那条「同一块键两边判定不同就报错」要扩到这几张表。

### 三.4 N4 看过、没打中新东西的几处

- 续跑：同一条命令停了再起，结论按（块键，种类，版本）复用、录入按（键，种类，录入版本）复用，两层各自续；没 GPU 时「等数据跑完再对比」，录了没比的块下一趟照录入复用、只补比。没造出续跑独有的错；R1 那一格在续跑上照样中（续跑就是「录过的不再跑」）。
- `import_from`：今天照抄录入表、路径、事实、核对、判法版本与结论块（副本 `verdict_store.rs:1566` 那个函数）；第 4 条「只拷结论」只要它不导新录入表，能做到。打中的是「或列族」撞整份 `cp -a`（R8），不是这个函数。
- 变异表：`crates/mutations.tsv` 改 `*.wgsl` 的有 236 行、改流水线三份停止文件（`crash_amplification.rs`、`crash_judge_dispatch.rs`、`verdict_store.rs`）的有 159 行（`awk -F'\t'` 按第二栏数）。按覆盖复用那一层把 `include_str!` 读的文件算函数外：`scripts/mutation/crates-mutation-rows.py:98`「编进去的其余文件（include_str! 读的、proc-macro 与构建脚本单元的、删掉了的）一律算函数外；」——改 WGSL 照现跑，新内核同样。变异复跑那三份脚本（`mutate.sh`、门禁阶段、`crates-mutation-rows.py`）都不设也不清 `SINGLEFS_CRASH_AMPLIFICATION_*`（`grep -c` 三份都是 0），没设时引擎用一趟一删的临时库，不跨趟复用录入；调用方环境里带着这个变量时会被继承，那时同 R1、R3 的复用照样发生，这一条我没核。
- 广谱变异：`scripts/mutation/broad-mutation.py` 里没有点名判定库或崩溃放量入口的跑法（`grep` 只命中一行 `gate-similar` 注释），不经两段形态的复用。没打中。
- e161 装置：调旧引擎（第一轮 A4）；第 6 条留旧引擎到 e161 有去处，它照旧编得过、量的仍是旧引擎。它的产物是第一轮 D2 的 CPU 参考数来源，拆两段之后不再代表录入段的速率（M1 要另量，第一轮已写）。

## 没打中的形状

- 自证之二的另一条绕法：`include_str!` 的参数不是字面量（经 `macro_rules!` 包一层、或 `concat!`）。闭包算法只认字面量（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/judging_closure.rs:501`「if let Some(literal) = text_at(position + 3).filter(|literal| literal.starts_with('"'))」），这样写闭包与自证都看不见那份 WGSL。今天 checker 档 `src/` 里读 WGSL 的五处（副本 `crash_judge_gpu.rs` 第 64–72 行）都是字面量，是一种要人故意写出来的形状，不算打中，只记一笔：自证之二若照闭包同一套切词实现，会一起瞎。
- 内核宿主不在任何闭包、只由派活文件引：第 3 条的自证照字面判红（探针 k-amp），抓得住。
- E6 改不改展开余量：不改（二.4）。
- 读故障标跨历史撞键（R10 的一种）：读故障标按「这一步发没发过东西」定义，发过就改了写表，盘面相同的两段历史读故障标也相同；造不出。
- 续跑、`import_from`、变异表、广谱变异、e161：见三.4。
- 取样：闭包探针七棵树、每棵改一处；双机模型三组真权重加三组构造权重，流水线块号 0..39916、给定状态表 20000 个随机指纹替身；历史写表 36 段 × 144 步、一组比重、一种盘宽；层 0 切片三个态数。崩溃状态 0 个，`closed_form_state_count` 累计 0 态。

## 这条腿自己的限度

- 没编 checker 档、没跑任何 checker 档测试，轻型也没跑。闭包探针、双机模型、层 0 切片模型都是照副本代码另写的小程序或改写；历史写表那一个依赖 harness 与 core 真跑历史。四样只在我的模型上量过，数不进 kb，要在入库装置上重做才能引。
- 探针 move、swap 两棵树没编译：多重集与按项多重集都只看词，不看编不编得过；真搬家多出的 `use` 我照常写了，实际要的可能多一两个。
- R4、R7、R10、N2、N3 是读代码推的，没造跑得起来的实例；R7 的「第 m 步多半判绿」是推的。
- 我提的改法全部被攻过零轮：

| 改法 | 修哪一格 | 标记 |
|---|---|---|
| 历史键按内容取（操作序列、`HistoryExecution` 整个、几何、取历史与取崩溃态判据所在文件摘要） | R1、N2 | 推的 |
| `12`/`14` 的录入版本照 H8 并上设点文件出发的闭包 | R1、R4 | 推的（探针只证了 CPU 结论那一把会变，没实现录入版本） |
| 派活那一刻把编进管线的源码摘要拼进那一种结论的 `extra_inputs` | R2 | 推的 |
| 写表函数先搬进各族叶子文件，族闭包里不许有 `*.wgsl`、CPU 判器入口与别族叶子 | R3 | 推的（探针量过今天三份入口的闭包，改法没实现） |
| 簇 0 验收改成按项多重集（去 `use`、`mod`、可见性） | R5 | 量过：move 不变、swap 变（`outputs/cluster0-acceptance.out`） |
| 己的检查改判「当结论用的那一份是 GPU 种类」 | R6 | 推的 |
| 第 7 条改按「依不依赖已判红的参照」分，RefFS 自己红之后的票单列 | R7 | 推的 |
| 块归属在录入那一刻写进库，之后照表分 | R8 | 推的（模型只量了不写进库时的漂移） |
| 第 5 条补「P12 只改恢复内核记录核对器一截」「P17 并进 P14 内核」 | R9 | 推的（余量数来自第一轮的模型） |
| 读 `12`/`14` 的结论种类用（块键，历史键）当键 | R10 | 推的 |
| 丁二写明层 0 用例改按块分、续跑走库 | N1 | 推的（对不齐是算的） |

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳；副本与我的模型上的数不算入库装置上的数。
- 没读禁读清单里的三处（同轮辩方腿的报告、模型目录与草稿目录）。
- 没量：录入段每态 Rust 恢复用时（M1）、观察者挂载写分组数（M2）、录入行压缩比（M3）、30 深全量态数、第三截表的写数。前三样第一轮判决已排给主 agent；后两样要编 checker 档（重型）。
- 跑前看了负载：别的会话在跑 `cargo test -p singlefs-checker-tier --release --features verdict-store,gpu …`（重型，不是我起的）与一条 harness 档 `cargo test`；没有性能测量在跑。我的三个小程序各用自己的编译目录，没等锁；第一次起闭包探针按 8G 上限排了 3 分多钟队，我拿 `timeout` 截了它（包装在排队，命令一行没跑，退 124），改 2G 重起跑通，这一次不算结果。
- 草稿目录 `~/.local/share/singlefs/work/m3-run-compare-split-r2-attack/` 里的七棵探针树、两个编译目录与两份小程序副本交回前删掉，留日志与输出原件。

## 要主 agent 跑的命令

没有必须由主 agent 跑才能交回的。要把这里的数做成入库装置上的数，最便宜的是闭包那一组：在入库的闭包算法上加一个只读子命令（整表逐行与任意入口的闭包清单），照 `build_trees.sh` 的七棵树复跑，比 `outputs/table-diff.out`、`outputs/record-entry-closures.out`、`outputs/cluster0-acceptance.out`。
