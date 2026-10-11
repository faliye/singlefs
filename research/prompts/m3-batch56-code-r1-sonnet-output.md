# m3-batch56-code-r1 正推腿（Sonnet）报告：删除线第 56 批代码轮第一轮（2026-10-10）

立场：正推。核「代码做的是不是条款说的」，逐条对条款原文与冻结副本的行；判定词一致 / 冲突 / 规则没说，每条带推翻条件。
格：正文第三节 K1–K6。冻结副本：`~/.local/share/singlefs/work/m3-batch56-code-r1/repo/`（下文代码行号都是它的）。kb 行号是 `/home/fy5090/code/singlefs/.claude/kb/` 里现查的。
本腿草稿、拷贝、日志：`~/.local/share/singlefs/work/m3-batch56-code-r1-sonnet/`（`copy/` 是冻结副本的拷贝，`target/` 是自己的编译目录，`logs/` 是日志，`mut.sh` / `runrow.sh` 是变异驱动，`mutdefs/` 是自造变异的原文与替换文）。
引文都是写进来之前 `grep -nF` 过的；「转述」二字标出的不是引文。

## 零、开工核对与跑法

| 项 | 命令 | 原样结果 |
|---|---|---|
| 冻结副本没动 | 在冻结副本里 `sha256sum -c research/prompts/m3-batch56-code-r1-snapshot/crates-sha256.txt` 计数 | `OK 14`（14 份全对，没有 FAILED） |
| kb 没动 | 仓根 `sha256sum -c research/prompts/m3-batch56-code-r1-snapshot/kb-sha256.txt` 计数 | `OK 5`（5 份全对） |
| 跑法 | 每条编译与测试：`nice -n 19 bash scripts/hosts/capped.sh 10 bash scripts/hosts/run-with-memory-cap.sh 10G cargo …`；用自己的编译目录 `target/`；同一时刻一条 cargo；变异都在 `copy/` 里改、改完 `cp` 回冻结副本原文并 `cmp` 相同 | 冻结副本没写过；`copy/` 里的 probe 文件（`zz_probe_*`）与变异不入仓 |
| 负载 | 开跑前 `ps` 看：没有 `qemu-system`、`vm-bench`、`e152`、`fio`；有别的会话的 `cargo test … -p singlefs-checker-tier --features verdict-store,gpu`（主 agent 的）在跑，机器 load 25–41；我的命令加 `nice -n 19` 照常跑，等锁没遇到，只是慢（一条 `checker_known_bad_images` 全文件变异 5 分钟左右） | 见各节 |
| 没跑 | 门禁阶段、checker 档测试（`cargo test -p singlefs-checker-tier`、GPU 对拍用例）一律没跑 | 凡是只有它们能核的写「复核不了」 |

<<K1_PLACEHOLDER>>

## K2　容器号三判（乙 G2）

条款（kb 行号现查）：
- D18 已定项 24 第 1 条：`.claude/kb/decisions/18-块里携带什么信息.md:576`：「容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1」。
- 同项第 3 条：`.claude/kb/decisions/18-块里携带什么信息.md:578`：「有新记录的那次发布才重写，不删除的发布照抄；满了才关」。
- 同项第 5 条（谱系）：`.claude/kb/decisions/18-块里携带什么信息.md:580` 含「随现行那一版、只增不删」。
- 布局表：`.claude/kb/layout/03-third-txn.md:18`：「只增不删；容器号从 1 起，下一个号 = 谱系树里最大号 + 1，零根时 1，0 不用（规则同墓碑容器）」；`.claude/kb/layout/03-third-txn.md:19`：「有新记录的那次发布才重写开放容器，没有的发布照抄；满了才关；第一版不回收」。
- kb 里 I-9.21、I-9.22 现文还写「严格递增」（`.claude/kb/invariants.md:309`：「墓碑树里容器号沿叶序严格递增」；`.claude/kb/invariants.md:310`：「墓碑树里容器号沿叶序严格递增，每个打包记录类型 1 容器记录数 ≤ 旋钮值」）；I-9.36、I-9.37 在 `invariants.md` 里零命中（`grep -nF 'I-9.36' invariants.md` 无输出）。这三句判法（第一片 1、相邻加一、每片至少一条）是代码先于 kb 的部分，正文第一节已说「还没写回」。
- 「每片至少一条」没有一句逐字条款：它是从「有新记录的那次发布才重写」（第 3 条与布局表）加「只增不删」推出来的；「相邻加一」是从「下一个号 = 最大号 + 1」加叶序升序、只增不删推出来的。判定词里「一致」都是对这两个推出，不是对逐字条款。

### K2.1　合法镜像上有没有误报

判：**一致**（没找到会让容器号不从 1 起、不连号、出现空容器的写侧路径）。

读码依据：
- 三棵容器树写侧只经两个追加函数：`crates/singlefs-core/src/container_tree.rs:476`「pub fn append_record_to_the_open_container<Record: Clone>(」与 `crates/singlefs-core/src/container_tree.rs:511`「pub fn append_records_to_the_open_container<Record: Clone>(」。
- 新片号：`crates/singlefs-core/src/container_tree.rs:548`「.map_or(1, |rightmost| rightmost.identity.container + 1);」；新片开出来时带着触发它的那一条：`crates/singlefs-core/src/container_tree.rs:556`「records: vec![record.clone()],」；只有最右那片没装满才往里追加：`crates/singlefs-core/src/container_tree.rs:537`「.is_some_and(|open| open.records.len() < records_per_container);」。所以首片 1、逐片加一、没有空片。
- `grep -n 'remove\|retain\|drain\|truncate\|\.pop()\|clear()'` 对 `container_tree.rs`、`tombstone_tree.rs`、`lineage_tree.rs`、`retirement_tree.rs` 四份跑，`wc -l` 得 0（我自己跑的，不是转述正文）。
- 各路径的「上一版」来源：删除批与退役 `crates/singlefs-core/src/transaction.rs:5381`「let tombstone_containers_before: Vec<TombstoneContainer> = previous」、`crates/singlefs-core/src/transaction.rs:5418`「let retirement_containers_before: Vec<RetirementContainer> = previous」、谱系 `crates/singlefs-core/src/transaction.rs:5588`「let containers_before: Vec<LineageContainer> = previous」，都取发布路径交来的上一版；崩溃恢复后的第一次发布，上一版是重建出来的所选根那一版（`crates/singlefs-core/src/mount.rs:1236`「let rebuilt = rebuild_version_in_the_unit_area_starting_at(」），抛弃段里写的容器在这条线上不存在；回退照抄现行那一版的三份容器：`crates/singlefs-core/src/mount.rs:7958`「tombstone_containers: current.tombstone_containers.clone(),」、`crates/singlefs-core/src/mount.rs:7961`「lineage_containers: current.lineage_containers.clone(),」、`crates/singlefs-core/src/mount.rs:7965`「retirement_containers: current.retirement_containers.clone(),」。
- 盘上读回来的容器也不会空：`crates/singlefs-core/src/recovery.rs:4241`「|| container.records.is_empty()」拒掉空容器。

造出来的：
1. 崩溃恢复抛弃段那一形（正文 K2.1 点名要造）。自写探针 `copy/crates/singlefs-harness/tests/zz_probe_k2_abandon.rs`：三棵树旋钮压到一片 1 条、一个节点 2 条，建 8 个空 inode，unlink 9、8 号到底，把「再 unlink 7 号到底」那一串发布的 62 次写按前缀截成 63 个崩溃镜像（第 k 个前缀 = 前 k 次写落盘，k = 0…62），每个镜像可写挂载一次（续做删除）、再用带旋钮的新入口跑池级 checker。原样结果：`PROBE writes=62` / `PROBE mounted_ok=63 mount_err=0 reds=0`（日志 `logs/probe-k2-b.log`）。同一探针改成「回退到 unlink 9 号那一版」的发布串：`PROBE writes=29` / `PROBE mounted_ok=30 mount_err=0 reds=0`（`logs/probe-k2-d.log`）。
   探针有牙：把 `crates/singlefs-core/src/container_tree.rs:548` 的 `map_or(1,` 改成 `map_or(2,`，基线镜像上带旋钮入口就红「墓碑树（树 65545）沿叶序第一片容器的容器号是 2，不是 1（容器号从 1 起）」；把 `+ 1` 改成 `+ 2`，红「沿叶序容器号不是逐片加一：容器 1 之后是容器 3」（`logs/wa.log`、`logs/wb.log`）。
   局限：只是前缀截断（每串 63 / 30 个状态），不是子集枚举；新片只在墓碑树（unlink 串）与谱系树（回退串）上出现，退役树没在这两串里开新片（退役记录要叶容器身份消失，空 inode 不触发）。
2. 固定种子随机历史 `container_trees_grow_a_level_when_the_root_is_full.rs` 的 `a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step`：见 K6 里我跑它的结果。
3. 产品容量的多容器镜像（退役 960、回退 1360）：见 K6，我跑了两条慢档。

推翻条件：任何一条能在「崩溃恢复后第一次发布 / 回退 / 根长高」之后让某棵容器树的沿叶序第一片号 ≠ 1、相邻差 ≠ 1 或出现 0 条的容器、而池级 checker 带正确旋钮值仍红的合法历史。上面的探针没找到。

### K2.2　坏镜像漏报：三判之外还有哪些漏

判：**规则没说 / 部分漏**（四处，前两处主 agent 已记，后两处是我的新发现；都没有逐字条款要求这些检查）。
1. 产品支（没装旋钮）只有一片容器时三判都不判，报不适用：`crates/singlefs-checker/src/walk.rs:11993`「None => 2,」（谱系 / 退役）与 `crates/singlefs-checker/src/walk.rs:12069`「None => 2,」（墓碑）。这一形第一片号 ≠ 1、或唯一一片是空的坏镜像不红。正文第二节已记「产品支一片容器不判」，我不重复算新发现；补一个事实：这个「不判」被恰好一条用例钉着（把 `None => 2` 改成 `None => 1`，`checker_known_bad_images` 全文件里恰好 1 条红，见 K6 的 c1）。
2. GPU 谱系记录表上限 256 条（正文已记）。
3. 首片号判的是 `== 1`，会拦 0；但没有用例拿 0 当首片：把 `==` 改成 `<=`（接受 0 和 1），`checker_known_bad_images` + `container_trees_grow…` 两份 247 + 6 条全绿（K6 的 c3）。这是判法对、钉子没有，不是判法漏。
4. 跨版本不判：同一棵树的后一版容器集应当是前一版的前缀加追加（「只增不删」，`18-块里携带什么信息.md:580`；退役「第一版不回收」，`03-third-txn.md:19`），三判只看每一版自己的树。所以「某一版比它的父版少一片容器、而这一版自己是 1…n 连号且无空片」不红。这条是读码推出，我没造镜像（需要同时改根的条目、容器索引与各层校验和）。规则里也没有一条 I-9.x 写「后一版容器集包含前一版」。
5. 最右片记录数超产品容量由别的条拦：`.claude/kb/invariants.md:32` I-1.7 写「记录数 × 记录宽 ≤ 声明长度且声明长度 ≤ 32768 − 136」，并给出 40 宽记录的容量 815（该行里谱系记录宽还写 16，D18 已定项 24 是 24，kb 写回时一起改）；容量由头 136 与登记宽算出，所以产品支最右片超容量由 I-1.7 拦，不用 I-9.21 / I-9.36 再判上限。

推翻条件：第 4 条——若有一条 I-9.x（或 I-7.x）已经在跨版本上比容器集前缀，我没读到；第 1 条的「钉子恰好一条」可被另一份用例集推翻（我只跑了 harness 两份文件）。

### K2.3　CPU 与 GPU

判：**一致**（读码，逐位）；对拍实测**复核不了**（要 GPU 用例，checker 档）。

对照（CPU 在 `singlefs-checker/src/walk.rs`，GPU 在 `singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`）：
| 判 | CPU | GPU |
|---|---|---|
| 首片是 1 | `crates/singlefs-checker/src/walk.rs:12003`「first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,」与 `crates/singlefs-checker/src/walk.rs:12078`「first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,」 | `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3151`「fn d_judge_container_numbers_from_one_by_one_and_no_empty_container(invariant: u32) {」里首片比 `D_FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2171`「const D_FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE: u32 = 1u;」） |
| 相邻加一 | `crates/singlefs-checker/src/walk.rs:12014`「left_container.checked_add(1) == Some(right_container);」、`crates/singlefs-checker/src/walk.rs:12090`「left.container.checked_add(1) == Some(right.container);」 | `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3158`「if (!(lt64(left_container, right_container) && eq64(add64(left_container, u64_of(1u)), right_container))) {」 |
| 每片至少一条 | `crates/singlefs-checker/src/walk.rs:12025`「let lineage_or_retirement_container_holds_a_record = record_count > 0;」、`crates/singlefs-checker/src/walk.rs:12105`「let tombstone_container_holds_a_record = record_count > 0;」 | `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3163`「if (d_container_record_count(container_index) == 0u) {」 |
| 谁判 | 产品支 ≥ 2 片（`crates/singlefs-checker/src/walk.rs:11993`、`crates/singlefs-checker/src/walk.rs:12069`），旋钮支 ≥ 1 片（`crates/singlefs-checker/src/walk.rs:11994`「Some(_) => 1,」） | 产品支 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3191`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3212`「if (d_container_count < 2u) {」；旋钮支 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3172`「if (d_container_count == 0u) {」 |
- 溢出：左片号 = u64::MAX 时 CPU `checked_add` 得 `None`，红；GPU `add64` 回绕成 0，若右片恰是 0 则等式成立，但 `lt64(MAX, 0)` 假，整句仍假，红。两边都红。
- 旋钮常量为 0 的那一份就是产品入口：`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:517`「&JudgeKernelSources::of_the_judge(),」是层 0 与放量的编法；`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:283`「pub fn of_the_judge() -> Self {」交 `ContainerCapacityKnobs::PRODUCT`；判法摘要罩的也是 PRODUCT 那三个 0：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:203`「container_capacity_knob_wgsl_constants(ContainerCapacityKnobs::PRODUCT).as_bytes(),」。带旋钮的编法只有对拍用例调：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2339`「let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(knobs);」。
- GPU 对拍用例覆盖的缺口（读码）：新加的 GPU 红例都走旋钮支（谱系 1 条、退役同号、谱系 >旋钮、谱系跳号 / 从 2 起 / 最右空），墓碑旋钮只有一条合法流（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13055`「fn the_gpu_judge_given_the_tombstone_knob_agrees_with_the_cpu_on_two_tombstone_containers_of_one_record(」），没有红例；产品支（I-9.21 / I-9.36）调同一个共享函数的两处 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3194`「d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_21);」与 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3215`「d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_36);」没有新红例，旧红例（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8522`「fn the_gpu_judge_agrees_with_the_cpu_when_a_non_rightmost_tombstone_container_is_not_full() {」、`…:8408`「fn the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full() {」）靠记录数红，把号判法摘掉也红同一位。→ 这两处调用被摘掉，GPU 对拍照绿（读码推，GPU 用例没跑，复核不了）。变异表 `crates/mutations.tsv` 里第 56 批的 GPU 行（22–30）也没有一行摘这两处。

推翻条件：GPU 对拍用例跑出 CPU 新入口与 GPU 某一位不同；或在产品支上造出「≥ 2 片、第一片非 1 / 相邻差 ≠ 1 / 最右片空」的流，GPU 与 CPU 不同。

## K3　旋钮进 GPU（乙 G6）

条款：旋钮值怎么给 GPU 是代码口径，kb 里没有逐字条款（D18 已定项 24 第 3 条 `.claude/kb/decisions/18-块里携带什么信息.md:578` 只写「关闭路径加只供测试的容量旋钮，旋钮不落盘」）；判决 G6 的验收是「带旋钮的镜像 GPU 与 CPU 新入口逐位相同」。下面的判定词都是对这条验收与代码自己的注释。

### K3.1　常量为 0 时是不是旧的产品判法

判：**一致**（除第 56 批 G2 新加的三判外，逐句相同）；旋钮支在常量 0 时不可达。

- 常量生成：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:242`「pub fn container_capacity_knob_wgsl_constants(knobs: ContainerCapacityKnobs) -> String {」，没装写 0。
- 旋钮支入口：墓碑 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3187`「if (D_TOMBSTONE_CONTAINER_CAPACITY_KNOB != 0u) {」，谱系 / 退役 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3208`「if (knob_records != 0u) {」，调用点实参是 `D_LINEAGE_CONTAINER_CAPACITY_KNOB`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3038`）与 `D_RETIREMENT_CONTAINER_CAPACITY_KNOB`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3880`）。常量为 0 时这三个条件恒假，剩下的就是产品支：`< 2` 片返回，再三判，再「最右那片之外每片 == 产品容量」。
- 旧产品内核的样子（转述，出处是改动全文里被删掉的行）：`< 2` 片返回，沿叶序 `lt64` 严格递增，最右片之外 == 容量。新的比它多：首片 == 1、相邻 `lt64 && add64 == `（比严格递增更严）、每片非空；容量那一句没动。
- 我自己量的（探针 `copy/crates/singlefs-checker-tier/examples/zz_probe_kernel_size.rs`，调仓里的 `kernel_inlined_size` 与 `checker_scan_kernel_source_given_the_container_capacity_knobs`，经 `run-with-memory-cap.sh 10G cargo run --offline -q -p singlefs-checker-tier --features gpu --example zz_probe_kernel_size`；不是 checker 档测试，也没用 GPU，只解析 WGSL）原样输出（`logs/probe-k3-size.log`）：
  `SIZE recovery 332666` / `SIZE checker-scan 388815` / `SIZE checker-walk 346278`；`SIZE scan-tomb1 388815`、`SIZE scan-lin1 388815`、`SIZE scan-ret959 388815`、`SIZE scan-all 388815`；七份 `VALIDATE … ok`（naga `Validator`，`ValidationFlags::all()`、`Capabilities::all()`）；`CONSTS scan-all const D_TOMBSTONE_CONTAINER_CAPACITY_KNOB: u32 = 815u; | const D_LINEAGE_CONTAINER_CAPACITY_KNOB: u32 = 1359u; | const D_RETIREMENT_CONTAINER_CAPACITY_KNOB: u32 = 2u;`。
- 判法摘要：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:203`「container_capacity_knob_wgsl_constants(ContainerCapacityKnobs::PRODUCT).as_bytes(),」把三个 0 的那一份罩进摘要，所以摘要变了（旧入库的判定作废，这是对的）；版本前缀 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:200`「let mut message = b"gpu-crash-judge-v5\n".to_vec();」没动，规则没说前缀该不该跟着升，内容变了摘要就变，不影响正确性。

推翻条件：把 `ContainerCapacityKnobs::PRODUCT` 编出的扫描内核与第 56 批之前的扫描内核在没有 G2 新判会红的输入上比出不同位；我只比了展开量与校验，没在 GPU 上跑（复核不了）。

### K3.2　展开量钉值是不是从原理算的

判：**一致**（数对上、可复现），但「从原理算」不成立——它是量出来的，不是推导出来的。

- 钉值与预算：`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1365`「388_815,」、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1366`「11_185,」；预算 `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:246`「pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;」；`400000 − 388815 = 11185`（算术）。
- 量法是仓里定义的数法：`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:262`「pub fn kernel_inlined_size(source: &str) -> KernelInlinedSize {」，函数自己的表达式数加每个调用点上被调函数展开后的表达式数。我用同一个函数在冻结副本源码上量回 332 666 / 388 815 / 346 278（见上），与三个钉值逐个相等。这是复现，不是独立第二条路：我和实现员用的是同一个数法、同一个 naga 版本（wgpu 30.0.1）。
- 「值不同不改展开之后的大小」：四种旋钮变体都是 388 815，成立（常量是字面量，naga 解析不折叠分支，表达式数与常量值无关）。
- 增量 `388815 − 387470 = 1345`：来自共享函数 `d_judge_container_numbers_from_one_by_one_and_no_empty_container` 被产品支与旋钮支的多个调用点各展开一份（谱系 / 退役那个入口有 2 处调用点），我没单独拆出每份大小（拆法需要逐函数量自己的表达式数，探针没做）；这一句是读码推，不是量出来的。

推翻条件：另一个版本的 naga 量出不同的数；或预算 400 000 不是驱动能编的真上限（正文之外，不归本腿）。

### K3.3　带旋钮的对拍是不是真在比 GPU 与 CPU 新入口

判：**一致**（读码）；实际跑**复核不了**（要 GPU，checker 档）。

- GPU 侧编的是带旋钮的内核：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2339`「let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(knobs);」，`…:2373`「GpuCrashJudge::on_card_compiling_these_kernel_sources(」。
- CPU 侧每个崩溃镜像交新入口：`…:2434`「if knobs == ContainerCapacityKnobs::PRODUCT {」之外的支走 `…:2441`「&check_pool_image_given_the_container_capacity_knobs(」，取违例位与判过而成立的位。
- 比的是整张池级 checker 违例位集合，GPU 判不了也算不一致：`…:2548`「if undecidable || gpu_checker_bits != *cpu_checker_bits {」。
- 范围：红位与恢复字仍按 CPU 判器（产品入口），池级 checker 位才换成新入口；所以对拍证明的是「GPU 扫描内核 = CPU 新入口」在这几条流上，不是整个判器换了旋钮。
- 缺口见 K2.3 末：墓碑旋钮没有红例；产品支新三判没有红例。

推翻条件：把对拍里 `bits != *cpu_checker_bits` 的比较弱化成只比某一位而用例照绿；我没改过它，读到的是整集比较。

## K4　M3 参照与等价留档（丙 G4）

条款（`.claude/kb/decisions/08-核心索引结构.md:171` 是已定项 6 的那一长段）：
- 「左吸收右与左从右借两样都做」；
- 「左从右借时借的条数取均分（借完两片条数相差不超过 1），右片保留身份、不写退役记录，它的分隔 key 改成它这一版的最小 key」；
- 「合并时机是写路径策略（单测 / 模型对拍），不进格式」。
最后一句决定了：先右还是先左、借完接着并、删空摘片之后怎么并，kb 里**没有逐字条款**，参照照 E176 M3 那一臂写（`.claude/kb/decisions/08-核心索引结构.md:224`：「合并与借都做」（E176 的结论，同一行还写借取均分））。所以 K4.1 里「拦住改坏」的判定词是对 E176 M3 与上面三句，不是对一条写着次序的条款。

### K4.1　五段历史与「各至少一次」拦不拦得住改坏

判：**一致**（拦住我造的 11 种改坏里的 9 种；另 2 种照绿，推等价，见下）。参照**仍不调** `inode_tree.rs`。

参照独立性：`crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs:29`「use singlefs_core::transaction::TransactionOutput;」、`…:30`「use singlefs_format::INODE_LEAF_RECORDS;」、`…:31`「use singlefs_harness::history::SeededRandomSource;」；`grep -n 'inode_tree\|remove_record_from_leaf_containers\|records_below_which'` 在这份用例里的命中只有第 3 行的注释与 76/104/110/130 行的局部字段名 `inode_tree: u64`，没有对 `singlefs_core::inode_tree` 的任何引用。共用前提三样：发布路径（被测物自己）出版本、`INODE_LEAF_RECORDS`=233、E176 的规则（本来的设计）。分隔 key 现在补比：`…:361`「"{what_changed}：每片叶的分隔 key 与参照不同"」。
五形各至少一次的断言：`…:538`「reference.times_both_neighbour_pairs_fit >= 1,」起五条，借后吸收另有第三段的精确断言（借 1 次、借后吸收 1 次）。

我造的改坏（`copy/` 里改 `singlefs-core/src/inode_tree.rs`，跑 `-p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference`；变异原文在 `mutdefs/`，日志 `logs/<名>.log`）：
| 改坏 | 结果 |
|---|---|
| 登记行 31（正推 m2）借完不接着并 | 红，`…reference.rs:343` |
| 登记行 32（攻方 k4-no-absorb-after-borrow） | 红，`:343` |
| 登记行 33（m5）左吸收整支关掉 | 红，`:343` |
| 登记行 34（m9）借完不改右邻分隔 key | 红，`:351`（分隔 key 那一句） |
| ka 先左后右（`absorb_until_no_pair_fits` 里两支互换） | 红 |
| kb 借的下限 117 → 116（`div_ceil(2)` → `/ 2`） | 红 |
| kc 借的条数向上取整 | 红 |
| ke 下限判 `>=` 改 `>` | 红 |
| kg 装得下判 `<= 233` 改 `< 233` | 红（71 秒，走满断言上界） |
| kh 左吸收后幸存片改取右片的分隔 key | 红 |
| ki 借的条数少 1 | 红 |
| **kd** 删空摘片之后从 `holding_position` 起并，不是 `holding_position − 1` | **绿** |
| **kf** 并没并过都去试借（`if !absorbed_at_least_once` 改 `if true`） | **绿** |
kd、kf 绿的理由（推，没有造出区分的可达形）：走得到的形状里每个非最右片 ≥ 117 条、相邻两片之和 ≥ 234（追加只落最右片：满 233 再开新片，所以 233 + 最右 r ≥ 234；删一条之后下面 K4.2 的论证说它对删除封闭）。kd：删空摘片时被摘的那片删前只有 1 条，两邻居各 ≥ 233，即各恰好 233，摘掉后那一对 466，从哪一边起都并不了。kf：并过之后幸存片 233 条，要借需少于 117；左邻吸收它时下标 `holding_position` 指到原右邻，它若不是最右就 ≥ 117，是最右就没有右邻，借都返回。两者都没有等价留档用例。

推翻条件：找到一个走得到的形状（追加 / 删除 / 借 / 并 / 回退到旧根 / 崩溃恢复能到的树）让 kd 或 kf 与实现的结果不同；或任一条红的改坏其实是被「族里别的用例」而不是这条参照用例拦下的（我只开了这一份用例目标，红的就是它）。

### K4.2　等价测试的族罩不罩得住走得到的形状

判：**一致**（没有「相邻两片之和 ≥ 234 而两种写法结局不同」的形，不论取值是不是在族里）。

- 族：`crates/singlefs-core/src/inode_tree.rs:1409`「const RECORD_COUNTS_OF_THE_EQUIVALENCE_FAMILY: [usize; 6] = [1, 2, 116, 117, 118, 233];」、`…:1411`「const CONTAINERS_IN_THE_EQUIVALENCE_FAMILY_AT_MOST: usize = 4;」；两种改写的枚举 `…:1298`「enum MergeRuleRewriting {」；用例 `…:1495`「fn stopping_every_absorb_pass_after_one_pair_changes_nothing_when_every_adjacent_pair_overflows_one_container(」、`…:1552`「fn not_absorbing_after_dropping_an_emptied_container_changes_nothing_when_every_adjacent_pair_overflows_one_container(」。每片只删头与尾两条。
- 我把族放宽去找反例：探针（追加进 `copy/` 的 `inode_tree.rs` 的 tests 模块，跑完已 `cp` 回原文）用 33 个取值（1–4、37、55–60、74、111–122、148、185、222、228–233）、1 到 4 片、相邻和 ≥ 234 的全部组合，每片删头尾，对实现与两种改写整份比。原样输出（`logs/probe-k4.log`）：`PROBE shapes=221256 removals=1744830 diffs=0`。
- 探针有牙（对照）：同一探针把「相邻和 ≥ 234」过滤关掉、片数限 3，`PROBE shapes=37059 removals=216710 diffs=20`（`logs/probe-k4-control.log`），例 `DIFF [1, 1, 1] del InodeNumber(2) NothingIsAbsorbedAfterDroppingTheEmptiedContainer`。所以等价成立靠的是这个前提，不是靠族取值。
- 论证（读 `crates/singlefs-core/src/inode_tree.rs:458` 的 `absorb_until_no_pair_fits` 与 `:485` 的 `borrow_from_the_right_neighbour_if_below_the_floor`）：删一条只让一对的和减 1，所以只有恰好 234 的一对能变成 233 并合；并完幸存片 233 条，与任何邻居相加 > 233，链接不下去（改写一与实现同）；借的情形下那一对本来 ≥ 234 且借不改和，借出片 ≥ 117，吸收下一片后若还能再并需下一对和 ≤ 233 而它们的和 ≥ 234，链同样断；删空摘片的邻居各恰好 233。删后相邻和仍 ≥ 234（前提对删除封闭，用例自己在 `crates/singlefs-core/src/inode_tree.rs:1516`「pair[0].records.len() + pair[1].records.len() >= 234,」断言）。追加只落最右片，所以走得到的树都满足前提；回退到旧根与崩溃恢复落到的树，是某个当时走得到的树。
- 变异：登记行 35 / 36（把装得下的判据放宽）把两个等价用例与别的旧用例一起打红（行 35：4 条红，含两个等价用例；行 36：8 条红，含两个等价用例；`logs/row35.log`、`logs/row36.log`）。注意这两行同时被旧用例拦，不是只靠等价用例。

推翻条件：找到一条走得到的路径产出相邻和 ≤ 233 的树（例如写侧能往中间片插入，kb 说插入永远落最右：`.claude/kb/decisions/08-核心索引结构.md:171` 的「插入永远落在最右的叶」），那时两种改写可能与实现分叉；探针只扫到 4 片，更多片靠论证里的局部性（一次删除只动一对邻居与它们的右邻）。

## K5　回退取树 ID 水位（甲 G5）

条款：D23 已定项 14 表里「水位」一格，`.claude/kb/decisions/23-journal的角色与格式.md:385`：「树 ID 水位照旧 = max(cur 那一版内存里的水位, 环里读得出的全部根的水位)」。

### K5.1　抽出前后行为相同吗

判：**一致**。
- 抽出后：`crates/singlefs-core/src/mount.rs:7865`「fn tree_identifier_watermark_of_the_rollback(」，体是环里全部根的水位折叠进 cur 那一项：`crates/singlefs-core/src/mount.rs:7873`「.fold(tree_identifier_watermark_of_the_current_version, u64::max)」；调用点 `crates/singlefs-core/src/mount.rs:8147`「let tree_identifier_watermark = tree_identifier_watermark_of_the_rollback(」，实参是 `&ring_roots`、`&current_table` 与 `current.root.tree_identifier_watermark`。
- 抽出前（转述，出处改动全文里被删的四行）：`ring_roots.iter().map(|root| root.tree_identifier_watermark).fold(current.root.tree_identifier_watermark, u64::max)`。迭代器、折叠初值、`max` 都与抽出后相同；`_current_table_not_consulted` 不进计算。纯重排。
- 实测：`singlefs-core` 库内 25 条（含两条新单测与 `inode_tree::tests`）全绿（`logs/core-lib-base.log`：`test result: ok. 25 passed`）；回退相关 harness 文件 `rollback_by_a_forward_publish`（3 过 22 忽略）、`unlink_idempotence_criteria_and_rollback_to_the_emptied_version`（5 过）、`inode_number_watermark_pressed_back_reissues_a_number`（3 过）全绿（`logs/k1-legit.log`）。

推翻条件：任何一次回退里，抽出前后的树 ID 水位不同（我没做新旧并行对拍，因为纯重排，读码已见同一表达式）。

### K5.2　单测钉的是不是已定项 14 那半句

判：**一致**（两条各钉一半，不漏）。
- 第一条 `crates/singlefs-core/src/mount.rs:9499`「fn an_abandoned_ring_root_whose_tree_identifier_watermark_is_above_the_current_version_sets_the_watermark(」：环里一条按 cur 实例表被抛弃的根（水位 65560 > cur 的 65549），断言取它——钉「全部根」，`crates/singlefs-core/src/mount.rs:9522`「"树 ID 水位取环里读得出的全部根：被崩溃恢复抛弃的那条照旧算"」。
- 第二条 `crates/singlefs-core/src/mount.rs:9528`「fn ring_roots_all_below_the_current_version_leave_the_current_tree_identifier_watermark() {」：环里根都低于 cur，断言取 cur——钉「与 cur 取 max」。
- 变异：登记行 2（改成只取没被抛弃的根）红在第一条；行 3（折叠初值改 0）红在第二条；其余一条都没红（`logs/row2.log`、`logs/row3.log`：各 `1 passed; 1 failed`）。
- 没钉的方向：改成「只取被抛弃的根」。第一条里没被抛弃的那条根（水位 65540）低于 cur，所以这种改法也给 65560；第二条两条根都低于 cur 取 cur，也过。这种改法在走得到的状态上与原式等价（没被抛弃的环里根的水位不会高于 cur 那一版内存里的），所以不算判别力缺口，记一笔。

### K5.3　不读的参数是不是在给错误写法开门

判：**规则没说**（没有条款写函数签名；这是设计取舍）。
- 事实：`crates/singlefs-core/src/mount.rs:7867`「_current_table_not_consulted: &InstanceTableRecords,」，下划线开头、不读；与 inode 号水位那一个并排：`crates/singlefs-core/src/mount.rs:7845`「fn inode_number_watermark_of_the_rollback(」收同一个 `current_table`，里面真用它去滤（`crates/singlefs-core/src/mount.rs:7829`「fn ring_roots_not_abandoned_by_crash_recovery(」）。
- 开门的意思：签名上有一张 cur 的实例表可用，写错的人（抄 inode 号水位那一段的人）写得出「只取没被抛弃的根」。登记行 2 就是这样写出来并编过的（编过、第一条单测红），所以这扇门是故意留给单测的。
- 不开门的代价：不收这个参数，行 2 那一错写得多写一个参数才写得出，单测要靠改签名才能红，等于没有构造态输入。
- 它不改语义：两条单测加行 2、3 钉住；clippy 对下划线前缀的未用参数不报（主 agent 说 clippy 过）。

推翻条件：有人在这个函数里把 `_current_table_not_consulted` 用起来而两条单测与回退用例仍绿——我造的行 2 说明这一形会红。

## K1　恢复的新次序（甲 G1）

条款：
- `.claude/kb/invariants.md:35`（I-1.10）：「判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：先于按字段表解条目」——只管「条目宽」那一判在解条目之前，**没有**写「逐条判代与 seq」排在「数条目」之前。逐条判代与 seq 先于数条目，来自判决 G1 的验收与第 52 批自己写的理由，kb 里规则没说。
- D5 对台阶行的口径：`.claude/kb/decisions/05-快照-空间记账机制.md:53`「台阶行随根 COW、一直留着」；`.claude/kb/invariants.md:155`（I-3.13）：「inode 号水位那一统计量的台阶行不判」（代段等于这一版 txg 那一条对台阶行豁免）。条款没写「台阶行的代必须早于根」，只有恢复代码里的通用判「代不晚于根」（见下）。

### K1.1　新次序下哪种坏法的成员变了；合法镜像被拒没有

判：**一致**（变的只有「台阶行的代写到根之后」这一族，是 G1 要的；合法镜像一个没被拒）。

两个读者的新次序（行号都在冻结副本 `singlefs-core/src/recovery.rs`）：
| 读者 | 次序（行号） |
|---|---|
| 重建（可写挂载用，`crates/singlefs-core/src/mount.rs:1236`「let rebuilt = rebuild_version_in_the_unit_area_starting_at(」） | 分配记录几何 2882 → 每盘一条 2888 → 分配记录代与跨度 2894 → 读记账树 2900 → 条目宽 2918 → 逐条判代与 seq 2923 → 数条目 2928 → 水位行在不在 2940 → 映射条目数 2944 |
| 冷走读（`recover` 与只读挂载用，入口 `crates/singlefs-core/src/recovery.rs:5766` 的 `walk_to_file_in_the_unit_area_starting_at`） | 先读记账树 5288 → 分配记录几何 5385 → 每盘一条 5387 → 分配记录代与跨度 5395 → 条目宽 5396 → 逐条判代与 seq 5399 → 数条目 5402 → 映射条目数 5407 |
- 新次序里，一个条目宽够、代不晚于根、seq 非 0 的合法记账叶，所有这些判照样一个个过：次序只是挪了位置，没有新加判，也没有去掉判。所以合法镜像不会被新拒。实测：`singlefs-harness` 里与记账 / 台阶行 / 回退 / 可写挂载相关的 6 份用例文件共 41 条全绿（`logs/k1-legit.log`：`accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf` 2 过、`core_review_tree_table_duplicates_and_slot_one_search` 13 过、`inode_number_watermark_pressed_back_reissues_a_number` 3 过、`rollback_by_a_forward_publish` 3 过 22 忽略、`unlink_idempotence_criteria_and_rollback_to_the_emptied_version` 5 过、`writable_mount_of_a_formatted_pool` 15 过）；`corrupt_on_disk_content_is_refused_instead_of_panicking` 35 条全绿（`logs/base-harness-1.log`）。多盘：这些池都是 2 盘（`3 + 6 × 2 = 15` 行），没有 3 盘以上的用例；次序与盘数无关，读码推。
- 自造的差分扫描（探针 `copy/crates/singlefs-harness/tests/zz_probe_k1_diff.rs`，不入仓）：以「最新根记账树根里 16 条、其中 1 条是台阶行」那个池为底，对第 14（台阶行）、15（这一版自己的水位行）、7（一条普通行）条条目，各改统计量（不改 / 改成 12）、代（不改 / 4 / 5，根的 txg 是 4）、seq（不改 / 0），按 key 重排、节点头最小最大 key 与各层校验和重算，得 33 张镜像；每张分别跑 `recover(…, JournalPolicy::Consult)` 与 `mount_writable`，旧次序（把登记行 0、1 两个变异换上去，即「数条目排回逐条判之前」）与新次序各跑一遍。原样结论（`logs/fuzz-new.out`、`logs/fuzz-old.out`，比对脚本 `fuzzdiff.py` 的输出）：
  `images 33` / 新次序恢复成员分布 `22 ('rec', '记账条目的代晚于根或 seq 为 0')`、`7 ('rec', 'FileRead')`、`4 ('rec', '叶里的条目不按 key 严格递增')` / `4 ('rec', '记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数', '记账条目的代晚于根或 seq 为 0') e.g. idx=14 stat=None gen=Some(5) seq=None` / 可写挂载同样 4 张 / `total images with any change: 4`。
  变了的 4 张都是第 14 条（台阶行）的代改成 5（晚于根的 4）：旧次序报条目数，新次序报代晚于根。别的 29 张，两种次序成员逐字相同，其中 7 张改完仍是合法镜像、两个读者都读得出文件（`FileRead`）。
- 读码：该变的只此一族。数条目那一判只看「解得开且统计量 12 且代 < 根 txg」才不算；`crates/singlefs-core/src/recovery.rs:4795`「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg」。逐条判在前，改变结局的只有「某条目 代 > 根」（此时旧次序里它要么被当普通条目多算一条而报条目数，要么刚好不影响条目数而在逐条判里报代）与「seq == 0」（条目数不受影响，两种次序都报 seq，成员相同）。所以变的只有「统计量 12 且代 > 根」这一类，别的坏法报的成员不变。
- 双坏镜像（同一张镜像上两处同时坏）的成员也可能变，这是任何次序重排的必然代价，不算「不该变」：冷走读里分配记录代与跨度从「映射条目数之后」挪到「条目宽之前」（改动全文里删去的两行在映射条目数那一判之后，现在在 `crates/singlefs-core/src/recovery.rs:5395`），所以「分配代晚于根 + 条目数不对」的镜像，冷走读旧报条目数 / 映射条目数、新报分配记录代；这一形我没造镜像（读码推）。

推翻条件：任何一张单坏法镜像，新旧次序下读者交回的 `detail` 不同，而坏的不是「台阶行的代晚于根」；或一张合法镜像（`recover` 能读出文件、`mount_writable` 能挂上）在新次序下被拒。扫描只覆盖了 3 个条目位置 × 2 × 3 × 2 个值，不是全空间。

### K1.2　三条路的次序一致吗；坏盘战役钉的成员受不受影响

判：**一致**（就这五判的相对次序）；战役本身**复核不了**（checker 档，不跑），读码判**不受影响**。

- 「三条路」其实是两个读者：重建（可写挂载读阶段与回退、水位重建都调它：`crates/singlefs-core/src/mount.rs:1236`、`crates/singlefs-core/src/mount.rs:7811`「match rebuild_version_in_the_unit_area_starting_at(」、`crates/singlefs-core/src/mount.rs:8049`「let target_version = match rebuild_version_in_the_unit_area_starting_at(」），与冷走读（`recover` 与只读挂载）。上面表里的相对次序在两个读者里是同一串：分配记录代与跨度 → 条目宽 → 逐条判代与 seq → 数条目 → 映射条目数。
- 不一致的只有「读树」那一步和水位行：冷走读 5288 先把记账树读完（结构判全，含 key 严格递增）再做分配记录各判，重建 2900 在分配记录代与跨度（2894）之后才读记账树；水位行在不在的判只有重建有（2940），冷走读没有（`grep -n InodeNumberWatermarkRowMissingFromTheAccountingTree recovery.rs` 的命中只有第 257 行定义、第 2516 行注释与第 2940 行这一处使用，冷走读 5170–5430 一带没有）。所以「分配代晚于根 + 记账叶 key 不严格递增」的双坏镜像，两个读者报不同成员；这条在第 56 批之前就存在（重建的分配代判 2894 没挪），不是这批新引入的，记一笔。
- 战役钉的成员（`singlefs-checker-tier/tests/bad_disk_input_campaign.rs`，读码）：记账树条目宽那一格两边钉 `EntryNarrowerThanItsFieldTable { what: "记账条目"`（条目宽判仍排在逐条判与数条目之前，没变）；水位行改标签那一格，可写挂载钉 `InodeNumberWatermarkRowMissingFromTheAccountingTree`（`crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs:406` 起），恢复钉 `FileRead（实例 2 第 9 代根`：改标签的那一行代等于根、seq 非 0，逐条判过、数条目不变（它仍不是代 < 根 的 12 号行），所以不受重排影响（推）。其余格钉的是分配记录、中央映射、树表，不碰这五判的相对次序。盲坏法（翻位、清零扇区、截尾）只钉「不 panic」。
- 第 52 批两条旧用例（条目宽先于数条目）在新次序下仍红在旧变异上：登记行 `第52批 冷走读记账条目数排回条目宽之前…` 与 `第52批 重建记账条目数排回条目宽之前…`，各把次序换回去，`a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery` / `…_by_the_writable_mount` 红（`logs/queue2.out` 的 old rows 段）。

推翻条件：战役里某格的实测成员在新次序下与它钉的不同（要跑 checker 档，我没跑）；或两个读者里这五判的相对次序有一处对不上（我读了 2882–2944 与 5385–5407 两段，一一对上）。

## K6　判别力：第 56 批新加与替换的变异行

方法：`crates/mutations.tsv` 里以「第56批」或「第 56 批」开头的行共 37 行（我用 python 数，锚点在冻结副本里逐行恰好命中一次：`rows 37 anchor!=1: 0`）。每行在 `copy/` 里把原文换成替换文，跑该行写的点名用例（命令原样取自该行第 5 段，加我的内存包装与线程上限），红了记「红」，跑完 `cp` 回冻结副本原文并 `cmp` 相同。驱动 `mut.sh`、`runrow.sh`，日志 `logs/row<行号>.log`（行号是我在 37 行里的序号 0–36）。

### K6.1　37 行逐行结果

| 行（序号 0–36） | 内容 | 我的结果 |
|---|---|---|
| 0、1 | G1 冷走读 / 重建数条目排回逐条判之前 | 红：`an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_recovery` / `…_by_the_writable_mount` |
| 2、3 | G5 只取没被抛弃的根 / 折叠初值改 0 | 红：`mount::tree_identifier_watermark_of_the_rollback_tests::an_abandoned_ring_root_…` / `…ring_roots_all_below_the_current_version_…` |
| 4、5 | G2 首片不判是 1（谱系 + 退役 / 墓碑） | 红：`lineage_…` 与 `retirement_containers_numbered_two_and_three_…`；`tombstone_containers_numbered_two_and_three_…` |
| 6、7 | G2 只判严格递增 | 红：`lineage_` / `retirement_containers_numbered_one_and_three_…`；`tombstone_containers_numbered_one_and_three_…` |
| 8、9 | G3 相邻整句恒真（攻方 c-incr-true / 正推 w1） | 红：`lineage_` / `retirement_containers_numbered_one_and_one_…` |
| 10、11 | G2 不判每片至少一条 | 红：`lineage_` / `retirement_containers_with_an_empty_rightmost_one_…`；`tombstone_…` |
| 12 | G3 谱系产品容量写成 959（慢档，release，`--include-ignored`） | 红（128 秒）；未变异基线绿（`logs/base-slow12.log`：`test result: ok. 1 passed`） |
| 13 | G3 退役产品容量写成 1359（慢档，960 次退役） | **没跑完**：前台限时 200 秒到（`logs/slow13.log` 末行 `exit=124`），复核不了，交主 agent |
| 14 | G3 最右那片也要求装满（慢档） | 红（168 秒，`logs/slow14.log`） |
| 15、16、17 | G3 谱系读不全 / 退役读不全 / 整版树表读不出时不报判不了 | 红：`an_unreadable_lineage_container_…`、`an_unreadable_retirement_container_…`、`an_unreadable_tree_table_…` |
| 18、19 | G3 旧墓碑旋钮入口给谱系 / 退役旋钮 | 红：`the_old_tombstone_knob_entry_judges_the_lineage_tree_…` / `…_retirement_tree_…` |
| 20 | G3 退役旋钮值加一 | 红：`a_retirement_container_holding_more_records_than_the_knob_…` |
| 21 | G2 相邻要求加二（合法随机历史上误报，慢档） | 红（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:672`，23 秒）；未变异基线绿（`logs/base-random-history.log`：`CONTAINER_TREES_RANDOM_HISTORY seed=7 containers(tombstone, lineage, retirement)=(37, 9, 4) levels(tombstone, lineage, retirement)=(6, 4, 2)` / `test result: ok. 1 passed`） |
| 22–30 | GPU 判器九行（首片 / 加一 / 整句不判 / 至少一条 / 旋钮加一 / 三棵不接旋钮 / 旋钮常量恒 0） | **没跑**：点名用例是 GPU 对拍用例（checker 档，要 GPU），不归我跑，复核不了。读码：每行改的是 WGSL 里一处调用或比较，行里点名的用例（`…lineage_containers_start_from_two` 等）对应读得到的红例；见 K2.3 末的缺口 |
| 31–34 | G4 参照用例四行（借后不并 ×2、左吸收关掉、借后不改分隔 key） | 红：`every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference`（`…reference.rs:343` 或 `:351`） |
| 35、36 | G4 把装得下的判据放宽，等价留档用例要红 | 红：行 35 红 4 条（含两个等价用例），行 36 红 8 条（含两个等价用例）；旧用例也红，所以这两行不是只靠新用例拦 |

计数：我跑红的 27 行（序号 0–12、14–21、31–36），没跑完 1 行（13），没跑 9 行（22–30）。27 + 1 + 9 = 37。命令：`grep -c '^test .* \.\.\. FAILED'` 逐份日志数过（见上面检查脚本输出，每行至少 1 条红）。老行（第 52 批两行、C586、第 24 批 K M5）也各跑了，红（`logs/queue2.out` old rows 段）。

### K6.2　还有哪几种改坏照绿

我自造的（变异原文在 `mutdefs/`，日志同名）：
| 改坏 | 跑什么 | 结果 | 是缺口吗 |
|---|---|---|---|
| **c3** 首片号判成 `<= 1`（接受 0） | `checker_known_bad_images` 全文件 + `container_trees_grow_a_level_when_the_root_is_full` | 全绿：247 + 6 条（`logs/c3.log`） | **是**。`.claude/kb/layout/03-third-txn.md:18` 写「0 不用」，没有用例拿首片号 0 造坏镜像 |
| **c789** 产品支（没装旋钮）的谱系 / 退役新三判（首片是 1、相邻加一、每片非空）各加 `|| knob_records_per_container.is_none()` 取消 | 过滤 `tombstone lineage retirement container` 的 96 条 + `container_trees…` | 全绿（`logs/c789.log`） | **是**（G2 验收写「三棵树各红自己那一条」，实际新红例全走旋钮支，I-9.36 的新三判没有红例） |
| **t789** 产品支墓碑 I-9.21 的新三判同样取消 | 过滤 `tombstone` 的 35 条 | 全绿（`logs/t789.log`） | **是**，同上 |
| c7、c8、c9、t7、t8、t9 六处合在一起、不过滤全文件 | 前台限时内没跑完（`logs/all6rest.log` 末行 `exit=124`；`logs/all6.log` 被我的外层限时截断，改的 walk.rs 已手工恢复并 `cmp` 相同） | 复核不了 | 未知：过滤之外的 151 条没跑 |
| **ca** 冷走读把分配记录代与跨度挪回逐条判之后（`crates/singlefs-core/src/recovery.rs:5395` 那一行挪到数条目之后） | `corrupt_on_disk_content_is_refused_instead_of_panicking` 35 条 | 全绿（`logs/ca.log`） | 是：次序统一里「分配记录代与跨度排前」是实现员自定、没有变异行也没有用例钉 |
| c5 相邻加一的 `checked_add` 换成 `wrapping_add` | 同 c3 两份 | 全绿 | 否：只在左片号 = u64::MAX 且右片号 = 0 时分叉，容器号到不了（等价） |
| kb2 回退水位改成只取被抛弃的根 | `tree_identifier_watermark_of_the_rollback_tests` | 全绿（2 条） | 否：走得到的状态上两者相同（没被抛弃的环里根的水位不高于 cur 内存里的），见 K5.2 |
| kd 删空摘片后并的起点 `holding_position − 1` → `holding_position` | 参照用例 | 绿 | 否（推等价，见 K4.1） |
| kf 并没并过都去试借 | 参照用例 | 绿 | 否（推等价，见 K4.1） |
| c1 谱系 / 退役产品支改成 1 片也判 | 两份 | 红，**只有 1 条**：`birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_same_line_and_the_every_readable_root_birth_invariants`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:3927`，断言回退到 A 之后 I-9.36 判定是 `NotApplicable(…)`，实际 `Holds`） | 弱：「产品支一片不判」被钉在一条附带用例上 |
- GPU 一侧（读码，没跑）：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3194`「d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_21);」与 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3215`「d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_36);」被摘掉，第 56 批的 9 行 GPU 变异都不覆盖（那 9 行摘的是共享函数体内的比较或旋钮分支），旧产品支红例靠记录数红同一位；所以这两个调用点没人看守。

推翻条件：上表任一「是缺口」的行，在别的用例（我没开的文件、checker 档用例、GPU 对拍）里被红了——我只开了 `singlefs-harness` 的 `checker_known_bad_images`（过滤）、`container_trees_grow_a_level_when_the_root_is_full`、`corrupt_on_disk_content_is_refused_instead_of_panicking`、`inode_leaf_merge_and_borrow_against_an_independent_reference` 与 `singlefs-core --lib`。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1.1 新次序的成员变化与合法镜像 | 一致 | 33 张差分镜像只有「台阶行代晚于根」4 张成员变（条目数 → 代晚于根），别的 29 张逐字相同，合法镜像（41 + 35 条用例、7 张仍可读的改动镜像）没被拒；双坏镜像的成员变化读码推、未造 |
| K1.2 两读者次序与战役钉 | 一致（相对次序）；战役复核不了 | 五判相对次序两读者同串；读树与水位行位置本来就不同（第 56 批前已有）；战役格读码判不受影响 |
| K2.1 合法镜像误报 | 一致 | 写侧只有两个追加函数，首片 1 / 加一 / 非空由构造保证；63 + 30 个崩溃前缀镜像、随机历史 seed 7（37/9/4 片）带旋钮入口一条不红 |
| K2.2 坏镜像漏报 | 规则没说 | 产品支一片不判；跨版本「后一版容器集含前一版」没有判（读码推，未造）；首片号 0 的坏镜像没有用例（c3 绿） |
| K2.3 CPU 与 GPU | 一致（读码）；对拍复核不了 | 三句 CPU / GPU 同形，溢出两边都红；旋钮 0 的那份就是 `of_the_judge()` 的产品编法；GPU 产品支新三判无红例 |
| K3.1 常量 0 = 产品判法 | 一致 | 旋钮支在常量 0 时不可达；四种旋钮变体的内核展开量都是 388 815，naga 校验过 |
| K3.2 展开量钉值 | 一致（量出来的，不是推出来的） | 用仓里同一个数法复现 332 666 / 388 815 / 346 278，与钉值逐个相等；预算余量 11 185 |
| K3.3 带旋钮的对拍 | 一致（读码）；实跑复核不了 | GPU 编旋钮内核、CPU 走新入口、整集违例位逐位比；墓碑旋钮无红例 |
| K4.1 五段历史拦不拦得住 | 一致 | 11 种我造的改坏 9 种红；kd、kf 绿并推等价；参照仍不调 `inode_tree.rs` |
| K4.2 族罩不罩得住可达形 | 一致 | 把族放宽到 33 个取值 × 4 片的 221 256 棵树 1 744 830 次删除，两种改写与实现 0 处不同；去掉「相邻和 ≥ 234」前提立刻有分叉 |
| K5.1 抽出前后行为 | 一致 | 纯重排，库内 25 条与回退相关 harness 全绿 |
| K5.2 单测钉不钉 D23 已定项 14 那半句 | 一致 | 两条各钉「全部根」与「与 cur 取 max」，登记行 2、3 各被一条拦；「只取被抛弃的根」不被拦（等价） |
| K5.3 不读的参数 | 规则没说 | 条款没写签名；这个参数是故意留给「只取没被抛弃的根」这一错写的构造态单测的，行 2 证明它会红 |
| K6 变异行判别力 | 一致（27 行红）；13 与 22–30 复核不了；有缺口 | 缺口：首片号 0（c3）、产品支新三判（c789、t789）、冷走读分配记录代位置（ca）无人看守 |

## 要主 agent 跑的命令

1. 第 56 批登记行 13（慢档，960 次退役，release）：工作目录仓根的冻结副本或主工作区；`SINGLEFS_HEAVY_TESTS` 不需要（harness 档）；`bash scripts/hosts/capped.sh 10 bash scripts/hosts/run-with-memory-cap.sh 10G cargo test --offline --release -p singlefs-harness --test checker_known_bad_images -- --include-ignored nine_hundred_sixty_retirements_at_product_capacity_fill_every_retirement_container_but_the_rightmost_and_redden_nothing`，先在没变异的树上跑一遍判基线绿，再按 `crates/mutations.tsv` 该行做变异跑红；我在前台 200 秒限时里没跑完。
2. 第 56 批登记行 22–30（GPU 对拍）：checker 档，要主 agent 提交时那一趟（`cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state`，上限与线程照主 agent 平时的）。
3. 上面 K6.2 的 c3 / c789 / t789 / ca 四种改坏要在更大范围复跑：`checker_known_bad_images` 全文件不过滤的一遍我前台没跑完；原文在 `research/prompts/m3-batch56-code-r1-sonnet-model/mutdefs/`（c3.old/c3.new 等）。

## 没做什么

- 不判别的腿的格、不替主 agent采纳或出判决：上面的「判定」只对 K1–K6 逐条对条款与代码，没有「采纳 / 不采纳」。
- 没跑门禁阶段、没跑 checker 档测试（含 GPU 对拍、`bad_disk_input_campaign`、`cargo test -p singlefs-checker-tier`）；凡是只有它们能核的，上面写「复核不了」。我跑过 `singlefs-checker-tier` 的一个自写示例（`examples/zz_probe_kernel_size.rs`，只解析 WGSL、量展开大小，不上 GPU），它不在仓里。
- 没跑 `cargo test` 全量、没动冻结副本；`copy/` 里的探针（`zz_probe_*.rs`、`examples/zz_probe_kernel_size.rs`）与变异都没入仓。
- 没造的：K1.1 的「双坏镜像」成员变化（只读码推）；K2.2 的跨版本漏报镜像；K4 里 4 片以上的可达形（只有局部性论证）；K6 的登记行 13、22–30。
- 没在 kb 或 `crates/` 里改任何一个字；报告以外只写了草稿目录与 `research/prompts/m3-batch56-code-r1-sonnet-model/`。
- 自己建的编译目录 `~/.local/share/singlefs/work/m3-batch56-code-r1-sonnet/target`（6.3G）与仓拷贝 `~/.local/share/singlefs/work/m3-batch56-code-r1-sonnet/copy` 交回前已删；脚本、变异原文、探针与日志留在 `research/prompts/m3-batch56-code-r1-sonnet-model/`（和草稿目录 `~/.local/share/singlefs/work/m3-batch56-code-r1-sonnet/` 里的同一份）。重跑要先把冻结副本拷成 `copy/`，并把 `.cargo/` 一并拷进去。
