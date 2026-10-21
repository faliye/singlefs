# m3-close-c693-r2 云端正推腿报告（Sonnet，2026-10-20）

立场：正推。从已定条款与 `crates/` 今天的实现推「应该是什么」，再与被判对象（第一轮判决的修补、E186 第二次跑的判决、调查员的读数）逐格比；不替主 agent 采纳或判决。正文 `research/prompts/_m3-close-c693-r2-body.md` 第三节 V1 到 V4 四格。方案按 `crates/` 今天的实现来谈（第 0 节）。第一轮已列的改动不重列；没读禁读清单里的文件。

每格结论标「量过」（本腿跑出输出，模型目录 `out/` 或草稿目录日志）或「推的」（读条款、读码推的）。

## 复跑命令与 sha256

```
# 仓根；V2 一段要在草稿目录拷一份 crates 编 harness 档测试目标，共用内存额度占满时设 RUN_WITH_MEMORY_CAP_WAIT_SECONDS=21600
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-c693-r2-sonnet-model/tools/rerun.sh \
  --repo . --frozen research/prompts/m3-close-c693-r2-snapshot --work <新的草稿目录>
# 末尾 name=rerun_compare 八行都 identical=1 即复出 out/ 下八份；name=v2_f2_regress 一行 passed=30
```

读的入库产物（开工后现算）：

```
f1dfd78bc92b7ed919af4484e334b379b615f3914b71a5cf36f513843439a86a  research/results/e186-unknown-tree-accounting-rederive.out.gz
ef2bd29e33654aa9d3dab846a98bd297d9fa74c14fbd096648da68316197705e  research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz
29cafdd37626b06fed52b4c3b8c66ce302be74a7e516cb0227e92e5e51610e58  research/results/e97-entry-encoding-single-generation-2026-10-20.out
885c238a93c4519e58a4dd1f1251ebd0669f25bdbca7629ee21acfe94caa9c07  research/results/e71-accounting-keys-single-generation-2026-10-20.out
```

模型目录 `research/prompts/m3-close-c693-r2-sonnet-model/` 每个文件的 sha256：

```
6cc8b2b72e47534366822f37735e47071eb36770a13852d2ea746ae96a3f7028  diffs/blacklist-mount.diff
8cbcc0d837fd90c491c3439b5898cb4dc8d6a9dd6eb94b7664e18947bc3dd335  diffs/blacklist-mounted_read.diff
869f7f93a007beaa0dcab14c55d5bc8a17c98085618620a5830ad599140dcc90  diffs/blacklist-recovery.diff
6cc8b2b72e47534366822f37735e47071eb36770a13852d2ea746ae96a3f7028  diffs/objectlevel-mount.diff
596e4927f7c4d659799b028626c5a173960eeb40624681afb39fc38f900e75dc  diffs/objectlevel-mounted_read.diff
893326540536dc3d13de5213a33a357536121c2050a9a6bae10e4e0f2e0441f0  diffs/objectlevel-recovery.diff
6cc8b2b72e47534366822f37735e47071eb36770a13852d2ea746ae96a3f7028  diffs/whitelist-mount.diff
596e4927f7c4d659799b028626c5a173960eeb40624681afb39fc38f900e75dc  diffs/whitelist-mounted_read.diff
4bf28e4c0a9fd4fdac6676080634e7cd78cdcac21a1974260f7860f5f23f1b65  diffs/whitelist-recovery.diff
24897c3bb8d1a0fedf3d4f1f4fafd002ebcb94c4ebc2c5fde686809380445f94  out/v1_node_cost.out
0e448bea3eca0670ac6c33bcdac85ef284f1b44b090248dd81b4769054ef76a3  out/v1_readings.out
a090136bfa1a3f45e95eb432f7ddb85d36abcc4000c073b4e566f580102cb196  out/v2-summary-blacklist.txt
f70b2553693867912e5b22d30be847e0e89bd15e358cc32d356d4bf2ed2dc9d6  out/v2-summary-none.txt
f705809aa23d36b3928c6ad0885347c470dc5686095e324c39d42b0433f89908  out/v2-summary-objectlevel.txt
e70e7b61885203afbab69b542dddd3391694b90db9a45b8d16371cbf82b2d7be  out/v2-summary-whitelist.txt
60e211a00dd1fdd8aa4dd70d248a5dbb2e43776bcc3af173e751eae4f89d72f7  out/v3_checks.out
58f3f89ac2cb20265ba8d1250b2cc6d8df8a1bf807e3435f944e87ff6a52b31b  out/v4_fold_checks.out
95b9366b3455521d3536e6c9dc1ede35a07a40756c66be20d66ccfe92b9a193c  tools/rerun.sh
b4e49cf7f8b975d519bff318b5efb2c8a06c9816e06a73082508a9ad8ad182b2  v1_node_cost.py
00cda4c372de9547e1801fcbb9a53344ac76601ab7a99750f4e8135ebfc81470  v1_readings.py
6f7fefcbcb36033a91a49eea1a68d67e391134ea47eeede778b05dbf678f5555  v2/apply_variant.py
33099f0e2b20f62170844cd4b0fda9136b6ca1b7d70ccc011e345b964d6e660d  v2/run_v2.sh
31a2be8fee3708c505dbbbb0e451c8acecf6e3e848ee3a2f91c2b40f919805c7  v2/summarize.py
89c6b0b77330bc3ec286d8d590d3c4873f2ba0148006143819c464653bf37bea  v2/tree_table_entry_with_an_unknown_flags_bit_in_the_newest_version.rs
cf21cea6a6d0088daef130feb80d794db8b273517d561e5d7a95117de3885e88  v2/v2_extra_entries.rs
e1d92cb5255c39e53c4e81ded40d346113079971c44a73cb4206eb62fb70ff46  v3_checks.py
5b263f49c4d101103ef57ae2a2bdbbb132e295aa52aa228c6f9bb9f4ffc1f12f  v4_fold_checks.py
```

V2 补丁量落点用的最小改法在 `v2/apply_variant.py`（`diffs/*.diff` 是它对三份源码的 diff）；打补丁前三份源码的 sha256 钉在脚本里：recovery.rs `67382a13…`、mounted_read.rs `5c46bc34…`、mount.rs `655aaad1…`，对不上时脚本打印 `pristine_sha256_differs`，落点读数不保证。
## 0 实现今天的样子（主要事实，读过的路径；行号现查，量过的标「量过」）

引文约定：kb 与代码的行常常很长，冲突两边抄的是冲突那一句（引号里逐字、括号里是那份文件自己的行号），不是整行。

| 事实 | 出处 |
|---|---|
| 记账树每次发布按「上一版 key 集合」与「这一版 key 集合」之差整棵重规划，没有墓碑条目 | 「for deleted in previous_keys.difference(keys_after_this_publish) {」(`crates/singlefs-core/src/code_two_tree.rs:655`) |
| 一次发布的行集 = 池级三行 + 每盘六行（代段 = 这次 txg）+ 从上一版照抄的水位台阶行；写者只给它认识的那个头（inode 树）写水位行 | 「entries.extend_from_slice(inode_number_watermark_steps_carried_from_the_previous_version);」(`crates/singlefs-core/src/transaction.rs:6968`) |
| 台阶行按统计量认、不按树认（`find` 只取这一版的第一条水位行）；写者自己写明克隆路径还没有 | 「`crates/` 今天没有克隆路径」(`crates/singlefs-core/src/transaction.rs:3682`)；checker 的 I-9.30 同样不按头分：「`crates/` 今天没有克隆路径，这一判不按树段分头」(`crates/singlefs-checker/src/walk.rs:13074`) |
| 记账树叶容量 477、内部 150 | 「格式算出来的节点容量：记账树叶 477、内部 150」(`crates/singlefs-core/src/transaction.rs:2561`) |
| 恢复判台阶行之外的记账行恰为 3 + 6 × 盘数 | 「if entries_other_than_the_inode_number_watermark_steps != 3 + 6 * device_count {」(`crates/singlefs-core/src/recovery.rs:5724`) |
| 树表条目 flags 有未知位 ⇒ `parse` 交回 `None` ⇒ 整张树表读不下去；读树表取条目的调用点 6 处（`grep -n 'tree_table_entries_each_kind_at_most_once(&' crates/singlefs-core/src/*.rs \| wc -l` 输出 `6`，量过） | 「_flags_this_reader_refuses => return None,」(`crates/singlefs-core/src/records.rs:753`)；「TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;」(`crates/singlefs-core/src/recovery.rs:2804`) |
| 走读失败后退回的三个触发点，退不退只看「系统配置见证没见证」，不看失败是什么 | 恢复 「Err(failure) => {」(`crates/singlefs-core/src/recovery.rs:7116`)；只读挂载 「Err(failure_of_the_selected_version) => {」(`crates/singlefs-core/src/mounted_read.rs:655`)；可写挂载 「Err(failure_of_the_selected_version) => {」(`crates/singlefs-core/src/mount.rs:8016`)；判法 「pub fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to<」(`crates/singlefs-core/src/recovery.rs:6893`)（它不收失败值）；harness 的挂载透明判官自己用「走读能不能通」再调同一个判法：「walk_to_file(reader, root, &mut mapping_fallbacks).is_ok()」(`crates/singlefs-harness/src/mount_transparency.rs:225`) |
| 未登记的树种类码在读树表之后被当损坏（`UnitMalformed`），不是「对旧读者只是看不到新树」 | 「what: "树的种类没登记",」(`crates/singlefs-core/src/recovery.rs:4124`)；「what: "树的种类没登记",」(`crates/singlefs-core/src/recovery.rs:6451`) |
| 每次发布的落盘出口只有一条：根槽 FUA 之后无条件轮换系统配置槽 | 「writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {」(`crates/singlefs-core/src/transaction.rs:1533`)，随后 「writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {」(`crates/singlefs-core/src/transaction.rs:1537`)（本腿重读过这两行） |

零命中（实现里没有的）：`grep -rn -i 'write_buffer\|write buffer\|WriteBuffer' crates --include=*.rs | wc -l` 输出 `0`（第一轮同一条命令；本腿复跑见「复跑命令」）；`grep -rnE 'fn [a-z_]*(destroy|delete|drop)[a-z_]*head|fn [a-z_]*clone[a-z_]*head|fn [a-z_]*head[a-z_]*(destroy|delete)' crates/singlefs-core/src --include=*.rs | wc -l` 输出 `0`。
## V1　D8 已定项 8 ㊀（旧写者遇到不认识的树，记账行怎么处置）

### V1.1 E186 第二次跑的判决撑不撑得住（量过 + 推的）

结论：**㊀ 不翻面撑得住，但撑它的不是第二次跑量出的数，是「没有任何一种合规读法让别的臂严格赢」。** 页上自己写了这一点：「分量来自 U5 的读法与反向接受条款的改写，不来自新量出的数」(`.claude/kb/experiments/186-不认识的树-记账维重推.md:179`)。第二次跑的 `unique_carry` 因此只能当「没有翻面的证据」引，不能当「`Carry` 最优的测量」引：产物行 「group=legal_pooled judged=yes judged_points=6400 excluded_points_h_new_0=3200 form=unique_carry consequence=carry_stands winner_sets=Carry:6400」(`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz:186278`)。

第二次跑改了什么：只改 U5 与后果表两处，别处一格不动，这一点产物自己判了：「name=verdict check=judgements_differ_only_in_u5 cells=172800 mismatches=0 holds=true」(`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz:186282`)。所以问题只在这两处改动该不该信，下面两条路各验一次。

**量过一：读法矩阵（我自己的脚本，只读两份入库产物的格行，不 import 别的腿的脚本）。** `python3 -I research/prompts/m3-close-c693-r2-sonnet-model/v1_readings.py <第一次跑产物> <第二次跑产物>`，输出 `out/v1_readings.out`；自证两格（第二次跑 `legal_pooled` 的 `Carry:6400` 逐点相同；第一次跑两个分组去掉 `h_new = 0` 的 1600 点后逐计数相同）都 `equal=1`，弄坏开关 `V1_BREAK=swap_acct` 退 3。被判点集 = 合法点里 `h_new > 0`（6400 点），三处读法各换一遍：第 6 位里 D8 已定项 12 那半句「所说的行不存在」算不算要改（U5 空真 / 要改）、㊀ 自己算不算一条（B1 不算 / B2 算）、第 7 位按行还是按记账树节点（`ceil(行数 / 477)`）、Q2 计不计。记账胜者集合（两个 `r_tree_dim` 分组各 3200 点）：

| U5 | ㊀ 自己 | 第 7 位 | `r_tree_dim = 1` 胜者 | `r_tree_dim = 2` 胜者 | `Carry` 不在胜者里的点 |
|---|---|---|---|---|---|
| 空真（第一次跑） | 不算 | 行 | `NoCarry` 1280、`NoCarry+随根留着` 1920 | `Carry` 3200 | **3200** |
| 空真 | 不算 | 节点 | `Carry+NoCarry` 1280、三条并列 1920 | `Carry` 3200 | 0 |
| 空真 | 算 | 行 / 节点 | `Carry` 3200 | `Carry` 3200 | 0 |
| 要改（第二次跑） | 不算 / 算 | 行 / 节点 | `Carry` 3200 | `Carry` 3200 | 0 |

Q2 计与不计逐行相同（16 种组合 = U5 两种 × ㊀ 自己两种 × 第 7 位两种 × Q2 两种）。**`Carry` 不在胜者里的只有「U5 空真、㊀ 自己不计、第 7 位按行」这一种（第一次跑的读法），`r_tree_dim = 1` 的 3200 点。** 这一种已被第一轮判决认定为判据写错（`research/prompts/m3-close-c693-r1-main-verification.md` 第二节 W1 一格）。换成「第 7 位排在第 6 位之前」（用户 2026-10-20 定的是第 6 位在前，这里只报）：按行读时 `r_tree_dim = 1` 的 3200 点 `Carry` 仍不在（两种 U5 都是），`r_tree_dim = 2` 的 1920 点也不在（随根留着独胜），按节点读时 `Carry` 回到胜者里（空真时与 `NoCarry` 并列），见 `out/v1_readings.out` 的 `order_seventh_before_sixth` 八行。所以：按节点读，`Carry` 在任何位次、任何 U5 下都不输；按行读，只有「行在要改条款之前」才输，而「行」一位在设备上兑现成 0 个节点（V1.2）。

**量过二（独立于装置的路径：数 kb 里现行条款）。** 第 6 位那个 `+1` 是不是装置造的？数点名搬运行的现行条款行：`grep -nF '旧写者搬运' .claude/kb/decisions/*.md .claude/kb/*.md .claude/kb/milestone/*.md crates/singlefs-checker/invariants/*.md`（剔掉 `.en.md`、`.ja.md` 与历史文件）只命中 D8 一行：「含已定项 8 ㊀ 那类旧写者搬运的记账行」(`.claude/kb/decisions/08-核心索引结构.md:383`)。㊀ 在现行 kb 里被点名的行：D8 的 261（定义）、282（射程）、287（E113 一条的取舍结论）、288（用户定案记录，不改）、383（这一句）与 `checks-owed.md` 452（C693 本行）。**取 `NoCarry` 要改 261（定义整句）、282（「不搬的代价」那一句）、287（取舍结论）、383（半句成了空指称）四处；取 `Carry` 只修 261、282、287 的措辞**（K 代残留与台阶行不换代段，见 V1.3），取舍不动，383 不动。装置的读法 B1 只数 383 一处（已定项 8 自己的 ㊀ ㊁ 登记为不进条款清单），读法 B2 把 261 也数进去；所以 U5 的 `+1` 是一笔真的文本修改，实际差距比 B1 记的还大。它有多重要取决于用户定的「第 6 位排在第 7 位之前」。

**推的：** 页上第二次跑 「第 7 位数的是行，不是写的节点或单元；少写几行变不变成少写节点没量」(`.claude/kb/experiments/186-不认识的树-记账维重推.md:173`) 已经写了第 7 位的局限；`r_tree_dim = 2` 是假设取样点（第二个带树维的统计量今天不存在），第二次跑之后 `r_tree_dim = 1` 也是 `Carry`，㊀ 的结论不再依赖这个假设。「不是盲测、判据跑后改」这一点没法补救，只能靠上面的矩阵把「结论不依赖那处改动」摆出来：改动之前的读法下 `Carry` 输，改动之后赢，**其余 14 种组合里它一直不输**。

**什么现象会推翻 V1.1：** 用户或主 agent 在第 6、7 位之外加一位让 `NoCarry` 严格赢，而且该位在设备上兑现（写节点数、写字节、崩溃窗口里的某个量）；或第 7 位被定成排在第 6 位之前且按行读；或 383 那半句被别的理由删掉（`NoCarry` 的 `+1` 消失，矩阵里的 `Carry` 与 `NoCarry` 并列，不再独胜，仍不输）。
### V1.2 W1-丁「按节点数第 7 位分不开」作为代价信息怎么写进 ㊀ 的射程（量过）

`python3 -I research/prompts/m3-close-c693-r2-sonnet-model/v1_node_cost.py`（输出 `out/v1_node_cost.out`；自证叶 477 / 内部 150 与 「格式算出来的节点容量：记账树叶 477、内部 150」(`crates/singlefs-core/src/transaction.rs:2561`) 写的值相等，`V1_NODE_BREAK=entry_without_seq` 退 3）。记账树每次整棵重写（第 0 节第一行），写出节点数 = 各层节点数之和；行数 = 3 + 6 × 盘数 + 台阶行 s 级 + `Carry` 多出的 `h_new × r_tree_dim` 行。两个量过的数：

- E186 两份产物的 172800 格，每格每代写行最大值：`grep ' name=cell ' <产物> | awk …` 两份都输出 `max rows_written_w2 = 47`（47 < 477：取样区里多写的行落在同一片叶里，0 个节点）。
- 把取样区扩到盘数 1..4096、台阶行 1..3 级（12288 点）：`Carry` 多写一片叶的点，`r_tree_dim = 1` 时 `h_new` = 1、2、4 各 25、50、101 点（2‰、4‰、8‰），`r_tree_dim = 2` 时 4‰、8‰、16‰；第一个点在 78 或 158 块盘。**所以「0 个节点」只在 E186 的取样区里成立，不是全称**（模型算术，不是入库装置的一次跑；进 kb 前主 agent 复跑这个脚本，或把它做成 E186 装置的附带量）。

### V1.3 ㊀ 的改后条文（三处，逐句；行号现查）

替换第一轮正推 1.5 节的三条。三处的出入分别是：台阶行不换代段（第一轮已指出）、上一版那次发布水位变了时同一行既是现行行又是台阶行（**第一轮条文漏了这一句**，V1.4 第 1 条）、K 代说法残留。

**① D8 已定项 8 ㊀（「写者必须把这类行原样带到新代（值不变、只换代号」(`.claude/kb/decisions/08-核心索引结构.md:261`)）整条换成：**

> - **㊀ 旧写者对不认识的记账行**（限「树 ID 不认识、统计量标签认识」这一半）：**写者必须把这类行带到新代**——「这一代的行」（代段 = 上一版 txg）值不变、代段换成这次发布的 txg；台阶行（D5 已定项 2 射程「inode 号水位的台阶行」，代段早于上一版 txg）不换代段，按台阶行的照抄规则原样随根留着，该规则按统计量认、不看树 ID，写者不必认识那棵树。上一版那次发布水位变了时，同一行既是现行行又是台阶行：台阶行照抄，现行值另写一行（代段 = 这次 txg）。有前端的写者把这类行当前端条目写、随发布排空到叶，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错。

**② D8 已定项 8 射程（「不搬的代价：D5（快照 / 空间记账机制） 已定项 2 定每条根的记账树只装这条根那一代的行、每次发布全写一遍 ⇒ 写者不认识的那棵树的行没人重写，下一次发布就消失」(`.claude/kb/decisions/08-核心索引结构.md:282`) 起，到该句括注为止）换成：**

> 不搬的代价：D5（快照 / 空间记账机制） 已定项 2 定每条根的记账树只装这条根那一代的行、每次发布全写一遍 ⇒ 写者不认识的那棵树的「这一代的行」没人重写，下一次发布就消失；台阶行不受这个影响（照抄按统计量认、不看树，只要那个头至少留着一级台阶，现行值——该维度元组下代段最大的那一行——仍读得出；建克隆那次给新头留一行是 D5 已定项 2 射程的条款，`crates/` 的克隆路径还没有，随里程碑十四）。仍取搬运的依据见依据里 E186 一条。**搬运的代价信息**：每个不认识的头每次发布多写 1 行（该头带树维的统计量有几个就几行，今天一个）、34 字节；E186 的取样区里每格每代写行至多 47、记账树叶容量 477，多写的行落在同一片叶里，0 个节点；盘数超过 78 之后（行数 3 + 6 × 盘数 + 台阶行 超过 477）多写的行有时压过叶边界、多写一片叶，在盘数 1 到 4096、台阶行 1 到 3 级的算术扫里占 2‰ 到 16‰ 的点（模型算术，不是入库装置的一次跑）。

**③ D8 已定项 8 依据（「记账那一维取 Carry 不取 Exempt」(`.claude/kb/decisions/08-核心索引结构.md:287`) 那一条）：括注换成「`Exempt` 建在已撤回的 K 代点删上，臂失去对象；记账那一维在每条根只装一代的口径下由 E186 重推，见下一条」，并在它后面加一条：**

> - E186（不认识的树·记账维重推）：每条根只装一代之后，三条记账臂在 11520 点 × 15 格上重比，字典序新补第 6 位「要改几条已定条款」。第二次跑（U5 判据改写之后；判据是在第一次跑的产物之后改的，不是盲测）被判点集（合法点里 `h_new > 0`，6400 点）上记账胜者处处是 {`Carry`}。这一条的分量来自第 6 位的算法，不来自新量出的数：取 `NoCarry` 时 D8 已定项 12 定案里「含已定项 8 ㊀ 那类旧写者搬运的记账行」那半句成了空指称、要改。第 6、7 位的读法各换一遍（U5 空真 / 要改、㊀ 自己计 / 不计、第 7 位按行 / 按记账树节点、Q2 计 / 不计）共 16 种组合，`Carry` 在 14 种里处处在胜者里，只有「U5 空真、㊀ 自己不计、第 7 位按行」一种（第一次跑的读法，已判写错）里 `r_tree_dim = 1` 的 3200 点上不在。（`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz` 第 186276 至 186278 行。）

不动的：D8 已定项 12 定案（383 行）、D5 已定项 2、I-3.13、D8 总表第 8 行进度（交主 agent，V2 一并说）、D8 欠一行的 C693（四块都判完再拿掉）、C159 一行（第一轮 1.5 节末两条照用，不重列）。

### V1.4 攻这三处条文（我自己找到的洞）

1. **漏了「同一行既是现行行又是台阶行」**。台阶的认法是「值与前一级不同才算」，上一版那次发布水位变了时它写的那一行同时是现行行和台阶行（「let row_of_this_version = self.accounting_entries.iter().find(」(`crates/singlefs-core/src/transaction.rs:3696`) 这一段，`find` 只取第一条）；`Carry` 若只写「台阶行不换、其余换」，这一行两头都不是。已补在 ① 里。
2. **台阶行不受影响这一句是个将来条件句**：它依赖「建克隆那次给新头留一行」，该条款在 「克隆头的台阶行按 (统计量, 树) 分，建克隆那次给新头留一行」(`.claude/kb/decisions/05-快照-空间记账机制.md:55`)，实现里是 TODO（第 0 节第三行）。已在 ② 里写明。
3. **第一轮的「多写的行没有变成多写的节点」是取样区内的话**，V1.2 已量出边界。
4. **㊀ 在 `crates/` 里到不了**（V1.5 第 1 条）：旧写者遇到不认识的树，今天先死在「树的种类没登记」，轮不到搬运。③ 的 E186 一条不讲这个，是 V1.5 的事。
5. 没打中的：把 ① 的「换代段」改成「不换」（两头都原样）会让 I-3.13 逐行判代段等于 txg 在被搬运的行上红，所以不取；我没找到别的洞。

### V1.5 `crates/` 要改的（第一轮列的 Carry 三样不重列，只列新的）

1. **前提，不做就没有这个场景**：`recovery.rs` 读树表后遇到没登记的种类码当损坏（第 0 节「未登记的树种类码」一行的 4124 与 6451 两处）。D15 的 compat 说新增 keyspace 对旧读者只是看不到新树，实现还没做到。量过（V2.2 的 `UnregisteredKind` 各行）：把一条必需的非核心树条目的种类码改成没登记的，恢复和只读挂载照读 R，可写挂载拒（见证了 R）或退回 R 之前一版（没见证）。额外多一条新种类条目的形状没造，推的。
2. `transaction.rs` `inode_number_watermark_steps`（3685 行）按 (统计量, 树) 分，「这一版那一行」按树各取一条（TODO 在 3682 行，和 V4 删头是同一处代码）。
3. `transaction.rs` `accounting_entries_of_this_publish`（6931 行）加一路输入：上一版里树 ID 不在写者认识的头集合里、统计量认识、代段 = 上一版 txg 的行，值不变、代段换成这次 txg、`seq` 1。
4. `recovery.rs` `accounting_entry_count_is_three_plus_six_per_device`（5711 行）：「恰为 3 + 6 × 盘数」改成「不少于，且多出来的行都挂在树表里不认识的头上」（推的：恢复只看镜像，数不出旧写者搬了多少行）。

### V1.6 W1-戊「按统计量的一条规则」（零轮）：㊀ 站住之后还要不要

不要，也不能说它被打过。规则 S =「有台阶行例外的统计量不带这一代那一行，其余统计量照 `Carry` 带」：在 `r_tree_dim = 1` 上等于 `NoCarry`，在 `r_tree_dim = 2` 上等于 `Carry`。按第 6 位数条款：S 要改 261、282、287、383（和 `NoCarry` 同四处），另外统计量登记表（D5 已定项 4 的清单，是冻结组件，D15）要多一列「有没有台阶行」，所以第 6 位比 `Carry` 多；它在第 7 位上省的是 `h_new` 行（V1.2：取样区 0 节点）。S 在 E186 的臂集里没有，没量，是推的、被攻过零轮。翻面观测：将来出现第二个带树维、而且也有台阶行的统计量，S 和 `Carry` 在那个统计量上都多写一行/头，仍没有区别；只有出现「带树维、没有台阶行、搬运行数大到按节点数也分得出」的统计量，才轮得到再比。

### V1.7 什么现象会推翻 V1 的结论

- 第 6、7 位之外再加一位让 `NoCarry` 严格赢，且该位在设备上兑现；或第 7 位被定成排在第 6 位之前且按行读（V1.1）。
- 把 E186 做成入库装置的新一次跑后，`nodes` 读法下 `r_tree_dim = 1` 的 `Carry` 不在胜者里（我的矩阵是读入库产物格行重算的，不是装置的一次跑）。
- 克隆路径与 I-9.11 落地后 `NoCarry` 的安全前提成立，且 383 那半句被另一个理由删掉：`Carry` 与 `NoCarry` 在第 6 位并列，只剩第 7 位按行的 `h_new` 行，按节点仍并列，㊀ 取舍变成「写一个不改变节点数的行要不要」，交用户。
## V2　㊁ 与实现的出入

### V2.1 先复核调查员的读数（量过）

在自己草稿目录的仓副本里（主工作区 `crates/` 拷来，与调查员副本只差那一份测试文件，`diff -rq` 为证），拷入调查员的测试文件（sha256 `89c6b0b77330bc3ec286d8d590d3c4873f2ba0148006143819c464653bf37bea`，与调查报告所写相同），经内存包装 2G、`capped.sh 31` 跑 `cargo test -p singlefs-harness --test tree_table_entry_with_an_unknown_flags_bit_in_the_newest_version -- --test-threads 1 --nocapture`：`test result: ok. 11 passed`，59 行 `W2PROBE` 与调查员 `run-final.log` 的 59 行逐行相同（`diff` 输出 `IDENTICAL_PROBES`，sha256 `497cbfda3d673714800957552c8f3106ba81209b864d7479c5e0376afe2a515e`）。调查员的读数在本机复现。

调查员只造了 inode 树条目带未知位这一形。我加了一份测试（`v2/v2_extra_entries.rs`，18 条）：同一套重写改**别的条目**——记账树（核心，读文件与准入要它）、livelist（非核心，根为空）、退役树（非核心，根为空）——各做「flags 置不认识的位」与「种类码改成没登记的 0x7777」两种改法，三种盘面、三处（恢复、只读挂载、可写挂载）各记落点。

### V2.2 落点表（量过；`out/v2-summary-*.txt` 是原表，下面是归并）

记号：拒 = 报错、盘上逐字节不变（可写挂载一栏都带 `disk_unchanged=true`）；R = 落在 R；P = 落在 R 之前一版（可写挂载取新实例号、之后恢复读出的是 P 的内容、R 的内容没了）。每格「恢复 / 只读 / 可写」。盘面：**见证** = 系统配置见证了 R；**崩** = 崩在 R 的根槽之后、轮换之前（R 没确认）；**零** = R 已确认而 R 那次轮换写的系统配置槽读回全零。

| 改的是什么 | 改法 | 见证 | 崩 | 零 |
|---|---|---|---|---|
| inode 条目 flags 未知位（调查的形） | 今天 | 拒/拒/拒 | P/P/P | P/P/P |
| | 乙 黑名单 | 拒/拒/拒 | 拒/拒/拒 | 拒/拒/拒 |
| | 甲 白名单 | 拒/拒/拒 | 拒/拒/拒 | 拒/拒/拒 |
| | 丙 对象级 + 甲 | 拒/拒/拒 | 拒/拒/拒 | 拒/拒/拒 |
| | 丁 承认现状（只改条款） | 同今天 | 同今天 | 同今天 |
| 记账条目 flags 未知位（核心树） | 今天 | 拒/拒/拒 | P/P/P | P/P/P |
| | 乙、甲、丙 | 拒/拒/拒 | 拒/拒/拒 | 拒/拒/拒 |
| livelist 或退役树条目 flags 未知位（非核心，根为空） | 今天 | 拒/拒/拒 | P/P/P | P/P/P |
| | 乙、甲 | 拒/拒/拒 | 拒/拒/拒 | 拒/拒/拒 |
| | **丙 对象级 + 甲** | **R/R/拒** | **R/R/拒** | **R/R/拒** |
| 记账条目种类码没登记 | 今天、乙 | 拒/R/拒 | P/R/P | P/R/P |
| | 甲、丙 | 拒/R/拒 | 拒/R/拒 | 拒/R/拒 |
| livelist 或退役树条目种类码没登记 | 今天、乙 | R/R/拒 | R/R/P | R/R/P |
| | 甲、丙 | R/R/拒 | R/R/拒 | R/R/拒 |
| 对照：inode 单元真损坏（校验和不重算） | 今天、乙、甲、丙 | 拒/拒/拒 | P/P/P | P/P/P |

读法：（1）**今天**，不认识的位在没见证的两种盘面上，三处一律静默落到 P，可写挂载还在 P 上取了号；这不止 inode 树，记账、livelist、退役树的条目带未知位都是。（2）**乙**只认 `what = "树表条目"` 这一个字串，未知位修好了，种类码没登记一栏原样不动。（3）**甲**把退回收窄到「字节丢了」，未知位与种类码两栏都不再静默退回，真损坏的对照仍退（F2 原本要的那一退保住）。（4）**丙**在甲之上让非核心树的条目带未知位时恢复与只读挂载照读 R，可写挂载拒（它要数全部映射单元、维护全部树）；对调查的盘面（inode 条目，核心树）丙与甲落点相同。（5）「记账条目种类码没登记」一栏里恢复拒而只读挂载落在 R：这是今天就有的恢复与只读挂载不一致（推的：只读挂载打开池没有读记账树），与退回无关。
**甲、乙、丙是补丁在草稿副本上的量过读数**，补丁在 `v2/apply_variant.py`（甲 17 行新增、乙 12 行、丙 33 行新增 2 行替换，`diffs/*.diff`）；它们是量落点用的最小改法，不是可入库的实现（丙的「跳过」是对象级 EIO 的近似，真实现要把条目留成『被拒收、位置还在』的一态，见 V2.3）。
**回归**：甲变体上已有的五个 F2 相关测试目标（恢复与两种挂载退回、读取字段随退回、挂载透明判官、可写挂载退回后重判记录、系统配置见证判不出）合起来 `2 + 4 + 16 + 5 + 3 = 30 passed`，乙变体同样 30 passed；乙、甲两个变体上调查员那 11 条都是 10 passed、1 failed，红的只有「钉住没见证时静默落到 P」那一条（它断言的正是要改掉的行为），其余 10 条（含真损坏对照）照绿。丙变体没跑回归（它的跳过只是量具）。
**没量的**：崩溃点重放里 F2 实际退回的失败种类分布（重型，归主 agent，见「要主 agent 跑的命令」）；额外多一条新种类条目（不是改一条旧条目的种类码）的形状；一块盘读得出一块读不出的混合盘面；GPU 判官（`crash_judge_recovery.wgsl` 里没有解树表条目 flags 的代码，只有单元和实例表行的 flags，读码）。
### V2.3 每个改法对 `crates/` 与条款的改动

**条款与实现现在的冲突（并排）：** 「未知位一律拒收——该条目 EIO（对象级，不拒整个池）」(`.claude/kb/decisions/08-核心索引结构.md:262`) ↔ 「TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;」(`crates/singlefs-core/src/recovery.rs:2804`)（整张树表读不下去）；D8 总表第 8 行写着 「未知 flags 位拒收该条目 状态：已定。 进度：已实现。」(`.claude/kb/decisions/08-核心索引结构.md:20`)；F2 的条款 「所选根走读失败、系统配置没见证它时退回见证到的那一版」(`.claude/kb/decisions/23-journal的角色与格式.md:398`) 与它认下的代价 「已确认的版本在较新的系统配置槽与它的单元同时读回全 0 时，恢复、可写挂载与只读挂载悄悄读回上一版」(`.claude/kb/decisions/23-journal的角色与格式.md:423`)（实现里同一句 「代价（判决同一条，用户认下）：已确认的一版在较新的系统配置槽与它的单元同时读回全零时，c_见证 落回它之前，读回的是上一版。」(`crates/singlefs-core/src/recovery.rs:6891`)）。**认下的代价要两个条件（系统配置槽读回全零，加它的单元也读回全零）；未知位那一形单元完好，只要一个条件，所以它在代价之外。**

**甲　F2 的退回分种类（白名单：只为「字节丢了」退）。**
- `recovery.rs` 「pub fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to<」(`crates/singlefs-core/src/recovery.rs:6893`) 加一个入参「所选那一版的走读失败是哪一类」（新枚举两个成员：字节丢了 / 读者拒收内容），内容拒收时第一句返回 `None`；字节丢了 =（恢复侧）「/// 一个单元两条位置条目都读不到校验和相符的那份。」(`crates/singlefs-core/src/recovery.rs:520`)、映射里没有这个 key、位置提示与映射都读不回字节三种；只读挂载的 「/// 树表条目解不开（条目长度、flags 未知位、预留 76 非零）。」(`crates/singlefs-core/src/mounted_read.rs:179`) 归内容拒收，它的 `UnitMalformed`（头坏了、载荷校验和不对）归字节丢了。
- 三个触发点把失败值交进去（7116、655、8016 行，第 0 节）；harness 的 `mount_transparency.rs` 那一处（第 0 节）同样要把失败值留下再交，否则判官和实现分叉。
- 白名单而不是黑名单的理由：乙（黑名单）只认一个字串，种类码没登记那一栏原样不动（V2.2 表）。
- 条款：D23 的 F2 一句（398 行）在「走读失败」后加括注「（失败限字节丢了：单元两条位置都读不到校验和相符的那份，或映射里没有 / 读不回那个 key；字节读到了、校验和对、读者不认的内容——树表条目的未知 flags 位、未登记的树种类码、预留位非零、字段取值越界——不算：那不是没落盘，退回上一版只会把已确认的发布静默丢掉，照报失败、拒挂载）」；认下的代价一句（423 行）不动，它描述的正是字节丢了那一形。

**丙　对象级（㊁ 的本意）。** 甲之上再做：
- `records.rs` 「pub fn parse_with_destruction_mark(bytes: &[u8]) -> Option<(Self, SnapshotDestructionMark)> {」(`crates/singlefs-core/src/records.rs:734`) 的 `None` 拆三态（合法 / 长度、预留、根指针合法而 flags 有未知位 / 损坏），`tree_table_entries_each_kind_at_most_once` 交回的条目带「被拒收、位置还在」一态；六处调用点按树分：读文件走读与只读挂载打开池对核心四棵树（inode、extent、分配记录、记账）遇被拒收就拒，其余树的被拒收条目跳过读、保留位置；`rebuild_version` 与 `allocation_records_under_root` 遇任何被拒收条目就拒（可写挂载要数全部映射单元）。
- （以上拆法是设计，推的，没实现。）消费者：「位置还在」的读者是 deadlist 合并（D5 已定项 16，进度未实现），所以丙随它做。
- 条款：D8 射程在「㊁ 拒收不丢数据依赖 ④ 的条目自带长度」那一句后加进度注与核心树限定：「条目 EIO 但位置还在 ⇒ 写者知道」(`.claude/kb/decisions/08-核心索引结构.md:282`) 这一句对核心四棵树不兑现——核心树的条目 EIO 就是整个文件系统读不了，恢复和两种挂载拒（V2.2 表），其余树的条目 EIO、恢复与只读挂载照读，可写挂载拒。
- 量过的部分：V2.2 表丙一栏（跳过近似）。没量的：保留位置交给 deadlist 合并（那一步没实现）。

**丁　承认现状（只改条款）。** D8 ㊁ 的括注「（对象级，不拒整个池）」改成「（整棵树表读不下去，与单元损坏同处置）」，D23 认下的代价一句扩成「…或树表条目带读者不认识的位…」。落点同今天（V2.2 表「今天」行），已确认的发布在没见证的盘面上静默落到上一版。**这改的是用户 2026-09-06 的定案（「用户定案 2026-09-06：四问」(`.claude/kb/decisions/08-核心索引结构.md:288`)）和 2026-10-07 认下 F2 时的知情范围，只有用户能选；正推只写出它是什么。**

### V2.4 与根环这一簇已定的 F2 改法有没有交集

判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 的两笔：C755 并集判退（「条文补三句」，同文件第 33 行）与 C751 取甲（共用实例表指针才收，第 50 行）。

- **改的是同一批函数，必须同批。** C755 三句落在恢复、只读挂载、可写挂载这三个触发臂和 F2 判法本身；甲的改法也落在这三臂的第一行和判法的第一句。分两批做会在同一处打架。
- **语义上不冲突，可以组合。** C755 的第一句「退不成就留在所选的那一版」把「退不成」定成留在所选版、报原失败；甲是「退回集合为空」的一种退不成。组合规则（零轮，推的）：并集判退里，**任何一处看到的失败是读者拒收的内容，三处都不退，各自报自己看到的失败**——并集的目的是三处落到同一版，一处看到内容被拒说明 R 在字节上完整，退过它就是丢已确认的发布。
- C751 取甲只管「退到哪一条」（共用实例表指针才收同实例的根），不管退不退，正交；C755 第三句「触发之后重扫一遍」：甲在重扫之前判，内容拒收的盘面省掉重扫。
- **「退回分支不看失败种类」要不要分种类：要。** 分法是白名单（字节丢了才退），理由是认下的代价那一句本来就只描述字节丢了。量到的支撑：V2.2 表（未知位、种类码两栏）与 30 条已有 F2 用例照绿。**没量到的是反方向的风险**：崩溃状态里 F2 实际退回的失败是不是都落在白名单三种里（若有落在 `UnitMalformed`、`InvariantViolated` 上的真退回，白名单会把它们改成拒）；这一格是重型，交主 agent（见「要主 agent 跑的命令」）。
- 根环判决第四节「D19 已定项 5 回看：没见证的 R 上数据单元读不出，可写挂载由照常挂 R 改成退回上一版」是字节丢了一类，甲保留它，不冲突。

### V2.5 落点，要不要立欠账

- **甲不用等 deadlist 合并，也不是「随 deadlist 合并那一步」**（第一轮正推的说法只对丙成立）：它不需要任何「位置还在」的消费者，量过。落点 = 与 C755 并集判退同一批实现（「判法函数三处是同一个，触发它的失败不同」(`.claude/kb/checks-owed.md:507`) 所在行，去向写里程碑三二·4；C755 实现批改挂哪个作业点，甲跟它）。
- **丙随 deadlist 合并**：`.claude/kb/milestone/04-fourth-txn.md` 一·2（删快照时 deadlist 并到更新一侧，D5 已定项 16），那是第一个读「位置还在」的读者。
- 攻方的说法「欠的是实现」：对，而且今天实现是**坏的方向**（静默丢已确认的发布），不是「没做完」；调查员的结论（比认下的代价少一个条件）量过。
- **要立的欠账（号待主 agent 取）：** ① F2 退回不看失败种类（要拦：未知位 / 种类码盘面在三种盘面三处不静默退回；怎么拦：`v2/v2_extra_entries.rs` 与调查员那份改成钉新结局，加真损坏对照；前置：无；去向：C755 同批）。② 树表条目带未知位整表拒，与 ㊁ 对象级不符（去向：04 一·2）。③ 未登记的树种类码被当损坏，旧写者看不到新树（D15 compat 在 `crates/` 没做到；去向：里程碑十七二·2 冻结契约检查加一格「新写者造的盘给旧读者读」，推的，开工前核）。
- D8 总表第 8 行「进度：已实现」不对（㊀ 一行没实现：`grep -rn -E '旧写者|不认识的树|unknown_tree|carry_unknown' crates --include=*.rs | grep -v '/tests/' | wc -l` 输出 `0`；㊁ 拒收有、对象级没有，还会静默退回），建议拆成「㊀ 未实现；㊁ 拒收已实现，对象级未实现」。这一行归主 agent 判。

### V2.6 什么现象会推翻 V2 的结论

- 崩溃点重放里找到 F2 真退回而失败种类不在白名单三种里（甲要改成黑名单加种类，或扩白名单）。
- 找到一个「已确认且读者拒收内容」的盘面，退回上一版是用户要的（例如把「读者比池新」当成可降级读）：那是用户改 ㊁ 与 F2 的知情范围（丁）。
- 今天的写者在盘不坏时造出「R 已返回而系统配置没见证 R」（调查员的前提；不是我复核的，我复核的是它给出的读数）。
## V3　D5 已定项 4 的依据句、「树高」与「代价曲线」几句

### V3.1 五句各自对不对得上产物（量过：`v3_checks.py`，输出 `out/v3_checks.out`）

脚本只读 E97 第五次重跑与 E71 第一次重跑两份入库产物，不读装置源码，不 import 别的腿的脚本。节点几何自证：E97 几何下四档 key 的叶扇出 1087 / 740 / 603 / 542 与产物 `cell_side_by_side` 行的 `leaf_fanout` 逐个相等，`crates/` 几何下 key 22 得 477 / 150，产物 32 行的 `new_leaves` 与本脚本按条目数算的叶数 0 处不等；`V3_BREAK=leaf_without_key_range` 退 3。第一轮攻方的 `w3.out` 是同一类算术，我按它的几行为准核了一遍，数全部相同（叶扇出、`devices_1_to_4096_differing=63`、79..141、四档节点字节），没有引它的脚本。

| 旧句（行号） | 对产物 | 说明 |
|---|---|---|
| 「四档 key 宽的树高恒为 2，多带一维的段宽不构成代价」(`.claude/kb/decisions/05-快照-空间记账机制.md:42`)（已定项 1 依据） | **对不上** | E97 登记的四个网格点上，四档树高每点彼此相同，但值是窄读 1、2、2、2，宽读 2、3、3、3，没有一格「恒为 2」；格点之间树高随 key 宽分开（下面 V3.2 ①）；节点字节随 key 宽涨（22 字节对 7 字节 1.667 倍与 2.000 倍），「不构成代价」只在绝对值上成立 |
| 「按分项计维与维度用满两条臂在同一格差 14.5 倍」(`.claude/kb/decisions/05-快照-空间记账机制.md:145`) 起一句（已定项 4 依据 E71） | **对不上，三处** | ① 14.5 倍是宽读臂对窄读臂，上界臂（维度用满）对窄读臂是 28.3 倍，且只是 `树数 64、设备数 8` 一格；15 格里宽读 / 窄读 1.00 到 128.47、上界 / 窄读 1.00 到 283.52（`e71_ratio_range` 一行）。② 「`1024 × 64` 时宽读那条臂过 1%」：旧口径下宽读臂 22、30 字节宽度都不过（11 759 616、16 035 840 字节），只有 50 字节宽度过（26 726 400），过界的是上界臂（三种宽度都过）。③ 「窄读四格全过」成立：「name=over_budget_count arm=per_statistic_narrow widths=registered_widths cells=45 old_over=0 new_over=0」(`research/results/e71-accounting-keys-single-generation-2026-10-20.out:291`) |
| 「宽读法下最紧的一格占预算 53%」(`.claude/kb/decisions/05-快照-空间记账机制.md:146`)（已定项 4 依据 E97） | **读法写反** | 53% 是**窄读**最紧的一格（旧口径 `8192 × 64`）：「name=narrow_tightest question=Q-E97-4 arm=acct_wide_19B old_node_bytes=11419648 old_grids=8192x64 old_percent=53 old_permille=531 old_over=0 new_node_bytes=458752 new_grids=8192x8,8192x64 new_percent=2 new_permille=21 new_over=0」(`research/results/e97-entry-encoding-single-generation-2026-10-20.out:229`) |
| 「而四档 key 宽的树高恒为 2」(`.claude/kb/decisions/05-快照-空间记账机制.md:159`)（已定项 5 表里统计量类型标签一行） | **对不上** | 同第一行；这一句的用处是替「标签取 2 字节」说话，该用的数是多 1 字节让叶扇出掉多少（下面） |
| 「四档 key（7 / 14 / 19 / 22 字节）的树高恒为 2，代价曲线是平的」(`.claude/kb/decisions/05-快照-空间记账机制.md:169`)（已定项 5 依据 E97） | **对不上** | 一代口径整批重写，代价是写出的叶数、随条目宽走，不是平的 |

### V3.2 改后条文（五处逐句；数字都有出处）

**① 已定项 1 依据（42 行）整条换成：**

> - E97（记账与分配记录的条目编码）：每条根只装一代之后（判据 10 第五次重跑，`research/results/e97-entry-encoding-single-generation-2026-10-20.out`，节点字节只数叶），四档 key 宽（7 / 14 / 19 / 22 字节）的记账树高在 E97 登记的四个网格点（树数 × 设备数 = 64 × 8、1024 × 64、8192 × 8、8192 × 64）上每点彼此相同（窄读 1、2、2、2；宽读 2、3、3、3），格点之间不保证：同一套几何下树数 1 到 8192、设备数 1 到 64 的 524 288 个点里，窄读有 17 440 点、宽读有 92 663 点四档树高不同；`crates/` 今天的行集（池级 3 行 + 每盘 6 行 + 一级台阶行）在 79 到 141 块盘上 7 字节与 22 字节 key 的树高不同。一代口径整批重写，代价是写出的叶数、随条目宽走，不是树高：`1024 × 64` 窄读 22 字节对 7 字节 1.667 倍（81 920 对 49 152 字节），宽读 2.000 倍；`64 × 8` 窄读四档都是一片叶（1.000 倍）。段宽是有代价的，窄读下绝对值小（最大一格 22 字节 507 904 字节，占预算 2.4%）。

**② 已定项 4 依据 E71（145 行）整条换成：**

> - E71（记账 key 的条目数与宽度）：每条根只装一代之后（第一次重跑，`research/results/e71-accounting-keys-single-generation-2026-10-20.out`），条目数比随 `树数 × 设备数` 涨，不是一个常数：30 字节宽度 15 格里宽读臂对窄读臂 1.00 到 128.47 倍、上界臂（维度用满）对窄读臂 1.00 到 283.52 倍，`树数 64、设备数 8` 那一格是 14.53 倍与 28.27 倍。过 `T_dirty` 的 1%（21 474 836 字节）的格：三种宽度 45 格里上界臂 5 格（旧口径 9 格）、宽读臂 3 格（旧口径 5 格）、窄读臂 0 格；`1024 × 64` 一格，上界臂在 22 与 30 字节宽度新口径不过（12 976 128、17 694 720 字节）、50 字节过（29 491 200），宽读臂三种宽度新口径都不过（5 879 808、8 017 920、13 363 200），旧口径只有 50 字节过（26 726 400）；`8192 × 64` 一格宽读臂在今天的 34 字节条目宽（`crates/` 的实现宽度）下新口径仍过。`统计量数 × 树数 × 设备数` 是一代的上界不是条目数（inode 号水位的台阶行除外：它随根留着，条目数随水位变过的发布数增长，不受这个上界管）。

**③ 已定项 4 依据 E97（146 行）整条换成：**

> - E97（记账与分配记录的条目编码）：一代口径（判据 10 第五次重跑，脏节点计）。窄读最紧的一格是 `8192 × 8` 与 `8192 × 64`（并列）：19 字节臂（E97 判的那一臂）458 752 字节占预算 2.1%，22 字节臂（`crates/` 今天的 key 宽）507 904 字节占 2.4%；旧口径同一格 19 字节臂 11 419 648 字节、53%，那是窄读不是宽读。宽读 `1024 × 64` 占 42%（22 字节臂 47%）、`8192 × 8` 占 47%（22 字节臂 53%）不过界，`8192 × 64` 占 337%（22 字节臂 376%）过界（旧口径三格都过界），且那一格反解没有可行的条目宽。这是「哪几项按树分」那一列的代价侧依据之一：今天只有第 12 项带树维、只为可写头维护，付的是窄读那一档的上界，宽读只在最大一格付不起；取舍本身站在「谁要它」上（本项完备性判据）。

**④ 已定项 5 表里统计量类型标签一行（159 行）：** 末尾「，而四档 key 宽的树高恒为 2」换成「；多出的 1 字节让记账叶的容量从 491 条掉到 477 条（`crates/` 几何的算术，少 2.8%），在一片叶装得下的池上不多写节点，多叶的池上最多多写同比例的叶」。

**⑤ 已定项 5 依据 E97（169 行）：** 「四档 key（7 / 14 / 19 / 22 字节）的树高恒为 2，代价曲线是平的」换成「一代口径下四档 key（7 / 14 / 19 / 22 字节）的树高在登记的四个网格点上彼此相同、格点之间不处处相同，代价曲线不是平的——整批重写时写出的叶数随条目宽走（叶扇出 1087 / 740 / 603 / 542；`1024 × 64` 宽读 5 046 272 / 7 405 568 / 9 076 736 / 10 092 544 字节，窄读 49 152 / 65 536 / 65 536 / 81 920 字节）」，分号后面（代段回绕）不动。

### V3.3 攻这五句：产物对得上吗，有没有在一个取样点上说全称

- 每一个数在 `out/v3_checks.out` 里都有一行（`e97_grid_point` 八行、`e71_1024x64` 四行、`e71_over_budget` 六行、`e71_ratio_range`、`crates_leaf_fanout_one_more_key_byte`）；E71、E97 产物里能直接引的行：「name=wide_over_cells question=Q-E97-3 arm=acct_wide_19B old_over_cells=3 old_over_grids=1024x64,8192x8,8192x64 new_over_cells=1 new_over_grids=8192x64」(`research/results/e97-entry-encoding-single-generation-2026-10-20.out:228`)、「name=acct_key_ceiling_new_caliber trees=8192 devs=64 reading=wide touched_entries=2670592 budget=21474836 max_entry_bytes=8 max_key_bytes=0 feasible=0」(`research/results/e97-entry-encoding-single-generation-2026-10-20.out:113`)、「name=over_budget_count arm=per_statistic_wide widths=current_implementation_width cells=15 old_over=1 new_over=1 new_over_cells=current_implementation:8192:64」(`research/results/e71-accounting-keys-single-generation-2026-10-20.out:294`)。
- **全称检查**：第一轮正推的「恒为 2 → 窄读 1、宽读 2」是把 `64 × 8` 一个点（产物里 48 行树高 `acct_geom_new_caliber` 的条目数只有 172 与 2944 两个值，`grep -c` 为证）写成了普遍句；①③⑤ 都带上了取样点，且把「彼此相同」限在四个登记点。
- **不是入库装置一次跑的数（进 kb 前要么主 agent 复跑 `v3_checks.py`，要么把它做成 E97 装置的附带量）**：524 288 个格点里的 17 440 与 92 663、`crates/` 行集的 79 到 141 块盘（第一轮攻方 `w3.out` 同数）、叶扇出 491 对 477。产物里直接有的：叶数、节点字节、过界格、可行条目宽。
- **取舍本身（只有第 12 项带树维）没被这些数打穿**：第一轮攻方 `w3.out` 里今天的十二项表在 `1024` 个头 × `64` 盘的窄读 「name=today_table_single_generation reading=today_narrow_heads heads_or_trees=1024 devices=64 entries=1412 height=2 nodes_rewritten=4 node_bytes=65536 over_budget=0」(`research/prompts/m3-close-c693-r1-opus-model/out/w3.out:34`)，「全部按树分」到 `8192 × 64` 才过界：「name=today_table_single_generation reading=all_items_by_tree heads_or_trees=8192 devices=64 entries=3178496 height=3 nodes_rewritten=6710 node_bytes=109936640 over_budget=1」(`research/prompts/m3-close-c693-r1-opus-model/out/w3.out:37`)（攻方副本模型，没入库）。取舍站在 D5 已定项 4 自己的完备性判据上：「没出现在任何一条判定里的就不要」(`.claude/kb/decisions/05-快照-空间记账机制.md:91`)，与这些数无关。

### V3.4 邻句

D8 已定项 1 射程「两块盘的池上除台阶行外常态 15 行，80 块盘起才分裂」（「两块盘的池上除台阶行外常态 15 行，80 块盘起才分裂」(`.claude/kb/decisions/08-核心索引结构.md:34`)）：不数台阶行 80 块盘起，数一级台阶行 79 块盘起（`crates_split_point` 两行：79 块盘 477 行一层、478 行两层）。建议改成「除台阶行外 80 块盘起才分裂，数一级台阶行 79 块盘起」。

### V3.5 什么现象会推翻 V3 的结论

- 入库装置按今天的十二项表重算，窄读不再是第 12 项的上界（例如每头不止一行）：③ 末尾「付的是窄读那一档的上界」要重写。
- 有人采用宽读（每树空闲字节、每树碎片度的读者出现）：③ 里宽读 `8192 × 64` 那一格从「最大一格付不起」变成取舍的承重点。
- 入库装置里叶数与我的按条目数算的叶数不符（32 行自证只覆盖 E97 登记的四个点）。
## V4　点删与墓碑

### V4.1 缩成什么题：并进 D8 已定项 13，不单列

**缩成的题**：开 write buffer 前端之后，记账行「上一代离开」落成前端里的墓碑（D8 已定项 12 写一律进前端；`crates/` 今天没有前端，是按 key 集合之差在内存里删，第 0 节第一行），排空到叶时，**墓碑对着的是上一次发布写进叶的值**，两者怎么折。答案：以叶上这个 key 的现值为起点、按到达序依次应用本次发布里该 key 的前端条目，墓碑就是删，不取最大。

**为什么现有条款盖不住它（并排）：** 已定项 13 管的是 「E 是墓碑时，折叠的结果就是墓碑，不许把墓碑当数值与 A–D 一起取最大」(`.claude/kb/decisions/08-核心索引结构.md:397`)，A–D 是同一次发布里的中间态；叶上的现值是**上一次发布**写的，不在 A–D 里，字面不管它。已定项 12 说的是 「落叶时同 key 比同一窗口的到达序」(`.claude/kb/decisions/08-核心索引结构.md:383`)，只比前端窗口，也不含叶上现值。D5 已定项 3 那一行是 K 代时的读法：「点删的墓碑不参加取最大——读一个 key 先看最新一条是不是墓碑」(`.claude/kb/decisions/05-快照-空间记账机制.md:75`)。三处各盖一半，起点这一格空着。

**并进 13 的三条理由**：① 规则是同一条（墓碑 = 删，到达序，不取最大），13 已经写了它，只差「起点」；② 检查是同一个（C383 的行本来就写着「墓碑那一半与 C344 同一个判据」：「墓碑那一半与 C344（单调统计量的点删遇墓碑没有会红的检查） 同一个判据」(`.claude/kb/checks-owed.md:277`)）；③ 单列一条要再写一遍同样的规则。**代价**：13 是用户 2026-09-19 只记录的定案，补「起点」是扩它的射程，交用户认（零轮措辞）。

**量过（我的模型，`v4_fold_checks.py`，输出 `out/v4_fold_checks.out`）**：行形、台阶认法、两种落法照第一轮攻方 `w4_frontend_fold.py` 重写（不 import 它；规则与它同一份，所以不算独立路径）。两条路——整棵重建、前端点删加到达序折叠——在三段历史 × 三种台阶规则（`crates`、`per_tree_all`、`per_tree_live`）× 每段 20 版上逐版逐行相同（九行 `two_paths differing_versions=0`），弄坏开关 `V4_BREAK=arrival_keeps_tomb_target` 退 3。朴素取最大折叠：

| 历史、台阶规则 | 落法 | 终态行数 | 陈旧行 | I-3.13 语法 | 条数判 | I-9.30 ① | **⑤ 台阶 H 严格递增（新）** | 活头（新） |
|---|---|---|---|---|---|---|---|---|
| 单头、两次建 inode，`crates` 今天 | 重建 | 18 | 0 | 0 红 | 0 | 0 | 0 | 0 |
| | 点删 + 到达序 | 18 | 0 | 0 | 0 | 0 | 0 | 0 |
| | **点删 + 朴素取最大** | **35** | **17** | **0** | **0** | **0** | **19 版红，第 2 版起** | 0 |

三道今天的检查对朴素取最大 20 版全绿（与第一轮攻方一致，复现）；新检查 ⑤ 第 2 版就红，重建与到达序折叠在三段历史 × 三种台阶规则上 0 处误红。机制：墓碑对着的上一代水位行被取最大留下，成了一行代段早于 txg、值与前一级台阶相等的行，三道语法检查都把它当台阶行。

### V4.2 检查换成什么形

1. **排空对拍**（要前端才有对象）：运行时断言——每次排空，叶上受影响的每个 key 的结果 = 参考重放（叶前值，本次发布内该 key 的前端条目按到达序，墓碑 = 删）。参考重放不与排空共用代码（D11 已定项 3 对陈旧检出的同类要求）。判别力自证：把排空改成「单调统计量取最大」，同一批用例由绿转红（模型：17 行陈旧）。这条换掉 C344 原来的「读一个 key 先看最新一条是不是墓碑」，因为后者是读路径的话，朴素取最大错在排空，读不出来。
2. **I-9.30 加 ⑤：同一 (统计量, 树) 下的台阶行按代段升序 H 严格递增**（台阶是水位变了的那次发布留的一行，相邻两级 H 相等就有一行不是台阶）。**不需要前端**，今天的行集上就能判；配一张坏镜像：好镜像的记账树里补一行（标签 12、树 = inode 树、代段 = txg − 1、值 = 前一级台阶的 H），今天的 I-3.13、条数判、I-9.30 ①②④ 都判不出（模型 0 红），⑤ 红。⑤ 是照台阶的定义推出的形，模型上量过，**真 checker 上没造**；对合法写者不误红的依据是水位单调、台阶认法「值与最后一级不同才算」（「pub fn inode_number_watermark_steps(&self) -> Vec<AccountingEntry> {」(`crates/singlefs-core/src/transaction.rs:3685`) 一段），读码推的。与草案 ③ 同族：「草案 ③（水位大于初值时有一级台阶 H ≥ 水位、g ≤ 最新根的 txg）没判」(`crates/singlefs-checker/invariants/09-inode树结构.md:42`)（C769），⑤ 判的是「多出来的台阶」，③ 判的是「丢了的台阶」，互补，不替代。
3. I-3.13 的语法认法不动（代段早于 txg 的水位行都跳过），改动全放在 ⑤。

### V4.3 删一个可写头时它的台阶行怎么离开

**与 `crates/singlefs-core/src/transaction.rs` 里那条克隆 TODO 是不是同一件事：不是。** 那条 TODO（「`crates/` 今天没有克隆路径」(`crates/singlefs-core/src/transaction.rs:3682`)）要的是「按 (统计量, 树) 分开取、建克隆那次给新头留一行」，是**建头**；它没有一个字说头删掉之后怎么办。checker 侧还有一条孪生的 「`crates/` 今天没有克隆路径，这一判不按树段分头」(`crates/singlefs-checker/src/walk.rs:13074`)，同样只说按头分。条款也是空白：「台阶行随根 COW、一直留着，管理员回退那次从 cur」(`.claude/kb/decisions/05-快照-空间记账机制.md:55`) 那一条只说台阶行一直留着。**按 TODO 字面做完（`per_tree_all`）仍留着删掉的头的台阶行。**

候选与各自的检查（模型 `clone_created_then_deleted`：第 2 次发布建克隆、第 4、5 次在克隆里建 inode、第 10 次删克隆）：

| 候选 | 终态行数 | 陈旧行 | 今天的 ①（不分树） | 活头检查（新） | 按树分的 ①（新） |
|---|---|---|---|---|---|
| H-1 删头那次发布起不再照抄它的台阶行，台阶按 (统计量, 树) 认、只照抄树表里还在的头（重建落法；`per_tree_live`） | 16 | 0 | **5 版误红**（不分树的 ① 在合法形上红，分树之后 0） | 0 | 0 |
| H-2 同上，前端里写成墓碑、到达序折叠（点删落法） | 16 | 0 | 同 H-1 | 0 | 0 |
| H-3 永留（TODO 字面，`per_tree_all`） | 19 | **3 行永留** | 16 版红（不分辨） | **11 版红，第 10 版起** | 0 |

H-1 与 H-2 逐版逐行相同（`two_paths`，`per_tree_live` 与 `per_tree_all` 两行都是 0）。**判法**：H-3 每创建并删除一个头至少留 1 行（模型 3 行 = 建克隆那一级 + 两次建 inode），随头的创建删除无界增长，且活头检查没法放过它，所以不取；H-1 / H-2 取哪个随前端开不开（第 0 节：今天没有前端，H-1），不是两个独立方案。

**要补的条款**（D5 已定项 2 射程「inode 号水位的台阶行」那一条，「克隆头的台阶行按 (统计量, 树) 分，建克隆那次给新头留一行」(`.claude/kb/decisions/05-快照-空间记账机制.md:55`) 之后加一句）：「销毁一个可写头那次发布（分批销毁时，把它的树表条目移出的最后一批）同时不再照抄它的台阶行：台阶行按 (统计量, 树) 认，只照抄树表里还在的头的。」**检查**：I-9.30 加 ⑥「水位那一统计量的每一行，树 ID 都是这一版树表里一个头的树 ID」（活头检查），① 改成按树分（walk.rs 那条孪生 TODO 本来要做的），都配坏镜像（留一行已删头的台阶、两个头的台阶互相串）。
**`crates/` 的改动**（做销毁头那一步时，今天没有这条路径，第 0 节零命中）：`transaction.rs` `inode_number_watermark_steps`（3685 行）与 `accounting_entries_of_this_publish`（6931 行）收「树表里还在的头」集合、按它过滤台阶；销毁头的最后一批发布里把该头条目移出树表与不再照抄台阶放在同一次发布（崩溃点上不会出现「条目没了台阶还在」或相反）；walk.rs I-9.30 的判官同样按头。

### V4.4 两条各落哪个里程碑作业点

1. **墓碑折叠**（缩题后的 C344 并入 C383）→ `.claude/kb/milestone/08-eighth-txn.md` 一·1（「write buffer 前端：走前端的三棵树（记账、反向索引、LRU）写进前端」(`.claude/kb/milestone/08-eighth-txn.md:39`) 那一行）：压着的条款与欠账一格加「C383（含并入的 C344）」，验收加「排空改成取最大的变异判红」。C383 在里程碑六四·1 的那一份（「C383（同一次发布里同 key 的中间态折叠没有检查）」(`.claude/kb/milestone/06-sixth-txn.md:34`)）管「同一次发布装多个事务」那个触发条件，不动；里程碑八 34 行「仓里已有的」里点名 C344 的那一处（「C344（单调统计量的点删遇墓碑没有会红的检查）」(`.claude/kb/milestone/08-eighth-txn.md:34`)）改点 C383。**检查 ⑤（I-9.30 台阶 H 严格递增）不依赖前端**，归 `.claude/kb/milestone/04-fourth-txn.md` 八·1（「删除机制的连带欠账：墓碑树、谱系与退役容器、inode 号水位几条判据的空白」(`.claude/kb/milestone/04-fourth-txn.md:157`)，C769 也在这一行），验收加「补一行等 H 的台阶的坏镜像判红」。
2. **删可写头时台阶行离开** → `.claude/kb/milestone/14-fourteenth-txn.md` 二·2（「销毁头：分叉点闸、分批续做、释放边界」(`.claude/kb/milestone/14-fourteenth-txn.md:50`)）：压着的条款加 D5 已定项 2 射程新句，验收加「销毁中途崩溃：台阶行与树表条目同发布离开，留一行已删头的台阶的坏镜像判红」；它的前提是一·1（「每头一棵树、树表里的头、引用计数旁表」(`.claude/kb/milestone/14-fourteenth-txn.md:39`)）里把 `transaction.rs` 那条 TODO 和 walk.rs 的孪生 TODO 做掉（台阶行与 I-9.30 ① 按 (统计量, 树) 分），C765 已经点名在这一行。

### V4.5 要动的条款、欠账（给 kb-scribe 现查行号）

| 位置 | 动什么 |
|---|---|
| D5 已定项 3 第 3 行（「点删的墓碑不参加取最大——读一个 key 先看最新一条是不是墓碑」(`.claude/kb/decisions/05-快照-空间记账机制.md:75`)） | 合并规则那句后加「取最大只用来排扫描重建时盘上同一个 key 的多个值（依据 D5 已定项 2 射程：「扫描重建按已定项 3 / 已定项 10 的合并规则排同一个统计量的新旧要看它」(`.claude/kb/decisions/05-快照-空间记账机制.md:53`)）；前端排空到叶、读一个 key 时前端与叶并存都不取最大，按 D8 已定项 13：以叶上现值为起点按到达序应用，墓碑就是删」，删掉「读一个 key 先看最新一条是不是墓碑…」那半句 |
| D5 已定项 3 欠（「**欠**：C344（单调统计量的点删遇墓碑没有会红的检查）。」(`.claude/kb/decisions/05-快照-空间记账机制.md:87`)） | 改成「**欠**：C383（同一次发布里同 key 的中间态折叠没有检查；C344 并入）。」 |
| D8 已定项 13 定案（「E 是墓碑时，折叠的结果就是墓碑，不许把墓碑当数值与 A–D 一起取最大」(`.claude/kb/decisions/08-核心索引结构.md:397`) 之后） | 加：「**折叠的起点是叶上这个 key 在上一次发布留下的现值（没有则无）**：排空 = 以它为起点、按到达序依次应用本次发布里该 key 的前端条目，墓碑 = 删；起点算在 A–D 之前，不算同一次发布里的一个中间态。开前端之后，记账行的『上一代离开』落成墓碑，对着的就是这个起点（模型：朴素取最大 20 次发布行数 16 → 35）。」同一句里「C344 … 实测…」括注改点 C383 |
| D8 已定项 12 连带（387 行）、欠（393 行） | 387 行「单调统计量的点删遇墓碑（D5 已定项 3 第 2 条）」改成「墓碑与叶上现值的折叠起点（已定项 13）」；393 行欠里 C344 换成 C383 |
| C344 行（「撞不上——它在 write buffer 前端或节点缓冲让同一个 key 多条并存时才发生」(`.claude/kb/checks-owed.md:252`)） | 搬进已还清表：题面缩成「开前端后墓碑与叶上现值怎么折」，并入 C383；检查换形（V4.2 第 1、2 条）；不是销账 |
| C383 行 | 「怎么拦」加排空对拍；去向加里程碑八一·1；「墓碑那一半与 C344 同一个判据」改成「墓碑那一半即原 C344」 |
| 新欠账（号待取） | 删可写头时台阶行怎么离开：条款空白，检查 I-9.30 ⑥ 与 ① 按树分；去向里程碑十四二·2 |
| I-9.30（`crates/singlefs-checker/invariants/09-inode树结构.md` 42 行） | 加 ⑤、⑥；① 改按 (统计量, 树) 分（里程碑十四之前只有一个头，字面不变） |

### V4.6 什么现象会推翻 V4 的结论

- 前端设计把「上一代离开」不落成墓碑（例如整树重建仍不经前端）：V4.1 的题就没有对象，C344 才是销账；但那样 D8 已定项 12 的检查句「前端树叶上的每一处逻辑改动都对得上一条前端条目」要给「key 消失」开例外（第一轮攻方的说法，读码推的）。
- 找到一类记账 key，代段不等于写下它那次发布的 txg 且值会被改（台阶行以外）：V4.1 的结构理由破了。
- 真 checker 上 ⑤ 在合法写者的镜像上红（例如管理员回退让相邻两级 H 相等）：⑤ 的形要改。
## 判定一览

标记：量过 = 本腿跑出输出（模型目录 `out/` 或草稿目录日志）；推的 = 读条款或读码推的。判定按每一格答的那件事：一致 / 冲突 / 规则没说。

| 格 | 判定 | 一句话 | 量过 / 推的 |
|---|---|---|---|
| V1.1 E186 第二次跑的判决撑不撑得住 | 一致 | ㊀ 不翻面撑得住，但分量是「没有合规读法让别的臂严格赢」：16 种读法组合里 `Carry` 在 14 种处处在胜者里，只有第一次跑的读法（已判写错）里 `r_tree_dim = 1` 的 3200 点不在；`unique_carry` 是「无翻面证据」不是「`Carry` 最优的测量」；`NoCarry` 比 `Carry` 多改 383 半句（kb 里数得出）| 量过（读法矩阵）+ 推的（条款数） |
| V1.2 W1-丁「按节点数第 7 位分不开」写进射程 | 规则没说 | 条文补一句代价信息：E186 取样区 0 节点（每格每代写行至多 47 对叶 477），盘数过 78 后 2‰ 到 16‰ 的点多一片叶，「0 节点」不是全称 | 量过（节点扫，模型算术） |
| V1.3 ㊀ 的措辞三处 | 冲突 | 「只换代号」对台阶行不成立、「不搬的代价」是 K 代说法、第一轮条文漏了「同一行既是现行行又是台阶行」，改后条文三条逐句已写 | 推的 |
| V1.5 ㊀ 的 `crates/` 落点 | 冲突 | 旧写者遇到不认识的树今天先死在「树的种类没登记」，㊀ 到不了；前提 1 处、搬运本身 3 处 | 量过（种类码一栏）+ 读码 |
| V1.6 W1-戊「按统计量的规则」 | 规则没说 | 不要：第 6 位比 `Carry` 多（也要改 261、282、287、383，另加冻结的登记表一列）；零轮 | 推的 |
| V2 ㊁ 与实现的出入 | 冲突 | 未知位今天走整表拒加 F2 退回，没见证的两种盘面三处静默落到 R 之前一版；范围不止 inode 条目，记账、livelist、退役树条目都是；认下的代价要两个条件，这一形只有一个 | 量过（调查员 9 个盘面 + 本腿加的 18 个，每个三处；调查员读数复现 59 行逐行相同） |
| V2 改法甲（F2 退回分种类，白名单） | — | 未知位与种类码两栏都不再静默退回，真损坏对照照退，已有 30 条 F2 用例照绿；不等 deadlist 合并 | 量过（草稿副本补丁） |
| V2 改法丙（对象级） | — | 对调查的盘面（inode 条目）落点与甲相同；收益只在非核心树：恢复与只读照读 R、可写拒；随 deadlist 合并 | 量过（近似）+ 推的 |
| V2 与根环 F2 改法的交集 | 冲突 → 可组合 | C755 并集判退与甲改同一批函数，必须同批；组合规则「任何一处看到内容被拒，三处都不退」是零轮；「退回分支要分种类」：要，白名单 | 推的（崩溃重放里 F2 真退回的失败种类分布没量，交主 agent） |
| V2 落点与欠账 | 规则没说 | 甲与 C755 同批；丙随里程碑四一·2；三笔新欠账；D8 总表第 8 行「进度：已实现」不对 | 推的 |
| V3 D5 五句 | 冲突 | 三句对不上产物（树高「恒为 2」、14.5 倍与「宽读过 1%」归错臂、53% 读法写反），改后条文五条逐句已写，每个数都有产物行或 `v3_checks.out` 行；不在入库装置里的数标明 | 量过 |
| V4 缩题与并入 | 冲突 | 第 3 行墓碑半句与 C344 不销账；缩成「前端墓碑对叶上上一次发布的现值怎么折」，并进 D8 已定项 13（补折叠起点一句，交用户认） | 量过（模型）+ 推的 |
| V4 检查换形 | 规则没说 | 排空对拍 + I-9.30 加 ⑤（台阶 H 严格递增，不依赖前端）；⑤ 在模型上第 2 版红、合法历史 0 误红，真 checker 没造 | 量过（模型） |
| V4 删头时台阶行 | 规则没说 | 与 `crates/singlefs-core/src/transaction.rs` 里那条克隆 TODO 不是同一件事（TODO 是建头）；取 H-1/H-2（删头那次发布不再照抄），H-3（TODO 字面）永留 3 行 | 量过（模型） |
| V4 两条的作业点 | — | 墓碑折叠 → 里程碑八一·1（检查 ⑤ 归里程碑四八·1）；删头台阶行 → 里程碑十四二·2（前提一·1） | 推的 |

## 要主 agent 跑的命令

1. 崩溃点重放里 F2 退回的失败种类分布（决定甲的白名单够不够；重型，归主 agent，轻型三条之一）：在一份仓副本里 `python3 -I research/prompts/m3-close-c693-r2-sonnet-model/v2/apply_variant.py --repo <副本> --variant whitelist`，然后在副本根 `SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=crash-point-replay.sh --quick; 范围=快档登记的项; 为什么=核 F2 退回分种类补丁在崩溃状态上没有新红' bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/crash/crash-point-replay.sh --quick`，对照未打补丁的同一条命令；看有没有一个状态在补丁后由「退回成功」变成「拒」。全量（不带 `--quick`）是重型，提交时跑。
2. 复跑本腿全部读数：`bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-c693-r2-sonnet-model/tools/rerun.sh --repo . --frozen research/prompts/m3-close-c693-r2-snapshot --work <新的草稿目录>`（外面加范围声明；V2 一段要编 harness 档，共用内存额度占满时设 `RUN_WITH_MEMORY_CAP_WAIT_SECONDS=21600`）。看末尾 `name=rerun_compare` 八行是否都 `identical=1`。
3. 进 kb 前把这几个数做成入库装置的一次跑：V1 读法矩阵与节点扫（E186 装置加 `nodes` 与 `u5` 读法两个附带量）、V3 的 524 288 点扫与叶扇出 491 对 477（E97 装置加网格点之间的树高附带量）。

## 没做什么

- 不替主 agent 采纳或判决；没改 kb、`crates/`、装置、产物；主工作区 `crates/` 一个字没动（`diff -rq` 副本与主工作区，副本比主工作区多两份测试文件）。
- 没读禁读清单（第二轮攻方的报告与模型目录）。读了第一轮两腿的报告与攻方模型、调查报告、E186 页与产物、四份 D（D5、D8、D23 的 F2 段）、checks-owed 相关行、里程碑 04、06、08、14、17 的相关行。
- 没跑重型测试、没跑 checker 档、没跑形式化与变异轻型；V2 的 cargo 全是 harness 档测试目标，经内存包装（2G，比派发提示的 4G 上限小）、`capped.sh 31`。过程里两次包装自己的结局：4G 排队 200 秒没排上、2G 排队 220 秒退 250，命令都没跑（日志在草稿目录）；之后改 `RUN_WITH_MEMORY_CAP_WAIT_SECONDS=21600` 起的都跑完。一次排队中的白名单回归与后一个补丁变体撞了源码，我用 `proc.py stop` 先停排队器、再停包装（各一个进程号），那次输出作废，重起一遍（`f2-regress-whitelist2.log`，30 passed）才用。
- 没造的盘面：额外多一条新种类条目（不是改旧条目的种类码）、一块盘读得出一块读不出的混合故障、v2 写者的持久顺序（调查员也没核）、GPU 判官在未知位盘面上的行为（读码：WGSL 里没有解树表条目 flags 的代码）。
- 没把 ⑤ 做进真 checker、没造那张坏镜像（模型上量过）；没量 H-1 / H-2 在真实多头路径上的样子（`crates/` 没有这条路径）。
- 没做的第二次独立抽样：本腿一条一次抽样，「没打中」的格（V1.5 第 5 条、V1.6）按规则算「没被攻过零轮」，主 agent 现查。

## 留下与删掉的草稿

- 留下，仓外，没入库：`/home/fy5090/.local/share/singlefs/work/m3-close-c693-r2-sonnet/` 下的日志（`baseline.log` 调查员读数复现，`extra-baseline.log`、`blacklist-*.log`、`whitelist-*.log`、`objectlevel-*.log` 各变体的 `W2PROBE` 全文，`f2-regress-blacklist.log`、`f2-regress-whitelist2.log` 两个变体上已有 F2 用例的回归），`seg/`（本报告各段的带占位源稿）、`model/`（模型脚本的开发副本，与模型目录同内容）。没入库的理由：落点表与补丁已进模型目录（`out/v2-summary-*.txt`、`diffs/`），全文日志里还有编译提示和秒数，是否要进 `research/results/` 交主 agent 定。
- 已删：本腿建的仓副本与编译目录 `/home/fy5090/.local/share/singlefs/work/m3-close-c693-r2-sonnet/repo`（含 `target`，3.8G）和补丁前原件副本 `…/pristine`、`…/pristine-repo`；复跑验证用的 `…/rerun2/repo`（含 `target`）。理由：再编要的东西都在模型目录里，`tools/rerun.sh` 每次从仓根现拷。

## cite-check 末行

```
$ python3 scripts/agents/cite-check.py research/prompts/m3-close-c693-r2-sonnet-output.md --root . --background research/prompts/_m3-close-c693-r2-background.md
  ✓ 核了 76 处引文、0 处裸行号，对上 76 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```

## 复跑验证的范围

`tools/rerun.sh` 整条跑过一遍（草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-c693-r2-sonnet/rerun2`，经内存包装 2G）：`name=rerun_compare` 八行都 `identical=1`，`name=v2_f2_regress exit=0 passed=30`。这一遍之后只改了 `v4_fold_checks.py` 的 `two_paths` 一段（由 `crates` 一种台阶规则扩到三种、九行）并重生成它的 `out/v4_fold_checks.out`；该脚本单独再跑与 `out/` 逐字节相同（`cmp` 无输出），没有再整条重跑。模型目录文件的 sha256 以本报告开头为准。
