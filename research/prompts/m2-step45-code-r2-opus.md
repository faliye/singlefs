你是里程碑「第二个事务」步 4 / 步 5 代码三方对抗**第二轮**（轮名 m2-step45-code-r2）的**云端攻方腿**。第一轮打中五格、主 agent 改了五处（判决 `research/prompts/m2-step45-code-r1-main-verification.md` 第四、五节）。这一轮**只攻这五处改法本身**——不再问「原来的代码错在哪」，问「改法有没有改对、有没有引进新的错」，而且不许重复第一轮攻过的角度（第一轮攻方报告 `m2-step45-code-r1-opus-output.md` 可读，用来避开）。

## 先读

1. 背景材料 `research/prompts/_m2-step45-code-r2-background.md`（正文 S1–S6 改法表、X1–X7 判据、分工、禁读清单；小节清单；附录一条款整段）。引条款从附录抄整行，行号写 kb 文件自己的（去 kb 文件里现查）。
2. 附录二 `research/prompts/_m2-step45-code-r2-diff.md`。
3. 第一轮判决与第一轮四条腿的报告（可读、不重复它们的角度）。
4. 自己去读代码（原仓只读）：`crates/singlefs-core/src/mount.rs`（`mount_rollback`、`rebuilt_allocator`、`reclaim_floor`、`raise_rollback_floor`）、`allocator.rs`（`isolate`、`rebuild_from_records`）、`recovery.rs`（`effective_rollback_floor`、`readable_roots`、`allocation_records_under_root`）、`crates/singlefs-checker/src/walk.rs`、用例 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`second_transaction_step_five_reuse.rs`。
5. 要改代码跑用例，把仓拷到你自己的目录 `/tmp/claude-1000/m2-step45-code-r2-opus/copy/`（`rsync -a --exclude target --exclude .git`），在副本上改、在副本上 `cargo test`；副本上的数照实报、注明是副本。草稿也放 `/tmp/claude-1000/m2-step45-code-r2-opus/`。

## 分到的攻击面（只攻这几格；X4、X5、X6 是本地攻方的，不碰）

- **X1 P2 的代价**：被抛弃的记录留在环里。造历史：所选根退到实例 1 的旧根（B、C、实例 3 的根都读不出）时 B 的记录（jsn 4、同实例、txg 4 > 3）被接上——这与 C332 那一族是不是同一格、P2 比 P1 多丢什么；所选根是 (1, 4)（B 的根在、之后都坏）时 P2 下它自己那条记录在、链首锚点在、下一条 jsn 5 是实例 2 的——停在哪；被抛弃实例 2 的记录在什么所选根下会被接上；jsn 全池最大 + 1 与 D23 已定项 14 第 3 条「新实例从前缀末 + 1 接着写」字面对不对（前缀末 = 施加的最后一条 = R_old 那条，还是环里最大那条）；环绕圈（24 槽是根环，journal 环是 768 MiB / 4 KiB 个槽）够不着。
- **X2 影子账每次挂载**：`newest_table = None`（最新根的实例表读不出）时按表判的被抛弃根一条都不隔离；被抛弃根的树表或分配记录树读不出时 `allocation_records_under_root` 报错、整个挂载开不了（合不合理，是不是该跳过那条根）；保守读法把 R_old 也引用的槽隔离，它被释放回收之后一直发不出去、账里算空闲——`df` 与实际可分配差多少、D28 第九项只报独占量；`is_released` 跳过：被抛弃根账里已释放、但它的上一版（也被抛弃）还引用的槽算不算它引用（例：C 的账里 B 的数据单元已释放，B 也是被抛弃根——B 的账里它未释放，所以隔离了；有没有一条被抛弃根引用而每条被抛弃根账里都是已释放的槽）。
- **X3 F_生效 做候选集**：抬 F 生效、回收、E 复用 50178 之后，把盘 1 的载体根改坏 ⇒ F_生效 回落到 0 ⇒ txg 0 的根与 A 又是候选，而 50178（第 0 代根的树表单元）已被 E 盖掉——回退到第 0 代根、到 A 各会怎样（`rebuild_version` 报错？走读 panic？成功但读到坏单元？）；候选集用 F_生效、checker 用最新根自己的 F，两边不一致的那段时间 I-2.1 / I-3.1 判什么；`effective_rollback_floor` 按「根落在哪块盘」用 txg 算区域再算盘——根槽写失败落到别处或区域映射变了怎么办。

## 规矩

- 每个打中给出具体的历史或输入，每一步指到被判对象里许可它的那一句（代码路径与行号，或附录原文）。
- 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
- 自己提的改法写明「只在我的模型上量过、被攻过零轮」。
- 没打中的写试过哪些形状、取样范围多大。
- 禁读：`research/prompts/m2-step45-code-r2-*-output*.md` 里别的腿的输出。

## 交付

- **分段写**：报告写进 `research/prompts/m2-step45-code-r2-opus-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。写了模型的放 `research/prompts/m2-step45-code-r2-opus-model/`，报告开头给复跑命令与每个文件的 sha256。
- 报告开头一张「各格判定一览」（X1 / X2 / X3 各一行），然后按格分节，末尾「没打中的形状」「这条腿自己的限度」。
- 引条款整行抄、行号写 kb 文件的；引代码写路径与行号。不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 60 分钟内交出报告。最后回复只写一句指向报告。
