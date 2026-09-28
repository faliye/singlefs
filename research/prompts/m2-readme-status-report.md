# README 与分片模板：报告

草稿目录 `/tmp/claude-1000/readme-status/`。改动前的 README 备份在 `README.before.md`（sha256 7f0923b2…418e，与动手前仓里的一致，那时 `git diff README.md` 只有「层 0 全量分到两台机器上跑（可选）」一节 18 行新增，是这一批的）。逐处替换的旧串 / 新串在 `edits/e01`–`e18`，都经 `research/scripts/replace-once.py` 定点替换、各命中 1 次。

## 一、仓根 `multi-host.env.example`

- 两份来源逐字节相同：`diff /tmp/claude-1000/gate-batch-m2-g2/multi-host.env.example /tmp/claude-1000/impl-shard-1/deliver/multi-host.env.example` 退 0；两份 sha256 都是 `72ae03d4a01a49ccc069dc27c62a03e31d42dc420e52a879fa67bd4c2c458bd4`。
- 用 `set -o noclobber; cat <G2 那份> > multi-host.env.example` 排他新建，仓里那份 sha256 同上（20 行）。
- 两份脚本在仓里（未跟踪）。⚠️ 它们被 G2 又改过一次，键比对按改后的这一版重做：`layer0-shard-run.sh` 4afc1b9c…952e、`layer0-shard-configuration-check.sh` 12d95b63…4f75，与 G2 `dev/research/scripts/` 下同名文件逐字节相同（`diff` 空）。配置检查这一次只改了 preflight 的 source 行，键表没动。
- 七个键逐字对上：

| 键 | 模板 | 配置检查 `LAYER0_SHARD_CONFIGURATION_KEYS`（第 24–25 行） | 驱动里用到的行 |
|---|---|---|---|
| PEER_SSH_HOST | 有 | 有 | 115、123、136 |
| PEER_REPOSITORY_DIRECTORY | 有 | 有 | 92、126、145、146、167 |
| PEER_CARGO_BIN_DIRECTORY | 有 | 有 | 113、115、132 |
| QUIESCE_STOP_COMMAND | 有 | 有 | 172、173、175 |
| QUIESCE_STOPPED_CHECK_COMMAND | 有 | 有 | 176、177 |
| QUIESCE_START_COMMAND | 有 | 有 | 97、98、103 |
| QUIESCE_STARTED_CHECK_COMMAND | 有 | 有 | 99、100、102 |

  驱动经配置检查的 `--emit-assignments` 取值（`layer0-shard-run.sh` 第 72 行调、第 76 行 `eval`），配置检查把七个键全打出来。
- 模板能被判法读通（在改前那一版配置检查上跑的，改后键表与解析段没变）：`SINGLEFS_MULTI_HOST_CONFIG=$PWD/multi-host.env.example SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1 bash research/scripts/layer0-shard-configuration-check.sh --emit-assignments` 退 0，打出七个键（前三个是占位值，四条清场命令是 `''`）。用了只供测试的开关，没连 ssh。
- 模板里没有真实值：`grep -nE 'faliye|<主机名片段>|fy5090|/home/|<本地模型服务>|tts|192\.168|10\.[0-9]+\.' <模板>` 零命中（退 1）；值只有 `peer-machine-alias`、`/absolute/path/on/the/peer/...` 与空串。仓根的 `multi-host.env` 没读、没改、没拷（只 `git check-ignore -v` 了它的路径）。
- 忽略规则：

```
$ git check-ignore -v multi-host.env.example; echo rc=$?
rc=1
$ git check-ignore -v multi-host.env
.gitignore:19:/multi-host.env	multi-host.env
$ git status --short multi-host.env.example
?? multi-host.env.example
```

  `.gitignore` 里与分片有关的只有第 18 行注释与第 19 行 `/multi-host.env`（`grep -n layer0 .gitignore` 两行）；第 16 行的 `.env` 只配名叫 `.env` 的文件，不配这两个名字。

## 二、README 缺描述的地方（一处一行，都已补）

| # | 缺什么 | 补在哪 |
|---|---|---|
| 1 | 没有讲项目现状的一节（「当前状态」在提交 be22bde 删掉后一直没有） | 新增「## 当前状态」，放在「为什么再造一个」之后 |
| 2 | 分片一节没说哪些用例会分片（只有登记了 `shard=across-machines` 的，今天是两条层 0 流） | 分片一节第 4 段 |
| 3 | 分片一节没说分片怎么跑、怎么并（0/2、1/2、`merge/2`，merge 那趟照单机判法判、写同一格标记）与被杀后续跑 | 同上 |
| 4 | 分片一节没说两台的前提：同一个 rustup 工具链（`rustc -Vv` 前三行与 host 行、`cargo -V` 逐字相同），本机 `rsync` / `ssh`，第二台 `python3` / `git` | 分片一节第 5 段 |
| 5 | 驱动脚本怎么单独调、`--selftest` 做什么、它是重型只在提交时或用户要求时跑 | 分片一节第 6 段 |
| 6 | 目录表没列仓根模板 `multi-host.env.example` | 目录表新行 |
| 7 | 目录表没列 `.claude/rules/` | 目录表新行 |
| 8 | 目录表没列 `.claude/agents/`（连同 `agent-common.md`、`main-agent.md`） | 目录表新行 |
| 9 | 目录表没列 `.claude/hooks/` | 目录表新行 |
| 10 | 目录表没列 `.claude/skills/` | 目录表新行 |
| 11 | 目录表没列 `.claude/warnings/` | 目录表新行 |
| 12 | 目录表没列 `.claude/singlefs-ai-sop/`（git 忽略的共享规范拷贝） | 目录表新行 |
| 13 | 目录表没列 `crates/mutations.tsv`（门禁 59 号复跑的变异表） | 目录表新行 |
| 14 | 目录表没列 `.cargo/mutants.toml` | 目录表新行 |
| 15 | `.claude/scripts/` 写「逻辑在 singlefs-ai-sop」说得不对：`lkmm.sh`（393 行，57 号的逻辑）、`fetch-deps.sh`、`gen-decision-items.py` 是项目自己的，只有 check / doc-lint / env / gate-lint / gate / naming-lint / shell-lint 七份是 12 行的转发包装（另有 28 行的 `preflight.sh` 替阶段找共享的 preflight 库，README 没单列） | 改那一行 |
| 16 | `.claude/gate.d/` 没提 `stage-inputs.tsv`（各阶段输入与崩溃枚举用例的登记表） | 改那一行 |
| 17 | `research/` 只写「实验装置、留存产物与复跑脚本」，没列 `prompts/`（1670 个跟踪文件，三方材料与报告）、`mutations/`、`e7-index-bench/`、`perf-by-milestone.md` | 改那一行 |
| 18 | `.claude/kb/` 写「第一个事务的字节表」说得不对：`layout/` 下有 `01-first-txn.md`、`02-second-txn.md` 两份 | 改那一行 |
| 19 | `crates/singlefs-core` 没提可写挂载与挂着的会话（`mount.rs`、`mounted_session.rs`）、准入（`admission.rs`）、挂载态的读（`mounted_read.rs`）、各棵索引树（`allocation_record_tree.rs`、`extent_tree.rs`、`inode_tree.rs`、`code_two_tree.rs`） | 改那一行 |
| 20 | `crates/singlefs-harness` 没提随机历史、崩溃注入、故障注入、坏盘输入、理想模型、断点续跑与分片（`history.rs`、`crash_injection.rs`、`fault_injection.rs`、`bad_disk_input.rs`、`model.rs`、`layer0_progress.rs`） | 改那一行 |

出处：目录与文件用 `git ls-files | awk -F/ '{print $1}' | sort | uniq -c`、`ls -A .claude`、`ls crates/*/src` 现查；`.claude/scripts/` 各份用 `grep -m3 -nE 'exec |singlefs-ai-sop/scripts'` 与 `wc -l` 现查；`stage-inputs.tsv` 的五行 `crash-case:` 用 `awk -F'\t' '$1 ~ /^crash-case:/'` 现查，`shard=across-machines` 只在 `layer0-first-stream`、`layer0-second-stream` 两行。

## 三、README 的状态句：逐句旧句 / 新句 / 出处

行号是改前 `README.before.md` 的行号。「新增」的句子没有旧句。

| # | 旧句 | 新句 | 出处 |
|---|---|---|---|
| S1 | 无（没有现状一节） | 「**在做第二个里程碑「第二个事务」，还没收尾。**」 | `CLAUDE.md:8`「当前里程碑：**「第二个事务」**…」；`.claude/kb/milestone/03-third-txn.md:3`「里程碑二「第二个事务」…收口之前这份不开工」；`records/2026-09-24-里程碑二收尾调度.md:11-22` 十步表；`ls briefs/` 只有 `2026-09-14.md`（十步的第 9 步「briefs 与里程碑更新」没做） |
| S2 | 无 | 上一个里程碑在两块盘上 mkfs、写一个文件、提交一次、丢掉进程里的一切再冷启动读回来；出口（层 0 全量崩溃点重放零违例）2026-09-14 满足，简报是 `briefs/2026-09-14.md` | `briefs/2026-09-14.md:3-4`；`.claude/kb/milestone/02-second-txn.md:7`「出口（步 7 层 0 全量零违例）2026-09-14 满足」；`CLAUDE.md:8`「出口已经满足」 |
| S3 | 无 | 这个里程碑要在同一个池里再正确提交一个事务，让四件事发生（释放、延迟后重用、第二个可写实例、管理员回退），每件被层 0 罩住 | `02-second-txn.md:18-20` |
| S4 | 无 | 「主线步 0 到步 7 的代码已经落地。」 | `02-second-txn.md:326`「步 0 到步 7 的代码已经落地」 |
| S5 | 无 | 收尾按用户定的十步走（逐项列名），调度与每批交回记在收尾调度 | `records/2026-09-24-里程碑二收尾调度.md:9`（用户原话）与第 13–22 行十步表 |
| S6 | 无 | 代码审阅提出的 38 条正分批在这个里程碑里修 | 同上第 205 行「代码审阅 38 条：…分批「照分批全在里程碑二修」」；之后第 206–229 行多数是这批（实审 A / B / C）的交回与开工 |
| S7 | 无 | C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）那一形用户定了要改 | 同上第 196 行「C554 用户定案：不许认，要改」；简称照 `.claude/kb/checks-owed.md:475` |
| S8 | 无 | 这一版的简报等收尾时写 | 十步表第 9 步；`ls briefs/` |
| S9 | 无 | 下一个里程碑十项设想是草稿、等这个里程碑收口才开工；第六项双机分片用户定了提前做、已接进门禁 54 号 | `03-third-txn.md:1,3`；`records/…收尾调度.md:199`「用户指示把里程碑三第六项…提前到现在做」；`.claude/gate.d/54-layer0-replay.sh` 文件头「双机分片（里程碑三第六项，用户 2026-09-27 定默认不分片）」一段 |
| S10 | 无 | 不能挂到系统里用：没有 FUSE、内核模块、目录与 POSIX 接口；快照、加密、压缩不在这个里程碑；别存数据；格式是软的 | `02-second-txn.md:26-28`「不在里面：目录与 POSIX 面、FUSE…快照与克隆、加密、压缩」；`grep -rnwiE 'fuse\|readdir\|DirectoryEntry\|directory_entry\|dirent\|posix' crates --include=*.rs` 零命中；`briefs/2026-09-14.md:6-7`；`.claude/rules/format-evolution.md` 开头 |
| S11 | 无 | 验证表「崩溃点重放」一行：两条流、登记的崩溃枚举用例逐条跑逐条记标记、固定脚本那条流重写重跑之前不作数、「门禁全绿只构成第一个事务在模型层的崩溃一致性证据」 | `CLAUDE.md:110`（「一句话版本」最后一条，限定词「层 0 用例只改到编得过、下游钉的值没有重核」照抄）；`.claude/gate.d/stage-inputs.tsv` 五行 `crash-case:`；54 号文件头 `--full` 一段 |
| S12 | 无 | 「在用的 79 条不变量里判 46 条」 | `grep -c '^\| I-.*已实现' .claude/kb/invariants.md` → `46`；`.claude/kb/invariants.md:17`「现共 79 条在用」 |
| S13 | 无 | 模型对拍：快档加五个取样点 | `.claude/gate.d/74-model-differential.sh:38` `SECTIONS=(…)` 六个元素 |
| S14 | 无 | QEMU：第一个事务、发布 B、第二个实例、发布 D 与抬 F；只有真实负载，没有崩溃注入 | `.claude/gate.d/55-qemu-first-transaction.sh:4-6` |
| S15 | 无 | 内存序：herd7 判 `litmus/` 每条 Never，每条配去掉屏障的对照 | `.claude/rules/implementation-workflow.md` herd7 那张表那一行；README 目录表 `litmus/` 一行 |
| S16 | 无 | 还没接的：「最终判据」「命名纪律（shell）」没有阶段覆盖 | `.claude/singlefs-ai-sop/scripts/gate.sh:548` `NOT_IMPL_KEYS=("模型对拍" "崩溃点重放" "最终判据" "命名纪律（shell）")`；`grep -rn '^# gate-covers:' .claude/gate.d/*.sh` 只有 54「崩溃点重放」、74「模型对拍」两行 |
| S17 | L35「一块 481 字节的结构」 | 「一块 489 字节的结构」 | `.claude/kb/decisions/22-单元原子性怎么合成.md:189`「合计 **489 字节**」；`crates/singlefs-format/src/lib.rs:226` `SYSTEM_CONFIGURATION_BYTES: u64 = 489` |
| S18 | L46 可变配置「改了付什么」只写「重建索引，**不丢数据**——索引是派生态，能从单元扫回来」 | 后面加「；第一版还没有重建索引这条路，所以今天实际改不了」 | `22-单元原子性怎么合成.md:226`「改它要走重建索引那条路，第一版没有这条路」 |
| S19 | L47 运行配置「**每次挂载都能重设，改了回写盘上**：…整理三水位」 | 同句后加「挂载时传值的通道与 `T_time`、`T_dirty` 的发布触发都还没做，今天按内置默认值写盘」 | `crates/singlefs-core/src/system_configuration.rs:160-162`「挂载选项通道不存在（C422…）…T_time 与 T_dirty 的发布触发没实现…拿内置默认值写盘」；`.claude/kb/checks-owed.md:364` C422 在未还的表里 |
| S20 | L48「系统运行量 \| 52 \| …槽世代号、整槽校验和、journal tail、实例代号」 | 「60」，列表末尾加「回退下界 F」 | `22-单元原子性怎么合成.md:196`「槽世代号 8、整槽校验和 32、journal tail 8、实例代号 4、回退下界 F 8 \| 60」 |
| S21 | L88 QEMU「真实负载 + 崩溃注入下的端到端行为，是准入的最终判据」 | 后加「今天门禁 55 号只接了真实负载与设备侧录制，崩溃注入还没接，所以「最终判据」仍列在门禁的未实现清单里」 | `55-qemu-first-transaction.sh:5-6`「不声明 gate-covers…这一道今天只有真实负载与设备侧录制、没有崩溃注入」；`gate.sh:548` |
| S22 | L90 崩溃点重放「屏障切段、段内任意一组写持久，…（第一个事务的全量在门禁 54 号）」 | 「写请求流按每块盘自己的屏障切段，段内任意一组整写持久（原地覆写多一种「新旧都读不出」的撕裂态），…由门禁 54 号跑：两条层 0 流加登记的崩溃枚举用例，哪些结果作数见「当前状态」」 | `.claude/kb/decisions/13-验证路线.md:69` 起「已定项 4」定案段（按设备切段、枚举单位是一次整写、原地覆写多取第三态）；`CLAUDE.md:110`「负载是两条流」 |
| S23 | L91 模型对拍「随机历史五段取样点」 | 「随机历史的快档加五个偏向某种历史的取样点，一共六段（段名以 74 号的 `SECTIONS` 为准）」 | `74-model-differential.sh:38` 六个元素 |
| S24 | L104 gate.sh 注释「（含层 0 快档与全绿标记核对、QEMU 真设备，要十几分钟；层 0 全量不在里面）」 | 「（含层 0 快档与逐条全绿标记核对、QEMU 真设备、herd7、crates 变异表复跑；层 0 全量不在里面）」；删了「要十几分钟」 | 「十几分钟」查不到量过的数：写它的那一轮自己记着没核（`research/prompts/_defs-gate54-tiering-r2-background.md:996`「「十几分钟」没核」），按「查不到的删掉」删；57、59 在 `gate.sh` 里：`.claude/rules/implementation-workflow.md`「两道都在 `gate.sh` 里」与分工表「54、55、57、59 靠复用」一行 |
| S25 | L106「层 0 全量标 ignored，这里只跑缩小版」 | 「登记的崩溃枚举用例（层 0 全量等）都标 ignored，这里只跑快档与缩小版」 | 五条 `crash-case:` 登记的用例函数前 4 行里各有一个 `#[ignore]`（逐条 `grep -n "fn <名>("` 再看前 4 行，五条都是 1） |
| S26 | L107「单跑层 0 崩溃点重放快档，并核这批输入有没有全量的全绿标记」 | 「单跑层 0 快档（两条流里不标 ignored 的用例），再逐条核登记的崩溃枚举用例各自那一格全绿标记」 | 54 号文件头「整轮门禁的默认」一段；「按整批输入分格的旧标记（singlefs-layer0-full-green.*）不再认」 |
| S27 | L108「判绿按输入哈希写一格全绿标记」 | 「逐条崩溃枚举用例按它自己的输入指纹复用或重跑，判绿写那一条的全绿标记」 | 54 号文件头 `--full` 一段 |
| S28 | L109「单跑 QEMU 两块 virtio 盘上的第一个事务」 | 「…第一个事务、发布 B、第二个实例、发布 D 与抬 F」 | `55-qemu-first-transaction.sh:4` gate-stage 行与六档模式说明 |

没改的现状句：L13「嗯，虽然我现在还在验证，但是俺寻思应该可以。」是作者自己的口吻，「还在验证」今天仍然成立，没动；L35「两块盘合起来全池四份」按两块盘说成立（`make_filesystem.rs:349` 起盘数可变，这句是举两块盘的例子），没动；「开工」一节克隆 `singlefs-ai-sop.git`：`git ls-remote https://github.com/faliye/singlefs-ai-sop.git HEAD` 与 `-zh` 同为 `478d0df9…`（GitHub 改名后的跳转），克隆得到，没动。

## 四、doc-lint

在最终的 README（sha256 `d1bf186c62979f85ada4abb9b6465a191f6ff13647076420665477b46f0afc41`，开跑前后各算一次相同；它逐字节等于备份加 e01–e18 这 18 处替换，`diff expected.md README.md` 空）上跑 `nice -n 19 bash .claude/scripts/doc-lint.sh`，退 1，原样末行：

```
  ✗ 文档铁律检查失败：0 个文件违规、0 处编号引用无定义、11 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 514，跳过 0）
```

11 处都不是 README 带来的：改前的基线同样是 11 处（`doc-lint-before.log` 末行逐字相同）；输出里点名的文件是 `.claude/kb/experiments-history.md`（1 处）与 `.claude/kb/experiments/158-择根与修复四岔路.md`（18 处引用），另有 O1–O3、F23、F24、M32、M34、U11、U13 九条编号登记问题；`grep -c README doc-lint-final.log` 为 0。改完 e17 之后起过一次 `DOC_LINT_VERBOSE=1`（e18 在它跑的过程中换上，读到的是哪一版说不清），输出第 464 行是「✓ README.md」。这 11 处不归这件活，没修。

## 五、没做什么

- 没跑 `layer0-shard-run.sh` 与它的 `--selftest`，没连第二台；模板只用配置检查加只供测试的开关读过一遍。
- `multi-host.env.example` 未跟踪（`??`）：README 目录表与分片一节指向它，要随提交一起进仓。没做任何 git 写操作。
- 两份分片脚本在我干活时被 G2 改过一次；README 按改后那一版写。G2 再改的话，分片一节第 4–6 段要对一遍。
- 看到但不归这件活、没动的：
  - `.claude/gate.d/74-model-differential.sh:4` 的 gate-stage 行写「随机历史快档与三个取样点」，同一文件第 38 行 `SECTIONS` 是快档加五个取样点；README 照 `SECTIONS` 写。
  - `.claude/kb/milestone/02-second-txn.md:476` 并行线一现状说多于一个数据单元被 `UndecidedClauseBlockingMoreThanOneDataUnit` 拒，`grep -rn UndecidedClauseBlockingMoreThanOneDataUnit crates --include=*.rs | wc -l` 为 0，且有 `crates/singlefs-harness/tests/second_transaction_parallel_line_one_multi_unit_file.rs`：那一句多半过时，README 没引它。
  - 本机 git common-dir 里 `crash-case-green` 标记 0 个（`ls "$(git rev-parse --path-format=absolute --git-common-dir)" | grep -c crash-case-green` → 0）：按逐条标记的新判法，还没有一条崩溃枚举用例记过全绿。这是本机状态、不进仓，没写进 README。
  - 「开工」一节克隆的是 `singlefs-ai-sop.git`，真名是 `singlefs-ai-sop-zh`（跳转能用）；没改。
- 没建编译目录、仓副本或 worktree；草稿目录里只有报告、备份、日志与 `edits/`。
