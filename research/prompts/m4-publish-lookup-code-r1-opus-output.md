# m4-publish-lookup-code-r1 攻方腿（Opus）报告（2026-10-20）

立场：在冻结副本的拷贝上造反例（改前改后写出不同的盘镜像、查到不同的项、改后 panic 而改前不 panic），T4 造改坏查法而测试不红的变异。第一轮，没有已有判决。
引代码一律写冻结副本里的绝对路径与行号（主工作区的 `transaction.rs`、`records.rs` 已与冻结副本不同：sha256 前 16 位主工作区 `178ec464ce490383`、冻结副本 `4f27be528003e1bf`）。副本上量的数都是副本上的数。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 索引与线性查找是不是同一项 | 没打中 | 223 段随机历史、23 段写死前缀加放开用户动作的历史，改前改后每一步的写流累计摘要与内存里那一版的摘要逐行相同；读码：取最先一项、身份类型的 Ord 是派生的、建索引之后表改不了 |
| T2 整理读版本一遍走叶与逐 key 查 | 没打中 | 三片叶的一版上把叶互换、互抄、同一角色多一项放前放后、删一片叶，16 种版本 × 897 把 key（含短一字节、长一字节、空 key）共 14352 次比对逐个相同，942 次两边一起 panic 且文案相同 |
| T3 照抄路径 | 没打中 | 三处查不到时的 panic 条件与文案逐字不变；`resolve` 每调一次现建一次索引，不跨轮留着；差分历史里没有一次 panic |
| T4 测试与变异判得出它要判的 | 打中两处 | ① 整理读版本只读半张映射（`.step_by(2)`）：点名用例 4 条全过、池级 checker 判绿，而两档镜像摘要都变了（`87adda…`→`763f44…`、`3344bc…`→`f06592…`）；② 增长行「整理读版本退回逐 key 查映射」三次量 354%、358%、362%，都在门槛 400% 以下，不红 |
| T5 有没有改到「行为不变」之外 | 没打中 | 补丁除查法之外只加了公开项、换了私有函数 `carried_unit` 的签名、改了导入与文档；panic 文案逐字相同；差分里写流（含 FUA、屏障与内容哈希）逐行相同 |

T4 两处打中都不是补丁行为的反例，是「点名的测试判不出」的反例；补丁本身在我取样的历史上没有一处与改前不同。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-publish-lookup-code-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo \
  --work <新的草稿目录>
```

它建四份树（改后、改前 = 补丁反向、半张映射变异、增长第九行变异），release 编差分装置、debug 编七·6 用例，跑一组小样本（随机历史每组比重 2 个种子、写死前缀两档各 1 个种子、T2 换叶对拍、半张映射变异上的七·6 用例、第九行变异上的增长慢档用例），打 `name=` 行，与 `out/rerun-expected.txt` 比。整条 `rerun.sh` 我没跑过（四份树编下来超过前台 240 秒），报告里的数都是下面逐批分开跑的；`tools/run_diff.sh` 是我逐批跑用的驱动，路径写死在我的草稿目录。

模型目录 `research/prompts/m4-publish-lookup-code-r1-opus-model/` 每个文件的 sha256：

```
3f735dbfd965665bf20238cc84674195514b684b666becec48e166b5f56e7483  attack_differential.rs
07c89c9be872290b46f27f582de2625a2b8e210fb518894d919aa6f07973478e  attack_leaf_read.rs
e98e7f0de31e3246bc7f439094b85782777a9f58a9d30ec726d88848b855cc4e  crates.patch
e7db96e6e1d07aefe565eeac23c663f8cb8663eb419223f4f6911d2b1d72147c  out/mutants.txt
371fc132e86e107a98f26cb94a68098beea8434bd6f56e9498839f5cffb0599f  out/random-histories-after.txt
371fc132e86e107a98f26cb94a68098beea8434bd6f56e9498839f5cffb0599f  out/random-histories-before.txt
9950a95270bd426747630ef75326a070644db8426e951ca65ba9c3bfcc457fde  out/rerun-expected.txt
40a156462dd7502e97281268114ea4725f7ed6a0a800cee2186eff6b80dae16c  out/scripted-after.txt
40a156462dd7502e97281268114ea4725f7ed6a0a800cee2186eff6b80dae16c  out/scripted-before.txt
8e739fa1652da282e2d3794c38fbc15e4c7b3a5246a67cdb8c60e86dc974e3a8  out/t2-leaf-read.txt
427364dd0eb937dd4f249fcc5cc6e17844f593c2deae4b04f62d4d360c4d56a0  tools/apply_once.py
6569083097b74ded56f4f3b00df665362b3fdc0b66555f8ad03d784729cc7384  tools/rerun.sh
b77d6c7977cd2781b1b950e2e7239b2e1e3e95b57dc66ab095cdc051675f072e  tools/run_diff.sh
```

`crates.patch` 是背景材料改动全文 `research/prompts/_m4-publish-lookup-code-r1-diff.md` 第 6–1358 行原样（`crates.patch` 那一段围栏里的内容）；改前树 = 冻结副本拷一份再 `git apply -R crates.patch`（`git apply -R --check` 先过）。`out/` 下 before / after 两份成对文件 sha256 相同，就是「改前改后逐行相同」的落盘形态。

## 装置：改前改后差分

同一份 `attack_differential.rs` 放进改前、改后两棵树的 `crates/singlefs-harness/tests/`，只用两边都有的 API，两份二进制跑同一组历史，每一步打一行，两边逐行比：

- 随机历史（`random_histories_print_digests`）：`GenerationWeights::NAMED` 的 14 组比重，每组种子 0–15，每段 60 步，经 `execute_history_handing_out_every_step_image` 跑。每一步打镜像（写过的扇区）的 sha256 与录制流到这一步的操作数，每一步结局的 Debug 摘要，每段末尾打结局、整条写流（`RecordedOperation::to_stream_line`，含写 / FUA / 清零 / discard / 屏障与内容 FNV 哈希）的摘要。
- 写死前缀加放开用户动作（`scripted_histories_print_digests`）：录制盘上照七·6 用例的配方造池（建 N 个文件、整份写、中途建快照、整份重写下标不是 4 的倍数的那几个、抬 F 四次、整理到停），之后按种子抽 40 步用户动作（建 inode、按号整份写、按号覆盖写、覆盖写 / 追加写第一个文件、截断第一个文件、unlink、建快照、删最旧的快照、抬 F、批大小 1–8 的整理 1–6 趟）。每一步打这一步结局的摘要（发成的每一版的摘要、整理那一趟的批数与结局、discard 列表；拒绝时打拒绝的 Debug 摘要）、写流累计摘要、现行那一版的摘要（单元按角色、落点、字节 FNV 哈希逐个进，别的字段照 Debug 进，所以 `units` 的次序也在里面）、中央映射树的节点数；panic 用 `catch_unwind` 接住，打文案后停。
- 前缀写死、用户动作放开扫（定义第 3 步）：前缀只固定造池那一段，后缀动作按 23 个种子各抽一遍。

为什么写流相同就够：崩溃状态由写流（屏障与 FUA 切段、段内写的子集）唯一决定，两边写流逐条相同（偏移、长度、种类、内容哈希都同），两边的崩溃状态集合就逐个相同，所以没再调 `enumerate_crash_states` 去枚举；两边要是有一行不同，才需要按定义第 3b 步枚举那一段。

装置会红（只在我的副本上量过）：把整理读版本改成只读半张映射（T4 ① 那个变异）之后，写死前缀 16x4 种子 1 那一段在第 39 步（前缀最后那次整理）起与改前不同，之后 41 行全不同（命令输出 `41`，`runs/selfcheck-mut-m2-B16s1.log` 对 `runs/d5-1-before-B16.log`，草稿目录里，装置是改覆盖写步之前那一版）。

取样与估时：
- 先跑一小段估时：release 下随机历史 4 组比重 × 1 个种子 × 40 步 4.93 秒；写死前缀 16x4 一段 9.18 秒、48x8 一段 85.95 秒（改成廉价版本摘要之后）。估全量（14 组 × 16 个种子 × 60 步，加 16x4 × 16、32x8 × 4、48x8 × 3）单份二进制约 25 分钟，两份并行分批跑，每批 3–4 件、每件经 `run-with-memory-cap.sh 2G`，前台每批不超过 228 秒；本机负载高（`uptime` 一度 load average 177.68），有三批件撞 228 秒被 `timeout` 停下，这几件只比两边都跑完的那几段（见下）。
- 缩了什么：48x8 只跑 3 个种子（原想 8 个），96x8 没跑；随机历史每段 60 步（未试更长）。

## T1　索引与线性查找是不是同一项：没打中

读码（正推的反面：找它不同的条件，没找到）：
- 建索引取同一身份最先那一项：「.or_insert(entry);」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:3205`），与 `iter().find` 同义；查是「self.first_entry_of_each_identity.get(identity).copied()」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:3215`）。
- `BTreeMap` 按 `Ord` 归并键，`Ord` 与 `==` 不一致才会把两个不同身份并成一个：`TransactionUnit` 是「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:2398`），它各成员带的十种位置 / 下标类型与 `InodeNumber` 我逐个 grep 过，都是派生的 `PartialEq, Eq, PartialOrd, Ord`，仓里没有对它们手写 `Ord` / `PartialEq` 的 impl（命令 `grep -rn "impl.*Ord for <类型>\|impl.*PartialEq for <类型>" crates/singlefs-core/src` 对十个类型都 0 行）。
- 建了索引之后改表再查：发布路径的 `previous` 在「let previous: Option<&TransactionOutput> =」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:12614`）绑一次，到函数末尾没有再绑（`awk` 列 12614–13800 行里 `previous` 的每处用法，没有 `let previous` / 模式绑定），它指的那一份（`previous_with_the_relocated_units_at_their_destinations`）是不可变绑定；索引在「let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:13630`）建，与改前 `previous.expect(...)` 指的是同一份。`carried_file` 就是这个 `previous`：「None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:12650`），所以第一个文件的数据单元改走 `units_of_the_previous_version` 不换表。`units` 是公开字段，但索引借着它，借用活着时别处改不动它，编得过就排除了「建索引之后改表」。
- 五张表里同一身份有没有出现不止一次：我没量（差分不依赖它：两种查法在重复时都取最先那一项）。

差分（反推：假设不同，本该看到某一步写流或版本摘要不同）：
- 随机历史 223 段（14 组比重 × 16 个种子少一段：第 10 组比重的种子 6 那一段在撞 228 秒被停下的那一件里没跑完，补跑那一件我从种子 7 起，漏了它，所以是 223 不是 224），全部 `ending=Completed`；改前改后每段的结局摘要、写流摘要、每一步镜像摘要与每一步结局摘要逐行相同：`out/random-histories-after.txt` 与 `out/random-histories-before.txt` sha256 相同（`371fc132…`），逐步行在草稿目录 `runs/d3-*`、`runs/d4-*`。这一批用的是改覆盖写步之前那一版装置文件，随机历史那一段代码与最终版逐字相同；用最终版又跑了一遍（`runs/d10-*`、`runs/d11-*`），两边都跑完的 32 + 27 + 7 + 10 + 8 + 6 段与两件跑完的 16 + 16 段逐行相同，改前那份与旧装置的同段也逐行相同。
- 写死前缀 23 段（16x4 种子 1–16、32x8 种子 1–4、48x8 种子 1–3）共 2120 步，0 步被拒、0 次 panic；中央映射树长到 3、4、5、6 个节点的步各 294、109、54、159 步；整理趟里发了版的 620 趟、共 1279 版。改前改后 2120 行逐行相同（`out/scripted-*.txt` sha256 同为 `40a15646…`）。

什么现象会推翻：一段改前改后某一步写流或版本摘要不同的历史；或者找到一处 `previous`（或 `carried`）在建索引之后指向了另一份 `TransactionOutput`。

## T2　整理读版本一遍走叶与逐 key 查：没打中

读码：
- 两边走叶用同一个函数同一个形状：逐 key 查是「let leaf_role = MultiLevelCodeTwoTree::CentralMapping.role_of_node(leaf, shape);」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:4247`），批量读是「let leaf_role = MultiLevelCodeTwoTree::CentralMapping.role_of_node(leaf, self.shape);」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:4330`），`self.shape` 是同一版的 `central_mapping_tree.shape`。「key 不在它路由到的那片叶里、几片叶都有同一把 key」两种走法都只看路由到的那片叶，答案一样。
- 叶的字节：逐 key 查「mapping_locations_for_key(&version.unit(leaf_role).bytes, key)」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:4248`），批量读「CentralMappingLeafRead::of_the_leaf_bytes(&units_by_role.unit(leaf_role).bytes)」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:4335`），两边都是 `units` 里这个角色最先那一项、都在第一次用到时才取（取不到同一文案 panic）。缓存按角色键，同一版里同一角色的字节不变。
- 三种结局：逐 key 查遇到切不动的条目就「return MappingLookup::EntryNarrowerThanItsFieldTable {」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:4707`），与这把 key 排在它前后无关；批量读同样整片叶报它（「return Self::EntryNarrowerThanItsFieldTable {」，`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:4281`），字节数都是最先那条切不动的条目的。同一把 key 几条：逐 key 查「if candidate == key {」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:4711`）逐条往后覆盖，批量读「locations_of_each_key.insert(key, locations);」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:4285`）后插覆盖，都是最后一条。key 长不是 27 时两边都按整串相等比，都查不到。
- 整理那一处换的只有查法：「.filter_map(|(_, key)| match central_mapping.locations_of_key(key) {」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/mounted_session.rs:2790`），收集的次序仍是 `mapped_units` 的次序，每把 key 的结局逐个相同则 `mapping_entries` 逐项相同。

对拍（`attack_leaf_read.rs`，只放进改后树）：48 个文件 × 8 个数据单元的池，现行那一版中央映射树 4 个节点、3 片叶（输出「M4T2 mapping_tree_nodes=4 leaves=3」）。在这一版上造 16 种版本：原样；三片叶两两互换字节（3 种，key 落在不是它路由到的那片叶里）、一片叶抄进另一片（3 种，几片叶有同一把 key）；每片叶在 `units` 末尾多一项全零的同角色单元、在开头多一项（6 种，重复角色、最先那一项是坏叶）；删掉一片叶的单元（3 种，走到它的 key 两边都 panic）。key 取每一版的全部进映射 key、三片叶里每一条条目的 key、全零、全 FF、短一字节、长一字节、空串，每版 897 把；同一个批量读者跨全部 key 复用，每次调用各包 `catch_unwind`，panic 比文案。命令输出（`out/t2-leaf-read.txt` 末行）：

```
M4T2 total keys_compared=14352 both_panicked=942 found=9808
```

`test result: ok. 1 passed`。原样那一版有 3 把 key 两边都 panic：长度不是 27 的那三把，两边同在「"key 的字节数等于这棵树的 key 宽：调用方从条目里切前 key 宽 个字节（D8 已定项 11）"」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/code_two_tree.rs:56`）那条 `assert_eq!` 上停（日志里 48 + 48 次）；删叶的三版各 299–304 把，两边文案同为「八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在」，panic 报的位置不同（逐 key 查报 `transaction.rs:3717:14`、批量读报 `transaction.rs:3227:14`，各 894 次），见 T5。

什么现象会推翻：一版、一把 key 上两种查法结局不同，或者同一个批量读者先查过别的 key 之后再查这一把结局变了（这一版对拍里同一个读者一直复用，没见到）。

## T3　照抄路径：没打中

- `carried_unit` 查不到：改前 `previous.unit(identity)` 的 panic 文案与改后共用一个常量「"八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在";」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:3184`），两条查法都「.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:3227`）。没有上一版时的 `expect` 文案，补丁逐处把 `previous.expect("…")` 换成 `units_of_the_previous_version.as_ref().expect("…")`，引号里的字逐字照搬（diff 第 324–383 行）；第一个文件那一处新加的 `expect` 在 `carried_file` 是 `Some` 时才走到，`previous` 必是 `Some`，走不到。
- `mapping_key_carried_from`：照抄全部角色那一圈改走 `mapping_key_of`，查不到的文案两处同为「.expect("照抄进来的角色在上一版里进映射、有一把 key")」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:3240` 与 `:7007` 两行，grep -nF 命中 2 处）；没有上一版时先 panic 的仍是「照抄的角色只出现在接着上一版的发布里」，次序同改前。快照判定那一圈仍调线性版本：「let mapping_key = mapping_key_carried_from(previous_version, *identity);」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:5652`）。
- 每轮现建还是跨轮复用：`settle_the_allocation_record_tree` 每轮调一次「let resolved = plan.resolve(」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:11067`），索引在 `resolve` 里面「previous.map(TransactionOutput::mapped_units_by_role);」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:7889`）每调一次现建一次，建在不可变的上一版上；没有跨轮复用，也就没有中间那一轮改表的情形。
- `file_other_than_the_first`：改前 `previous.and_then(…file_other_than_the_first…)`、改后 `files_of_the_previous_version.as_ref().and_then(…first_entry_of…)`，后面同一个「.expect("照抄的文件在上一版里")」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:12770`）；索引建在同一个 `previous` 上（「previous.map(TransactionOutput::files_other_than_the_first_by_inode);」，`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-core/src/transaction.rs:12750`）。
- 差分：23 段写死前缀历史与 223 段随机历史里，改前改后都没有一次 panic；三种照抄（第一个文件的数据单元、照抄的文件、照抄的树节点）在前缀的整份重写、截断、unlink、删快照之后的每一次发布里都走到（前缀里 924 次整份写、后缀 145 次按号整份写与 56 次 unlink）。

什么现象会推翻：一段改后 panic 而改前不 panic 的历史，或两边 panic 文案不同。

## T4　测试与变异判得出它要判的：打中两处

### 等价测试比的是什么

用例里比的是新索引 API 与测试里自己写的线性参照：「fn is_the_same_entry<Entry>(indexed: Option<&Entry>, found_linearly: Option<&Entry>) -> bool {」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-harness/tests/publish_lookup_by_identity.rs:336`）按地址比，中央映射那一格的参照是保留下来的逐 key 函数 `central_mapping_locations_in_the_version`，这两样参照是真的旧查法，不是「两边都走新索引」。但发布路径与整理里的调用点（照抄时用哪张索引、查哪个身份、整理读版本那一圈怎么收）不在比对里：它们只经三样被判——池级 checker（「let violations = violations_of_the_pool_checker(&memory_pool_of(&devices));」，同文件 `:539`）、「batches_published >= 1,」（同文件 `:561`）与产品代码自己的断言；镜像摘要只打不判（「"publish_lookup_by_identity shape={shape:?} batches_published={batches_published} images_sha256={}",」，`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-harness/tests/publish_lookup_by_identity.rs:545`），「改前改后镜像逐字节相同」那句验收只在实现员手工比过。

### 打中 ①：整理读版本只读半张映射，点名用例不红

变异（我的副本，在冻结副本的拷贝上）：`mounted_session.rs` 第 2790 行那一圈前面加 `.step_by(2)`，整理读版本时每隔一把 key 才查一次：

```
         .mapped_units
         .iter()
+        .step_by(2)
         .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {
```

`publish_lookup_by_identity` 的 4 条非慢档用例（`--test-threads 4 --nocapture`，经 `run-with-memory-cap.sh 3G`）原样输出：

```
== b1-1-after.log        （未变异的改后副本）
publish_lookup_by_identity shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=87addacdc948bbbc00b1e0f2894f5ffe45f1e93a24f7645bd4092edcb3f2bf75
publish_lookup_by_identity shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=3344bce43504554dd5e0ca94c286b1a3b70f9ac362a374898760405876761b19
test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 25.62s
== b2-2-mut-m2.log       （.step_by(2)）
publish_lookup_by_identity shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=13 images_sha256=763f4477ce2f2d78947048a8f8d37327601b302c81bd8eb0fdaa6596427d89e4
publish_lookup_by_identity shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=39 images_sha256=f065921a3fc7507c1d89d4fba41acc5fe7ef5f6c9a302f48b4244a86e9998a5c
test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 25.13s
```

同一处换成 `.skip(1)`（漏查第一把 key）同样 4 条全过，16x4 那一档镜像变了（`batches_published=23 images_sha256=9c63e79f…`）；换成 `.take(len - 1)`、`.rev()` 镜像不变（两档摘要与未变异相同），这两个在这段历史上等价，不算。全部原样在 `out/mutants.txt`。

点名用例之外：同一个 `.step_by(2)` 副本上我又跑了调用整理的 11 个 harness 测试目标里的 6 个，3 个红（`compaction_intent_check_fallbacks_reached_by_transient_read_errors` 2 条、`compaction_intent_is_logged_and_resumed_after_a_crash` 3 条、`compaction_intent_issue_writes_the_empty_segments_counted_before_the_issue` 1 条），这 3 个目标在未变异的改后副本上全过（2、5、3 条）。所以整个 harness 档大概率判得出它，判不出的是七·6 自己的用例与这一行变异点名的那个目标。另 5 个目标没跑。

打中之后的四句：
1. 分不分辨臂：这一格问的是「七·6 的用例与变异判不判得出查法坏了」，它分得出「判得出」与「判不出」两种情形；它不说明补丁本身错（补丁在我的差分里与改前逐行相同）。
2. 被判的系统当时看不看得到：看得到。用例已经算出了镜像摘要和发了几批，只是没拿它与改前比；改前二进制编不进七·6 用例（用到新 API），要比就得把造池那串操作抽成两边都能编的样子，或把改前那一份摘要钉成常量。
3. 满足的是哪一句：正文 T4「T4 造一个改坏查法而测试不红的变异」与「等价测试比的是不是改前改后真不同的路径」后半——比对只覆盖索引 API，整理读版本那个调用点没有参照。
4. 跑前条款给的改法：正文 T4 没写改法。下面「自己提的改法」各修哪一格见表。

### 打中 ②：增长行「整理读版本退回逐 key 查映射」在 400% 上不红

变异是 `crates/mutations.tsv` 第 4975 行（冻结副本）原样那一个替换：`mounted_session.rs` 那一圈改回逐 key 调 `crate::transaction::central_mapping_locations_in_the_version(version, key)`（编得过，两条 warning：`central_mapping` 没用到、不必 `mut`）。慢档用例 `a_compaction_batch_cpu_grows_less_than_the_threshold_from_512_to_2048_data_units`（debug，`--include-ignored --exact`，经 `run-with-memory-cap.sh 3G`，与别的两件并行）三次与未变异两次、另一行增长变异（第 4974 行）一次：

```
== b5-1-after.log
publish_lookup_by_identity growth smaller_median=40000412 larger_median=118999924 growth_in_percent=297
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 102.36s
== b5-2-grow-r8.log
publish_lookup_by_identity growth smaller_median=52009953 larger_median=331990329 growth_in_percent=638
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 181.30s
== b5-3-grow-r9.log
publish_lookup_by_identity growth smaller_median=117000100 larger_median=414930741 growth_in_percent=354
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 105.08s
== b6-1-after.log
publish_lookup_by_identity growth smaller_median=38000697 larger_median=112990929 growth_in_percent=297
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 101.47s
== b6-2-grow-r9.log
publish_lookup_by_identity growth smaller_median=112976439 larger_median=404951286 growth_in_percent=358
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 105.13s
== b6-3-grow-r9.log
publish_lookup_by_identity growth smaller_median=113000539 larger_median=409992548 growth_in_percent=362
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 103.78s
```

（`out/mutants.txt` 里这几份日志的原样摘行；`==` 行是日志名：after = 未变异的改后副本，grow-r8 / grow-r9 = 第 4974 / 4975 行那两个增长变异。）门槛是「const HIGHEST_GROWTH_OF_THE_BATCH_CPU_IN_PERCENT_FOR_FOUR_TIMES_THE_DATA_UNITS: u64 = 400;」（`/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/crates/singlefs-harness/tests/publish_lookup_by_identity.rs:808`）。第九行变异让小池一批的 CPU 从约 38–40 ms 涨到约 113–117 ms（三倍），大池从约 113–119 ms 涨到约 405–415 ms，倍数只从 297% 涨到 354–362%：逐 key 查每把 key 整片重解叶（与 P 无关的大常数）占了大头，平方项在 512→2048 上还没盖过它。第八行（索引退回在 `BTreeMap` 上逐项找）638%，红。

打中之后的四句：
1. 分不分辨臂：分得出这一行变异「红 / 不红」；两行增长变异里第八行红、第九行不红，所以「两行增长变异证红」这一格在第九行上不成立。
2. 被判的系统当时看不看得到：用例只看大池与小池中位的比，看不到绝对值；第九行的代价主要是 P 的一次项乘一个大常数，比值几乎不动。它看得到的东西（两档的绝对 CPU）里其实有分别（小池三倍），只是判据没用它。
3. 满足的是哪一句：正文 T4「两行「增长」变异在门槛 400% 上分不分得出（实现员估「整理读版本退回逐 key 查映射」那一行可能不红）」——分不出，与实现员的估计一致。
4. 跑前条款给的改法：正文没写。注意量的条件：debug、本机重负载（同批另有两件并行，`uptime` 那一刻 load average 177.68），倍数是线程 CPU 的比，负载对它有影响；三次都在 354–362%，与门槛差 38 个百分点以上。

### 七行「按身份查」变异各自红在哪（只跑了第 3、4、5 行）

在冻结副本的拷贝上逐行打上，跑 `every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units`（debug，`--exact`）：

```
== row-copy grow-r8 rc=101          （第 4969 行：单元索引漏了 units 最前面那一个）
thread '…sixteen_files_of_four_data_units' (1245462) panicked at crates/singlefs-core/src/transaction.rs:3227:14:
八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在
== row-copy mut-m1 rc=101           （第 4970 行：照抄角色的映射 key 一律取 inode 根那一把）
thread '…sixteen_files_of_four_data_units' (1245584) panicked at crates/singlefs-core/src/transaction.rs:8207:9:
assertion `left == right` failed: 进映射的单元与留存的映射条目各一把 key，互不相撞（…）
== row-copy mut-m2 rc=101           （第 4971 行：照抄的文件按下一个号查）
thread '…sixteen_files_of_four_data_units' (1245657) panicked at crates/singlefs-core/src/transaction.rs:12770:26:
照抄的文件在上一版里
```

（线程名与断言文案里的「…」是我为行宽删的，整行在 `out/mutants.txt`。副本名是我复用的草稿目录名，括号里是它这一次装的变异。）三行都红在产品代码自己的 `expect` / 断言上，不是红在用例的线性参照比对上：第 4969 行在第一次照抄时 panic，第 4970 行撞上发布路径「key 互不相撞」的断言，第 4971 行在最后一个文件找不到「下一个号」时 panic。也就是说这三行的判别力来自产品断言；把第 4971 行换成「查到另一个存在的文件」、第 4970 行换成「取另一个也存在的角色而 key 不撞」这类变体，能不能红取决于池级 checker，我没造。第 1、2、6、7 行没跑（读码：第 1 行在第一次照抄时同样 panic；第 2、6 行只靠合成小表 / 合成重复 key 的那两条用例；第 7 行靠 48x8 那一档的多叶比对）。

### 自己提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修打中 ① | 修打中 ② | 依据 |
|---|---|---|---|
| 甲：七·6 两档等价用例把 `images_sha256` 钉成常量（数取自改前二进制跑同一串操作；造池那串操作要抽成改前也编得过的样子） | 修（推的：`.step_by(2)` 与 `.skip(1)` 都改了镜像摘要，量过的是「摘要变了」，没量钉常量之后的用例） | 不修（推的：第九行不改镜像） | `out/mutants.txt` |
| 乙：在用例里对整理读版本的结果另比一遍逐 key 参照（要把 `version_as_the_compaction_reads_it` 开成测试可调，或在 `cfg(test)` 下产品里自比） | 修（推的） | 不修（推的） | 读码 |
| 丙：增长判据改数次数而不量 CPU（每批整理解了几片叶、按身份查了几次，平方路径的次数随 P 平方涨） | 不修 | 修（推的：第九行每把 key 整片重解叶，解叶次数从「叶数」涨到「key 数」，次数比不受常数影响） | 读码 |
| 丁：增长门槛降到约 330% | 不修 | 也许（推的：未变异 297%、第九行 354–362%，余量只有二三十个百分点，debug 加重负载下没量过稳不稳） | `out/mutants.txt` 五次读数 |
| 戊：增长两档拉开（512 → 8192） | 不修 | 也许（推的：平方项在更大的 P 上才盖过常数，没量） | 无 |

## T5　有没有改到「行为不变」之外：没打中

- 补丁原文（`crates.patch`）里产品代码的改动只有：新加 `EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION` 常量与 `FirstEntryOfEachIdentity`、`CentralMappingOfAVersionReadLeafByLeaf` 两个公开类型和 `units_by_role`、`mapped_units_by_role`、`files_other_than_the_first_by_inode` 三个公开方法；`TransactionOutput::unit` 的 `expect` 改成引那个常量（文案逐字相同）；私有函数 `carried_unit` 的第一个参数从 `&TransactionOutput` 换成索引；五处调用点换查法；`mounted_session.rs` 的 `use` 换了一项；文档注释。没有删掉或改签名的公开项，没有碰盘上格式的代码（`records.rs`、`unit.rs` 不在补丁里）。
- 唯一看得见的副作用是 panic 报的源码位置：同一个「八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在」，照抄路径与整理批量读现在报 `transaction.rs:3227`（`FirstEntryOfEachIdentity::unit`），改前报 `TransactionOutput::unit` 那一行（冻结副本 `:3717`）。T2 对拍里删叶那三版两边各 894 次就是这个样子。文案相同，结局（panic）相同；要是有哪处测试或日志判据按 panic 位置匹配，会看到不同——我没找这样的判据。
- 写出的次序：差分里写流逐条（偏移、长度、种类、内容哈希、屏障位置）相同，现行那一版的摘要（含 `units` 的次序）逐步相同，见 T1。

什么现象会推翻：补丁里还有一处我没列的产品行为改动（逐行看模型目录 `crates.patch` 第 1–438 行可核，第 439 行起是新用例），或者差分里一行不同。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| T1、T3、T5 | 随机历史：`GenerationWeights::NAMED` 14 组比重（含快照、整理、截断、按号写、回滚、抬 F、掉电重挂、冷启动） | 种子 0–15、每段 60 步，223 段（缺第 10 组种子 6），全部 Completed；操作次数前六件里 PublishOverwrite 3062、CloseAndMountWritable 1102、CreateInode 990、RollBackWhileMounted 797、RaiseRollbackFloor 475、Truncate 159、CreateInodes 155、Unlink 131、CompactWhileMounted 120、掉电重挂 70、WriteOfAnInode 51、CreateSnapshot 21、DestroySnapshot 8 |
| T1、T3、T5 | 写死前缀（七·6 配方加一个快照）+ 按种子放开的 40 步用户动作 | 16x4 种子 1–16、32x8 种子 1–4、48x8 种子 1–3，共 23 段 2120 步；映射树最多 6 个节点；后缀里整理 129 步、按号整份写 145、按号覆盖写 118、unlink 56、建快照 62、删快照 71、截断 63、抬 F 74 |
| T2 | 多叶版本上换叶：互换、互抄、同角色重复项放前放后、删叶；key 含不在路由叶里的、几片叶都有的、长度不对的、空的 | 1 个池（48x8）、16 种版本、14352 次比对 |
| T4 | 整理读版本那一圈的 `.take(len-1)`、`.rev()`（镜像不变，等价）、`.skip(1)`、`.step_by(2)`（镜像变、用例不红） | 七·6 用例两档 |

## 这条腿自己的限度

- 「没打中」只抽了一次样（这一条腿一次），照规则不能单独支撑「没打中」。
- 差分只比到写流与内存里那一版的摘要；读路径（挂载、恢复读回）不在补丁里，没另比。
- 崩溃状态没有逐个枚举：两边写流逐条相同，崩溃状态集合由写流决定，所以没调 `enumerate_crash_states`；这一步是推的，不是枚举出来的。
- 48x8 只 3 个种子、没有 96x8 以上的形状；随机历史没有一段走到中央映射树多于一片叶（4 GiB 盘、内容短），多叶只在写死前缀那 7 段里出现。
- 增长读数是 debug、重负载下的线程 CPU，五次读数，同批有并行件。
- 五张表里同一身份是否出现不止一次，我没量。
- 整条 `tools/rerun.sh` 没跑过；它的步骤与我逐批跑的一致，但路径与样本量是重写的。
- 副本上量出的数（镜像摘要、增长百分比、红不红）都是副本上的数，不是入库装置上的数。

## 没做什么

- 不判正推那几格；不替主 agent 采纳改法。
- 没跑 `.step_by(2)` 副本上调用整理的另 5 个 harness 目标、没跑整个 harness 档。
- 没跑七行「按身份查」里的第 1、2、6、7 行。
- 没跑任何门禁阶段、checker 档测试、崩溃轻型、变异轻型，没起 GPU。

## 要主 agent 跑的命令

- 复跑本报告的小样本（经内存包装，工作目录随意，约 10 分钟，要放后台）：
  `bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-publish-lookup-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo --work <新的草稿目录>`；看 `name=differential` 两行是不是 `identical`、`name=half_mapping_mutant` 是不是 `4 tests_ok` 且两档摘要与未变异不同、`name=growth_row_nine` 的 `growth_in_percent` 是不是在 400 以下且 `test result: ok`。

## 草稿与清理

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1-opus`：逐批日志在 `runs/`（交回前保留，供核对）；仓副本 `after/`、`before/`、`mut-m1/`、`mut-m2/`、`grow-r8/`、`grow-r9/` 与编译目录 `target-*` 已删（草稿目录现 13M，只剩日志、`dev/` 里的装置源文件与两个辅助脚本）。
- 我在开工时误把两个只含 `scripts/` 拷贝的目录建在了仓根（`after/`、`before/`，相对路径写错），当场删掉，`git status --porcelain -- after before` 0 行。
