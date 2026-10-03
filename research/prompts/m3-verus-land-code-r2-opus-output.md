# m3-verus-land-code-r2 云端攻方（Opus）报告（2026-10-01）

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 H6 S01 S02 S03 S04 S05 S06 S07 S08 S09 S10 S11 S12 S13 S14 S15 D01 K02 K19 K20 G08a C6 -->

攻击面：正文第四节分工表「云端攻方（Opus）」那一行，H1–H6，重点 H2、H4、H5。被判对象读冻结副本 `/tmp/claude-1000/m3-verus-land-code-r2/tree/`（跑完核过 `crates-sha256.txt`，215 份全对），没读主仓 `crates/`。

## 复跑

```
cd /home/fy5090/code/singlefs
bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 6 \
  bash research/prompts/m3-verus-land-code-r2-opus-model/rerun.sh /tmp/claude-1000/m3-verus-land-code-r2/tree <工作目录>
```

Verus 是 `~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus`，带 `--no-cheating --rlimit 10`。harness 那一段用 `CARGO_PROFILE_FLAG=--release`。这一轮实际跑的那一次用的是 `harness-rows.sh` 第一版（判红那一句有错，见 H4-a），`rerun.sh` 调的是改好的版本。

模型目录 `research/prompts/m3-verus-land-code-r2-opus-model/` 每个文件的 sha256：

```
3149e42531bf7a44ad37b86781f548ed040b4993b4e409afa774c8c92d92b842  ./apply-row.py
355de30f53a837a8cd68ccc5dd96bed152f0aef6b6e4bb9a102f7b68a44fb9d5  ./attack-rows.tsv
c8f3b8079297c7a63d090b8dff3afdad67cf00048c5b9dabcd7d24cec0d7ea63  ./call-pattern-rows.tsv
fad385db17bc0c09d300c43268a41405dd3aad8487a8794f58085ca63fdda390  ./dead-code-row.tsv
3d55332f5bde9a1d482c7bae3eee9f687510fdfef332fab6b95c8e546951a37e  ./harness-rows.sh
ce5640634374e447f6b761872022cddcbf73b72d8610a92912d8712b3f66e50b  ./make-attack-rows.py
30d4382fd569521aa618945f4257b3da4f9e81abcead00bb2a9a3844e812f93a  ./make-dead-code-row.py
a1f6cccf8903e2f8c9cd1289a14e97d541c65b8f3c5081a0801fb69d0820bece  ./reclassify.py
bb216bcbc7e4f5cb25d8c6a53fdf04ae03e90215b1a99f9d28f67030e3929bda  ./rerun.sh
d8bbf1e9bdfb25a35ee850d7a15eb052e838b6dcf9405fa3a273e7493f69eaab  ./results/harness-call-pattern-rows-reclassified.tsv
e0d6a08c0742d1c11e538c3c0e83ccf4d6f7964008f819afd2259e7a5ca2c4e7  ./results/harness-call-pattern-rows-summary-first-classifier.tsv
9924c2392dfdecea326775381ed4e7673700746b8930a540c92f32a653e9387d  ./results/harness-call-pattern-S06-failure.txt
786edc3214d0a8c143a7a00b0584aa7b4752ce62da6f53451abb1d790a790ff8  ./results/harness-crates-rows-reclassified.tsv
058f71881d53c95bab6a142327fda288814f45c12385a8e08f754be04f56d23d  ./results/harness-crates-rows-summary-first-classifier.tsv
21661440f310076943769afab9ef1c783762a218ca278ce421fc6dd6323d1081  ./results/verus-attack-rows-summary.tsv
ecaf74f845d5f64a97defd29bab8e3d9c77310a2d29141b12a03cae29a2dc60d  ./results/verus-crates-core-rows-summary.tsv
8000263f7323ed6b0a639e37cc637970866df06e168b62fa8bf8b4d2326215c5  ./results/verus-dead-code-row-summary.tsv
38526522b0222e85def8374dca716cf658087cc432aba267fcc2002c2bced249  ./verus-rows.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| H1 | 没打中（只核了两条新断言） | 8 处调用点建表都是「键 = 字段」，计数器 48 位，产品路径上两条断言触发不了 |
| H2 | **没打中（字面判据）**；记一条口子 | 15 行只改可执行代码的变异：改判定结局的 6 行全红，照样验得过的只有 3 行等价变异和 5 行只改读盘形状的。spec 把每个结局字段都钉成了函数；没钉住的是回调调几次、按什么次序调（核头注释第 9 行声称与原文相同） |
| H3 | 没打中 | 两种核失败各有成员，`match` 没写通配臂 |
| H4 | **没打中（字面判据）**；一条欠的检查、一处连带改动 | 第 4 批 9 行新变异加两行改写变异，在副本上全红、红在点名的那一条。读盘形状变异 S03、S05 在唯一一条数挂载读盘的用例上不红（H4-c）。K20 那一行锚在 H5-1 的死子式上（H4-b） |
| H5 | **打中 8 处**（中 2、低 6） | 死分支加死子式，Verus 量过可删（H5-1，中）；核与 core 两套同名类型靠模块路径分开（H5-3，中）；冗余判定、语义写在注释里、K19 注释与代码不符、魔法数、手抄辅助函数、同一结局两个名字（低） |
| H6 | 没打中 | 指纹、数据行、输出三处与 Python 一致，只有一处碰不到的差异 |

零轮改法（只在攻方副本上量过或推的，被攻过零轮）：

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 删掉 C540 扫描里的死分支、`next_counter` 与「≥ 下一号」，补一句引理调用（`dead-code-row.tsv` D01） | H5-1 | **量过**：`verus-rows.sh` 原样输出「2	rc=0	verification results:: 32 verified, 0 errors	rlimit_lines=0」 |
| K20 那一行变异改锚在扫描起点（`index + 1` 改 `index + 2`） | H4-b | 推的 |
| 核里的类型改名带上角色，或者 core 直接用核的类型 | H5-3 | 推的 |
| 胶水的「新根从哪来」改成两成员枚举 | H5-4 | 推的 |
| spec 加幽灵读盘日志，或者补一条「有候选时挂载那一步环读次数」的 harness 用例 | H4-c / H2 的口子 | 推的 |

## H2　spec 补完之后证到了什么（重点）

### 量法

冻结副本的核 crate 拷到 `/tmp/claude-1000/m3-verus-land-code-r2-opus/verus/base/`，原样跑 Verus 0.2026.09.20 `--no-cheating --rlimit 10`：「verification results:: 32 verified, 0 errors」（挂钟 4.68 秒）。之后每行变异在干净副本上套一处、跑一次、删掉副本（`verus-rows.sh`）。两张表：

- 攻方自己的 15 行（`attack-rows.tsv`）：只改可执行代码，不碰 spec、不变式、引理；每行跑前写了猜测。
- `crates/mutations.tsv` 里改核 `lib.rs` 的 24 行原样拿来跑 Verus，问的是：这些 harness 变异里有没有 Verus 看来等价（spec 判不出差别）的。

### 结果（`summary.tsv` 原样，第 1 列是表里的物理行号）

攻方 15 行（行号 2–16 依次是 S01–S15）：

```
2	rc=1	verification results:: 31 verified, 1 errors
3	rc=0	verification results:: 32 verified, 0 errors
4	rc=0	verification results:: 32 verified, 0 errors
5	rc=0	verification results:: 32 verified, 0 errors
6	rc=0	verification results:: 32 verified, 0 errors
7	rc=0	verification results:: 32 verified, 0 errors
8	rc=0	verification results:: 32 verified, 0 errors
9	rc=1	verification results:: 31 verified, 1 errors
10	rc=1	verification results:: 31 verified, 1 errors
11	rc=1	verification results:: 31 verified, 1 errors
12	rc=1	verification results:: 31 verified, 1 errors
13	rc=1	verification results:: 31 verified, 1 errors
14	rc=0	verification results:: 32 verified, 0 errors
15	rc=0	verification results:: 32 verified, 0 errors
16	rc=1	verification results:: 31 verified, 1 errors
```

`crates/mutations.tsv` 改核的 24 行（第 23、24、36、226、355、406、414、448、512–517、613、614、1003、1004、1495–1498、1502、1503 行）：**24 行全红**，23 行是「31 verified, 1 errors」，第 1004 行是「30 verified, 2 errors」（日志里有 1 行 rlimit）。harness 变异里没有一行在 Verus 看来是等价变异。

### 逐行判读

| 行 | 改了什么 | Verus | 判读 |
|---|---|---|---|
| S01 | C540 扫描删掉「计数器 ≥ 下一号」那一判 | 红 | **不是语义差别**：红在两处证明里的 `assert(is_a_later_record_of_the_same_instance(…))`（第 1118、1149 行），证明要用这一判给出的事实。补一句引理调用后照样验得过，见 D01 与 H5 第 1 条 |
| S02 | 锚点读不出那一支删掉序号那一判 | 32 verified | 等价变异：注释自己也说这一判冗余（第 1409–1410 行），Verus 坐实 |
| S03 | 反向链回调每个候选调两次 | 32 verified | spec 不管读盘次数 |
| S04 | 关掉点名验证时照样调点名回调 | 32 verified | spec 不管 Skip 下读不读盘 |
| S05 | 逐条判定之前先调点名回调 | 32 verified | spec 不管读盘次序 |
| S06 | 主循环每条记录（含水位之下、跨实例的）都调一次反向链回调 | 32 verified | spec 不管对谁读盘 |
| S07 | 末条提交标记那一判挪到反向链回调之前 | 32 verified | spec 不管判定次序（结局相同，少读一次） |
| S08 | C540 扫描遇到 txg 越过这一条的照样往后看 | 红 | spec 钉的是带截断的那种读法（第 243 行注释写的 K10 前提） |
| S09 | 没带提交标记时只拦事务号变小 | 红 | — |
| S10 | 在飞上限按已施加条数数 | 红 | spec 钉的是「过了逐条判定的条数」 |
| S11 | 水位之上条数把实例代号更大的也数进来 | 红 | spec 钉的是同实例那种读法，见下面「与 D23 字面」 |
| S12 | 点名单元验过的条数只数施加了的 | 红 | — |
| S13 | 链上下一号取所选根的实例代号 | 32 verified | 等价变异（候选都与根同实例） |
| S14 | `verification_failed += 1` 改成 `= 1` | 32 verified | 等价变异 |
| S15 | 上一条没带提交标记时，这一条自己带提交标记就照接 | 红 | — |

**判定：照字面判据（「一处只改可执行判定的变异在改后核上照样 verified ⇒ 打中 H2」）没打中。** 照样 verified 的 8 行里，S02、S13、S14 是等价变异（在满足 `requires` 的输入上，结局逐个相同），S03–S07 只改回调调用的次数、次序和对象，不改任何一个判定的结局。改变判定结局的 6 行（S08–S12、S15）全红。对确定的回调（同一个下标两次调用交回同一个值）来说，`replay_follows_the_prefix_rules` 已经把每个结局字段都钉成函数：`admitted` 只能是「候选里从头起、到第一个判不过的或到上限为止」那一段，`applied` 只能是它在最后一个末条处截短，报告五个数和交回的根都跟着定死。

四句问答（S03–S07）：
1. 分不分辨臂：这一轮没有臂，被判对象只有一份核。这几行照样 verified，说明 spec 管不到「读盘的形状」，不说明哪一个判定错了。
2. 被判系统看不看得到：Verus 看得到回调调了几次（`ensures` 关系），但 spec 只要求「据以判的那一次交回了什么」（`back_chain_broken.ensures((position as usize,), false)`，第 270 行），没写调用次数与次序。
3. 满足的是判据字面的哪一句：不满足 H2 判据的「只改可执行判定」，S03–S07 改的是副作用，按字面不算 H2 的打中。**我把它归到 H4「测试与变异证不证得到名字说的那件事」**：核头注释第 9 行写「读盘次数与次序同原文」，这一句今天没有 spec 罩着，见 H4-c 量的 harness。
4. 改法碰不碰得到：跑前条款给 H2 的改法是「补 spec 并把那一行进 verus-mutations.tsv」。对读盘形状来说，Fn 回调的 `ensures` 关系表达不出「调用次数」，要表达就得把回调换成带幽灵日志的写法（例：核交回一个 `Ghost<Seq<(usize, ReadKind)>>`，spec 写出它等于判定走过的那一串）。这一步只是推的，没实现，零轮。

### 与 D23 字面（已定项 4、7、8、14、18、22）逐句对

逐句对照的结果：读者规则四格、提交标记、反向链、点名单元、在飞上限、jsn 严格连续都写进了 spec，判法与条款同向。字面上对不上的只有两处，都是 spec 选了较窄的那种读法，Verus 都钉住了（S08、S11 红）：

- C540 的「之后」带 K10 截断：spec 注释自己写明「同实例 txg 随计数器不降（K10）时与」字面「之后任一条同 txg」逐态相同（第 243 行），4-1 报告第二节 ③ 也写了。不是这一轮新找到的，不记打中。
- 报告的 `above_water` 只数同实例：spec 写的是 `&&& outcome.report.above_water == candidates.len()`（第 596 行），候选的定义是 `record.instance == root_instance && record.checkpoint_txg.0 > root_checkpoint_txg.0`（第 123 行）。已定项 14 定案句的字面是「恢复只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录」（`.claude/kb/decisions/23-journal的角色与格式.md:355`），注 2 又写水位按实例代号为主比。照字面，「水位之上」包括实例代号更大的记录。对施加没有影响（注 1 写明跨实例即停），影响的只有报告里这一个数的口径。`JournalScanReport.above_water` 没有文档注释（`recovery.rs` 第 301 行），条款也没定义报告字段。**记「口径没写」，不记打中**：要改的是在字段上写明「同实例、txg 大于所选根的条数」，不是改 spec。

### `#[verifier::opaque]`、空真、回调两判

- opaque 让 `passes_…`、`fails_…`、`admitted_record_passed_every_check` 对求解器不展开，不改后置条件的含义。上面 6 行改判定的变异全红，13 行原有改判定的变异主 agent 现跑全红，说明后置条件没有空真；reveal 写漏只会让证明过不去，不会让它空真。
- 回调两判写成了后置条件（第 265–298 行），按下标对得上。限度在上面第 2 句：spec 是「某一次调用交回过这个值」的关系，不是「只调了这一次」。对不确定的回调（同一下标两次交回不同值，例如真盘上一次读失败一次读成功），S03 那种调两次取或的核也验得过。产品回调每次都重读盘（`judge_back_chain_against_the_previous_record_on_disk` 逐盘 `reader.read`），所以「确定」这一前提靠的是读盘稳定，spec 里没写。

## H4　测试与变异（第 4 批新加的，重点）

### H4-a　第 4 批新加的 9 行和改写的两行，在副本上逐行套、只跑点名的用例

冻结副本拷到 `/tmp/claude-1000/m3-verus-land-code-r2-opus/tree-base/`，`harness-rows.sh` 逐行套上、`cargo test --release` 只跑那一行第 5 段点名的目标、再从冻结副本拷回原文件（跑完 `diff -rq` 冻结副本与副本的 `crates/`：相同）。选的是第 4 批新加、能在 harness 档跑的全部 9 行（第 1497–1505 行），加上 4-1 改写时替换文换了写法的两行（第 512、513 行，第一轮是 `found` 变量，今天改成清空重压 / `if false`）。第 1506–1508 行（E165）与第 517、518 行（层 0）的目标在 checker 档包里，没跑。

第一版 `harness-rows.sh` 把 cargo 测试红时打的那一行「error: test failed」也当成编译错，标成 BUILD_ERROR（原样留在 `results/*-first-classifier.tsv`）；`reclassify.py` 从同一批日志重判，脚本已经改好。重判结果原样：

```
1497	RED	required=two_last_record_flags_in_the_chosen_roots_publish_return_the_error_with_only_that_publishs
1498	RED	required=last_record_flags_of_other_publishes_of_the_same_instance_leave_the_anchor_alone_and_the_n
1499	RED	required=two_last_record_flags_in_the_chosen_roots_publish_return_the_error_with_only_that_publishs
1500	RED	required=a_table_whose_keys_disagree_with_its_records_trips_the_glue_assertion_before_the_kernel_ru
1501	RED	required=a_record_whose_counter_is_the_largest_value_trips_the_glue_assertion_before_the_kernel_run
1503	RED	required=with_an_in_flight_limit_of_one_only_the_first_of_two_single_record_publishes_above_the_cho
1502	RED	required=last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chai
1504	RED	required=stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_t
1505	RED	required=torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_only_the_anch
512	RED	required=when_the_last_record_of_the_chosen_roots_publish_is_unreadable_its_readable_earlier_record
513	RED	required=two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_befor
```

**11 行全红，红的都是点名的那一条。** 逐行看名字说的那件事证没证到：

- 第 1497、1498 行（K19，核的锚点筛选不看 txg）：一条看报错带的计数器只有 [2, 3]，一条看别的发布的末条标志不让恢复停。证到了。
- 第 1499 行（胶水换回 `unreachable!`）：红在 panic 上。证到了。
- 第 1500 行（键等于字段的断言失效）：核拿到乱序的切片照样交回 `Ok`（`above_water: 2, prefix_applied: 0`），用例自己的 `panic!` 消息不含期望的那一句，所以红。证到了。
- 第 1501 行（`counter < u64::MAX` 的断言失效）：**红在核里 `lib.rs:1743` 的「attempt to add with overflow」上**，不是红在「胶水没拦」这一判上；`#[should_panic(expected = …)]` 比的是消息，消息对不上，所以红。用例名说的那件事（断言在核之前拦下）证到了。但这一行能红，靠的是工作区打开了溢出检查（`[profile.release] overflow-checks = true`）；溢出检查关掉时核回绕成 0、照样交回，用例的 `panic!` 同样对不上，也红。不算打中。
- 第 1502 行（K20）：读回 C（`prefix_applied: 2`）。证到了。锚点的问题见 H4-b。
- 第 1503 行（K02）：证到了。「测到的是不是产品路径」：这份用例绕过 `recover`，`choose_root` 用默认环的系统配置，`scan_journal` 和 `replay_journal` 用 3 槽的那份（用例第 153–181 行，文件头第 8–11 行写明了为什么），`recover` 本身一次只用一份配置。走到的是产品的 `replay_journal`（在飞上限在胶水里由 `ring_bytes` 现算）和产品的核；没走到的只有 `recover` 交 `ring_bytes` 的那一句，那一句另有反向链读盘落点的用例管着。不算打中。
- 第 1504、1505 行（G08a）：两条都红在「施加出来的根的 F 取记录新根段里的那一份」那句断言上。证到了。抬 F 的写法是改交给写者的上一版的 F（`writable_mount_takes_a_second_instance.rs` 第 90 行），不是产品里抬 F 的那条路；作为分得出两份 F 的取样点够用。
- 第 512、513 行：替换文换了写法，语义与第一轮相同（读法甲取同 txg 里 jsn 最大的；多于一条照接、取第一条）。证到了。

另外用 Verus 跑了 `crates/mutations.tsv` 里改核的全部 24 行（H2 那一节），没有一行等价，所以第 4 批改写的 18 行里没有一行变成「怎么跑都杀不死」的变异。

### H4-b　K20 那一行锚在死子式上

`crates/mutations.tsv` 第 1502 行把 `later.counter.0 >= next_counter` 改成 `> next_counter`，锚的子式正是 H5-1 里 Verus 证明可删的那一个。变异本身有意义：它模拟的是「只从紧挨着的下一条之后找」这种错，副本上也确实红。问题在于照编码纪律删掉死子式之后，这一行的锚点就腐化了，到时候得换写法。这是「锚点塑形产品代码」的同一族问题（第一轮 C6 那一处被删掉了）。**判定：按 H4 的字面判据没打中**（变异与用例都证得到名字说的那件事），记成 H5-1 的连带改动：H5-1 改掉之后这一行跟着要改。改法（零轮）：H5-1 删掉死子式时，把这一行改成在扫描入口改起点（例：`let mut later_index = index + 1;` 改成 `index + 2`），语义同样是「跳过紧挨着的下一条」。这个改法是推的，没跑。

### H4-c　核头注释说「读盘次数与次序同原文」，Verus 和 harness 都不管

`lib.rs:9` 写着「核走到那一步才调，读盘次数与次序同原文」。H2 里 S03–S07 五行只改回调的调用次数、次序和对象，Verus 全部验得过。在 harness 上跑了其中三行，结局原样（重判后）：

```
2	GREEN	required=ANY
4	GREEN	required=ANY
5	RED	required=ANY
```

（第 2 行是 S03，第 4 行是 S05，第 5 行是 S06。）S06 对水位之下、跨实例的记录也调反向链回调，红在 `mounted_read_by_offset.rs:689` 挂载那一步的环读次数上：

```
assertion `left == right` failed: 挂载那一步全环扫一遍、两块盘各一遍：这就是 196 608 那一档，计数器数得出它
  left: 393218
 right: 393216
```

S03（每个候选调两次反向链）、S05（点名回调提前到所有判定之前）在这条用例上是绿的。harness 里数挂载时读盘次数的断言**只有这一处**（`grep` 过 `ReadCountingPoolReader` 的全部用法：`mounted_read_by_offset.rs`、`multi_unit_file_write_and_read.rs`、`tree_nodes_and_the_central_mapping_on_mounted_read.rs`，后两份只数挂载之后的读）。而且这一处用的是健康镜像，水位之上一条候选都没有，回调根本不调。所以「对候选多读、换次序读」这一类变化，**今天没有任何断言会红**：Verus 量过（S03–S07 验得过），harness 推的（只跑了这一条用例，其余没有读计数的用例结局不变，这一点是推的）。

四句问答：不分辨臂；Verus 看不到（spec 不写调用次数）；按字面不满足 H4「一行变异或一条测试证不到它的名字说的那件事」，因为那句话写在注释里，不是测试名也不是变异名；改法见 H2 第 4 句（幽灵读盘日志，零轮，推的），或者在 harness 里补一条「水位之上有候选时，挂载那一步的环读次数等于全环扫描加每个候选一次反向链重读」的用例。**判定：没打中，记一条欠的检查**。注释写的这一句今天只靠第一轮的对拍（一次性的，没入库）撑着。

## H5　编码纪律（第 4 批新写的代码，重点）

规则出处：`.claude/singlefs-ai-sop/rules/code-discipline.md`。下面每条的原文与位置都在末尾「引文」一节里，按 cite-check 认的写法列出。

| 号 | 文件:行（冻结副本） | 违的那一条 | 证据 | 轻重 |
|---|---|---|---|---|
| H5-1 | `lib.rs` 第 1070–1072 行（计数器是 u64 最大值就交回 false 那一支）、第 1073 行 `next_counter`、第 1090 行 `&& later.counter.0 >= next_counter` | 「用不到的代码删掉」 | 入参按 (实例代号, 计数器) 严格升序，切片里在后面的同实例记录计数器一定更大：这一判恒真；计数器是最大值时后面不会有同实例记录，循环本来就交回 false。**量过**：三处一起删掉、进 if 时补一句已有引理的调用（`dead-code-row.tsv`），Verus「32 verified, 0 errors」。第一轮 C6 删的是另一处死子式（第 240 行那一处），这两处没删。另外 `crates/mutations.tsv` 第 1502 行（K20，主 agent 合入时改的锚点）锚在这一个死子式上：照纪律删掉之后，这一行的锚点就腐化了，见 H4-b | 中 |
| H5-2 | `lib.rs` 第 1415–1416 行 `|| record.ordinal_within_publish.0 != 1` | 「用不到的代码删掉」 | 第 1409–1410 行的注释自己写明这一判与第 1425 行那一判冗余。**量过**：S02 删掉它，Verus「32 verified, 0 errors」。保留它可以说是为了对照条款注 1 的字面，但注释只交代了「冗余」，没交代「为什么留」 | 低 |
| H5-3 | `lib.rs` 第 24、30、36、39 行，与 `singlefs-core` 的 `address.rs` 第 26、30 行、`journal.rs` 第 34、72 行 | 「名字自己站得住，不靠模块路径补全」；「重复要生成，不许手抄」 | 核里新立的 `InstanceGeneration`、`CheckpointTxg`、`JournalRecordOrdinalWithinPublish`、`JournalRecordPlaceInPublish` 与 `singlefs-core` 里的同名类型是同一个概念、两个类型。胶水只能靠路径分开（`recovery.rs` 第 3044 行 `singlefs_replay_prefix::CheckpointTxg(record.checkpoint_txg.0)`），`JournalRecordPlaceInPublish` 的两个成员在胶水第 3050–3057 行手抄映射一遍。全仓搜 `CheckpointTxg` 会落到两个类型上。判决第一轮 C6 定的是「在核 crate 里自己立」，没有定「取同一个名字」。改法（零轮，推的）：核里的类型改名带上角色（例 `KernelCheckpointTxg`），或者让 `singlefs-core` 直接用核的类型、只留一份 | 中 |
| H5-4 | `recovery.rs` 第 3037 行的注释，第 3058 行 `new_root_source: Some(index)` | 「类型能表达的交给类型，类型表达不了的写进名字，都别写进注释」 | 「`None` 是所选根自己，`Some(下标)` 是最后施加的那次发布的末条」这层含义只写在注释里，类型是裸 `Option<usize>`。改法（零轮）：在胶水里立一个两成员的枚举（所选根 / 记录表里第几条） | 低 |
| H5-5 | K19 用例 `recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs` 第 26–27 行与第 31 行 | 注释与代码对不上；魔法数字 | 第 26 行注释写常量是「记录与根共用的 fsid 低 8 字节」，可根用的是 `filesystem_identifier: [0x53; 16]`（第 31 行），低 8 字节是 0x5353535353535353，不等于第 27 行的 `0x5349_4E47_4C45_4653`：**两者并不共用**。今天读者是空池，根的 fsid 也不参与取前缀（反向链重读拿的是记录自己的 fsid），结局不受影响，错的只是注释。`0x53` 没有起名 | 低 |
| H5-6 | K02 用例 `journal_replay_applies_no_more_records_than_the_in_flight_limit.rs` 第 36、47、62、104–105 行 | 「魔法数字写成命名常量」 | `slot_offset(target, 4096)` 与 `flip_byte(…, 100)`（第 62 行）、`+ 60`（第 47 行）、`% 253`、`* 7`（第 36 行）、4100 / 2500 与 3 / 11（第 104–105 行）都没起名。前两个是沿用旧用例的写法（`slot_offset(target, 4096)` 在 harness 用例里出现 10 次），新文件照抄了一份 | 低 |
| H5-7 | `writable_mount_takes_a_second_instance.rs` 第 83–99 行 | 测试代码 DRY「放宽」，仍算手抄 | `overwrite_in_process_raising_the_floor` 与同文件第 64 行的 `overwrite_in_process` 只差第 90 行那一句。纪律对测试代码是放宽，只记一笔 | 低 |
| H5-8 | `lib.rs` 第 1310 行 `BrokenBeforeTheNamedUnits` 与第 327 行 `fails_a_check_before_the_named_units`；第 1311 行 `BrokenAtTheNamedUnits` 与第 292 行 `named_units_reported_unverified` | 「一个语义概念全仓一个名字」 | 同一个结局，可执行一侧叫 Broken，spec 一侧叫 fails / unverified。第一轮 C6 定过「同一失败统一一个名字」，这一处是第 4 批新起的枚举 | 低 |

`#[allow]` 的 reason（第 1314、1484 行）写了为什么，3-6 的 `large_enum_variant`（e165 装置第 901–902 行）也写了理由。新 newtype 与两个枚举（`JournalRecordPlaceInPublish`、`NamedUnitVerification`）本身照 C6 的要求做了。

## H1　改后核与改前等价（次要，正推腿为主）

没有重做对拍（正推腿那一格）。只核了两条新断言在产品路径上会不会触发：

- 键等于字段：调 `replay_journal` 的 8 处（`recover`、`mount.rs` 的 `read_stage`、`mounted_read.rs` 的 `mount_read_only`、E165 装置、E158 装置、3 份 harness 用例）建表都是 `(parsed.instance, parsed.counter)` 或 `(record.instance, record.counter)`（E165 装置第 771、789、3308 行，`scan_journal` 第 2924–2926 行）；只有 K19 第 3 条用例故意对调。产品路径上触发不了。
- `counter < u64::MAX`：计数器从盘上 6 字节读来（已定项 9 的 48 位），三处产品调用点都只交 `scan_journal` 的表。触发不了。

**没打中。** 读盘形状（H1 题面的「读盘次序」）今天只有 spec 之外的东西管着，见 H4-c。

## H3　报错路径与 G6（次要，正推腿为主）

核的两种失败在胶水里各有一个成员（`recovery.rs` 第 3094–3121 行）：`RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided` 带所选根的实例代号、txg 与核交回的计数器（按切片次序，也就是升序）；`ChosenRootCheckpointTxgHasNoSuccessor` 映射到 `UnitMalformed { what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR }`。`CoreFailure` 只有这两个成员，`match` 没写通配臂，漏一种编译不过。没有被吞的。两种失败谁先报由 spec 钉死：`Err(ChosenRootCheckpointTxgHasNoSuccessor)` 一臂要求 `flagged.len() < 2`。**没打中。**

## H6　E165 装置的 `--pinned-cases`（次要）

对了三处接口：

- 指纹：装置的 `input_fingerprint_of` 取最后一个「 state=」之后的值（不许带空白），截到「;first=」之前，算 `sha256(域|N|值)`；`research/scripts/e165-pin-red-examples.py` 的 `state_before_outcome` 用 `(?:^| )state=(\S+)$` 取同一段，再 `split(";first=")[0]`。只有一处不同：行首就是 `state=`（前面没有空格）时 Python 认、装置不认。逐态行都以「E7RESULT name=state 」起头，碰不到这一处。
- 数据行：两边都跳过空行和 `#` 开头的行，八段制表符分隔。
- 输出：装置对每个找得到状态的行打一行逐态行（漂了也打，同时记问题、最后退 1），找不到的行不打。表头约定「按数据行的次序每行打一行结果」，在有问题的那一次两边行数对不上，但装置这时已经退 1，`--replay` 照退出码判红，不会误判成绿。

3-6 的三条单测与三行变异（第 1506–1508 行）读过，各自拦得住名字说的那一种改坏（截 `;first=`、八段、指纹比全文）；这三行的目标在 checker 档包里，**没跑**。**没打中。**

## 没打中的形状

| 形状 | 取样范围 | 结局 |
|---|---|---|
| 只改可执行判定、让 Verus 照样验得过（H2 判据） | 攻方 15 行（S01–S15），加 `crates/mutations.tsv` 改核的全部 24 行 | 改变判定结局的 6 行与 24 行全红；验得过的只有等价变异（S02、S13、S14）与只改读盘形状的（S03–S07） |
| spec 与 D23 已定项 4、7、8、14、18、22 字面对不上 | 逐句对（读者规则四格、提交标记两句、反向链、点名单元、五条口径、注 1、在飞上限语义、jsn == expected） | 只有 C540 的 K10 截断（已知）与 `above_water` 只数同实例（口径没写）两处，都不记打中 |
| opaque 造成空真 / reveal 写漏 | 改判定的变异全红；改后核原样 32 verified | 没打中 |
| `crates/mutations.tsv` 改写后语义漂了 | 第 4 批改写的行逐列比第一轮冻结副本（22 行有变化），语义逐行对过；改核的 24 行 Verus 全红；第 512、513 行在 harness 上红 | 没漂 |
| 第 4 批新用例证不到名字 | K19 四条、K20 一条、K02、G08a 两条，对应 9 行变异在副本上全红 | 没打中 |
| 胶水两条新断言在产品路径上触发 | 8 处调用点的建表方式逐处读 | 触发不了 |
| 核失败映射错或被吞（H3） | `CoreFailure` 两个成员逐个对 | 没打中 |
| E165 `--pinned-cases` 与 Python 两边各读一种（H6） | 指纹、数据行、输出三处 | 只有「行首就是 state=」一处不同，碰不到 |
| 改法本身是否为性能改 core | 本报告提的四个改法（H5-1、H5-3、H5-4、H4-b）与两条欠的检查 | 都只动死代码、类型名或测试，没有一条是为了快 |

## 这条腿自己的限度

- **只抽了一次样**（攻方一条），「没打中」的格按 `.claude/rules/three-way-inference.md` 那一节，要再抽一次才算数。
- harness 只跑了 14 行变异，都是 release 档，每行只跑点名的那一个目标。整包 debug 全量在本机 30 分钟只跑完 5 个目标，被后台时限停了；17 个目标的 release 基线跑了 4357 秒，全绿 149 条。所以读盘形状那一类（S03、S05）只在唯一一条数挂载读盘的用例上量过，「别的用例结局也不变」是推的。S04、S07 只跑了 Verus，没上 harness。
- 第 4 批改写的 18 行里，第 1497–1505、512、513 行之外的那些行只跑了 Verus（证明不等价），没在 harness 上套。层 0 与 E165 的那几行目标在 checker 档，没跑。
- 没做对拍（H1 归正推腿）。
- spec 能把结局钉成函数这一判，依据是读 spec 加上改判定的变异全红，没写「给定具体输入、由后置条件推出 prefix_applied 等于几」的 Verus 冒烟。
- 副本上量出的数不是入库装置上的数。

## 没做什么

- 没跑 checker 档：E165 三行单测与变异（第 1506–1508 行）、层 0 两行（第 517、518 行），也没跑门禁。
- 没读禁读清单里的两项（同轮正推的报告与模型目录）。
- 没判正推、辩方那几格；没替主 agent 采纳；四个改法都是零轮、只在攻方副本上量过（H5-1 量过 Verus；H5-3、H5-4、H4-b 是推的）。

## 引文（cite-check 认的写法；冻结副本写绝对路径）

- 「核走到那一步才调，读盘次数与次序同原文」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:9`）
- 「同实例 txg 随计数器不降（K10）时与」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:243`）
- 「back_chain_broken.ensures((position as usize,), false)」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:270`）
- 「record.instance == root_instance && record.checkpoint_txg.0 > root_checkpoint_txg.0」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:123`）
- 「&&& outcome.report.above_water == candidates.len()」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:596`）
- 「if record.counter.0 == u64::MAX {」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:1070`）
- 「let next_counter = record.counter.0 + 1;」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:1073`）
- 「if later.instance.0 == record.instance.0 && later.counter.0 >= next_counter {」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:1090`）
- 「两判之间只有反向链回调，它只会断、不改状态。」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:1410`）
- 「|| record.ordinal_within_publish.0 != 1 {」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:1416`）
- 「pub struct CheckpointTxg(pub u64);」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:30`）与「pub struct CheckpointTxg(pub u64);」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-core/src/address.rs:26`）
- 「BrokenBeforeTheNamedUnits,」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:1310`）
- 「expected_next = Some((record.instance, JournalCounter(record.counter.0 + 1)));」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:1743`）
- 「checkpoint_txg: singlefs_replay_prefix::CheckpointTxg(record.checkpoint_txg.0),」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-core/src/recovery.rs:3044`）
- 「new_root_source: Some(index),」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-core/src/recovery.rs:3058`）
- 「pub above_water: usize,」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-core/src/recovery.rs:301`）
- 「记录与根共用的 fsid 低 8 字节；读者是空池，不拿它比。」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs:26`）
- 「const FILESYSTEM_IDENTIFIER_LOW_BYTES: u64 = 0x5349_4E47_4C45_4653;」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs:27`）
- 「filesystem_identifier: [0x53; 16],」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs:31`）
- 「image.flip_byte(device, slot_offset(target, 4096), 100);」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs:62`）
- 「previous_with_the_raised_floor.root.rollback_floor = raised_floor;」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs:90`）
- 「while_mounting.reads_inside_the_journal_ring,」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-harness/tests/mounted_read_by_offset.rs:690`）
- 「later.counter.0 > next_counter」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/mutations.tsv:1502`）
- 「用不到的代码删掉，注释掉的代码也删。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:181`）
- 「魔法数字写成命名常量：一个数只许有一处定义」（`.claude/singlefs-ai-sop/rules/code-discipline.md:178`）
- 「名字自己站得住，不靠模块路径补全」（`.claude/singlefs-ai-sop/rules/code-discipline.md:32`）
- 「类型能表达的交给类型，类型表达不了的写进名字，都别写进注释。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:78`）
