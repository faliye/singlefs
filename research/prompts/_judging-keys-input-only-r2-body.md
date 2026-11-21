# judging-keys-input-only-r2：键只有输入数据，第一轮的修补与耗时、复用（2026-11-20）

## 一、题面

第二轮（设计推论轮），接 `research/prompts/judging-keys-input-only-r1-main-verification.md`。第一轮两条腿都打中，七格挂起（判决「二、判决」第 1–7 条）；用户在腿交回后又定了三格：崩溃判定的键 = 块的输入数据 + 判器引到的被检测产品代码的指纹；录入键由产品代码的屏障加场景决定、录得全不全归录入检测；编译档、线程数、工具链与依赖版本都不进键（`records/2026-11-19-判定键只算产品与测试代码.md` 第 99–105 行）。用户另要这一版「比上次耗时和复用上提升」（同一份第 103 行）。

这一轮攻三件事，方案按 `crates/` 今天的实现与集成副本里合好的补丁来谈：

1. 第一轮判决「三、改法」表里的六条零轮改法本身：每条修不修得了它点名的那一格，会不会带来新的假绿、假红或走不出去的红。
2. 用户新定的三格：产品代码进键之后，「判器引到的产品代码」这一截怎么取、取宽取窄各会怎样；录入键只由屏障加场景定，录入检测要回答什么才算「录全了」；工具链与依赖版本不进键之后，换一次工具链会不会让判定在键不变时结局该变。
3. 耗时与复用：新键（加第 1 条里被采纳的改法）比上一版快多少、多复用多少，花在哪一段；哪一条改法会把省下的又花回去。

第一轮已攻过、这一轮不重复的角度：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱（第一轮的分工）；判器改了对拍不重拍、强制现判只一块、漏判类空向量、观察者不重跑、判法换代无作废手段、态号到态的映射、旧格式待判行、字面冲突这八格本身是否成立（已现查坐实，只攻修补）。

## 二、实现今天的样子（主 agent 的观测，2026-11-20）

- 集成副本 `~/.local/share/singlefs/work/judging-keys-integration/tree`：主工作区经 `scripts/hosts/minimal-copy.py` 取，打了验证一侧 `~/.local/share/singlefs/work/judging-keys-input-data-only/patch/crates.patch`（93 份文件）与变异表、服务一侧 `~/.local/share/singlefs/work/judging-keys-service-side/services.patch`（29 份文件），外加两处修补（`~/.local/share/singlefs/work/judging-keys-integration/tree-fixes.md`）。集成副本上验证包、GPU 服务、函数单元图、实验包的 clippy 都过了，点名测试大半绿、其余在跑；证红没跑成（算键库那一类，归别的会话在修）。
- 对拍键：`crates/singlefs-verification/src/crash_library/pairing_inputs_and_conclusions.rs`（集成副本）第 371 行起的文档写「改产品恢复，对拍重拍；两边判器与比法自己的代码不进」。
- 强制现判每趟每流一块：`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:430`；哨兵 3 块（同一份第 16 行）；给定状态表写死不强制（`crates/singlefs-verification/src/crash_amplification/given_states_engine.rs:1074`）。
- 录入键：`crates/singlefs-verification/src/judging_units_entries.rs` 的 `RECORDING_SCENARIOS`（集成副本，150 行，每条崩溃枚举用例一把，按包过滤在 `crates/singlefs-unit-graph/src/entry_groups.rs`）；块键的写表按屏障切段（`crates/singlefs-verification/src/crash_identity/step_write_table_and_parent_image.rs:17-23`）。
- 上一版基线（调查员报告 `~/.local/share/singlefs/work/judging-keys-baseline-costs/result.md`，数都带出处）：四族最近的不指定范围的趟没有一趟跑完；崩溃放量判完的 9 项驱动合计 142,123 秒，录入段约 84,710 秒（60%，几乎全是观察者逐态录入），等卡约 24,784 秒，判段 32,629 秒；观察者录入每态 1646.26 微秒；崩溃点重放一条大用例 22,440 秒里 22,437.6 秒是观察者录入，各片各编一遍验证包 17–24 分钟；崩溃放量 9 项现判 605,888,874 态、复用 285,868,471 态（32%）；库里镜像判定留下 585 把键、观察者结论 327 把键。

推的、没量过的：新键下同一份库的复用率；第 1 条各改法每趟多判多少态。

## 三、要回答的问题

1. 第一轮「三、改法」六条逐条：它修的那一格修得了没有（构造一个改动，按改法走一遍，看还中不中）；它自己带来的新问题；落在 `crates/` 或 `scripts/` 哪一处、改什么。推不出反例写「推不出」并写查过哪些路径。
2. 「判器引到的产品代码」：从哪几个入口收、按什么过滤成「产品」；收窄了漏哪类产品改动（构造一个），收宽了把哪类非产品代码又带回键里（构造一个）。
3. 录入按屏障：哪几类产品或场景的改动让该录的写流变了而屏障编号与场景键都不变，或反过来；「录入检测」要钉住哪一条判据，落成哪一道会红的检查。
4. 工具链与依赖版本不进键：构造一个工具链或依赖变了、写流与判定结局该变而键不变的例子，或写「推不出」并写查过哪些路径。
5. 耗时与复用：按基线的分段，估新键下每段的变化（量级与出处）；哪一条改法、哪一处实现会让耗时不降反升；要量的数写成能交给任务服务的命令。

## 四、压着的条款

D13（验证路线） 已定项 17 整条在附录；用户定案原话在 `records/2026-11-19-判定键只算产品与测试代码.md` 第 82–105 行；第一轮判决 `research/prompts/judging-keys-input-only-r1-main-verification.md`。
