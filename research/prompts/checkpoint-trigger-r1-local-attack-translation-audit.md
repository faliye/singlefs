# 转述核对表：checkpoint-trigger-r1-local-attack（2026-09-22）

逐句核对 `research/prompts/checkpoint-trigger-r1-local-attack.md`（本地攻方：K3 三支打不打架 + K1
里「journal 环占用这个量今天在 crates/ 里存不存在」那一格）里每一条 fact 与中文原文。
kb 行号现查各自文件（工作区 2026-09-22，`git status` 显示这些 kb 文件是未提交的工作区改动；
开工快照 `checkpoint-trigger-r1-start-snapshot.sha256` 与现存文件逐个 sha256sum 核对一致，
无漂移）。格式：英文项（fact 编号）/ 原文文件:行 / 首稿缺的 / 定稿。

提示本身不引用任何文件名加行号（按共用约束「英文提示里的每一句转述」一节与主 agent派发提示
「提示里明令答复不写代码行号与文件行号」），只在这份核对表里写死来源文件与行号；核对表与
运行记录里，样本自带的行号（若模型自己写出行号）一律标「模型自给、未核」。

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| Fact 1: 触发判据原文 | `.claude/kb/decisions/16-发布语义.md:68`（「触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)。两个阈值是系统配置声明值，挂载时求值一次（形态与 D21（权威态与派生态的分界）「单元几何挂载时求值一次」同型）。」） | 首稿把「形态与 D21 同型」压缩成一句泛指的类比，未点名 D21 的编号 | 保留：D21 的具体编号与「单元几何」措辞对本轮四道判断不承重，只需保留「这是同一种模式：也在别处用于某些几何值」这层事实；不点名编号避免引入未加载的决策上下文 |
| Fact 2: 「两种规则」枢纽句 | `.claude/kb/decisions/16-发布语义.md:133`（「T_dirty 与 T_time 是两种规则：T_dirty 走夹取——有效值 = min(字段, 环长 ÷ F)，不动；T_time 走警告——越界不拒绝，只给警告。同一段里两个字段两种规则，不按同一规则读。」） | 无遗漏，逐句对应：夹取规则/警告规则/「不按同一规则读」的告诫三部分均译出 | 按字面译，这是 K3 判断3的枢纽事实 |
| Fact 3: T_time 的值与区间 | `.claude/kb/decisions/16-发布语义.md:122`（节选：「T_time 默认值 5 s，取可行区间 [200 ms, 5 s] 的上端——默认丢失窗口是 5 秒，取更小只减少摊销、不多买任何东西，拦住丢失的是那 5 秒本身。T_time 用户可设，不限制：挂载时传的值覆盖盘上的字段并回写；超出 [200 ms, 5 s] 不拒绝，只给警告（用户 2026-09-20）。」） | 无遗漏；「用户可设，不限制」「超出不拒绝只警告」「2026-09-20 用户定案」三个限定词全部译出 | 按字面译 |
| Fact 4: T_dirty 的值与夹取机制 | `.claude/kb/decisions/16-发布语义.md:122`（节选：「T_dirty = 2 GiB（初值，写代码时再调）；它不由丢失窗口定……T_dirty 是可调值，不是格式常量……挂载时还要再夹一次：有效 T_dirty = min(系统配置里那个字段, 环长 ÷ F)；环长是 mkfs 参数（默认 768 MiB、约束环 ≤ 设备容量 ÷ 4）……默认环下有效值 = min(2 GiB, 256 MiB) = 256 MiB，系统配置里 T_dirty 仍写 2 GiB、字段不动。」） | 无遗漏；「初值写代码时再调」「不由丢失窗口定」「可调值不是格式常量」「环长是 mkfs 参数」「约束环 ≤ 容量÷4」「字段本身不变」六处限定词全部译出 | 按字面译 |
| Fact 5: I-8.1 定义 + 未实现 + 口径注 + 警告 | `.claude/kb/invariants.md:76`（表格行：「环大小 ≥ F × 任一事务的最坏 journal 占用，F ≥ 2……未实现 ⚠️ 口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）：ckpt_cost 按当时各记录树的高现算……哪一种没定。」）与 `:84`（「D23（journal 的角色与格式）的死锁 3 全靠 I-8.1（环几何够大），而 I-8.1（环几何够大） 的 checker 未实现。『写进本清单』不等于『有东西在拦』……」） | 首稿把 D28 已定项 4 的具体编号略去，只译「一个相关的成本数字」；「口径注」日期 2026-09-13 保留 | D28 已定项 4 的编号对本轮四道判断不承重（K1/K4 的射程才需要它，K1 分类与 K4 都不归本地攻方），只需保留「ckpt_cost 现算会长、mkfs 声明值该不该跟着复算」这层未决事实 |
| Fact 6: C432 整行 | `.claude/kb/checks-owed.md:383`（C432 整行五列：简称「一个 checkpoint 间隔的 journal 占用没有条款」；要拦什么列；怎么拦列；前置列；出处列） | 出处列（`.claude/kb/milestone/02-second-txn.md` 与 `research/prompts/m2-s1-r2-main-verification.md` 两个路径指针）未译入 fact 6——这一列只是文件路径指针，本身不含可判断的实质内容，且提示明令答复不许引用文件路径 | 略去出处列，在此登记为有意省略；其余四列（简称/现象/怎么拦/前置）逐字译入，无摘句 |
| Fact 7: 候选乙（第三支）原文 | `research/prompts/_checkpoint-trigger-r1-body.md:27`（「触发 ⟺ (now − 上次 checkpoint ≥ T_time) ∨ (脏字节数 ≥ T_dirty) ∨ (这个 checkpoint 间隔里 journal 环已占用的字节 ≥ 环长 ÷ F)」）与 `:29`（「第三支不依赖 λ、不依赖带宽、不依赖机器快慢：机器快，占用涨得快、它先到；机器慢，时间那一支先到。T_time 因此可以保持「用户可设、不限制、只给警告」（用户 2026-09-20 定案）不改。」） | 无遗漏；公式本身与「不依赖三样」的理由、「T_time 不用改」的结论均译出 | 按字面译；这是 body.md（这一轮的正文材料），不是 kb 文件，行号仅供本核对表内部溯源，未写入提示本身 |
| Fact 8: D23 已定项1「甲」形态 | `.claude/kb/decisions/23-journal的角色与格式.md:40`（「取甲：每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录。祖先不延后。脏叶是全部脏叶，不只被 fsync 的那个文件的：fsync = 一次全量发布。」） | 无遗漏 | 按字面译；只取「甲」的定义句，未带入同一已定项后面「两根轴不正交」等与 K3/K1-存在性 无关的论证段（那些论证段在附录里完整保留，供别的腿引用） |
| Fact 9: 增补1第二轮 I7 发现（WAL类假设） | `research/prompts/m2-s1-r2-main-verification.md:23`（I7 行：「攻方判『不能同真』：λ = 2785 fsync/秒……seq 每次 fsync 8 条 4 KiB 记录 ⇒ 间隔占用 = λ·P·8·4096，T_time 取它自己的默认值 5 s 就越过环 ÷ F（F=3 在 2.94 s 越，F=2 在 4.41 s 越）；T_dirty 夹取救不了——WAL 类下数据在 fsync 那一刻已落盘、不是脏页，那一支永不触发」）与 `_checkpoint-trigger-r1-body.md:19`（「T_time 是时间，环 ÷ F 是字节量，两者之间隔着一个速率；那个速率（λ 与每次 fsync 写多少记录）因机器、因负载而异，而条款里没有任何地方定义它、也没有任何地方要求测它。」） | 无遗漏；λ 数值、8条4KiB记录、越界时刻（2.94s/4.41s）、T_dirty 救不了的理由、以及主 agent 接受的更正（速率因机器而异、条款未定义它）均译出 | 按字面译；明确标注这是「假设的 WAL 类形态」而非今天实际取的「甲」形态，避免模型把 λ=2785 的场景误当成今天系统的真实行为 |
| Fact 10: crates/ 现查（本次会话自己跑的 grep） | 本次会话命令与输出（非中文原文翻译，见下方「命令与输出」一节留痕）：`grep -rn "T_time" crates/ --include=*.rs`（4 处，均在 `crates/singlefs-core/src/system_configuration.rs`）；`grep -rn "T_dirty" crates/ --include=*.rs`（4 处，同一文件）；`grep -rn "checkpoint_trigger" crates/ --include=*.rs`（0 命中）；`crates/singlefs-core/src/system_configuration.rs:141`（注释原文「也没有触发路径——T_time 与 T_dirty 的发布触发没实现」）；`grep -rni "occup\|since_last_checkpoint\|checkpoint_interval\|interval_bytes\|dirty_bytes\|occupied_bytes" crates/ --include=*.rs`（0 命中，除已知的 JOURNAL_SAFETY_FACTOR/in_flight_limit 相关行外无匹配）；`crates/singlefs-format/src/lib.rs:165-169`（`JOURNAL_SAFETY_FACTOR = 3`，`journal_in_flight_record_limit` 函数体：`ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR`） | 无中文原句可核；已现查每一条命令与输出（本报告下方「命令与输出」一节留痕） | 如实转述命令与输出；函数名（`time_threshold_seconds`、`dirty_threshold_bytes`、`journal_in_flight_record_limit`）按名称转述，提示正文未写文件名或行号 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| Judgment 1/2/3/4 全部四条 | `research/prompts/_checkpoint-trigger-r1-body.md:45`（K3 那一行：「三支是 ∨，谁先到谁触发。有没有一格让两支同时到、而它们要求的动作不同？T_dirty 在 WAL 类下永不触发这件事，加了第三支之后还成不成立、要不要顺带改 T_dirty 的定义」）与 `:39-46` 表格里 K1 一行第一句（「这个量在运行时是不是一个随时读得出的量」——本地腿只取其中「今天存不存在」这一半，不取「随时读得出」与「落在 fs-design 哪一格」那两半） | 四道 Judgment 本身是本地攻方腿按主 agent 派发提示「怎么交」一节给的四个问题拆出来的任务框架，不是某一句中文的逐字翻译；Judgment 4 额外加了一句「不评价这个量该不该存在、该落在哪个格」 | 正文 K3 那一行天然含三问（两支同时到动作不同吗/T_dirty永不触发还成立吗/要不要改T_dirty定义），本稿把前两问合并进 Judgment 1、后两问合并进 Judgment 2，另立 Judgment 3 单独处理「第三支该走夹取还是警告」（这是主 agent 派发提示单独列出的第三问），Judgment 4 单独处理 K1 存在性那一半；Judgment 4 末尾那句限定是为了不让本地腿的答复越界踩进「落在 fs-design 三格哪一格」（正推腿的范围）与「随时读得出吗」（这两者都需要判断运行时代价与记账归属，超出「今天存不存在」这个纯事实问题） |
| Fact 9 开头「concerning a hypothetical write ahead log style form rather than the form actually chosen and described in fact 8」 | `research/prompts/m2-s1-r2-main-verification.md:23` 本身没有这句限定，这句限定是本稿从 C432（fact 6）与 D23 已定项1（fact 8）合并推出的 | I7 那一行原文单独读时容易让人以为 λ=2785 的场景是今天系统的真实运行状态；但 fact 6（C432）与 fact 8（甲形态）合起来说明 I7 测的是「重开 WAL 类形态」这个假设情形，今天实际选的是甲、甲下这个缺口不发作。不加这句限定，模型可能把 fact 9 的越界结论当成对今天系统的直接观测，而不是对一个假设替代形态的推演，四道判断都会因此站错立足点 |

## 命令与输出（fact 10 依据，本次会话现查，留痕核验）

```
$ grep -rn "T_time" crates/ --include=*.rs
crates/singlefs-core/src/system_configuration.rs:139:/// 字段表里归这一档的 3 行：可调值段的「T_time 4」「T_dirty 8」「整理低 / 高 / 停三水位 8 × 3」。
crates/singlefs-core/src/system_configuration.rs:141:/// 也没有触发路径——T_time 与 T_dirty 的发布触发没实现，整理三水位恒 0 = 内置默认
crates/singlefs-core/src/system_configuration.rs:151:    /// T_time（D16（发布语义） 已定项 5）。
crates/singlefs-core/src/system_configuration.rs:554:            "系统运行配置：T_time 4 + T_dirty 8 + 整理三水位 24"

$ grep -rn "T_dirty" crates/ --include=*.rs
crates/singlefs-core/src/system_configuration.rs:139:（同上一行）
crates/singlefs-core/src/system_configuration.rs:141:（同上一行）
crates/singlefs-core/src/system_configuration.rs:157:    /// T_dirty（D16（发布语义） 已定项 5）；挂载时的有效值另按环长夹取，那不改盘上这 8 字节。
crates/singlefs-core/src/system_configuration.rs:554:（同上一行）

$ grep -rn "checkpoint_trigger" crates/ --include=*.rs
（零命中，退出码 1）

$ grep -rni "occup\|since_last_checkpoint\|checkpoint_interval\|interval_bytes\|dirty_bytes\|occupied_bytes" crates/ --include=*.rs
（零命中，退出码 1）

$ sed -n '165,169p' crates/singlefs-format/src/lib.rs
pub const JOURNAL_SAFETY_FACTOR: u64 = 3;
/// 在飞记录数上限 = 环槽数 ÷ F，语义是重放前缀最多这么多条（D23（journal 的角色与格式） 已定项 18，2026-09-14 用户定案）。
pub const fn journal_in_flight_record_limit(ring_bytes: u64) -> u64 {
    ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR
}
```

## 没做什么（本核对表）

- 未核对 K1 分类、K2、K4 相关的 fact 与 judgment——这份提示只覆盖 K3 与 K1 存在性那一半，
  按分工表不碰另外三面。
- 未判两次抽样之间的答复方向是否一致——那是运行记录与主 agent 的事，不是这份核对表的事。
