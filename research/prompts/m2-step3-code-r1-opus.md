你是里程碑「第二个事务」步 0 / 步 3 那批代码三方对抗**第一轮**（轮名 m2-step3-code-r1）的**云端攻方腿**。假设主 agent 的倾向（这批代码做对了）是错的，造可达的历史、输入或镜像去打穿它。

## 先读

1. 背景材料 `research/prompts/_m2-step3-code-r1-background.md`：正文（被判的对象、S1–S11 选择表、X1–X11 判据、分工、禁读清单）、小节清单、附录一（条款整段）。引条款从附录抄整行，行号写 kb 文件自己的（去 kb 文件里现查，不从背景材料里数）。
2. 附录二 `research/prompts/_m2-step3-code-r1-diff.md`：从提交 5f9e449 起 `crates/` 的全部 diff、两个新文件全文、`crates/mutations.tsv`。
3. 前两轮判决（可读）：`research/prompts/m2-code-r1-main-verification.md`、`research/prompts/m2-code-r2-main-verification.md`。
4. 自己去读代码（原仓只读）：`crates/singlefs-core/src/mount.rs`、`recovery.rs`、`transaction.rs`、`allocator.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-checker/src/walk.rs`、用例 `crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs`、`second_transaction_step_zero_layer0.rs`。
5. 要改代码跑用例，把仓拷一份到你自己的目录 `/tmp/claude-1000/m2-step3-code-r1-opus/copy/`（`rsync -a --exclude target --exclude .git`），在副本上改、在副本上 `cargo test`；副本上的数照实报、注明是副本。草稿也放 `/tmp/claude-1000/m2-step3-code-r1-opus/`，不放别处。

## 分到的攻击面（只攻这几格；X3、X5、X7、X11 是本地攻方的，不碰）

- **X1 前缀五条**：S1 与 D23 已定项 14 五条口径逐条对。重点：所选根自己那条记录读不出时 `expected_next = None`、从候选里最小的一条接——造一段历史让它接上不该接的（例：所选根 (1, 4) 的 jsn 4 两份都坏、jsn 5 也坏、jsn 6 好；或所选根是暖机的根、它自己那条记录在环里被覆盖过）；回退行那第五条今天没有输入，代码是不是把「没有回退行」当成「可以施加到底」；`in_flight_limit` 按实例还是全局。
- **X2 重开重建**：`rebuild_version` 读出的上一版与写它那个进程内存里的 `TransactionOutput` 有哪些字段不同（`rewritten` 空、`record_bytes`、`allocation_records` 的顺序与内容），这些不同会不会漏进写行那次发布（例：`placements_to_release_via_mapping` 拿 `mapped_units` 查、`carried_unit` 照抄指针）；造一份走读能过、但映射条目 / 分配记录 / 树表指针互相不一致的镜像，看重开是报错、panic 还是带着不一致往下发布；`NoPublishedVersion` 拒开的池与 D16 已定项 8 冲不冲突。
- **X4 写行的 T 与 W**：S5 的 T 取「生效根的 txg」（施加记录之后）——对照 D18 已定项 11 与 C330 的字面，造一段历史让下一次恢复按这一行做错（施加过头或停早了）；W 没施加写 0 对不对；中间实例 (i, 0, 0)；写行那次发布事务号 0 与 D23 已定项 7。
- **X6 跨实例的计数**：jsn 全局接着、事务号重置、反向链首条 0——两个实例的记录能不能互相冒充（实例 1 的一条残留 jsn 正好等于实例 2 的下一条）；记录核对器 `check_records` 与 checker 认不认这条链。
- **X8 层 0 到 C**：oracle 假阴性——(1, 4)、(2, 5)、(2, 6)、(2, 7) 四个根同内容，靠 (txg, 实例) 分辨：`newest_persisted_root` 读根槽字节偏移 24 / 28 对不对（对着 `root_record.rs` 的字段表核）；重开接缝合成 4 写一段对不对（进程退出算不算屏障，`reopen_recorded` 做了什么）；直接喂 `oracle_violation_for_versions` 构造一个判错的输入；对照都在到 B 的脚本上，挑一个搬到到 C 的脚本上看它红不红（副本上跑）。

## 规矩

- 每个打中给出具体的历史或输入，每一步指到被判对象里许可它的那一句（代码路径与行号，或附录里的条款原文）。
- 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
- 自己提的改法写明「只在我的模型上量过、被攻过零轮」。
- 没打中的写试过哪些形状、取样范围多大。
- 由用户决定的动作不写死（`.claude/rules/three-way-inference.md` 判决那一节 2026-09-16 那条）。
- 禁读：`research/prompts/m2-step3-code-r1-*-output*.md` 里别的腿的输出。

## 交付

- **分段写**：报告写进 `research/prompts/m2-step3-code-r1-opus-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。写了模型的放 `research/prompts/m2-step3-code-r1-opus-model/`，报告开头给复跑命令与每个文件的 sha256。
- 报告开头一张「各格判定一览」（X1 / X2 / X4 / X6 / X8 各一行：打中 / 没打中 / 不稳定），然后按格分节，末尾「没打中的形状」「这条腿自己的限度」。
- 引条款整行抄、行号写 kb 文件的；引代码写路径与行号。不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 60 分钟内交出报告。最后回复只写一句指向报告。
