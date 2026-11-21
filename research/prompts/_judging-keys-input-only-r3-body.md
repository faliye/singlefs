# judging-keys-input-only-r3：键只有输入数据，最终轮（2026-11-20）

## 一、题面

第三轮（设计推论轮，最终轮），接 `research/prompts/judging-keys-input-only-r1-main-verification.md` 与 `research/prompts/judging-keys-input-only-r2-main-verification.md`。论证前两轮的结论与判定过程，交主 agent 判断；第三轮之后不开第四轮，这一轮里新冒出来的形态写进判决的交用户表、标「零轮」。

这一轮三面，提示里明令不许重复前两轮已攻过的角度（第一轮：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱；第二轮：第一轮的修补本身、产生结论的方法、从未被看过的地方，加耗时与复用）：

1. 「已定」项撑不撑得住：D13（验证路线） 已定项 17 现行正文（附录）加用户后定的三格（崩溃判定的键 = 块的输入数据 + 判器引到的被检测产品代码的指纹；录入键由产品代码的屏障加场景决定，录得全不全归录入检测；编译档与线程数不进键），这一套作为一个整体，在前两轮挂起的格都不修的前提下，能不能当「已定」交出去；哪几句必须改、哪几句可以照留。
2. 未定项清单的完整性：前两轮交给用户的岔路（第二轮判决「交用户的」那一段）是不是全了；漏了哪一个要用户定的选择；有没有哪一条其实不用用户定、照已定条款就判得出来。
3. 那一轮报出的数字能不能核：前两轮腿报的数（第二轮判决「一、腿与核查」与「二、判决」里引的每一个数），逐个说清是量的还是推的、出处在哪、主 agent 交了读数的那几条（下面第二节）对不对得上。

## 二、实现今天的样子（主 agent 的观测，2026-11-20）

- 集成副本 `~/.local/share/singlefs/work/judging-keys-integration/tree`（主工作区 + 验证一侧与服务一侧两份补丁 + `~/.local/share/singlefs/work/judging-keys-integration/tree-fixes.md` 两处修补）：clippy 全绿；判器引到的产品代码那一组的入口在集成副本 `crates/singlefs-verification/src/judging_units_entries.rs` 第 1266–1276 行（六个入口一组，只留五个产品包的单元）；观察者那几组只留产品包，观察者版本 = 判定项标识 + 这一组的键 + 另读的输入（补丁报告 `~/.local/share/singlefs/work/judging-keys-input-data-only/patch/result.md` 第 17 行）。
- 集成副本构建出的键表（攻方第二轮腿拷进 `~/.local/share/singlefs/work/judging-keys-input-only-r2-opus/keys-table-ed3b64-4e85.tsv.gz`），主 agent 复算：判器引到的产品代码一组 1,770 个单元（singlefs-core 1,673、singlefs-format 64、singlefs-replay-prefix 33）；键表带 10 行 `toolchain`；231 组。
- 主 agent 按两腿给的命令交了读数，日志存在 `~/.local/share/singlefs/work/judging-keys-r3-readings/task-<号>.log`：六个判器入口各自的单元图闭包（`unit-graph closure … --judgment-kind product-behavior`）、`crash_library_reader keys`（库里判过的态按键拆）、验证包在主工作区与集成副本里各一次 `cargo build --release --timings`（构建脚本新旧各要多久）。
- 工具链与依赖版本：用户三次弹窗只问 crates 变异、广谱变异与实验变异表 `mutate.sh` 三族的键（`records/2026-11-20-拆分带出的门禁与变异红.md` 第 248 行起那一段）；崩溃判定与录入键没有用户定案，集成副本照 D13（验证路线） 已定项 17 第一条留着工具链行与收窄的 `Cargo.lock` 行。
- 上一版基线：`~/.local/share/singlefs/work/judging-keys-baseline-costs/result.md`。

推的、没量过的：产品改动在历史提交里多常改到判器引到的产品代码那一组；按比例强制现判与 CPU 抽检各要多付多少挂钟。

## 三、要回答的问题

1. 已定项 17 加用户后定三格，作为一个整体：逐句判「照留 / 必须改 / 要用户定」，必须改的写出改成什么（对 `crates/` 或 kb 的哪一处），依据写文件:行。
2. 第二轮判决「交用户的」那三条岔路之外，还有哪一个选择要用户定；那三条里哪一条照已定条款就判得出。每条写候选的定义、什么观测会让选择翻面、量到哪一步够判。
3. 前两轮判决里引的每一个数：量的还是推的、出处、与第二节读数日志对不对得上；对不上的写差多少、哪一句结论跟着变。
4. 前两轮挂起的格（第二轮判决「二、判决」两张表与「用户后定的格」五条）里，哪几格结论在第二节读数下翻面。

## 四、压着的条款

D13（验证路线） 已定项 17 整条在附录；用户定案原话在 `records/2026-11-19-判定键只算产品与测试代码.md` 第 82–105 行与 `records/2026-11-20-拆分带出的门禁与变异红.md` 第 248 行起那一段；前两轮判决 `research/prompts/judging-keys-input-only-r1-main-verification.md`、`research/prompts/judging-keys-input-only-r2-main-verification.md`。
