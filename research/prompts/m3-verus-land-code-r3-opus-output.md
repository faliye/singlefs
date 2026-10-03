# Verus 核接回 `crates/`：代码轮第三轮攻方（Opus）报告（2026-10-01）

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 H6 J1 J2 J3 J4 J5 S03 S04 S05 S06 S07 K20 A01 A02 A03 A04 A05 A06 A07 H00 H01 H02 H03 H04 H05 H06 H07 H08 H09 H10 H11 -->

攻击面：正文第四节分工表「云端攻方（Opus）」那一行，J1–J4，重点 J1、J2、J3。被判对象只读冻结副本 `/tmp/claude-1000/m3-verus-land-code-r3/tree/`（对 `research/prompts/m3-verus-land-code-r3-snapshot/crates-sha256.txt` 跑 `sha256sum -c`，不是 OK 的行 0 行）。下文引代码一律写冻结副本的绝对路径。副本上量出的数都是副本上的数。

## 复跑

模型目录 `research/prompts/m3-verus-land-code-r3-opus-model/`。第二轮、第三轮冻结副本与 fcc4d340 的 `recovery.rs` 原样放在 `inputs/`（`*.read`，sha256 与快照清单、`git show fcc4d340:…` 逐字相同）；冻结副本不在了，就把 `inputs/` 里两份核放回 `<根>/crates/singlefs-replay-prefix/src/lib.rs` 再跑。

```bash
# 工作树：第三轮冻结副本拷一份（不带 target）
rsync -a --exclude target --exclude .git /tmp/claude-1000/m3-verus-land-code-r3/tree/ <草稿>/tree/
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="<理由>" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1800
# 一、核的差分（old＝第二轮核，new＝第三轮核，外加 K20 两种写法与 H5-2 退回）：四个族，内存 512M、线程 10
bash research/prompts/m3-verus-land-code-r3-opus-model/kernel-diff/rerun.sh /tmp/claude-1000/m3-verus-land-code-r2/tree /tmp/claude-1000/m3-verus-land-code-r3/tree <草稿>/tree <出口>
# 二、Verus：攻方 7 行 rlimit 10；verus-mutations.tsv 全表 rlimit 10 / 20 / 40（每次 Verus 经内存包装 1G，--num-threads 2，四行一批）
cp -a /tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix <草稿>/verus-clean/
bash research/prompts/m3-verus-land-code-r3-opus-model/verus-batch.sh <草稿>/verus-clean/singlefs-replay-prefix <出口>
# 三、harness（--release，每条 cargo test 经内存包装、capped.sh 10）
CARGO_PROFILE_FLAG=--release HARNESS_MEMORY_CAP=3500M bash research/prompts/m3-verus-land-code-r3-opus-model/harness-rows.sh <草稿>/tree /tmp/claude-1000/m3-verus-land-code-r3/tree research/prompts/m3-verus-land-code-r3-opus-model/harness-attack-rows.tsv <出口>/rows 2 3 4 5 6 7 8 9 10
bash research/prompts/m3-verus-land-code-r3-opus-model/harness-batch-2.sh <草稿>/tree /tmp/claude-1000/m3-verus-land-code-r3/tree <出口>
```

挂钟（副本上量）：差分穷举族 27 秒，三个抽样族各 0 秒（程序自报整秒）；Verus 一次 7 秒左右（未改的核，rlimit 10）；harness 单个目标一行 16–56 秒，整包一行 206–335 秒。

模型目录每个文件的 sha256（`find … -type f | sort | xargs sha256sum`，32 份）：

```text
3149e42531bf7a44ad37b86781f548ed040b4993b4e409afa774c8c92d92b842  research/prompts/m3-verus-land-code-r3-opus-model/apply-row.py
7c7f1243057d2f51a4f724773759d49476de2cf19078ccbe4cd8fa77cd635465  research/prompts/m3-verus-land-code-r3-opus-model/harness-attack-rows-2.tsv
64d6543b872fc0556835d421ab63f9ab532cf6ea65ad43ca1225fbf92ffb6bb8  research/prompts/m3-verus-land-code-r3-opus-model/harness-attack-rows-3.tsv
a2113b466f1f140538b5931cfc0635b54f354aa123e142cc5e23ac5fcc5e4bb3  research/prompts/m3-verus-land-code-r3-opus-model/harness-attack-rows.tsv
c977807769ef6af143a092ff008869123961b9fb5d6a0617a1363b47dd9605f3  research/prompts/m3-verus-land-code-r3-opus-model/harness-batch-2.sh
2bc51e33ed194ab19a0009f0298a0f3fa2f151ef42dc015e1f46859237b4f209  research/prompts/m3-verus-land-code-r3-opus-model/harness-rows.sh
f04814901d9521a1f72bf64aae530157c643fe47f3b0a37ac06695feb9418fdb  research/prompts/m3-verus-land-code-r3-opus-model/inputs/fcc4d340-recovery.rs.read
2510f4648f1dd7da8367cecdc59fa395126c6f3d4955d42bcf1221434a038791  research/prompts/m3-verus-land-code-r3-opus-model/inputs/r2-frozen-singlefs-replay-prefix-lib.rs.read
5f29497341979dba2aad7befe067f117d9b207392da64d7e76651d3a63514190  research/prompts/m3-verus-land-code-r3-opus-model/inputs/r3-frozen-singlefs-replay-prefix-lib.rs.read
f699980e7cdd0d917c771aad8355ba249f16722b90fd6edb72a48912b2977a5f  research/prompts/m3-verus-land-code-r3-opus-model/kernel-diff/Cargo.toml
17c1d63bab3cf6b57b1e529fadd464932a78b66c0e3fac75713e6ed66e87a3ae  research/prompts/m3-verus-land-code-r3-opus-model/kernel-diff/make-kernels.py
b88625953f924dd9c9cf6cec419e5524425bb899fe32d018e6a17617095cbb53  research/prompts/m3-verus-land-code-r3-opus-model/kernel-diff/rerun.sh
a39d238392ad20435c57d5127642d93524dd7a0df11cda2b525d12beacd3fe02  research/prompts/m3-verus-land-code-r3-opus-model/kernel-diff/src/main.rs
ff4c97157236e054582487cee3097bff93396234bda1ea26c4995dc4ea0b9074  research/prompts/m3-verus-land-code-r3-opus-model/results/harness-attack-rows-2.tsv
11fab29222c20ef00faf5eb3e172c21a7e6ab6f4cc0b15772b3603bb44abbb26  research/prompts/m3-verus-land-code-r3-opus-model/results/harness-attack-rows-all-harness.tsv
d548c02886853c00c7767bf23b29460e69e6fe6ebf3088beb2ce252a737a8eed  research/prompts/m3-verus-land-code-r3-opus-model/results/harness-attack-rows-k20.tsv
52a6941d9d4480f3635d6d2b799aacbbbc39d1f65204bae5424c31e2cb3cdf90  research/prompts/m3-verus-land-code-r3-opus-model/results/harness-attack-rows.tsv
b585b24aae0ca52294596a3419e537f9f2cc1d97e0ce625a80aee421232f81f7  research/prompts/m3-verus-land-code-r3-opus-model/results/harness-control-row-litmus-failures.txt
de71f925371857fd7194555dcd24e34886351b262a5a7be5df24cb4369daf243  research/prompts/m3-verus-land-code-r3-opus-model/results/kernel-diff-chain.txt
a7b4e406536f6545d4673737f08066aacdb9704b228f78c888778492b4a2097d  research/prompts/m3-verus-land-code-r3-opus-model/results/kernel-diff-drops.txt
661859bd2ebcb25e0d4475e80db2a359f8a29603b6a14df73da1196bc29f4915  research/prompts/m3-verus-land-code-r3-opus-model/results/kernel-diff-exhaustive.txt
633865749330f7789bd058b06727b2a326094feebc220baa555902b643d1e905  research/prompts/m3-verus-land-code-r3-opus-model/results/kernel-diff-random.txt
b133eb319955ee0ddb5a72dbb8426d70aea3259896d619a00a50ed19da49c680  research/prompts/m3-verus-land-code-r3-opus-model/results/verus-attack-A03-rlimit20-40.tsv
cd60864c53526f3daa285820eaf88ed942fe2794f3736a861efb59faf6267d41  research/prompts/m3-verus-land-code-r3-opus-model/results/verus-attack-rlimit10.tsv
e4c58a005d29c387bc18a1b8d28a2b8e67fb29fdf8456d63a5b5d5361ae2976f  research/prompts/m3-verus-land-code-r3-opus-model/results/verus-fragile-rows-errors.txt
9e542f9c70c947127ea668c9db8a055b3b1a5cf5263c37774d0e80765f3c1137  research/prompts/m3-verus-land-code-r3-opus-model/results/verus-table-rlimit10.tsv
8c75fb65b0e241a955ddde42b0b208bb13db9ad3138bb6edf7c7c22dd5e60d65  research/prompts/m3-verus-land-code-r3-opus-model/results/verus-table-rlimit20.tsv
1668bdb1e09d49dffc920dfeee7fac82b83414193ba6b4e421d2fca569fe571a  research/prompts/m3-verus-land-code-r3-opus-model/results/verus-table-rlimit40.tsv
7ff7024b1f6d058a9c00781a8f6442878e2fbc267d6d9bec7c19da4a3a78ba10  research/prompts/m3-verus-land-code-r3-opus-model/verus-attack-rows.tsv
831ed3b601eb104516d9d4886b0291eeae8a869f1614ccd45aaf9b5efddf6b45  research/prompts/m3-verus-land-code-r3-opus-model/verus-batch.sh
aa821ece9f0f9ea50708929bc110be3b8f722f2d7cb9c5b1927400e787e56af8  research/prompts/m3-verus-land-code-r3-opus-model/verus-judge-row.py
4f31c7450377912710af08686b09811667e88b3b6eb88c0fa2decb3e9f5be399  research/prompts/m3-verus-land-code-r3-opus-model/verus-rows.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| J1 纯重构 | **打中**（回调调用） | 结局在 375 815 052 个输入上零差异；回调调用在 H5-2 那一类输入上不同：锚点读不出、水位之上第一条候选 txg＝根＋1、序号不是 1 时，改前核不调反向链回调，改后核调一次（穷举族 69 121 920 / 360 815 052 个输入）。H5-2 退回之后与改前核逐输入相同。H5-1、H5-3、H5-4、H5-8 没有差异 |
| J1 附带：核里两句注释 | **打中**（注释与代码不符） | 「判的次序同原文，读盘次序跟着它」与「判定照提交 fcc4d340 … 搬过来」在 H5-2 之后不再成立 |
| J2 新用例钉不钉得住读盘次数与次序 | **打中** | S07（末条提交标记那一判挪到反向链回调之前）套到今天的核上 Verus 照样验过、新用例照样绿、整个 harness 包里红的与对照行一样（只有两条和核无关的 litmus 用例）；另有 S04 和攻方加的 H06–H09 四行同样漏过。核头注释第 11 行说读盘次数与次序「由 harness 用例 … 钉住」，用例只罩住「两次单条发布、都过、锚点读得出」这一种形状 |
| J3 变异表改动 | **打中**（rlimit） | `verus-mutations.tsv` 第 36、45、48、87 行的期望片段把 rlimit 用尽那一个错误也数进去了：rlimit 改成 20 或 40，这四行仍然红，但都判成「输出里没有期望的片段」；rlimit 10 上 93 行全部对得上。K20 那一行改锚之后变成更宽的一种错（新写法单独打中 843 552 个输入，旧写法单独打中 0 个），点名的用例对两种写法都红。「步 3 三方第一轮…」「P6 后一半…」两行没变 |
| J3 删掉「下一号改成加二」 | 照判据没打中 | 删掉的是一行改注解的变异，不是改代码的；「跳过紧挨着的下一条」这一类改代码的错，改前、改后都没有必红世界行罩着。攻方在 Verus 上套这类改法（A01、A02），两行都红 |
| J4 编码纪律 | **打中** 6 处 | 同一个量两个名字，核里注释自己写了；胶水照旧手抄枚举映射；核里两句注释与代码不符；新用例的期望值写成字面数；新用例与在飞上限用例之间手抄了三个常量和两个辅助函数；`writable_mount_takes_a_second_instance.rs` 里 5-1 改过的函数还留着 `+ 60` |

## J1　5-1 的改法是不是纯重构

### 装置

`kernel-diff/src/main.rs` 用 `#[path]` 把五份核编进同一个二进制：第二轮冻结副本的核（old）、第三轮冻结副本的核（new），第 1502 行变异的改前写法（old_k20）与改后写法（new_k20），以及 H5-2 退回（new_revert_h52）。生成脚本是 `kernel-diff/make-kernels.py`，每处原文必须恰好命中一次。同一个输入喂给五份核，比两样东西：结局（报告五个字段、交回的根、两种报错及所带的计数器、panic），以及两个回调的调用日志（哪个回调、对哪个下标、按什么次序）。输入都满足核的 requires（按 (实例代号, 计数器) 严格升序；计数器 ≤ 12）。分四个族：

- exhaustive：根 txg 1，最多 3 条记录。计数器取 {1..5} 的升序子集；txg 取 {1,2,3}，序号取 {1,2,3}，末条标志、提交标记两种，事务号取 {1,2}。回调这边：反向链断在哪些下标取全部子集，点名单元三种，验或不验，在飞上限取 {1, 16}。
- random：最多 8 条，实例取 {0,1,2}，根 txg 取 {0,1,2,u64::MAX}，各取 500 万。
- chain：照合法写者写出若干次发布，再按概率丢记录、改字段。
- drops：同 chain，但只丢记录、不改字段。合法写者的记录只因丢失而缺，就是这个形状。

### 原样输出（`results/kernel-diff-*.txt`，只抄判定行）

| 行 | exhaustive | random | chain | drops |
|---|---|---|---|---|
| total | 360815052 | 5000000 | 5000000 | 5000000 |
| new_vs_old_outcome_diff | 0 | 0 | 0 | 0 |
| new_vs_old_calls_diff | 69121920 | 487254 | 152831 | 64043 |
| new_vs_old_calls_diff_in_h52_shape | 69121920 | 487254 | 152831 | 64043 |
| new_vs_old_calls_diff_outside_h52_shape | 0 | 0 | 0 | 0 |
| h52_shape_without_calls_diff | 0 | 0 | 0 | 0 |
| revert_h52_vs_old_outcome_or_calls_diff | 0 | 0 | 0 | 0 |
| any_panic | 0 | 0 | 0 | 0 |
| h52_shape_with_previous_counter_present | 7999488 | 39282 | 44986 | 0 |

一条样本（random 族 EXAMPLE 行原样）：`Input { root_instance: 1, root_txg: 2, records: [Rec { instance: 1, counter: 11, txg: 3, tx: 1, commit: false, ordinal: 2, place: Last }], … } => old Observed { outcome: Ok { above: 1, applied: 0, passed: 0, failed: 0, max_tx: 0, root: 18446744073709551615 }, calls: [] } new Observed { … calls: [('B', 0)] }`

### 哪一句许可了这个差异

- 改后核锚点读不出那一支只判 txg：「if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0 {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1419`）。接着先调回调「if back_chain_broken(index) {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1425`），然后才判序号「if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1428`）。
- 原文（fcc4d340）在调反向链之前就把序号判掉了：「} else if record.checkpoint_txg != chain_start_txg_without_anchor」（`research/prompts/m3-verus-land-code-r3-opus-model/inputs/fcc4d340-recovery.rs.read:3114`）、「|| record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST」（`research/prompts/m3-verus-land-code-r3-opus-model/inputs/fcc4d340-recovery.rs.read:3115`），之后才是「match judge_back_chain_against_the_previous_record_on_disk(」（`research/prompts/m3-verus-land-code-r3-opus-model/inputs/fcc4d340-recovery.rs.read:3121`）。
- 核里的注释仍然说次序同原文：「// 一个候选过不过逐条判定。判的次序同原文，读盘次序跟着它」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1310`）。锚点读不出那一支的注释只说结局不变：「// 那一判对链首照样判，两判之间只有反向链回调，它只会断、不改状态。」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1414`）。「不改状态」只对结局成立，对回调调用不成立。

### 产品上看得见吗

胶水里的反向链回调，只有当「同一实例、计数器小 1」的那条在记录表里时才读盘：「let Some(previous) = records.get(&(record.instance, previous_counter)) else {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:2965`）；不在表里就「return BackChainJudgement::PreviousRecordNotOnDisk;」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:2966`），一次读都不发。在表里时按「for device in reader.device_identities() {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:2971`）逐块盘去读。

`h52_shape_with_previous_counter_present` 那一行数的就是「多调的那一次真会读盘」的输入，结果如下：

- drops 族（合法写者，只丢记录）里是 0。原因是推的：合法写者写下的序号不为 1 的记录，前一条一定同属一次发布、txg 也是根＋1，它要是读得出，自己就是第一条候选，于是轮不到这一条打头。
- 改过字段的族里不是 0（chain 族 44 986，exhaustive 族 7 999 488）。这需要盘上有一条校验和对得上、字段却与合法写者不符的记录，即原文注释里说的「改过的镜像」。

所以多出的这次读盘只在被改过的镜像上出现（推的，没在 harness 上造这样的镜像）。崩溃与丢失造得出的状态上，多出来的只有一次不读盘的回调调用。

### 四句

1. 分不分辨臂：分辨。改前核与改后核只在这一类输入上不同；把 H5-2 退回去，与改前核逐输入相同（revert 那一行四个族都是 0）。
2. 系统当时看不看得到判别它的东西：看得到。锚点、txg、序号都是核手里的入参。
3. 满足判据的哪个分句：正文第五节第一条「改后核与第二轮冻结副本的核在某个输入上结局或回调调用不同 ⇒ 打中 J1」里的「回调调用不同」。结局那一半没打中。
4. 改法在打中的那几格上还中不中（正文第五节第一条只写了「写回代码」，没有点名改法，下面列攻方自己的，都是零轮）：
   - 改法甲，把 H5-2 退回：打中的格全消（revert 那一行为 0，量过）。代价是 Verus 在登记的 rlimit 10 上验不过，只剩主循环一处 rlimit 用尽：`rc=1	4	rlimit=10	verified=31	errors=1	rlimit_error_lines=1	proof_errors=0	expected=-	verdict=RED`（`results/verus-attack-rlimit10.tsv`）。rlimit 20、40 上 32 verified, 0 errors（`results/verus-attack-A03-rlimit20-40.tsv`）。要么补证明提示，要么调高登记的 rlimit。
   - 改法乙，保留 H5-2，改两句注释：第 1310 行改成写明锚点读不出、序号不为 1 时先调反向链回调；第 5 行「照 fcc4d340 搬过来」加一句例外。这样打中的格不消，只是把它写成已知的差异（推的）。

### J1 正文点名的两个子问

- H5-1：被删的那一支是「计数器是最大值就交回 false」。删掉之后靠什么罩住，攻方的看法如下（推的，由 Verus 验过）：`readable_record_of_the_same_publish_follows` 的 requires 只有下标在界内和入参升序两条：「records_ascend_by_instance_and_counter(records@),」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1066`）。严格升序下，计数器取最大值的那条之后不会再有同实例的记录，扫一遍也交回 false。所以这一支靠的是升序，不靠计数器加一不溢出那条 requires。产品上的升序由胶水断言键等于字段来保：「*key,」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:3037`），再加上 BTreeMap 本身按键有序；「record.counter < u64::MAX,」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:3043`）仍在。差分输入的计数器最大只到 12，没有在 u64 最大值上跑过，这一点只靠 Verus 和上面的推理。
- H5-2：「首条序号为 1」只剩一处判，没有锚点、锚点读不出、锚点在别的发布这几种链首都由第 1428 行那一判管住。四个族里结局差异都是 0，可以印证。没打中的只是结局，回调调用打中了，见上。

## J2　新用例证不证得到「读盘次数与次序」

### 用例罩住的形状

用例摆的场景见文件头：「B（txg 4、jsn 4）、C（txg 5、jsn 5）两次单条发布的记录与单元都在」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs:10`），挂载走的是「let mounted = mount_read_only(&logging).expect("只读挂载");」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs:226`）。它只罩住这一种形状：锚点读得出，两次单条发布，两条都过点名验证、都施加，在飞上限碰不到，没有一条候选被断。核头注释却说「//! 挂载时每个候选读几次盘、按什么次序读，由 harness 用例」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:11`），没写只罩一种形状。

### S03–S07 与攻方加的几行套到今天的核上

harness 列来自 `results/harness-attack-rows.tsv`、`results/harness-attack-rows-2.tsv`、`results/harness-attack-rows-all-harness.tsv`；Verus 列（rlimit 10）来自 `results/verus-attack-rlimit10.tsv`，S03、S05、S06 的 Verus 列是第二轮的数，这一轮没重跑。第一张表跑的是新用例加 `mounted_read_by_offset`，cargo 没带 `--no-fail-fast`。

| 行 | 改法 | Verus | 新用例 | 整个 harness 包 |
|---|---|---|---|---|
| H01＝S03 | 反向链回调每个候选调两次 | 第二轮：验过 | **RED**（`2	RED	rc=101	passed=12	…	failed=mounting_rereads_the_previous_record_once_then_each_named_unit_of_every_candidate_in_journal_order`） | 没跑 |
| H02＝S04 | 关掉点名验证时照样调点名回调 | 没跑 | 绿（`3	GREEN	rc=0	passed=13`）；`mount_read_only` 传的是验，碰不到 | 没跑 |
| H03＝S05 | 点名回调提前到逐条判定之前 | 第二轮：验过 | **RED**（第 4 行） | 没跑 |
| H04＝S06 | 主循环每条记录先调一次反向链 | 第二轮：验过 | **RED**（`2	RED	rc=101	passed=0	…	failed=mounting_rereads_…`，`harness-attack-rows-2.tsv` 只跑新用例那一行） | 没跑 |
| H05＝S07 | 末条提交标记那一判挪到反向链回调之前 | **验过**（A04 `32 verified, 0 errors`） | **绿**（`6	GREEN	rc=0	passed=13`） | 红的只有对照行那两条 litmus 用例（`passed=661`） |
| H06 | H5-2 退回 | rlimit 10 上 rlimit 用尽，20 上验过 | 绿（第 7 行） | 同对照 |
| H07 | 不带末条标志的候选点名回调调两次 | **验过**（A05） | 绿（第 8 行） | 同对照 |
| H08 | 反向链断了之后再调一次点名回调 | **验过**（A06） | 绿（第 9 行） | 没跑 |
| H09 | 到了在飞上限之后对下一个候选先调一次反向链再停 | **验过**（A07） | 绿（第 10 行） | 没跑 |

对照行 H00 只改一句注释，整包跑下来红在 `the_rotation_barrier_litmus_writer_follows_the_last_three_steps_of_every_recorded_publish`、`litmus_writer_threads_follow_the_recorded_publish_order` 两条，`passed=661`。H05、H06、H07 红的也是这两条，通过数也一样。这两条在副本上 panic 的位置见 `results/harness-control-row-litmus-failures.txt`，与核无关。没查它们为什么红，推的是副本里缺 `crates/` 之外的文件。

### 四句

1. 分不分辨：分辨。S07 改的是判定次序：候选的末条不带提交标记时，今天的核先调反向链回调再断，S07 不调就断。H07 让多条记录的发布里每条非末条多读一遍点名单元，多条记录的发布在产品上很常见，一次写超过一条记录装得下的量就是。
2. 看不看得到：用例在块层逐次记读，看得到。它没摆出这些形状，所以没看见。
3. 判据分句：正文第五节第二条「S03–S07 里有一行套到今天的核上、新用例照样绿，而核头注释说那一类由这条用例钉住 ⇒ 打中 J2」。S07 一字不差地满足：Verus 验过，用例绿，第 11 行的注释说读盘次数与次序由这条用例钉住。
4. 改法（第二轮判决第二节 H2、H4 两行给的是两条路：spec 加幽灵读盘日志，或者补用例；5-1 选了补用例）：
   - 照现在补的这条用例：在 S07、H06–H09 那几格上照样绿（量过，上表）。
   - spec 加读盘日志：推的，能罩住所有形状，没实现。
   - 最小的改法是把第 11 行收窄成「只钉住锚点读得出、两次单条发布都施加的那一种形状」，再补三格（零轮，推的）：锚点读不出；多条记录的发布；一条候选在末条提交标记或在飞上限上被断。

### `mount_read_only`、可写挂载与 `recover` 是不是同一条路

三处都调 `replay_journal`，核是同一份：`mounted_read.rs:559`、`mount.rs:4551`、`recovery.rs` 里 `recover` 那一处（「JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {」，`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:4187`）。但块层上读多少次不一样：

- 可写挂载的读阶段走缓存：「let cache = ReadStageCache::empty_over(devices);」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/mount.rs:4586`）。同一位置第二次读命中缓存：「if let Some(bytes) = self.bytes_read_by_placement.borrow().get(&placement) {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/mount.rs:4430`）。反向链要重读的那个槽，扫环时已经按同一个 (盘, 偏移, 长度) 读过，在可写挂载上不会再去盘上读（推的，没量）。
- `recover` 不验点名单元那一臂（`ConsultWithoutNamedVerification`）就是 S04 那一类会读的地方；用例走的是 `mount_read_only`，传的是验，碰不到。

所以这条用例钉的是只读挂载的块层读。可写挂载上同样的核调用读几次盘，没有用例钉。

## J3　变异表的改动

### 期望片段把 rlimit 用尽那一个错误也数进去：换 rlimit 就误判

表头写着期望片段「只取」的是 Verus 判定行里的错误数：「# 只取「verified, M errors」，不带 N」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/verus-mutations.tsv:4`）。这个 M 把 rlimit 用尽也算作一个错误。门禁那一格先把「只剩 rlimit 用尽」的情形排除掉，再拿期望片段去比原始输出与拼出来的判定行（`research/scripts/verus-proof-check.py` 的 `check_must_red_worlds`）。

全表 93 行套到冻结副本上，用门禁同样的判法（`verus-judge-row.py`）分别在三个 rlimit 上判：

| rlimit | GATE_OK | GATE_RED_fragment_missing | 输出里夹着 rlimit 用尽的行 |
|---|---|---|---|
| 10（登记值） | 93 | 0 | 13 |
| 20 | 89 | 4 | 6 |
| 40 | 89 | 4 | 5 |

误判的四行（`results/verus-table-rlimit20.tsv` 原样）：

```text
rc=1	36	rlimit=20	verified=31	errors=1	rlimit_error_lines=0	proof_errors=1	expected=verified, 2 errors	verdict=GATE_RED_fragment_missing
rc=1	45	rlimit=20	verified=30	errors=2	rlimit_error_lines=0	proof_errors=3	expected=verified, 3 errors	verdict=GATE_RED_fragment_missing
rc=1	48	rlimit=20	verified=30	errors=2	rlimit_error_lines=0	proof_errors=2	expected=verified, 3 errors	verdict=GATE_RED_fragment_missing
rc=1	87	rlimit=20	verified=31	errors=1	rlimit_error_lines=0	proof_errors=1	expected=verified, 2 errors	verdict=GATE_RED_fragment_missing
```

同样四行在 rlimit 10 上都是 `verdict=GATE_OK`，每行都有一条「Resource limit (rlimit) exceeded」（`results/verus-fragile-rows-errors.txt`，逐行列出 10 与 20 两个 rlimit 上的 error 行）。四行在 20 上照样红、证明错照样在，只是 rlimit 那一个错误没了，M 少一，期望片段对不上，门禁报「红了，但输出里没有期望的…」。四行的名字：第 36 行「ensures·引理：已施加的与攒着的接起来改成不是整串」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/verus-mutations.tsv:36`）、第 45 行收下的计数器多看一条、第 48 行 `maximum_transaction_of` 位置到不了末尾、第 87 行已施加加攒着的条数严格小于下标。其中 45、48、87 是 5-1 这一批改过期望的；36 这一批没改，在改前的表里就这样。

复现（一行）：

```bash
bash research/prompts/m3-verus-land-code-r3-opus-model/verus-rows.sh <干净副本>/singlefs-replay-prefix /tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/verus-mutations.tsv 20 <出口> 36 45 48 87
```

四句：

1. 分不分辨：只打中这四行，别的 89 行换 rlimit 照样对得上。
2. 看得到：门禁看得到 rlimit 用尽，也会单独把它排除在「只剩 rlimit」那一种情形之外，却没把它从期望里的计数剔掉。
3. 判据分句：正文第五节第三条「期望片段换 rlimit 或换版本会误判，给出复现命令就算打中 J3」。
4. 改法：
   - 登记表 `.claude/gate.d/verus-pins.tsv` 第 33 行写着「要调高这个数，先在新数上把核与必红世界表整张重验一遍再改」。照这条流程，换 rlimit 时这四行会被发现，但要逐行改期望。
   - 零轮、推的：期望片段改成只数证明错的那个数，门禁比的时候先剔掉 rlimit 那一个错误。或者这四行换一种改法，让它们在 rlimit 10 上不撞 rlimit。

换 Verus 版本会不会误判，这一轮没量（只有一个版本）。

### K20 那一行（`crates/mutations.tsv` 第 1502 行）改锚之后，是更宽的另一种错

改前那一行的替换文是 `>= next_counter` 改成 `> next_counter`，只跳过计数器恰好是下一号的那一条：「if later.instance.0 == record.instance.0 && later.counter.0 >= next_counter {」（`research/prompts/m3-verus-land-code-r3-opus-model/inputs/r2-frozen-singlefs-replay-prefix-lib.rs.read:1090`）。今天这一行锚在「let mut later_index = index + 1;」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1071`），改成 `index + 2`，跳过的是切片里下一个位置上的那条。下一号缺了（读不出）的时候，被跳过的就是更后面的那条。

差分（原样，`results/kernel-diff-exhaustive.txt`）：`old_k20_outcome_differs_from_old=1173024`、`new_k20_outcome_differs_from_new=2016576`、`new_k20_differs_while_old_k20_does_not=843552`、`old_k20_differs_while_new_k20_does_not=0`。新写法打中的输入是旧写法的严格超集，所以更好杀。一条只在「下一号缺了」那种形状上起作用的用例，能杀新写法、杀不了旧写法，表上看不出来。

点名的用例两种写法都杀（`results/harness-attack-rows-k20.tsv`：H10、H11 都是 `RED	rc=101	…	failed=last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag`）。所以今天的表没有因此漏掉错，只是这一行判的已经不是原来那一判。按正文第五节第三条第一分句（「一行变异改锚后不再是原来那一判」），这一行算打中，程度轻。

改法（零轮）：原文锚「if later.instance.0 == record.instance.0 {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1086`），替换文加 `&& later.counter.0 != record.counter.0 + 1`。这就是攻方的 H11：照原来那一判写，点名用例红（量过）；Verus 上也红，红在不变式与溢出（A02，量过）。胶水已经断言计数器小于最大值，运行时加一不会溢出。

### 「步 3 三方第一轮…」「P6 后一半…」两行

读过：「步 3」改前是 `false && txg != 起点 || 序号 != 1`，按优先级等于只判序号；改后是 `false && txg != 起点`，序号交给第 1428 行判。两种写法都是「不看 txg」，结局相同。「P6 后一半」改前把两处序号判都关掉，改后只剩一处、关掉它，都是锚点读不出时链首不判序号。两行判的都还是原来那一判，没打中。

### 删掉「下一号改成加二」之后 Verus 一侧还罩不罩得住

删掉的那一行改的是循环不变式 `next_counter == … + 1`，属于改注解的行；随着 H5-1 删掉 `next_counter`，它本来就该删。「跳过紧挨着的下一条」作为改代码的错，在改前表里的 13 行改判定中也不在。所以照判据第三条第二分句（「之前有」而现在没有）没打中。

攻方在 Verus 上直接套了两种改法（`results/verus-attack-rlimit10.tsv`）：A01（`index + 2`）`verified=30	errors=2`，A02（只跳下一号）`verified=31	errors=1`，都红。证明管得住这一类，只是表里没有一行登记它。补一行（零轮）：建议用 A02，它红在第 1077 行那条不变式上。

## J4　编码纪律（5-1 新写的代码）

判据取 `.claude/singlefs-ai-sop/rules/code-discipline.md` 两节：`.claude/singlefs-ai-sop/rules/code-discipline.md@名字：光看名字，就说得出它是什么、做什么`：「**一个语义概念全仓一个名字**」；`.claude/singlefs-ai-sop/rules/code-discipline.md@注释、断言、常量、重复`：「**魔法数字写成命名常量**：一个数只许有一处定义，名字或类型里带上单位。」以及「**重复要生成，不许手抄。**」。

1. **同一个量两个名字，核里的注释自己写了**：「// 说的是同一个量，名字分开、全仓一搜只落到一处」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:23`）。改名之后，同一个概念从「两个同名类型」变成了「两个名字」（`PrefixKernelCheckpointTxg` 与 core 的 `CheckpointTxg` 等四对）。规则的原话是一个概念一个名字，带角色的不同名字留给不同的概念，所以这一笔账算不上还清。选改名、不选一处定义，5-1 给的理由是 core 在 `journal.rs` 里给其中两个类型写了固有方法（`journal.rs:41` 的 `impl JournalRecordPlaceInPublish {`、`journal.rs:74` 的 `impl JournalRecordOrdinalWithinPublish {`）。这条理由挡得住「core 在外 crate 的类型上写固有方法」，挡不住「把这几个方法搬进核 crate、写在 `verus!` 之外，core 再 `pub use` 过来」（推的，没验 Verus 认不认 `verus!` 之外的固有方法）。
2. **胶水照旧手抄枚举映射**：「JournalRecordPlaceInPublish::LastRecordOfThePublish => {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:3061`）往下，两成员的 match 是手写的。5-1 只换了名字，手抄还在。第 1 条要是走一处定义，这段映射也就跟着没了。
3. **核里两句注释与代码不符**（与 J1 同一件事）：第 1310 行「判的次序同原文，读盘次序跟着它」，第 5 行「//! 判定照提交 fcc4d340 的」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:5`）。H5-2 之后，锚点读不出那一支的次序与原文不同。第 11 行「由 harness 用例 … 钉住」也说多了（见 J2）。
4. **新用例的期望值写成字面数**：「(12, 12, 50),」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs:274`）、「(10, 0),」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs:306`）。50 等于 2 加 2×12×2，10 等于 5 个槽乘 2 块盘。这两个数都推得出来，却直接写成字面值，单位只写在断言消息里。第一轮 C6、第二轮 H5 都把测试里的魔法数判成违例，同一类。
5. **两份测试之间手抄**：新用例第 32、37、39 行的 `CONTENT_BYTES_OF_PUBLISH_B`、`OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE`、`BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER`，与 `journal_replay_applies_no_more_records_than_the_in_flight_limit.rs` 第 39、44、47 行同名同值。辅助函数 `overwrite_in_process`（新用例第 115 行、在飞上限用例第 59 行）与 `tear_root_slot`（第 130 行、第 74 行）也是照抄的，新用例文件头自己写了「造法同」那一份。两份都已经 `mod common;`，本可以放进 `common`。同一个数因此有两处定义。
6. **5-1 改过的函数里还留着魔法数**：「FIXED_WRITE_TIME_SECONDS + 60,」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs:88`）在 `overwrite_in_process` 里（第 71 行起），这个函数正是 5-1 为 H5-7 合并过的。同一个 60，在在飞上限用例里 5-1 起名叫 `OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE`，到这里没起。同一个文件第 600、684 行的 `content_of(3300, 17)` 也还是字面数。

看过、没判违例的：`NewRootSource` 两个成员名与核里 `new_root_source` 的角色对得上（核只是原样交回，`ChosenRootItself` 交给 `chosen_root_source`）。H5-4 的映射与改前 `Option<usize>` 逐个相同（两份冻结副本的 `recovery.rs` 做 diff，只有改名与枚举）。`journal_replay_…` 新写的注释「根记录 magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 之后」，对 `root_record.rs` 第 82–87 行的写入次序：树表指针从第 36 字节起，长 86 字节（`NODE_POINTER_BYTES`），100 落在里面，注释属实。`RollbackFloorHandedToTheWriter` 两个成员与它的用法对得上。

## 没打中的形状

- J1 结局：四个族合起来 375 815 052 个输入，结局、报告五字段、报错与 panic 的差异都是 0。H5-1 去掉的计数器最大值那一支，差分没有在 u64 最大值上取样（核的 requires 不让这样的输入进来），只有 Verus 加上面的推理。
- J1 越出 requires 的输入（不升序、键与字段不等）：没造。胶水用断言挡在入口，产品走不到；第二轮已判这是契约收紧。
- J3「步 3」「P6 后一半」两行：只读、没跑。改前改后各套一次、比结局，没做。
- J3 换 Verus 版本：本机只有一个版本，没量。

## 这条腿自己的限度

- 差分输入的计数器最大到 12，txg 最大到 4，最多 8 条记录；在飞上限只取了 {0,1,2,3,16}。
- 「多出的那次回调在产品上会不会读盘」用的是一个替身判据：第一条候选「计数器小 1 的那条在不在表里」。没有在 harness 上造改过的镜像去量块层读。
- 可写挂载经缓存、不会多读那一句是推的，没量。
- 整包 harness 的对照行红在两条 litmus 用例上，原因没查（推的：副本里缺 `crates/` 之外的文件）。所以「整包里没有别的用例钉住」这句，只对能在副本上跑的那 661 条成立。
- Verus 的数都是副本上量的。门禁本身没跑（重型）；`verus-judge-row.py` 是照 `check_must_red_worlds` 抄的判法，不是门禁本身。
- 攻方提的改法（H5-2 退回或改注释、第 1502 行改用 A02 的写法、Verus 表补 A02、期望片段不数 rlimit、J2 补三格、J4 各处）都只在攻方自己的模型上量过，被攻过零轮。

## 没做什么

- 没判 J5（归辩方）；没替主 agent 采纳。
- 没跑任何重型测试，也没跑门禁 checker-tier-formal-verification。
- 内存包装：差分用 512M，Verus 1G，harness 单目标 2400M–3500M、整包 6G。没用派发给的 12G：整机 slice 排不下，第一次起时包装退 252，就按包装的提示把上限降到实测需要的量；Verus 与 harness 的上限受包装的峰值表约束（超过实测峰值两倍时包装退 251，那一批作废重跑，见 `results/harness-attack-rows.tsv` 第 11、12 行的 `WRAPPER_EXIT_251`，改用 2400M 重跑的结果在 `harness-attack-rows-k20.tsv`）。
- 跑之前看了负载：起第一次编译前 `ps` 看到别的会话的 `vm-bench.sh` 与 `qemu-system`（pid 1939071 那一串），等它退出才开始编译（`proc.py wait` 报「都已退出」）。
- 交回前删掉的草稿：仓副本 `/tmp/claude-1000/m3-verus-land-code-r3-opus/tree`（含 target，8.2G），核的干净副本 `verus-clean`（132K），锚点自查副本 `anchor-check`（136K），生成的核 `kernels-generated`（424K），Verus 基线那一份核副本。各批的日志与输出留在 `/tmp/claude-1000/m3-verus-land-code-r3-opus/{diff-out,verus-out,harness-out}`，判定行已拷进模型目录 `results/`。
