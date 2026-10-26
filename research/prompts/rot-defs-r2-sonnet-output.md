# rot-defs-r2 辩方腿（K1）：第一轮判决第三节那张表落没落、落得对不对（2026-10-19）

立场：辩方，复核 `research/prompts/rot-defs-r1-main-verification.md` 第三节那张「要再改的」表（第 61–74 行）与它依据的逐格判（第 33–53 行），对着批 4 的改动（`research/prompts/_rot-defs-r2-body.md` 第五节甲，diff 行号见下）判：落了没有、落得是不是那一格要的东西；工具员与判决字面不同的三处判是格错了还是改偏了；判决别的格有没有把钩子现状说错。

## 一、现查的基线

- 被判定义的 sha256 对开工快照：`sha256sum -c research/prompts/rot-defs-r2-snapshot/defs-sha256.txt` 十二行里十一行 OK，一行 FAILED：`.claude/hooks/lib_heavy_tests.py`（快照 7cae21b8…，此刻 30f7e232…，改过：自证从工具员报告的 722 种变成此刻 725 种，`python3 .claude/hooks/lib_heavy_tests.py --selftest` 退 0）。八份加四份定义、`run-scope-guard.sh`（1a62a70f…）、`heavy-test-guard.sh`（37426dc2…）、`lib_shell_words.py`（0ba1e4e8…）与工具员报告里记的换上之后的 sha256 一致。下面凡喂钩子的结果，用的是此刻的 `lib_heavy_tests.py`。
- 判定看的 diff：`git diff HEAD -U0` 对八份定义现取，行号是此刻定义里的行号；正文 diff 里对应的行号一并给（`_rot-defs-r2-body.md:行`）。
- 喂钩子：自写的 `feed.py`（草稿目录 `/home/fy5090/.local/share/singlefs/work/rot-defs-r2-sonnet/feed.py`）把命令当 PreToolUse 的 JSON 喂 `.claude/hooks/run-scope-guard.sh` 与 `.claude/hooks/heavy-test-guard.sh`，`agent_type` 按角色填；只喂不起命令。两道钩子的退出码就是下面写的 exit。喂的声明「为什么」一段都填实到 12 字以上（定义里的 `为什么=…` 是待填的占位，原样抄会被拒，这是设计）。
- 轻型自证（`selftest-registry.tsv` 登记为 light）：`python3 scripts/gate/rerun-failed-tests.py --selftest` 退 0「✓ rerun-failed-tests 自检通过（7 格…」；`bash scripts/gate/merge-and-verify.sh --selftest` 末句含「--skip-apply 时补丁反着核得过才往下验、没打在树上的退 1 不编」。

## 二、一句话结论

- 判决第三节那张表（十三份定义、钩子、脚本的行）在主工作区全部落了，没有「没落」、没有字面落偏；表里点名的 A1–A9、B1–B3、正推第 5、10、16、21 项逐格判定见第三节。
- 工具员与判决字面不同的三处：第 1 处（`agent-common.md` 第 87 行）是判决那一格说错了钩子，工具员改对；第 2、3 处是判决那一格语焉不详，工具员改得没偏，第 2 处补进的脚本行为今天为真。
- 判决别的格把钩子现状说错的，我找到一处实打实的：A5 那一格给主 agent 的「数法」`crash-point-replay.sh --list-items --with-keys` 今天是重型（它起 `cargo run -p singlefs-checker-tier`），不带 `SINGLEFS_HEAVY_TESTS` 前缀主 agent 也被拒。另有三处「落了但没写全」的旁注，见第五节。

## 三、判决第三节那张表逐格：落了没有、落得对不对

判定只用三个词：落了 / 没落 / 落偏（落了，但不是那一格要的东西）。「判决行」是 `rot-defs-r1-main-verification.md` 里那一格的行（逐格判）与要再改的表行（第三节）。

| 格（判决行） | 判定 | 证据 |
|---|---|---|
| A1 写回员 `--write` 被范围声明闸按整轮拒（逐格判 36；表行 63、64） | 落了 | 判决那一格要的写法：「能过的只有 `--write --check decision-items-sync` 加 `种类=items` 声明」（`research/prompts/rot-defs-r1-main-verification.md:36`）。定义里：「`doc-decisions.sh --write --check decision-items-sync` 重生成」（`.claude/agents/kb-scribe.md:22`，正文 diff 139/140）；「同一次一并跑 `SINGLEFS_RUN_SCOPE='种类=items;」（`.claude/agents/kb-scribe.md:32`，diff 148–151）；「然后 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）」（`.claude/agents/kb-scribe.md:33`）；「写完正文自己跑一次 `SINGLEFS_RUN_SCOPE='种类=items;」（`.claude/agents/kb-scribe.md:35`，diff 153/154）；主 agent 两处：「各自不跑 `doc-decisions.sh --write --check decision-items-sync`」（`.claude/main-agent.md:29`，diff 219/220）、「`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格」（`.claude/main-agent.md:97`，diff 252/253）。钩子侧：门禁文件头「--write [--check decision-items-sync] [项目根]」（`.claude/gate.d/doc-decisions.sh:39`），`--write` 只重写「重写 decisions.md 的分项清单生成块与索引表」（`.claude/gate.d/doc-decisions.sh:39`），收窄到这一格没有丢刷新的块。喂 `run-scope-guard.sh`：写回员带 items 声明（为什么填实）exit 0；写回员裸 `--write` exit 2；主 agent 裸 `--write` exit 2。没落的残留：`.claude/rules/path-moves.md` 第 33 行还是裸 `--write`，见第五节 H5 |
| A2 攻方 3b 与重型闸冲突（逐格判 37；表行 65、66） | 落了 | 定义：「调枚举函数的原型只 `cargo build`，经内存包装直接执行编出来的二进制（不 `cargo run`、不 `cargo test`）」（`.claude/agents/three-way-attack.md:36`，diff 184/185）；写范围一行带「SINGLEFS_RUN_SCOPE='种类=target; 跑什么=<crate> 的测试目标 <目标>」（`.claude/agents/three-way-attack.md:53`，diff 193/194）。闸侧：「许可的用法：攻方原型 `cargo build` 之后经内存包装直接执行编出来的二进制」（`.claude/hooks/heavy-test-guard.sh:37`），并写明「闸看不出这个二进制链接了什么、不判，守这一句靠攻方腿」（`.claude/hooks/heavy-test-guard.sh:38`）。我在草稿目录造了一个依赖 `singlefs-checker-tier` 的探针包（只有 Cargo.toml，不编），以攻方腿身份喂 `heavy-test-guard.sh`：`cargo build --manifest-path` exit 0；经包装 `cargo run --manifest-path` exit 2；经包装 `cargo test --manifest-path` exit 2；经包装直接执行 `target/release/probe` exit 0；不经包装直接执行 exit 2。与判决那一格「冲突改前就在，这一批只改准了路径」（`research/prompts/rot-defs-r1-main-verification.md:37`）相符，解法的两半（定义与闸）都落了 |
| A3 实验执行员 1b 裸跑 `cargo test` 按整轮拒（逐格判 38；表行 67） | 落了（`scripts/experiment-apparatus` 的装置） | 判决要的：「`cargo test --release --bin e<号>-<简称>` 加 target 声明放行」（`research/prompts/rot-defs-r1-main-verification.md:38`）。定义：「cargo test --release --bin e<号>-<简称>」（`.claude/agents/experiment-runner.md:31`，diff 84/85），前面带「SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 e<号>-<简称> 的单测; 范围=1 个测试目标」（同行）。喂两道钩子，当前目录 `scripts/experiment-apparatus`、命令 `cargo test --release --bin e14-discrimination`（该目录 `Cargo.toml` 里现成的 bin），经包装、为什么填实：`run-scope-guard.sh` exit 0，`heavy-test-guard.sh` exit 0；旧写法裸 `cargo test` exit 2。入库装置（`crates/singlefs-checker-tier/src/bin/e*.rs`）的内联单测没覆盖，见第五节 H3 |
| A4 分诊第 5 步缺 `NOT_RUN_NO_GPU_ADAPTER` 一支与基线不绿（逐格判 39；表行 68） | 落了 | 定义：「基线不绿 N 行」（`.claude/agents/mutation-triage.md:33`，diff 169/171）；「那一行是 `ok`、输出里又有一行 `NOT_RUN_NO_GPU_ADAPTER` 点名它，记」（`.claude/agents/mutation-triage.md:34`，diff 170/172）。与规则同一支：「那一行是 `ok`、输出里又有一行 `NOT_RUN_NO_GPU_ADAPTER` 点名它，记」（`.claude/rules/mutation-sampling.md:79`）；门禁侧「点名的测试打 NOT_RUN_NO_GPU_ADAPTER 后返回成功的那一条不算抓到、不算没红」（`.claude/gate.d/checker-tier-crates-mutation-replay.sh:103`），基线行的写法「基线不绿 0 行」（`.claude/gate.d/checker-tier-crates-mutation-replay.sh:113`） |
| 正推第 21 项 分诊第 2 步与 `mutate.sh` 按条锚点矛盾（逐格判 40；表行 68） | 落了 | 定义：「原文在源文件里不是恰好一次的那几条列出来（第七类），打 `⚓`、不跑，别的条照跑」（`.claude/agents/mutation-triage.md:31`，diff 166/167）。脚本：「不起 cargo、不进按条判定库、不复用；别的条照跑照判、照入库」（`scripts/mutation/mutate.sh:47`）。两处说法一致，第 5 步的 `⚓` 栏走得到 |
| A5 main-agent 第 10 行摘短 D13 已定项 9（逐格判 41；表行 64；另立账 76） | 落了（照格抄）；格里的「数法」命令今天是重型，见第五节 H1 | 判决要的：「数法 `bash scripts/crash/crash-point-replay.sh --list-items --with-keys`」（`research/prompts/rot-defs-r1-main-verification.md:64`）。定义逐字落了：「条件照 D13（验证路线） 已定项 9 那一句（单元键对不上、且对不上的键里有判器单元变了），数法 `bash scripts/crash/crash-point-replay.sh --list-items --with-keys`」（`.claude/main-agent.md:10`，diff 210/211）。kb 那一句现存：「阶段把作废块数与变了的判器单元清单打出来」（`.claude/kb/decisions/13-验证路线.md:256`），摘句问题消除，改成指路 |
| A6 main-agent 第 94 行「不换树」与驱动反着（逐格判 42；表行 64） | 落了 | 判决要的：「不包暂存树、不换版本；判的是主工作区，先照第 2b 步 ① 核它等于暂存区」（`research/prompts/rot-defs-r1-main-verification.md:64`）。定义：「不按提交选条；判的是主工作区，先照第 2b 步 ① 核它等于暂存区」（`.claude/main-agent.md:95`，diff 249/250）。出处两处现存：「没有「一版」，不包暂存树、不换版本：」（`.claude/skills/crash-test/commit-stages.md:23`）与「① 驱动判的是主工作区（`--source` 默认本仓根），要它等于暂存区」（`.claude/skills/crash-test/commit-stages.md:24`），定义与第 2b 步同一说法 |
| A7 main-agent 第 52 行丢规则例外（逐格判 43；表行 64） | 落了（见第四节第 3 处） | 判决要的：「收进括注的指路里，不写短版」（`research/prompts/rot-defs-r1-main-verification.md:64`）。定义：「只改注释的产品代码文件开不开、射程与例外都照」（`.claude/main-agent.md:53`，diff 235/236）。规则现存：「只改了注释的产品代码文件也不走」与「文档注释与 `// SAFETY:` 注释的增删照代码算」（`.claude/rules/implementation-workflow.md:10`），定义不再给结论、整句交规则，没有短版 |
| A8 验证段第 1 步无条件 `--skip-apply`、脚本不核补丁打没打（逐格判 44；表行 64、73） | 落了；定义没写「混批分两趟」，见第五节 H4 | 定义：「写段打过的补丁目录带 `--skip-apply`（只编、只验，脚本先逐份核补丁已打在树上，核不过退 1），没打过的不带（脚本来打）」（`.claude/main-agent.md:51`，diff 232/233）。脚本侧：「对给的每个补丁目录里的 crates.patch 在要验的树上跑 `git apply --check --reverse`」（`scripts/gate/merge-and-verify.sh:48`），「有一份核不过、或给的补丁目录不在，就退 1、逐份点名、一个都不编」（`scripts/gate/merge-and-verify.sh:49`）。`bash scripts/gate/merge-and-verify.sh --selftest` 退 0，末句含「--skip-apply 时补丁反着核得过才往下验、没打在树上的退 1 不编」。判决自己记的「在没打补丁的树上验出绿」没核动（`research/prompts/rot-defs-r1-main-verification.md:82`）现在由脚本自证里造的没打补丁的树补上了 |
| A9 harness 档没带声明；实现员合成一条命令只在开头 export 一份（逐格判 45；表行 69、70） | 落了 | 定义：「SINGLEFS_RUN_SCOPE='种类=target; 跑什么=<crate> 的测试目标 <自己的目标>; 范围=1 个测试目标; 为什么=…' cargo test -p <crate> --test <自己的目标>」（`.claude/agent-common.md:56`，diff 62/63）；「串成一条时每一段各带自己的范围声明，不用一份 export」（`.claude/agents/implementation-writer.md:40`，diff 104/105）。喂 `run-scope-guard.sh`：形式化轻型与 `cargo test -p singlefs-harness --test foo` 各带自己的声明 exit 0；开头一份 `export SINGLEFS_RUN_SCOPE='种类=quick…'` 管到两段 exit 2 |
| B1 八份里没改到的命令句（逐格判 46；表行 64、65、69、72） | 落了 | 六处都有：`--full` 补 whole 声明（`.claude/main-agent.md:95`）；写回的 `--write` 两处（`.claude/main-agent.md:29` 与 `.claude/main-agent.md:97`）；攻方 `cargo test` 补 target 声明（`.claude/agents/three-way-attack.md:53`）；实验执行员 1b（`.claude/agents/experiment-runner.md:31`）；「每条开头已带 `SINGLEFS_RUN_SCOPE='种类=target; …'` 声明」（`.claude/agents/implementation-writer.md:34`，diff 97/98）。脚本侧 `rerun-failed-tests.py` 文件头：「SINGLEFS_RUN_SCOPE='种类=target; 跑什么=<包> <挑目标的参数> 里上一趟红的用例」（`scripts/gate/rerun-failed-tests.py:11`），`--selftest` 退 0（7 格，含打出的命令带声明且闸放行）。主 agent 的 `--full` 整条喂两道钩子（whole 声明加 `SINGLEFS_HEAVY_TESTS=user-request`）都 exit 0 |
| B2 investigator 原样复现整包命令（逐格判 47；表行 71） | 落了 | 定义：「输入命令是整包、整工作区或整道门禁的，收窄到点名的目标或格再复现，收不窄的写进要主 agent 跑的命令」（`.claude/agents/investigator.md:31`，diff 126/127）。喂 `run-scope-guard.sh`：调查员整包 `cargo test -p singlefs-harness` 带 whole 声明 exit 2（与判决「复跑第九格：整包带 whole 声明退 2」（`research/prompts/rot-defs-r1-main-verification.md:47`）同），收窄成 `--test foo` 加 target 声明 exit 0 |
| B3 `run-scope-guard.sh` 文件头旧句（逐格判 53；表行 74） | 落了 | 判决那一格指的旧句在第 32 行。现在：「scripts/gate/merge-and-verify.sh：whole（挑包在脚本里做：只跑补丁改到的包与依赖它们的包，--all-packages 与它文件头写的另几种照旧全跑；」（`.claude/hooks/run-scope-guard.sh:32`）。`grep -c '4-3i' .claude/hooks/run-scope-guard.sh` 现在 0 |
| 正推第 5 项 范围声明闸条目比 hook 实管的窄（逐格判 51；表行 70） | 落了，字面与判决不同，改对了（见第四节第 1 处） | 定义：「崩溃放量三份入口同样要声明（`crash-amplification.sh` 不带 `--item` 是 whole），整轮门禁、`cargo test` 整包与整工作区都是 whole」（`.claude/agent-common.md:87`，diff 71/72）；后半句「命令位置上执行的驱动脚本读进去逐行判，脚本里每条要声明的命令各自带声明」对应钩子文件头「命令位置上执行的别的 shell 脚本」（`.claude/hooks/run-scope-guard.sh:38`） |
| 正推第 10 项 实现员 7a 没要求给合表那一行（逐格判 50；表行 69） | 落了 | 定义：「报告给合表那一行：项名、第二列键、第三列放量分项、第四列轻型取自重型的哪一项」（`.claude/agents/implementation-writer.md:48`，diff 113/114）。合表头的四列说法与之对得上，且「④ 不空、都在 ② ③ 的测试目标里」（`.claude/gate.d/crash-point-replay-items.tsv:10`） |
| 正推第 16 项 main-agent 三处「`--all-packages` 才跑全部」说窄了（逐格判 48；表行 64） | 落了 | 三处都改：「补丁只改 checker 档包时测试本次未跑」命中第 7、29、91 行（`.claude/main-agent.md:7`、`.claude/main-agent.md:29`、`.claude/main-agent.md:91`；diff 206/207、219/220、244/245）。脚本侧另三种全跑：「照旧全跑的：带 --all-packages；没给补丁目录；」（`scripts/gate/merge-and-verify.sh:33`）与「补丁改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/」（`scripts/gate/merge-and-verify.sh:34`），改到不落在任何包里的路径同在第 34 行；定义四项都列了 |
| 表行：`.claude/hooks/heavy-test-guard.sh` 文件头（表行 66） | 落了 | 见 A2 |
| 表行：`scripts/gate/rerun-failed-tests.py`（表行 72） | 落了 | 见 B1 |
| 表行：`scripts/gate/merge-and-verify.sh`（表行 73） | 落了 | 见 A8 |

补一条 A9 的喂法细节：开头一份 `export` 的那条被拒的原因是第二段，原话「声明的是轻型（quick），命令跑的是点名的测试目标（target）」，与判决 A9「按种类对不上拒」一致。

## 四、工具员与判决字面不同的三处：判决那一格错了，还是工具员改偏了

出处：工具员报告 `/home/fy5090/.local/share/singlefs/work/m3-close-w4-defs-and-guards/report.md` 第 134–137 行（「与判决原文字面有出入、要 rot-defs-r2 判的三处」）。

### 第 1 处　`agent-common.md` 第 87 行范围声明一句：判决那一格错，工具员改对

- 判决写：「崩溃放量三份入口、整轮门禁、`cargo test` 整包与整工作区都是 whole」（`research/prompts/rot-defs-r1-main-verification.md:70`）。
- 钩子文件头：「scripts/crash/crash-amplification.sh：带 --item 是 items（N = 分项数）；不带 --item 的」（`.claude/hooks/run-scope-guard.sh:29`）；「crash-amplification-item.sh、crash-amplification-two-hosts.sh 起一个测试目标：target，N = 1」（`.claude/hooks/run-scope-guard.sh:30`）。三份入口里只有 `crash-amplification.sh` 不带 `--item` 是 whole。
- 喂 `run-scope-guard.sh`（主 agent 身份，声明的为什么填实）：`crash-amplification-item.sh` 声明 whole exit 2、声明 target 加「1 个测试目标」exit 0；`crash-amplification-two-hosts.sh` 同样 whole exit 2、target exit 0；`crash-amplification.sh` 不带 `--item` 声明 whole exit 0，带 `--item foo` 声明 items 加「1 项」exit 0。照判决字面抄进定义会把两份入口写成 whole，是假话。
- 工具员落成「崩溃放量三份入口同样要声明（`crash-amplification.sh` 不带 `--item` 是 whole），整轮门禁、`cargo test` 整包与整工作区都是 whole」（`.claude/agent-common.md:87`）。判：改对。残留一点不全：括注只交代了第一份入口，另两份是 target 没写；对子 agent 无影响，因为三份入口都起 checker 档 cargo，「scripts/crash/crash-amplification-two-hosts.sh（双机驱动）：每一种跑法都起 checker 档 cargo」（`.claude/hooks/heavy-test-guard.sh:69`），子 agent 一律不跑。
- 推翻它的现象：`run-scope-guard.sh` 对 `crash-amplification-item.sh` 声明 whole 放行（今天 exit 2）。

### 第 2 处　`main-agent.md` 验证段第 1 步：判决那一格语焉不详，工具员补的是真话，没改偏

- 判决写：「写段打过的补丁目录带 `--skip-apply`，没打过的不带」（`research/prompts/rot-defs-r1-main-verification.md:64`）。
- 工具员落成：「写段打过的补丁目录带 `--skip-apply`（只编、只验，脚本先逐份核补丁已打在树上，核不过退 1），没打过的不带（脚本来打）」（`.claude/main-agent.md:51`）。多出的括注是丁-76 改完之后脚本的行为：「对给的每个补丁目录里的 crates.patch 在要验的树上跑 `git apply --check --reverse`」（`scripts/gate/merge-and-verify.sh:48`）、「有一份核不过、或给的补丁目录不在，就退 1、逐份点名、一个都不编」（`scripts/gate/merge-and-verify.sh:49`）。`merge-and-verify.sh --selftest` 退 0 且含这一格。判：没改偏，补进去的句子今天为真。
- 判决那一格的缺口（不是工具员的错）：它按补丁目录分「打过 / 没打过」，而脚本的开关是整次调用一个。脚本自己的说明写的是「打过的与没打过的分两趟」（`scripts/gate/merge-and-verify.sh:197`），定义没写这一句，见第五节 H4。
- 推翻它的现象：对一棵没打补丁的树带 `--skip-apply` 跑 `merge-and-verify.sh`，今天不是退 1（脚本自证的新格就造了这棵树，退 1）。

### 第 3 处　`main-agent.md` 第 52 行（现在第 53 行）：判决那一格只说方向，工具员的落法合格

- 判决写：「收进括注的指路里，不写短版」（`research/prompts/rot-defs-r1-main-verification.md:64`）。
- 工具员落成：「只改注释的产品代码文件开不开、射程与例外都照」（`.claude/main-agent.md:53`）；前半句「整段只改测试代码的不开代码轮」不再带「或只改注释的」。规则里：「只改了注释的产品代码文件也不走」和「文档注释与 `// SAFETY:` 注释的增删照代码算」都在同一行（`.claude/rules/implementation-workflow.md:10`），定义把结论整个交给规则，不留短版，正是判决那一格要的。同一文件第 7、29、91 行三处「只改测试代码的不开代码轮」口径一致。判：没改偏。唯一的代价是读者要多开一次规则才知道结论，判决那一格选的就是这个代价。
- 推翻它的现象：规则第 10 行把只改注释的产品代码文件改成要走代码轮，而定义没跟着变（今天规则与定义不矛盾）。

## 五、判决别的格有没有同样把钩子现状说错

### H1　A5 那一格给的「数法」今天是重型（说错了钩子现状）

- 判决那一格给的数法：「数法 `bash scripts/crash/crash-point-replay.sh --list-items --with-keys`」（`research/prompts/rot-defs-r1-main-verification.md:64`），另立账一节把它描述成读库：「`crash-point-replay.sh --list-items --with-keys` 打的是块数与判法单元指纹，不是「变了的判器单元清单」」（`research/prompts/rot-defs-r1-main-verification.md:76`）。
- 脚本此刻的实现：`--with-keys` 不是读文件，而是「cargo run --release --quiet --offline -p singlefs-checker-tier --bin crash_library_reader -- keys」（`scripts/crash/crash-point-replay.sh:1545`），起的是 checker 档包的 `cargo run`；文件头自己写了「起只读读库程序」（`scripts/crash/crash-point-replay.sh:22`）。
- 喂 `heavy-test-guard.sh`：单独的 `--list-items` 主 agent、调查员都 exit 0；加 `--with-keys` 主 agent 不带前缀 exit 2（首行写「崩溃点重放的重型」）；主 agent 带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request` exit 0；调查员带哪个前缀都 exit 2。钩子 `run-scope-guard.sh` 那边倒是不要声明：「crash-point-replay.sh 与 crash-amplification.sh 带 --list-items（含 --with-keys）」（`.claude/hooks/run-scope-guard.sh:48`），所以两道钩子对这条命令说法不一，重型闸才是真拒的那一道。
- 后果：工具员照格落进定义，`.claude/main-agent.md:10` 给主 agent 的数法不带前缀会被拒。该句的场景是里程碑收尾、提交之前，落在提交流程里，加 `SINGLEFS_HEAVY_TESTS=commit` 就放；但定义没写，也没写它起 cargo。这是判决那一格漏看重型闸，不是工具员改偏。
- 推翻它的现象：主 agent 不带前缀喂 `bash scripts/crash/crash-point-replay.sh --list-items --with-keys` 给 `heavy-test-guard.sh` 退 0（今天 exit 2）。
- 判决另立账说的「阶段打不出变了的判器单元清单」仍成立，`crash_library_reader` 的输出只有三个计数加指纹：「CRASH_LIBRARY_KEYS	library=/library/crash-state-verdicts	recorded_blocks=6	judged_under_the_current_key=2	judged_only_under_another_key=3	never_judged=1」（`crates/singlefs-checker-tier/src/bin/crash_library_reader.rs:14`）。这一点我没找到推翻。

### H2　正推第 5 项 / 判决第 70 行把崩溃放量三份入口写成 whole

见第四节第 1 处，已由工具员改对；判决那一格需要更正。

### H3　A3 只看了 `scripts/experiment-apparatus` 的装置，没看入库装置（漏看，不是说错）

- 定义第 31 行的例子 `cargo test --release --bin e<号>-<简称>` 是 `scripts/experiment-apparatus` 下装置的写法；入库装置住在 checker 档包里：「**另有四条只对入库装置生效**」（`.claude/agents/experiment-runner.md:34`）。
- 闸把 checker 档包 src/bin 下装置的内联单测算 checker 档：「src/bin/ 下的装置（它们的内联单测编成同名测试二进制）」（`.claude/hooks/lib_heavy_tests.py:870`）。喂 `heavy-test-guard.sh`（实验执行员，经包装）：`cargo test --release -p singlefs-checker-tier --bin e171_three_oracle_collusion` exit 2，在仓根不带 `-p` 的 `cargo test --release --bin e171_three_oracle_collusion` 也 exit 2（首行都是「cargo test 跑到 singlefs-checker-tier 包的测试（checker 档…）」）。
- 就是说：执行员照第 31 行跑 `scripts/experiment-apparatus` 的装置单测放行；跑入库装置的单测被拒，定义没说那种交主 agent。交 K2 攻方腿对「实验执行员走一遍」判走错与否。
- 推翻它的现象：喂上面两条命令里任何一条退 0。

### H4　A8 那一格没说混批要分两趟（说得不全）

脚本的出路写的是「打过的与没打过的分两趟」（`scripts/gate/merge-and-verify.sh:197`）；判决与定义（`.claude/main-agent.md:51`）都按补丁目录分「带 / 不带」，读者会以为一次调用里可以逐目录分开。混批不会漏验（带 `--skip-apply` 时没打的那份核不过退 1、不编，不带时打已打的那份被打补丁脚本拒），是一次浪费，不是错过红。

### H5　A1 那一格没扫到规则文件里的同一写法

「跑 `bash .claude/gate.d/doc-decisions.sh --write` 重新生成」（`.claude/rules/path-moves.md:33`）仍是裸 `--write`；喂 `run-scope-guard.sh`，主 agent 裸 `--write` exit 2。规则文件不在这一轮被判的十二份定义里，只记账，改归 tooling-writer。

### H6　旁注：三处例子只是声明片段，不带内存包装

`.claude/agent-common.md:56`、`.claude/agents/three-way-attack.md:53`、`.claude/agents/experiment-runner.md:31` 的例子都是「声明加 cargo」。我喂 `heavy-test-guard.sh`：实现员 `SINGLEFS_RUN_SCOPE='种类=target; …' cargo test -p singlefs-harness --test foo` 不经包装 exit 2（首行「不经内存包装跑编译出来的代码被拒」），套 `bash scripts/hosts/run-with-memory-cap.sh 4G` 后 exit 0。包装的要求另在同一份文件：「一律经 `bash scripts/hosts/run-with-memory-cap.sh <上限> <命令…>` 跑」（`.claude/agent-common.md:58`），所以是读者要自己合起来读，不算判决那一格漏。是否把包装写进例子，由主 agent 定。

### 判决里查了没发现钩子现状说错的格

A1（声明与 `--write` 收窄）、A2（链接 checker 档的探针 `cargo run` / `cargo test` 拒，`cargo build` 加包装下直接执行放行）、A4、A6、A7、A9、B1、B2、B3、正推第 10、16、21 项：判决那一格对钩子、脚本、规则的说法逐条对得上（证据见第三节）。

## 六、什么现象会推翻整份复核

- 第三节任何一行：把那一行的证据命令在此刻的钩子、脚本上重喂，结果与表里写的 exit 码或首行不同。
- 第四节三处判语：各自的推翻条件写在各自一节末。
- 第五节 H1：主 agent 不带前缀喂 `--list-items --with-keys` 给重型闸退 0。

## 七、没做什么

- 没判 K2、K3（攻方的格），也没判第五节乙四份定义（`experiment-designer.md`、`investigator.md`、`three-way-defense.md`、`three-way-forward.md` 在 `04ceaa904` 里的改动）；`investigator.md` 只看了批 4 动的第 31 行。
- 没复跑工具员报告里的全部自证（lib_heavy_tests 722 种、heavy-test-guard 1321 种、run-scope-guard 85 格、runner-dispatch-guard 420 种、dispatch-prompt 76 项）：只跑了 `python3 scripts/gate/rerun-failed-tests.py --selftest`（退 0）、`bash scripts/gate/merge-and-verify.sh --selftest`（退 0）、`python3 .claude/hooks/lib_heavy_tests.py --selftest`（退 0，725 种，比工具员记的 722 种多，因 `lib_heavy_tests.py` 在快照之后又改过，没追是谁改的、改了什么）。
- 没起 cargo、没真跑 `crash-point-replay.sh --list-items --with-keys`（它起 `cargo run`，也被重型闸拒）；H1 的判断来自脚本第 1545 行与喂闸的结果。
- 没读禁读清单里的两份文件（`rot-defs-r2-opus-output.md`、`rot-defs-r2-main-verification.md`）。
- 没替主 agent撤销或维持判决，只交复核结果；第五节各项该怎么改（改定义、改闸、更正判决）不替主 agent 定。
- 没改仓里任何文件；草稿目录里只有喂钩子的 `feed.py`、`q.sh` 和一个只含 `Cargo.toml` 与占位文件的探针目录（已删，不留编译目录）。
