# m2-rollback-forward-r3 云端辩方腿（Sonnet）报告

轮名 m2-rollback-forward-r3（设计轮第三轮，最后一轮）。任务：复核第二轮判决 `research/prompts/m2-rollback-forward-r2-main-verification.md`，替被判出局或被打中的一方辩，回答四个指定问题。**全部结论来自对冻结副本 `/tmp/claude-1000/m2-rollback-forward-r3/tree/crates/` 的现读（grep + 逐行读）与对 r1/r2 判决、r2 三条腿报告原文的核对，不新增测试、不建原型**，因此不建 `research/prompts/m2-rollback-forward-r3-sonnet-model/`（该目录只在写了模型时才建）。

## 问① 用户定的 MAX（没有 HOLD）是不是真的必须加 HOLD？X3 那一格要几次故障、在故障模型里算不算合法历史？

**X3 的故障数（现查 `m2-rollback-forward-r2-opus-output.md`）**：该腿自己在 X3 的「四句」第 3 句明写「**故障数：两次崩溃 + 盘 0 上两个根槽坏**」（`m2-rollback-forward-r2-opus-output.md:222`，整句原文：「3. 字面：I-7.4（候选根引用的块被重新分配）与共用问句；故障数：两次崩溃 + 盘 0 上两个根槽坏。」）。拆开看这三件事的性质不同：

- 两次「崩」（步 2：崩在盘 1 那一次落盘之前；步 4：写行发布的单元落了、根没落）都是**逐点崩溃**——在一串正在进行的写序列里选一个中断点，这是崩溃一致性测试的标准做法，不是外部注入的「故障」。r3 正文自己的 K1 问题栏目就明写要测「崩在那次发布的每一点、之后最新 1..k 条根读不出、**回退之后接着写再回退**」（`_m2-rollback-forward-r3-background.md:20`），把「崩溃 → 恢复 → 接着写 → 再崩溃」链起来测是这一轮正文自己点名的范围，不是攻方另外发明的形状；r1 的 H6（`m2-rollback-forward-r1-opus-output.md:175-184`，经 r2 辩方核过，`m2-rollback-forward-r2-verifier-output.md:97-101` ✓）本身就是「暂时读不出 → 撤故障 → 再重开 → 写 → 再改坏 7 条根」的链式历史，同样是崩溃 + 之后再来一次损坏，被判决当作决定性证据用来判「影子账还要」——可见链式的崩溃/损坏序列在本项目里从未被当成「不合法」而排除过。
- 真正的外部损坏（在这个意义上的「一次故障」）只有一件：步 5「盘 0 上 8、9 两条根槽改坏」。这与 X2（「盘 1 上带 F 的根坏，2..3 个根槽」，`m2-rollback-forward-r2-opus-output.md:174`）是**同一量级、同一类**的单点损坏，不是加码。

**结论：X3 是合法历史，够不着「故障太多不现实」这条辩护**——它只比 X2 多了两个标准崩溃点（本项目自己的测试纲领要求测这类链），真正的外部故障计数与 X2 相同（1 次、2 个根槽）。这一条辩护角度**够不着**。

**但「MAX 必须加 HOLD」这句话本身够不着它自己想证的东西**——三点理由：

1. **X3 打中的正是早于这一整轮就已经立案、尚未编号解决的 C419**，不是一个 HOLD 专属才能补的新洞。`checks-owed.md:367`（C419）整行写着：「形态：F 只住根记录、根槽不镜像 ⇒ 那条根坏一个字节 F 就回落，已经被复用的块会被更早的根重新引用。」（`.claude/kb/checks-owed.md:367`）——这与 X3 的机制字面相同（根槽坏一个字节 ⇒ F 回落 ⇒ 已复用的块被更早的根重新引用），而该条目自己写明「用户 2026-09-17 打回重议、至今没有编号」，判别力自证要的实验是「造一份『F 已生效、随后带 F 的那条根被改坏一个字节』的镜像，断言 F 不回落……判别力自证：把 F 的载体改成单份不冗余，第一条必须由绿转红」（同一行）。X3 做的正是这份自证实验的一个具体实例，不是在 C419 范围之外发现了新问题。
2. **HOLD 本身不解决 C419 诊断的病根**。r2 判决自己写「病根是 F 只住在根记录里，与 C419 取哪种规则无关」（`m2-rollback-forward-r2-main-verification.md:58`），依据是 X4：两块盘上带新 F 的根全坏时，`max` 与 `max+hold` 都回落（`m2-rollback-forward-r2-opus-output.md:227` 起，`RBF2 g3_control rule=max ... kill_all_carriers ... F_after=0`）。也就是说 HOLD 只是把「要坏几个根槽 F 才回落」从 2 个（X3）推高到 6..8 个（X4），并没有让 F 的载体变成冗余——C419 诊断要的正是「载体冗余」，HOLD 给的是「时间窗口收窄」，两者不是同一件事。
3. **r2 判决自己交给用户的三选项里没有「必须选 HOLD」这句话**：「交用户的」表里写的是「取最大值（用户 2026-09-25 定；X3 打中）/ max+hold（攻方提；X2、X3 都不中）/ 另记在系统配置里」，且注明「max+hold 与另记都是零轮」（`m2-rollback-forward-r2-main-verification.md:108`）——三个选项并列，没有把 HOLD 定成唯一出路。这句话本身站得住，我没找到 r2 判决文本里出现「MAX 必须加 HOLD」这个强断言；如果这一轮的攻方或后续判决把「X3 打中 MAX」直接读成「必须加 HOLD」，这个推论够不着——它跳过了「HOLD 是否真正解决 C419」这一步（X4 已经现成地否定了这一步）。

**同样打中别的臂**：X3 是设计给分辨臂用的（打中 `max`，不打中 `min_of_max`、`max+hold`），这是它的设计目的，不是缺陷；但 X4（不分辨臂）恰恰证明「靠规则切换」这条路线本身买不到 C419 要的东西，`min_of_max`（今天）、`max`、`max+hold` 三者在 X4 上一样回落，只是需要坏的根槽数不同——这与 K4 正文「每条臂新开的失败面」的问法一致，答案是「HOLD 开的新失败面比它堵的旧失败面更贵才闭合，本身没有全闭合过」。

**推翻条件**：若有人证明「盘 0 上根槽坏两条」这一步本身在合法故障模型里不成立（例如根槽必须成对冗余、坏一条不可能不带另一条坏），则 X3 与 X2 用的这个基础故障就都不成立，两者一起推翻，不只是 X3；目前 kb 里没有这样的条款（`.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 2 只定根环参数，没有定根槽本身镜像——这正是 C419 待重议的那句）。若有人补上「F 另记在系统配置里」的实测（`checks-owed.md:367` 判别力自证那句要的实验），且它在 X3、X4 上都不中，则「必须加 HOLD」这句话的对立候选站得住，问题回到用户在三选项里怎么选，不是我这条辩护能替用户定的。

## 问② I-7.9 保持今天的写法、只给卸载那一串开例外，行不行？

**不行——这不是一个独立于 (a)(b)(c) 之外的第四条路，它在结构上必然坍缩成 (b) 或 (c) 之一，不存在「保持写法不变又能开例外」这回事。**

现查 I-7.9 的判定函数 `judge_rollback_floor_raises_against_their_ceilings`（`crates/singlefs-checker/src/walk.rs:3047-3111`，材料员补注核过的落点）与它调用的 `rollback_floor_ceiling_before_the_raise`（`walk.rs:2966`）：两者的入参只有 `reader: &dyn ImageReader`、`geometry: &PoolGeometry`、`roots: &[(u64, u64, crate::RootView)]`、`cache: &mut IndexNodeCache`——**没有任何入口告诉它「这次抬 F 是从哪条代码路径发出的」**。判定逻辑本身（`walk.rs:3103`：`judgements.judge("I-7.9", raised_floor <= ceiling.lowest_possible, ...)`）只比较「这条根带的 F」与「拿它之前的根算出的上限」，两者都是从磁盘镜像解析出来的数值，不含调用来源。

这与 X6 已经现查坐实的事实完全一致：「同一起点走 B1 卸载，与当准入抬 F 抬到同一个 F，两份盘逐字节相同，I-7.9 判的一字不差」（`m2-rollback-forward-r2-opus-output.md:270-274`），四句第 2 句原文「看不到：按判据自己的那一节，这是『判别子观测不到』——要求 I-7.9 对这两段历史判得不同，**任何只看盘的判法都做不到**」（`m2-rollback-forward-r2-opus-output.md:278`）。这句「任何只看盘的判法都做不到」是一个对判定函数输入的结构性论断，不是只对 X6 那一个构造出来的历史成立——只要 I-7.9 的判定继续只吃 `ImageReader` 解析出来的字节，它就永远分不出「这条抬 F 的根是卸载那一串发的」还是「是准入发的」，因为这两类历史在磁盘上可以逐字节相同。

「保持今天的写法、只给卸载那一串开例外」要求判定函数在**不改变它读什么、不改变它的判据字面**的前提下，多认出一类历史该被豁免。这在逻辑上只有两条出路：
- 给磁盘上多写一个字节（记号），让 `ImageReader` 能读到「这条根是卸载写的」——这就是选项 (b)，且它本身就承认要动格式；
- 让判定挪到一个不是只读磁盘、而是知道调用方是谁的位置（层 0 / 录制流）——这就是选项 (c)，且它本身就承认要放弃池级 checker 覆盖。

没有第三条能做到「函数签名不变、判据字面不变」还能分辨。这条辩护**够不着**：它不是一个更省事的选项，是 (b)/(c) 的一个没写清楚代价的别名。

**同样打中别的形态**：这个论证不依赖 X6 那一个具体历史，是从 `walk.rs:3047-3111` 与 `walk.rs:2966` 的函数签名直接得出的结构性事实，对任何「卸载写出的根」与「准入抬 F 写出的根」在磁盘上恰好同值的历史都成立，不止 X6 造的那一条。

**推翻条件**：若未来 I-7.9 的判定函数改签名、吃进除 `ImageReader` 之外的输入（哪怕只是一个「这次挂载的入口类型」的运行期标记，不落盘），则「不改字面开例外」在技术上可能成立，但那已经不是「池级 checker 只读镜像」的今天这个函数，是选项 (c) 的一种变体（把「知道入口」的信息带进池级 checker），需要重新论证它是否还算「池级」。

## 问③ 「截断删掉不改落点」只量了 13 条根，够不够？

**够——而且「够不够」这个问法本身问错了对象：这不是一个需要更多样本的经验结论，是可以从代码结构直接证明、且 kb 早在这一轮之前就已经写明的必然事实，13 条根验证的是这条结构事实的一个实例，不是在对它抽样。**

现查 `rollback_high_water_of_root`（`crates/singlefs-core/src/recovery.rs:1161-1167`）：
```
pub fn rollback_high_water_of_root(reader: &dyn PoolReader, root: &RootRecord) -> Option<u64> {
    instance_table_of_root(reader, root)?
        .rows
        .iter()
        .find(|row| row.instance == root.instance && row.is_rollback)
        .map(|row| row.applied_transaction_high_water)
}
```
它要求：`root` 自己指着的实例表里，存在一行 `row.instance == root.instance` 且 `row.is_rollback == true`。现查 `instance_rows_to_write`（`crates/singlefs-core/src/mount.rs:1737-1761`）：一次挂载写的行覆盖区间是 `[first_row_instance, instance_to_acquire)`——**严格小于** `instance_to_acquire`（这次挂载要拿到的新实例号），而 `is_rollback: true` 那一行的 `.instance` 字段取的是 `previous_row.instance`（回退目标 `target.instance`，即 R_old 所属的、更早的实例号），同样严格小于 `instance_to_acquire`。也就是说：**任何一次挂载写进它自己那张实例表里的每一行，`.instance` 字段永远小于这张表自己所属的实例号**；而 `root.instance`（对属于这个新实例自己的任何一条根）恰好等于 `instance_to_acquire` 本身。`row.instance == root.instance` 因此对**任何**属于「回退实例自己」的根都不可能成立——这与测的是哪 13 条根无关，是 `instance_rows_to_write` 的取值范围（`mount.rs:1741-1742`：`let first_row_instance = previous_row.instance.0.max(1); (first_row_instance..instance_to_acquire.0)`）决定的一个恒真命题。

这与 kb 里早于这一整轮设计就写明的结论完全一致：`.claude/kb/decisions/23-journal的角色与格式.md:404`（kb 快照同一行）整句「回退行写在新实例的实例表里，只有新实例的根指得到那张表；一次恢复要落到 R_old 上，(txg, 实例) 比 R_old 大的每一条根——新实例的全部根也在内——都得读不出，于是带回退行的那张表从任何一条可读根都够不着，第五条**在任何可达的历史上都取不到真**」，并且直接点名了这个函数：「定案之前实现取 P2、第五条照写但不起作用（预想，`crates/singlefs-core/src/mount.rs` 的 `mount_rollback` 与 `crates/singlefs-core/src/recovery.rs` 的 `rollback_high_water_of_root`）」（同一行 404）。「在任何可达的历史上都取不到真」是 kb 自己的原话，不是我这条辩护的推论；X8 的 13 根实测复现的正是这句话，不是在为它积累置信度。

**同样打中别的形态**：这条结构论证对根环几何（8 槽、双盘）、回退深度、实例代数无关——只要 `instance_rows_to_write` 与 `rollback_high_water_of_root` 的代码不变，对任意条数的根都成立，不止 13 条。

**推翻条件**：若 `instance_rows_to_write` 或 `rollback_high_water_of_root` 的实现改了（例如按 kb 23-journal:404 末句「用户 2026-09-24 定回退见证……第五条怎么改成读见证……随实现交代码三方」那样把判定改成读见证表而不是读实例表自身），这条结构论证连同「删掉截断不改落点」的结论都要重新核——那时函数字面已经不是今天这一份。

## 问④ 第二轮判决里哪一句够不着它引的证据？

**`m2-rollback-forward-r2-main-verification.md:79`**：「第一轮判决 F3 那一行『可以删（推的）』这一轮量过了。」这句话与同一份判决**第 83 行、同一个 G5 小节里、只隔 3 行**的另一句自相矛盾。

现查三处原文：

1. **r1 判决那一行**（`m2-rollback-forward-r1-main-verification.md:42`）：「实例表回退行（flags bit0）与重放遇回退行截断到 W | 可以删（推的） | 攻方原型不写回退行、截断那一支走不到，没攻；下一轮攻」——这一行的判词「可以删（推的）」**同时覆盖两件事**：① 实例表回退行本身（flags bit0 这个标志位）、② 重放遇回退行时的截断逻辑。证据栏自己写明两者在 r1 都「没攻」。
2. **r2 判决第 79 行**：「截断删掉：量过，不改落点。同一份旧镜像上对 13 条根逐条调 `rollback_high_water_of_root`，全部 `None`……第一轮判决 F3 那一行『可以删（推的）』这一轮量过了。」——这句话字面说的是**整行**（即①②两件事）都被这一轮量过了。
3. **r2 判决第 83 行，同一个 G5 小节**：「旧镜像带回退行（flags bit0）：删掉之后的代码照读这一行、照挂，前提是实例表行的解析不动（**推的，没量**）。」——这句明确承认①（回退行本身、flags bit0 的解析）这一轮**没有量过**，与 opus 报告原文一致（`m2-rollback-forward-r2-opus-output.md:311`：「旧镜像带回退行（flags bit0）：删掉之后的代码照读这一行、照挂——前提是实例表解析不动（原型没动）；若删代码时把 bit0 的解析也删成『未知标志就拒』，旧镜像挂不上（**推的，没量**）。」）。

三处并排：r1 那一行判的是①②合在一起的「可以删」；r2 这一轮只测了②（13 条根、`rollback_high_water_of_root` 全 `None`），①（回退行/flags bit0 本身删不删、解析动不动）明确「推的，没量」——就在同一份判决的下三行里自己写着。第 79 行「那一行……这一轮量过了」把「量过了一半」写成了「那一行量过了」，够不着它引用的 r1 那一整行判词，是这一轮判决文本内部的自相矛盾，不需要跨文档才能发现。

**这处够不着不改变 G5 的实质结论**：X8（旧镜像落回被抛弃版本那一格）与截断本身「删掉不改落点」的判定都不依赖①有没有被量过，问②答③的逻辑分析（本报告问③）进一步说明②是一条对任意历史都成立的结构事实。但第 79 行的措辞如果被下游（交用户表、kb 写回）直接引用成「F3 那一行已经验完」，会把①（回退行本身能不能删、旧格式标记解析要不要跟着删）这个仍然「没量」的问题悄悄带过去——r2 判决自己在第 5 节「回看决策」里没有把这一处列成待办，是一个真实的遗漏点。

**同样打中别的臂**：这不是针对某一条实现臂的问题，是判决文本自己在总结「量过了什么」时的记账错误，与 K1-K7 任何一格的实现选择无关。

## 复核表

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| G3 X3「打中，分辨臂」（`m2-rollback-forward-r2-main-verification.md:55`） | **站得住**：故障计数（2 崩溃点 + 1 处根槽损坏，与 X2 同量级）是合法历史，「故障太多」这条辩护够不着 | `m2-rollback-forward-r2-opus-output.md:222`（故障数原文）；`_m2-rollback-forward-r3-background.md:20`（K1 问题栏目要求测链式崩溃）；`m2-rollback-forward-r2-verifier-output.md:97-101`（H6 链式历史核实 ✓） |
| 交用户表「C419 生效规则」暗含的「MAX 必须加 HOLD」读法（若下游这样理解 `m2-rollback-forward-r2-main-verification.md:108`） | **够不着**：X3 是 C419 既有欠账的一个实例，HOLD 本身被 X4 证明不解决 C419 诊断的病根（F 载体不冗余），判决自己列了第三候选「另记在系统配置里」，没有断言 HOLD 是唯一出路 | `.claude/kb/checks-owed.md:367`（C419 原文）；`m2-rollback-forward-r2-main-verification.md:58`（X4「病根……与 C419 取哪种规则无关」）；`m2-rollback-forward-r2-opus-output.md:227` 起（X4 原样行） |
| K6「I-7.9 保持今天的写法、只给卸载那一串开例外」（本轮正文分工表指定要辩护的候选） | **够不着**：结构上必然坍缩成 (b) 记号或 (c) 挪判定位置之一，函数签名（只吃 `ImageReader`）决定它做不到 | `crates/singlefs-checker/src/walk.rs:3047-3111`、`:2966`（现查函数签名）；`m2-rollback-forward-r2-opus-output.md:278`（「任何只看盘的判法都做不到」原文） |
| G5「截断删掉：量过，不改落点」，13 条根（`m2-rollback-forward-r2-main-verification.md:79`） | **站得住，且比字面写的更强**：不是经验采样，是可从 `instance_rows_to_write` 的取值范围证明的恒真结构事实，与 kb 既有条款一致 | `crates/singlefs-core/src/mount.rs:1737-1761`（现查代码）、`:1741-1742`（取值范围）；`crates/singlefs-core/src/recovery.rs:1161-1167`（现查代码）；`.claude/kb/decisions/23-journal的角色与格式.md:404`（kb 快照同一行，「在任何可达的历史上都取不到真」原文） |
| `m2-rollback-forward-r2-main-verification.md:79`「第一轮判决 F3 那一行『可以删（推的）』这一轮量过了」 | **够不着**：与同一小节第 83 行自相矛盾，r1 那一行判的是回退行本身 + 截断两件事，这一轮只测了截断一半 | `m2-rollback-forward-r1-main-verification.md:42`（r1 原判词）；`m2-rollback-forward-r2-main-verification.md:83`（「推的，没量」）；`m2-rollback-forward-r2-opus-output.md:311`（opus 自己的原话） |

## 没做什么

- 不判 K1-K7 本身对不对、代价多少（这一轮分工给 Opus）；没写模型、没建原型、没跑任何测试或崩溃点扫描——全部结论来自对冻结副本源码（`crates/singlefs-core/src/mount.rs`、`recovery.rs`，`crates/singlefs-checker/src/walk.rs`）的现读与对 r1/r2 判决、r2 三条腿报告原文的比对，因此没有 `research/prompts/m2-rollback-forward-r3-sonnet-model/` 目录。
- 没有读禁读清单里的文件：这一轮别的腿的报告与模型（`m2-rollback-forward-r3-opus-*`、`m2-rollback-forward-r3-local-attack*`）、它们的草稿目录、这一轮的核查员报告——全部未读。
- 不判「本地攻方」分工的两张算术表（K4 三条臂回落到 0 要坏几个槽、K3 去重下各模式留几个状态）。
- 不撤销或维持第二轮判决——只交复核结果，判决是否改写、C419 的生效规则最终怎么定，都由主 agent 与用户定。
- 问②只论证了「保持写法不变 + 开例外」这一个具体候选做不到，没有替 (a)(b)(c) 三条改法本身打分——那是 K6 的分工，不归这一格辩护。
- 问①没有替用户在 MAX / max+hold / 另记系统配置三个候选里选一个——只指出「必须加 HOLD」这句话如果被这样读，够不着它自己的证据；三选项的取舍权衡不在这一格的分工范围内。
- 没有编译、没有跑门禁、没有跑层 0 / QEMU / 全量 `cargo test`；本次只用 `Read`/`Bash`（`grep -n`、`sed -n`、`awk`）对冻结副本与 kb 快照现读，没有起任何编译或测试进程，交回前无后台任务、无草稿目录产物需要清理（`target/` 不存在，因为没有编译过）。
