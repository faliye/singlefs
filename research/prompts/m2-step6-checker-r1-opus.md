你是里程碑「第二个事务」步 6「checker 新接 I-7.4 与 I-4.8」代码三方对抗**第一轮**（轮名 m2-step6-checker-r1）的**云端攻方腿**。被判的是一处改法 U1：池级 checker 按回退候选集里每条根各判一格 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），判据是走那条根时 I-2.1（校验和与内容匹配） 的违例数或走读失败数有没有变大。你的立场是造可达的历史或镜像打穿它。

## 先读

1. 背景材料 `research/prompts/_m2-step6-checker-r1-background.md`（正文 U1 改法表、Y1–Y6 判据、分工、禁读清单；diff；附录一条款整段）。引条款从附录抄整行，行号写 kb 文件自己的（去 kb 文件里现查）。
2. 自己去读代码（原仓只读）：`crates/singlefs-checker/src/walk.rs`（`check_pool_image`、`Walk`、`walk_root`、`read_referenced_unit`、`visited_units` 的每一处）、`crates/singlefs-checker/src/image.rs`（`Judgements`）、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/src/crash.rs`（层 0 每个崩溃状态怎么跑 checker）、用例 `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`（复用窗口置 0 那条：E 落回 50176、A 是候选）、`second_transaction_step_zero_layer0.rs`。
3. 要改代码跑用例，把仓拷到你自己的目录 `/tmp/claude-1000/m2-step6-checker-r1-opus/copy/`（`rsync -a --exclude target --exclude .git`），在副本上改、在副本上 `cargo test`（`nice -n 19`）；副本上的数照实报、注明是副本。草稿也放 `/tmp/claude-1000/m2-step6-checker-r1-opus/`。

## 分到的攻击面（只攻这几格；Y3 是本地攻方的，不碰）

- **Y1 复用漏判**：同一个 (盘, 槽) 被最新根与更早的候选根各引用一次（复用之后指着同一槽的不同对象）——`visited_units` 让第二次走跳过解容器，校验和那一步还判不判；复用之后新对象与旧位置条目的校验和相同的可能（位置条目校验和是什么、罩哪一段）；被复用的单元只被映射条目引用、不被树表引用时走不走得到；被抛弃根不在候选集里但它引用的单元被盖——这一格 I-7.4 该不该红（条款字面是候选集）。造一段合法历史（固定脚本上的覆盖写、回退、抬 F、复用都可以用）让候选根的单元被盖而 I-7.4 绿。
- **Y2 归错根**：两条候选根共享一个坏单元时违例记在先走的根上、后走的看不到增量：报出来的根 txg 与真坏的根会不会不同；I-4.8 最新根那一格与 I-7.2（最新根走得完） 在每个镜像上是不是永远同红同绿（那就是重复判定）；`violation_count` 只数 I-2.1，走读失败里不经 I-2.1 的那些（头用不了、条目区解不开）算进 `walk_failures` 了吗。
- **Y5 层 0**：层 0 每个崩溃状态都跑 checker：根槽写到一半、单元写到一半、记录写到一半的崩溃状态里，更早的候选根引用的单元是不是恒完整——找一个按恢复语义合法的崩溃状态让 I-7.4 或 I-4.8 红（在副本上把 `second_transaction_step_zero_layer0` 的快用例或到 C 的枚举跑一遍，贴新两条的评估次数与违例）。

## 规矩

- 每个打中给出具体的历史、镜像或崩溃状态，每一步指到被判对象里许可它的那一句（代码路径与行号，或附录原文）。
- 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
- 自己提的改法写明「只在我的模型上量过、被攻过零轮」。
- 没打中的写试过哪些形状、取样范围多大。
- 禁读：`research/prompts/m2-step6-checker-r1-*-output*.md` 里别的腿的输出。

## 交付

- **分段写**：报告写进 `research/prompts/m2-step6-checker-r1-opus-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。写了用例的放 `research/prompts/m2-step6-checker-r1-opus-model/`，报告开头给复跑命令与每个文件的 sha256。
- 报告开头一张「各格判定一览」（Y1 / Y2 / Y5 各一行），然后按格分节，末尾「没打中的形状」「这条腿自己的限度」。
- 引条款整行抄、行号写 kb 文件的；引代码写路径与行号。不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 45 分钟内交出报告。最后回复只写一句指向报告。
