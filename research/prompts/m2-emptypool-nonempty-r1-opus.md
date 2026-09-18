你是里程碑「第二个事务」空池可写挂载与「非空」按树表认的代码三方对抗**第一轮**（轮名 m2-emptypool-nonempty-r1）的**云端攻方腿**。被判的是三处改法：V1「非空持久有效根」按树表里 inode 树与 extent 树的根指针与前一条有效根比（树表读不出就拒绝抬 F）；V2 只做过 mkfs 的池可写挂载（取号 1、零单元写行与暖机、同一进程里发第一个文件版本）；V3 树表 0 条的一版上要写实例表行、回退到树表 0 条的根、实例表或树表不是 mkfs 写的，都在任何落盘之前拒绝。你的立场是造可达的历史或镜像打穿它们。

## 先读

1. 背景材料 `research/prompts/_m2-emptypool-nonempty-r1-background.md`（正文 V1–V3、Z1–Z7 判据、分工、禁读清单；diff；附录条款整段）。引条款从附录抄整行，行号写 kb 文件自己的（去 kb 文件里现查）。
2. 实现员报告 `research/prompts/m2-emptypool-nonempty-r1-implementer-report.md`（它自己列的设计判断与「没做什么」）。
3. 自己去读代码（原仓只读）：`crates/singlefs-core/src/mount.rs`、`recovery.rs`、`transaction.rs`、`allocator.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-checker/src/walk.rs`，用例 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`、`second_transaction_step_three_formatted_pool_layer0.rs`、`second_transaction_step_five_reuse.rs`、`second_transaction_step_four_rollback.rs`、`common/mod.rs`。
4. 要改代码跑用例，把仓拷到你自己的目录 `/tmp/claude-1000/m2-emptypool-nonempty-r1-opus/copy/`（`rsync -a --exclude target --exclude .git`），在副本上改、在副本上 `cargo test`（`nice -n 19`；宿主上门禁 54 号在 release 下跑层 0 全量，你不跑 release 全量枚举）；副本上的数照实报、注明是副本。草稿也放 `/tmp/claude-1000/m2-emptypool-nonempty-r1-opus/`。

## 分到的攻击面（只攻这几格；Z6 是本地攻方的，不碰）

- **Z1 非空判法的漏判与误判**：一次改过用户可见状态的发布，inode 树与 extent 树的根指针会不会与前一条有效根逐字节相同（同样的内容写回、落点复用到同一个槽、位置条目与校验和相同）；一次没改用户可见状态的发布（写行、暖机、抬 F、回退、实例切换）会不会让这两棵树的根指针变（照抄时出生代、位置条目被改写）；「前一条有效根」在回退、两次回退、实例切换、F 刚抬过、被抛弃根还在环里的历史上取到的是不是该比的那一版。造一段历史让上限多算或少算一格。
- **Z2 拒绝的时机**：三处拒绝真的在任何落盘之前吗——`instance_generation_to_acquire` 与 `acquire_instance` 两次算号之间有没有别的写，两盘超级块不一致、一盘槽坏时两次会不会算出不同号；回退与抬 F 的拒绝之前有没有读路径以外的副作用（分配器、录制流、缓存、暖机）。造一段历史让拒绝返回时盘上已经有一次写。
- **Z3 空池挂载之后的第一个文件版本**：`publish_first_file` 写死 txg 3——在什么池上 txg 3 不成立却走进这条路（根环里有 txg > 0 的根、零单元暖机次数不是 2、两盘超级块代号不一）；第一个文件版本换下 mkfs 第 0 版树表时，释放、defer、记账与 mkfs 同一个进程里跑第一个事务那条路逐字节相同吗（在副本上把两条路的镜像逐字节比）。
- **Z4 层 0 第三条流**：闭式与段序列与第一个事务相同——新流的枚举是不是真的跑了重开之后的恢复与挂载（还是只重放了与第一个事务相同的写）；零单元发布的崩溃状态（记录写到一半、根写到一半）恢复出来是什么；checker 在 mkfs 之后与挂载之后报不适用的那 8 条会不会在崩溃状态上掩住违例。在副本上跑快测、读枚举代码，贴原样输出。

## 规矩

- 每个打中给出具体的历史、镜像或崩溃状态，每一步指到被判对象里许可它的那一句（代码路径与行号，或附录原文）。
- 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
- 自己提的改法写明「只在我的模型上量过、被攻过零轮」。
- 没打中的写试过哪些形状、取样范围多大。
- 禁读：`research/prompts/m2-emptypool-nonempty-r1-*-output*.md` 里别的腿的输出。

## 交付

- **分段写**：报告写进 `research/prompts/m2-emptypool-nonempty-r1-opus-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。写了用例或模型的放 `research/prompts/m2-emptypool-nonempty-r1-opus-model/`，报告开头给复跑命令与每个文件的 sha256。
- 报告开头一张「各格判定一览」（Z1 / Z2 / Z3 / Z4 各一行），然后按格分节，末尾「没打中的形状」「这条腿自己的限度」。
- 引条款整行抄、行号写 kb 文件的；引代码写路径与行号。不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 60 分钟内交出报告。最后回复只写一句指向报告。
