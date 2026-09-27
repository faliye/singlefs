# E142 第十九次跑：问题单（2026-09-27）

不是为岔路建的实验，照 `.claude/rules/three-way-inference.md`「交岔路时写岔路单」写同格式的问题单：只写问题与要量的量，不写倾向、不写已有的数。可以原样交给实验设计员。

题面：录制器自实审 B3a-2 起按设备记屏障（一道池屏障两块盘记两行）、原地覆写补第三态（`research/prompts/m2-rev-b3a2-implementer-report.md`），346f5e6 之后几批又改了第一个事务写出的单元数（`m2-rev-b3a3c-implementer-report.md`：第一个文件版本 8 → 12 个单元）；E142 第十八次跑的产物是这些改动之前跑的，门禁 69 号第一类红在「E142 的输入与产物头的指纹对不上」，门禁 52 号核的 `.claude/kb/layout/01-first-txn.md` 第八节段序列登记句等它重出，门禁 55 号拿它当虚机段序列的基准。

| # | 问题 | 要量的量 | 够判条件 | 状态 |
|---|---|---|---|---|
| 1 | 今天的代码上，第一个事务（mkfs → 取号 → 暖机 → 第一个文件）的录制流分几段、每段几写、两态与三态的闭式状态数各是多少 | 装置打出的 `name=segments` 行（mkfs、instance_acquisition、warm_up、transaction、post_mkfs_stream 各一行：段数、`closed_form`、`kinds`）；三态的 `closed_form` 另一行 | 五行都打出来、与 `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs` 钉的数逐字相同；产物头的输入指纹与今天的 `crates/` 对上（门禁 69 号第一类转绿） | 开着 |
| 2 | 第十八次跑的四个新字段与三条条款（D22 已定项 9、D15、回退下界 F）在今天的代码上还成立吗 | `r18_system_configuration_fields_ok`、`r18_root_flags_ok`、`r18_g8_ok`、`r18_g9_ok`、`control_outcome_matrix_ok`、`g7_states_ok` 这几行 | 每行有值；与第十八次不同的逐个写出是哪一处改动带来的 | 开着 |
| 3 | 与第十八次比，段序列里哪些行变了、各因为哪一批改动 | 第十八次产物与第十九次产物逐行 diff | 每处差异点名一个实现员报告 | 开着 |

## 这张单子怎么用

设计员写重跑登记 `research/prompts/e142-r19-prereg.md`（准入 `inputs-changed` 登记 `crates/` 与装置源码），执行员跑完把三行的状态改成「已够判」并给出产物名；`replay.sh` 的 E142 行改指新产物；`.claude/kb/layout/01-first-txn.md` 第八节的句子与状态数由主 agent 派书记员按新产物写。
