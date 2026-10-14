# m3-defs-b6-r2 辩方腿（Sonnet）报告

日期 2026-10-13。格 X1–X7。立场：复核第一轮判决 `research/prompts/m3-defs-b6-r1-main-verification.md`——L1–L7 落到第二轮冻结副本里，对第一轮两腿每一条打中是不是修到了、有没有改过头、第一轮哪一格判重判轻。
记号：A = 第二轮冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo`（改后）；B = 第一轮冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r1/repo`（改前）；C = 零轮之前 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r2/repo`。表里的「退出码」是 A/B/C 的顺序（只有 A/B 的写两个）。
模型目录 `research/prompts/m3-defs-b6-r2-sonnet-model/`；每条喂法的原样输出在它的 `out/` 里，下面写「out/文件名」。

## 0 方法与边界

- 喂法：钩子入口喂 JSON 或 `--check-prompt`（派发闸、重型测试闸）；判法函数直接调（`lib_abandoned_rounds.py`、`lib_long_run.py script / exempt-check`、`host-dispatch.py` 的 `header_hosts`、`dispatch-prompt.py` 的 `guard_type_tables`）；门禁格只抽格里的 python 在小树上原样跑（`tools/run_htd_cell.py`、`tools/run_pv_cell.py`，不起 `--check`）。没跑任何 `--selftest`，没起 cargo，没起重型测试。
- 三份副本只读：`rerun.sh` 里 python 一律 `-B`。开工中我自己的一次 `adhoc.sh` 调 `python3 -I dispatch-prompt.py`（`-I` 不认 `PYTHONDONTWRITEBYTECODE`）在 A、B 的 `.claude/gate.d/` 下各留了一个 `__pycache__/lib-forbidden-notations.cpython-312.pyc`；发现后已逐个删掉（含空目录），`adhoc.sh` 改成 `-B -I`，之后 `find <三份副本> -newer <开工时间戳> -type f` 无输出。
- 冻结副本核对：`sha256sum -c research/prompts/m3-defs-b6-r2-snapshot/frozen-copy-sha256.txt` 在 A 里 33 份非 OK 0 行。
- 开工看负载：`ps` 有别的会话的 `cargo test`（经内存包装），没有 `qemu-system`、`vm-bench`、`e152`、`fio`；我的命令全加 `nice -n 19`；没起 cargo，派发提示的线程上限 31 用不上。
- 复跑：整条经 `run-with-memory-cap.sh 2G`、分两趟（单趟超过前台 240 秒），命令与结果见第 11 节。
- 私有配置：`multi-host.env` 一个值都没读出来、没写；A 里的那一份只交给 `tools/pv_path_probe.py`（抽格里的取词代码）数个数，输出只有个数；模型目录里 `gate-samples/doc-text/pv-tar-mirror-red/multi-host.env` 是第一轮攻方留的编造样本。

## 1 一览（逐条在第 2–6 节）

| 格 | L | 打中的组数（第 2–7 节每张表的行） | 修到 | 部分修到 | 没修到 | 没动（本批不在 L1–L7） | 改过头 |
|---|---|---|---|---|---|---|---|
| X1 | L1 | 10 | 6 | 1（V1-乙：形式修到，说明行里同一句仍到得了子 agent） | 1（DP-1「书记员」） | 2（内存上限无钩子；已判行理由不看内容） | 0 |
| X2 | L2 | 九种写法 + lib.rs + printenv（按写法数 11） | 10 | 0 | 1（printenv 读法，两版都漏，L2 没承诺） | 0 | 1（O-3：排除表涨 2 行，与格的只缩不涨冲突） |
| X3 | L3 | 4 | 3（V5-丙 C2、C8；V5-乙说明） | 0 | 1（V5-甲，L3 本不含） | 0 | 0 |
| X4 | L4 | 3 | 2（日期根；一收一拒的形状） | 1（别的轮名：仓里有材料的名字修到） | 0 | 0 | 1（O-1：`<名>-r<数字>` 形但仓里没有材料的名字，改前不收，改后收） |
| X5 | L5 | 2 | 2 | 0 | 0 | 0 | 0 |
| X6 | L6 | 6 | 3（定义原话被拒、词表删除、非规范句拒） | 1（规范句后的说明里仍放一半、余量、中文数词、按核数现算） | 0 | 0 | 1（O-2：规范句后接 host-resources 等现算词，改前拒，改后放） |
| X7 | L7 | 6 | 1（V8-甲的 a 份：头不是规范句被拒，只是形式） | 2（sha256 入表并核；表是自证） | 1（V8-甲：规范句头 + 现算正文三道全过） | 2（stage-owners.tsv 第 48 行文字未动；E171 六份两边读法不一） | 0 |

「改过头」按正文第五节：第一轮判对的一段历史，改后判错。三处都只在一类写法上成立，下面给了喂法与退出码；三处都是 L 的改法本身带进来的，不是 L 没做完。

## 2 X1（L1：⑮ 只认整数）与 X6（L6：⑭ 多机一行只认规范句）

喂法：`tools/feed_matrix.py`（第一轮正推腿 `feed.py` 的全部用例原样搬来，加第二轮新增；A、B、C 三份副本的 `runner-dispatch-guard.sh --check-prompt`）→ `out/fm-X1.txt`、`fm-V1.txt`、`fm-V2.txt`、`fm-V8.txt`、`fm-X6.txt`（共 195 个标签）；`tools/v1_feed.sh` 喂攻方腿的 27 个括注变体与 8 个多机变体 → `out/o-v1-*.txt`、`o-v8-*.txt`；`tools/feed_stdin.py` 喂 stdin 入口 → `out/stdin.txt`。⑮ 的判法：「`THREAD_LIMIT_VALUE = re.compile(r"[0-9]+")`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1002`）；放行行：「`valve = valve_reason(prompt, "线程上限已判") is not None and not break_switch_is("thread-valve-ignored")`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1081`）。

### 2.1 第一轮每一条打中的去向（X1）

| 来源（第一轮报告里的编号） | 喂法 / out | B | A | 判 | 依据 |
|---|---|---|---|---|---|
| 正推 AC-2、MA-1：括注里中文数词按字符判，误拒统一取登记值、万一内存不够、十分稳定；不套现算词表，放过按核数取、忙时降低、负载高就少起 | fm-V1 的「括注 统一/万一/十分」「V8x 括注 按核数取/忙时降低/负载高就少起/nproc/host-resources 现算」 | 误拒 2，漏放 0 | 全部 2 | 修到 | 括注一概拒，拒绝说明点名「括注一概不认，说明另起一行写线程上限说明」；`out/fm-X1.txt` 同形用例 |
| 正推 AC-3、MA-3、HG-6：括注里的尖括号放行（与「别的尖括号按认不出数拒」冲突） | fm-V1「括注里的尖括号 <登记值>、<按机器忙闲定>」；stdin 第 4 例 | 0 | 2 | 修到 | 同上；`out/stdin.txt` 第 4 例新 2 旧 0 |
| 正推 CL-2、FW-2、DF-2、IN-1、TW-1：线程上限已判能顶缺行，定义没写 | fm-V1「<类型> 没有线程上限行、写已判」；fm-X1「已判 顶缺行」（clerk、forward、defense、investigator 各一） | 0 | 0 | 修到（文字） | 定义补了：「这一行也能顶替缺的」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agents/clerk.md:24`），共用约束 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:85` 与 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/main-agent.md:53` 同；钩子行为没变，写已判行仍放、不写仍拒 |
| 正推 HG-4：钩子注释说 forward、defense 的定义许编译 | `grep -nF` 三份定义 | 注释与定义不符 | 注释改了 | 修到 | 「three-way-forward、three-way-defense 的定义只跑模型目录里的 tools/rerun.sh、不编译」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/runner-dispatch-guard.sh:69`）；forward 里「编译」零命中，defense 只有「要改、要编译先拷进自己的草稿目录」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agents/three-way-defense.md:36`）——讲的是拷贝副本，与注释不冲突 |
| 正推 DP-1：dispatch-prompt.py 把 clerk 叫「书记员」（那是 kb-scribe 在 ⑧ 里的叫法） | `grep -nF` | 有 | 仍有 | 没修到 | 「书记员、前向腿、防方腿」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/scripts/agents/dispatch-prompt.py:20`）；L1 改了这份文档串里别的句子，这一处没动 |
| 正推 IN-2、TW-2：「内存上限」没有任何钩子判 | — | 无 | 无 | 没动 | 不在 L1–L7；第一轮判决也没列 |
| 正推 AC-7、MA-6（V2）：已判行的理由不看内容，「只看到负载忙不算」靠主 agent | fm-V2：理由写「机器忙负载高…」、写一个字 x | 放（0） | 放（0） | 没动 | 第一轮判 V2「站得住」；A、B、C 三份退出码相同，`out/fm-V2.txt`。注意 L1 让三种类型把已判行当缺行的出路，承重变大（见第 6 节判轻） |
| 攻方 V1-甲：括注抄定义原话（全仓唯一的登记位、双机时两台各取这一个数、第二台同）被拒 | o-v1 变体 A、R、L4 | 2 | 2 | 修到（以文字对齐的方式） | 退出码不变，但改前定义许括注、拒是误伤；改后定义写明括注一概不认（「整数后面接任何字都拒」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:85`；「这一行里整数后面不接任何字（括注也拒）」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/main-agent.md:53`），拒是对的 |
| 攻方 V1-乙：按负载降、忙闲看空闲核数、小机器对折、双倍、第２台１６、另台 sixteen 放行 | o-v1 变体 D、D2、E、H2、I2、I3、I4 | 0（C 也 0） | 2 | 修到（形式）；部分（通道） | 括注一概拒。但通道只是挪了位置：说明另起一行「线程上限说明：…」不判，里面写数字、÷、一半、31/2 都放（fm-X1「说明里写数字」「说明里写 ÷ 与一半」A=0）。同一句话仍能到子 agent 手里——L1 自己写明了这条取舍，不是漏做 |
| 攻方专问：九种类型里 clerk、forward、defense 多数派发不带重活却被逼写线程上限 | fm-X1「已判 顶缺行」 | 出路只在钩子说明里 | 定义与骨架都写了 | 修到 | 骨架说明见 `out/adhoc.txt`（C-3 段），共用约束 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:85` |

### 2.2 X1 专问：只认整数之后正当的说明写在哪、「线程上限说明」会不会被当第二个「线程上限」

- 写在另起一行的「线程上限说明：…」：`out/fm-X1.txt` 里 clerk、forward、defense、investigator 各喂「线程上限：31」加一行说明，A=0。说明行放在前面、或里面写数字、÷、一半、31/2，A=0。
- 不会被当成第二个「线程上限」：钩子的行匹配是「`THREAD_LIMIT_LINE = re.compile(LINE_START + r"线程上限[* \t]*[：:][* \t]*(.*?)[ \t]*$", re.M)`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1000`），「线程上限」与冒号之间只许星号、空格、制表符，「说明」两个字夹在中间就不匹配。喂法：只有说明行没有线程上限行，A=2 且原因是「没有线程上限：N 一行」（forward、defense、clerk、investigator 都是）；「线程上限（说明）：31」A=2 同因。
- 说明行里套一行「 线程上限：16」（行首空格）是另一行，照判，A=2。
- 边界（A 退出码，B、C 相同的没标）：「线程上限:31」半角冒号 0；「线程上限： 31」0；「**线程上限**：31」0；「> 线程上限：31」0；「线程上限：031」0；「线程上限：31。」2；「+31」2；「31.0」2；「3１」2；行尾制表符 0；两行都写 31 的 0。
- 已判行：理由空、写「无」「没有」「<理由>」时 A=2（缺行仍拒）；写「机器忙，降一半」A=0；已判行写在括注里「31（线程上限已判：忙）」A=2（B=0）。

### 2.3 第一轮「判对」的 ⑮ 历史重喂（有没有改过头）

`fm-V1.txt` 里 B=A 的行（31÷2、31/2、31 一半、31，即 15、31（本机），第二台 16、全角数字、半角括号、两段括注、整数后接逗号、缺行、30 不写已判、30 写已判）全部 A=B；A 与 B 不同的只有「括注」类（B=0，A=2），是 L1 的本意。`SINGLEFS_THREAD_CAP=16` 时「线程上限：16」A=B=2、「31」A=B=0（`out/env-cap.txt`）。攻方 27 个变体里没被第一轮判中的 F、G、G2、W、W2（第二台另起一行、已判行写负载）A=B=C=0，Z（031）0，X、Y（编号列表、表格）2，K、V（句号、31 线程）2：A 与 B 一致，没改坏。

### 2.4 X6：⑭ 多机一行只认规范句（对应攻方 V8-乙、正推 MA-9、DP-4）

规范句：「`CANONICAL_HOSTS_SENTENCE = "台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量"`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_long_run.py:82`）；说明里只拦数字：「`HOSTS_NOTE_DIGIT = re.compile(r"\d")`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_long_run.py:84`）。

| 来源 | 喂法 / out | B | A | 判 | 依据 |
|---|---|---|---|---|---|
| 攻方 V8-乙 M1–M3：定义原话「不看忙闲」「在跑」「核数」被拒 | o-v8 M1–M3；fm-X6「规范句 + ；不看忙闲」「+ ；说明写按核数现算」 | M1–M3 拒（2） | 不带规范句的仍拒（2）；规范句后接「；不看忙闲」放（0） | 修到 | 词表删了，定义原话可以挂在规范句后面；拒绝说明把规范句整句写出来，可照抄（`out/fm-X6.txt`；原因行见 `o-v8-after.txt` 的首行） |
| 攻方 V8-乙 M6「的一半」、M7「按此刻余量少给」、M4「（31）」、M5「机器忙也不降」放行 | o-v8 M4–M7 | 0 | 2 | 修到（这几行原样） | 它们都不是规范句 |
| 同上，规范句 + ；说明 | fm-X6「规范句 + ；说明写一半」「+ ；按此刻余量少给」「+ ；机器忙时降」「+ ；每台取十六」「+ ；英文数词」 | 这几行带的是改前写法，B 不适用（B 拒的是缺 heavy-thread-default 的写法） | 全部 0 | 部分 | 说明里只拦数字（A 对「；每台 31 线程」「；全角数字１６」是 2）；中文数词、英文数词、一半、余量、降，都放。lib_long_run.py:21 自己写明「说明里写的是什么钩子只判有没有数字」 |
| 正推 MA-9 / DP-4 / HG-8：文字没写词表与「否定句也算」，最自然的合规写法被拒 | fm-V8「标准写法 + 不按核数现算」「不看忙闲」「照 main-agent 字面」 | 2 | 2 | 修到 | 文字不再讲词表：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/main-agent.md:55` 只讲规范句；但「照字面写的」A 仍拒，因为不是规范句，原因行指到规范句 |
| 改过头候选 O-2：规范句后面接现算词 | fm-X6「规范句 + ；host-resources 现算」「+ ；available_parallelism」「+ ；按核数现算」 | 2（B 的词表拒） | 0 | 改过头 | 第一轮判对（正推 HG-8 判「一致」：八个词在多机行里各自出现都拒）；L6 删词表后，规范句 + 现算说明放行——句子自相矛盾（「不现算」后面接「按核数现算」）而钩子看不到。依据「别的写法一律拒，不用现算词表去拦」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_long_run.py:19`）是故意的，但后果是 R1 已封死的一块重新开了 |
| 「现有脚本与定义里的写法能不能照抄过闸」 | `tools/x6_sweep.sh` → `out/x6-sweep.txt`；fm-X6「规范句原样」；`dispatch-prompt.py clerk` 骨架 | — | 见右 | 能 | 仓里 9 份带长活声明的脚本：3 份（crash-amplification.sh、crash-point-replay.sh、broad-mutation.py）头里写 unshardable、不写 hosts，6 份 E171 入口写现算头在豁免表里，A、B 都 rc=0；没有一份脚本的 hosts 行是登记值读法，所以「照抄过闸」没有现成样本可试，只能拿骨架那一句试：规范句原样 A=0；带直角引号、反引号、半角逗号、少一个空格各 A=2；行尾空格、行首空格 A=0（`fm-X6.txt`）。骨架的占位里整句夹在「…」里（`out/adhoc.txt` C-3 段），整段原样贴回去仍是 `<填 …>` 占位，⑬ 拒（fm-X6「骨架占位原样」A=B=C=2） |

## 3 X2（L2：heavy-thread-defaults 退回宽判法）

喂法：`tools/run_htd_cell.py <判法所在副本> <被判树>` 从那份副本的 `code-tooling.sh` 抽出 heavy-thread-defaults 格的 python，在被判树里原样跑（基准状态 none）。判法取 A、B、C 三份，被判树取：攻方留的 `feeds/v3/sample`（14 份装置）、我造的 `feeds/x2/sample`（不读环境变量的 8 种写法）、各副本自己的真树。输出 `out/v3-*.txt`、`out/x2-fp-*.txt`、`out/x2-real-*.txt`、`out/x2-rule-diff.txt`、`out/x2-base-growth.txt`。

### 3.1 第一轮每一条打中的去向

| 来源 | 喂法 | B | A | 判 | 依据 |
|---|---|---|---|---|---|
| 攻方 V3：经包装函数读（project_wrapper.rs、harness_wrapper.rs、wrap_helper.py）、`ENV = os.environ` 别名（alias_env.py）、`os.environ.copy()`（copied_env.py）、`use std::env::var as`（use_alias.rs）、`std::env::vars()`（vars_iter.rs）、shell `|| X=8`（or_default.sh）、`&& X=8`（and_default.sh） | v3 样本，`out/v3-*.txt` | 九份都不点名 | 九份都点名（C 也九份） | 修到（9/9） | 引号里的全大写名字出现就算读：「`OWN_VARIABLE_QUOTED = re.compile(r"[\"'](" + THREAD_VARIABLE + r")[\"'](?!\s*\]\s*=(?!=))")`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:2008`）；`||`/`&&` 后的钉数是新增形：「`CONDITIONAL_ASSIGNMENT_FORM = "写死的线程数默认（|| / && 赋值）"`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:1983`） |
| 攻方 V3-乙：`#[cfg(test)] mod tests;` 之后的读（lib.rs:8）两版都漏 | v3 样本 lib.rs | 不点名（C 也不） | 点名 | 修到 | 外置模块不跳过：「`RUST_EXTERNAL_MODULE = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*;")`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:2018`） |
| 攻方 V3 对照：tests.rs（测试模块文件）、baseline_read.py、if_default.sh | v3 样本 | 点名 | 点名 | 不变 | tests.rs 是声明的测试模块文件，攻方当误报；两版同 |
| 攻方 V3：`printenv_read.sh`（`$(printenv SAMPLE_PRINTENV_WORKERS)`）两版都漏 | v3 样本 | 不点名 | 不点名 | 没修到 | L2 没承诺这一形（变量名不在引号里、也不是 `$X` 读）；`out/v3-after.txt` 里没有它 |
| 正推「V3 对照」if_default.sh（`if [[ -z "${X:-}" ]]; then X=8; fi`） | v3 样本 | 点名 | 点名 | 不变 | — |

### 3.2 X2 专问：退回宽判法之后第一轮之前那批误判回来了几处，排除表兜没兜住，是不是只缩不涨

- **真树上回来 2 处**（`out/x2-rule-diff.txt`：A 的判法在 A 的树上、排除表清空，被判行 94；B 的判法 92；C 的判法 92；A 比 B 多、比 C 多的是同样 2 行，B 比 A 多 0 行）：
  - `scripts/agents/dispatch-prompt.py:269`——`"THREAD_LIMIT_REQUIRED_AGENTS" in long_run_table.group(2)`，字符串比较，不读环境变量（正是 W01）；
  - `scripts/experiments/replay.sh:1892`——`[[ "$REPLAY_JOBS" -lt 1 ]] && REPLAY_JOBS=1`，夹下限，不是默认值。这一处是 L2 自己新加的 `&&` 钉数形带出来的新误判，第一轮没有。
- **合成样本上回来的类**（`out/x2-fp-*.txt`，8 种都不读环境变量）：Python `in` 比较、名字元组、字典实参（写给子进程）、没人用的常量、argparse metavar、Rust `==` 比较、Rust match 臂——A 7 种全点名，C 7 种全点名，B 0 种；加上 shell 夹下限（`fp_clamp.sh`）A 点名、C 不点名（C 没有这条形），共 A 8 种。文档串（`fp_docstring.py`）与环境前缀（`true && X=8 make -j 2`）三份都不点名，正确。
- **排除表兜住**：两处各有一行：「`scripts/agents/dispatch-prompt.py:"THREAD_LIMIT_REQUIRED_AGENTS" in long_run_table.group(2)`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/heavy-thread-exempt:82`）与「`scripts/experiments/replay.sh:&& REPLAY_JOBS=1`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/heavy-thread-exempt:83`）。A 的格在 A 的树上无基准跑：`out/x2-real-after.txt` 末行「可疑写法 94 处全在排除表里（表 66 行，每行都命中）」，B 在 B 的树上同为 94 处 66 行。表行数没变是因为删了两行陈旧锚串（B 的 .claude/heavy-thread-exempt 第 30、33 行，runner-dispatch-guard.sh 的拒绝说明与自证汇总句里列词表的那两处）、加了这两行。
- **只缩不涨：不满足**。格的判法按「路径:锚串」对基准表：「`grown = [(number, path, anchor) for number, path, anchor in exempt_rows if (path, anchor) not in base_keys]`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:2332`），红的说法是「`print(f"  ✗ 排除表涨了：这几行基准 {base_name[:12]} 那一版里没有：")`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/code-tooling.sh:2364`）。把基准表设成 B 的表或 C 的表，对 A 的树跑同一段 python：都退 1，点名第 82、83 行两行（`out/x2-base-growth.txt`）。所以 L2 要求的「W01 进排除表一行」与格自己的「只缩不涨」互相打架：主工作区有 git 基准时，这一格会在提交时红，出路写的是「交主 agent 问用户再加」。集成副本复核（verify9）绿，是因为副本没有 git 基准（`基准里还没有这张表（或不在 git 仓里），这一条没比`）。是不是红取决于主工作区 HEAD 那一版表里有没有这两行，我读不到，写进第 8 节「要主 agent 跑」。
  - 判：**改过头（O-3）**（L2 的文字本身带进来的矛盾），不是修漏。
- 第一轮判对的历史重喂：攻方 V3 对照的 tests.rs、baseline_read.py、if_default.sh 同一结果；真树 `out/x2-real-*.txt` 三份都绿；装置自己读的线程变量数 A 72、B 58、C（自己的树）72（宽判法数得多，不是回归）。

## 4 X3（L3：重型测试闸）

喂法：`tools/v5_feed.sh`（在工作根底下造 3 份仓副本，把 Bash 工具的 hook JSON 喂 `heavy-test-guard.sh` 入口）→ `out/v5-copy-*.txt`；`tools/v5_chdir_feed.sh` → `out/v5-chdir-*.txt`；`tools/v5_env_matrix.py`（48 例：30 例按「cargo 真正落在 checker 档包」手判应拒，18 例应放）→ `out/v5-env-matrix.txt`。

| 来源 | 喂法 | C | B | A | 判 | 依据 |
|---|---|---|---|---|---|---|
| 攻方 V5-丙：`env -iC <checker 档目录> cargo test`（C2）放行 | v5-chdir C2 | 0 | 0 | 2 | 修到 | env 选项照 getopt 切：`env_split_string` 在 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_shell_words.py:521`，拆完原地换进命令再判（`skip_prefixes_expanding`，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_shell_words.py:577`） |
| 攻方 V5-丙：`env -S 'cargo test -p singlefs-checker-tier'`（C8）放行（两版都放） | v5-chdir C8 | 0 | 0 | 2 | 修到 | 同上 |
| 攻方 V5-乙：拒绝说明写「由主 agent 在集成副本里跑」，主 agent 不带前缀照做也被拒 | v5-copy S2、S5（子 agent）、S6、S7（主 agent） | S2、S5、S6、S7 全 0 | S2、S5 拒、S6、S7 拒 | 同 B：S2、S5、S6、S7 都 2 | 修到（说明） | S6、S7 本来就该拒（主 agent 不带前缀）；改的是说明：A 的 S2、S5 说明里多了 `SINGLEFS_HEAVY_TESTS=user-request`、工作目录是集成副本根、不带前缀也被拒、改的是共用模块时跑 import 它的入口——对应的代码行「`（主 agent 不带这个前缀，在集成副本里也被这道闸拒；`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_heavy_tests.py:1448`）；B 的同一条说明停在「由主 agent 在集成副本里跑」。喂法 JSON 的原样输出见 `out/v5-copy-after.txt` 第 S5 行 |
| 攻方 V5-甲：副本里只改入口 import 的库，入口脚本与主仓 sha256 相同，自证放行（S1、S1b、S4） | v5-copy | 0（C 也 0） | 0 | 0 | 没修到 | L3 没承诺，第一轮判「另立账」。文字补了限度：「`入口脚本 import 的库改过的同样不起，闸只看入口那一份的正文`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:81`）——定义对子 agent 说不要起，钩子仍不拦，两句并存 |
| 正推：V5 无句子（本腿没判 V5） | — | — | — | — | — | — |

### 4.1 X3 专问：env 拆法的边界、有没有改过头

`out/v5-env-matrix.txt` 48 例：A 判错 0 例；B 判错 16 例（全是漏放）；C 判错 23 例。覆盖的写法（A 全部判对）：`env --` 之后再接 `env -C`；`-u` 带值贴写（`-uRUSTFLAGS -C 目录`）与另起；带等号的长选项（`--chdir=目录`、`--split-string=串`、`--unset=X --chdir=目录`）；缩写的长选项（`--chd`、`--ignore`）；单独的 `-`；`-0`；`-iS 串`；`-S` 串里再嵌 `env -C`、`env -S`、`sh -c "…"`、串开头的选项与赋值；`-S` 串在前、`-C` 在后（拆开后 `-C` 落在 cargo 的参数里，不是 env 的选项，按「放」判，A 放）；对照应放的 18 例（harness 包的测试、`env -i cargo build`、`env -u X cargo fmt`、`env -S 'cargo build'`、`-C ../singlefs-harness`、缺值畸形的 `env -C`/`env -S`/`env -u`、`-S` 串里只是字符串「echo singlefs-checker-tier」、`env X='a b' cargo build`）A 全放。畸形 `-S` 串（引号不闭合、`\c`、`\_`、`${X}`、`#` 注释、嵌套三层、空串）18 例没有崩（无 Traceback），退出码 0 或 2。没找到改过头。
仍罩不住的是第一轮就写在钩子文件头的那一类：目录写在变量里（v5-chdir C6，A=B=C=0）。

## 5 X4（L4：叫停轮登记行——别的轮名按仓里认得的全集判、免出处只给四行旧名单）

喂法：`tools/v6_matrix.py`（47 例：第一轮正推腿 `v6.py` 的 29 例原样搬来 + 第二轮新增；每例造小树，库两种读法 + 派发闸 ⑥ 端到端；B 有 `--legacy-ok`（第一轮门禁格的读法），A 的库已无这个开关，只判一种）→ `out/v6-matrix.txt`；真表 → `out/x4-real-table.txt`；`tools/make_gate_trees.sh` 的 7 棵树 → `out/gate-trees-lib.txt`、`out/adhoc.txt`、`out/nolib.txt`。

### 5.1 第一轮每一条打中的去向

| 来源 | 喂法 | B（⑥ 读法 / 门禁读法） | A（库，门禁与 ⑥ 同一份） | 判 | 依据 |
|---|---|---|---|---|---|
| 正推 TI-3、AC-6、HG6-1、HG6-5、LIB-1、DPR-1、DPR-5、DPR-2 与攻方 V6：日期 2026-10-11 登记、没出处——门禁读法收、⑥ 不收，「不收就是红」不成立；表头日期是腿发出日不是登记日 | 「日期 2026-10-11 没写出处」「日期 2000-01-01 没写出处」「新叫停轮日期 2026-10-11（腿发出日）无出处」；小树 legacy-sourceless | 不收 / 收 | 不收（⑥ 拒） | 修到 | 免出处改成点名：「`LEGACY_SOURCELESS_ROUNDS = ("c196-r1", "d16-item1-r2", "c364-r3", "smalldata-ttime-r1")`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_abandoned_rounds.py:59`），门禁与 ⑥ 同读；日期不再起作用；A 里凡是登记了 mtest-r1 的例子，库收的 ⑥ 都放行、库不收的 ⑥ 都拒，没有一收一拒（只有占名单名那几例库判的是 c196-r1、⑥ 看的是开着的 mtest-r1，不是同一轮，不算）。真表：A 五行全收，B 的 ⑥ 读法 4 行不收（`out/x4-real-table.txt`） |
| 正推 TI-1、AC-5、HG6-2 与攻方注：出处那一行另有别的轮名 m3-close-b1（无 -r 形）、OTHER-R2 | 「另有不以 -r<数字> 结尾、也没登记的别的轮名 m3-close-b1」（树里没有它的材料）；「树里有 _m3-close-b1-body.md」「只有 -snapshot/」「只有 -main-verification.md」「只有 -opus-output.md」；OTHER-R2 | 全收 | 无材料：收；有 body、snapshot、main-verification 任一：不收；只有腿报告：收；OTHER-R2、Other-r2（即便树里有 _other-r2-body.md）：收 | 部分修到 | 认法：「`ROUND_MATERIAL_FORMS = (re.compile(r"^_(.+)-body\.md$"), re.compile(r"^(.+)-main-verification\.md$"), re.compile(r"^(.+)-snapshot$"))`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_abandoned_rounds.py:71`）。仓里没有任何材料的名字、大小写不对的名字、只有腿报告的名字，认不出（第一轮就认不出，没变） |
| 攻方 V6 专问：10-12 之前免出处的行还剩哪些一收一拒 | 同第一行 + 名单内旧行（c196-r1 日期 2026-10-11 无出处）、占名单名（新日期的 c196-r1）、名单名带空白、名单名大小写 | A 形（门禁收 ⑥ 不收）可达 | 无一收一拒；名单名 c196-r1 即便写新日期也收（库收，⑥ 按开着的轮判，本例里 mtest-r1 没有登记所以照拒） | 修到 | 名单按名字不按日期；名单名大小写不对（C196-R1）不收 |

### 5.2 改过头：`<名>-r<数字>` 形、仓里没有任何材料的名字

- 喂法与退出码（`out/v6-matrix.txt`）：「出处那一行另有 <名>-r<数字> 形的轮名」（other-r2，树里没有材料）：B 库不收、⑥ 拒；A 库收、⑥ 放行。「别的轮名是已归档的旧轮（材料被删），登记表也没有」（old-archived-r3）同：B 不收，A 收。树里加上 `_other-r2-body.md` 后 A 不收（拒）。
- 第一轮判对的行为：正推 TI-1 的「不」只指 m3-close-b1 那一类（无 -r 形）；`<名>-r<数字>` 形的 B 一直认，正推 HG6-2 判「一致」，V-8 喂法「other-r2 不收」。A 把这一支只留在弄坏开关里：「`alternatives = ([ROUND_NAME_FORM] if "other-name-by-form" in modes else [])`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_abandoned_rounds.py:154`）。
- 文字与代码不一致：规则文字写「不只认 -r<数字> 形」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/rules/three-way-inference.md:121`），读作「全集之外再加 -r 形」；代码是「只认全集，不再认 -r 形」。按文字读，other-r2 应该仍不收。
- 真仓上的量（`recognised_round_names` 在 A 上跑）：认得的轮名 310 个；只有 1 个不以 -r<数字> 结尾（test-granularity-rename）；只靠 main-verification.md 认出的 120 个，只靠 body 或只靠 snapshot 认出的各 0 个，只靠登记表认出的 4 个（c196-r1 等）；有腿报告文件却四类都认不出的 1 个（_c510-date-gate-r1）。**漏取 main-verification 这一类，310 个里 120 个认不出**；漏取 body 或 snapshot 单类不丢（各被别的类盖住）。
- 影响：今天的真仓里别的轮几乎都有材料，被认得，改过头的只会出现在「写错的轮名」「已经没有材料的轮」「还没开材料就写进叫停原话的轮」。后果是 B 能挡的一条（出处那一行带别的 -r 形轮名）A 不挡了，这一行会被收下，那是别的轮的叫停原话。
- 判：**改过头**（文字与代码二选一改，不进本腿）。

### 5.3 X4 专问：「仓里认得的轮名全集」怎么取、漏取一类会怎样、旧行名单与登记表现有各行

- 怎么取：`research/prompts/` 下 `_<轮>-body.md`、`<轮>-snapshot/`、`<轮>-main-verification.md` 三种文件名的 `<轮>`，加登记表第一列（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_abandoned_rounds.py:134`）。目录读不了就只剩登记表。
- 漏取一类：见 5.2 的量；另外只有腿报告（`<轮>-<模型>-output.md`）的轮不认（喂法：「树里只有 -opus-output.md」A 收）。
- 旧行名单与登记表现有各行：登记表 5 行（c196-r1、d16-item1-r2、c364-r3、smalldata-ttime-r1、m3-batch63-code-r1），A 的库全收；前四行靠名单免出处，最后一行带出处。名单四个名字全在表里，没有悬空；表里每一行不是名单名就带出处，没有「既不在名单又没出处」的行。名单写死在库里，没有检查把它与表绑在一起（表删一行名单不会红；名单里的名字被拿来登记新叫停轮时 A 也收，见 5.1 第三行）。
- 门禁格本身（`doc-process-records.sh --check abandoned-rounds`）子 agent 起不了，见第 8 节。

## 6 X5（L5：private-values 路径整串比对、命中段整段打码）

喂法：`tools/run_pv_cell.py` 抽 A、B、C 各自 `doc-text.sh` 里 private-values 格的 python，在样本 git 仓里原样跑（样本根里的 `multi-host.env` 是第一轮攻方留的编造配置，假值 `/srv/sample-private/peer-tree`）；`tools/x5_shapes.sh` 造 6 种路径形状 → `out/x5-shapes.txt`；`tools/pv_path_probe.py` 用 A 根下真配置只数个数 → `out/x5-path-probe.txt`。

| 来源 | 喂法 | C | B | A | 判 | 依据 |
|---|---|---|---|---|---|---|
| 攻方 V7：私有值带 `/`（真配置 8 个词里 6 个带），tar 镜像路径逐段打码打不到，红行把值原样打出 | 样本 pv-tar-mirror-red（假值） | 退 0，不比路径 | 退 1，红行含 `research/results/from-peer/srv/sample-private/peer-tree/…` | 退 1，红行是 `research/results/〔文件名含私有值〕/research/results/e999.out` | 修到 | 「`def shown_path(path):`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/doc-text.sh:444`）整串在整条路径里找，命中段整段打码、重叠并段 |
| 形状 1–6（`out/x5-shapes.txt`）：值跨四级目录；值是目录名的前缀（peer-tree-2）；同一路径里出现两次；主机名替身是文件名的一部分；不含值的对照；只含值的后半段 | 6 个未跟踪文件 | 退 0，输出 0 处含假值 | 退 1，3 行含假值，2 处打码 | 退 1，0 行含假值，6 处打码；对照与后半段都没被点名 | 修到，没有误报 | 前缀形状整段（到下一个 `/` 为止）打码；副作用是命中那一级的上一级目录名（mirror、from-peer、e1）也被并进打码段，信息损失、不泄值 |

### 6.1 X5 专问

- 误报：真配置 8 个词（含本机主机名）在 A 根（14446 个文件）的相对路径上命中 0 个（`out/x5-path-probe.txt`，只打个数）；取词有 8 个字的下限（`MINIMUM_LENGTH`），常见目录名进不来。私有值是路径前缀的形状见上表第 2 行，打码到段尾，不误点别的文件。
- 别的通道：红行只列「文件、行号、键名」，路径经 `shown_path`；**有一处没经**：「`fail(f"取不到 {path} 相对基准 {base[:12]} 的改动（git diff 退 {difference.returncode}）`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/doc-text.sh:473`）把原路径打出来，这是 git 失败的分支，路径里带私有值时会泄；B 里同形、L5 没动，不是 L5 的修漏而是 L5 的射程之外。
- 判：L5 对 V7 修到；第一轮判对的「路径误报 0」在 A 仍成立。

## 7 X7（L7：豁免表每行带 sha256）与 X6 里的 V8-甲（E171 下一次修订）

喂法：`tools/x6_rev12.sh`（在 A、B 的拷贝里放 E171 修订 12 入口三份：a 攻方原样（文件头改成登记值的说法，非规范句）、c 只把 `# long-run-hosts:` 换成规范句而正文照旧起修订 11 的现算编排、h 老实拷贝修订 11）→ `out/x6-rev12.txt`；`tools/x7_sha.sh`（豁免表里一份 E171 入口上做 13 种改动）→ `out/x7-sha.txt`；`tools/hd_header.py` → `out/hd-header.txt`。

| 来源 | 喂法 | B | A | 判 | 依据 |
|---|---|---|---|---|---|
| 攻方 V8-甲：改头不改身——修订 12 入口只改文件头、正文照起现算编排，三道全过；老实拷的反被拒 | x6_rev12 的 a 份 | lib_long_run script 0、long-run-check 0、exempt-check 0、heavy-thread-defaults 0 | script 1、long-run-check 5（头不是规范句） | 这一份修到（形式） | 头必须逐字是规范句 |
| 同上，c 份（规范句头 + 现算正文） | x6_rev12 | 0、0 | script 0、long-run-check 0、exempt-check 0、heavy-thread-defaults 0 | **没修到** | 与 B 的结构一模一样：规范句头过，h 份（老实）A、B 都拒（script 1、long-run-check 5）。L7 自己的说法是「文件头写的读法与脚本真做的事是否一致…钩子看不到，由写文件头的人保证」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/long-run-live-hosts-exempt:5`）与「E171 下一次修订的入口不进这张表」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/long-run-live-hosts-exempt:5`），没承诺修；真正做现算的是 `rows_orchestrator.py` 里「按 sha256 钉住的冻结层，一个字节不改」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/heavy-thread-exempt:55`），而它不在豁免表的 sha256 里 |
| 正推 DR-2、LR-4、EX-3：「被 sha256 钉住」无人核，现行文件与找得到的钉都不同 | x7 的 0、3、4、5、9、10、11 | exempt-check 不比 sha256 | 多一个换行、CRLF、去掉末尾换行（3、4、5）：exempt-check 1、lib_long_run script 1、long-run-check 5；表里 sha256 写成全 0 或大写（10、11）：1、1、5；脚本改名（9）：1、1、5 | 部分修到 | 表里每行带 sha256 并核：「`LIVE_HOSTS_EXEMPT_ROW = re.compile(r"^(?P<path>[^\s#]+)[ \t]{2,}(?P<sha256>[0-9a-f]{64})[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_long_run.py:86`）。六份现行文件的 sha256 与表一致（`out/x7-sha.txt` 第 0 行 exempt-check 0）；但与第一轮正推找到的钉（`m3-leaf-death-r3-snapshot/tree-sha256.txt`）六份仍都不同（`sha256sum` 对照，本腿现查，没入 out/） |
| 同上：表自证 | x7 的 6、7：改脚本并同步改表里的 sha256 | — | exempt-check 0、script 0、long-run-check 0 | 部分修到 | 基准比较只取路径：「`LIVE_HOSTS_EXEMPT_BASE_ROW = re.compile(r"^(?P<path>[^\s#]+)[ \t]{2,}")`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/lib_long_run.py:87`），同一次改动里脚本和表一起改，格不红；所谓「钉住」是表与脚本自洽，不是外部的钉 |
| 正推 SO-1：stage-owners 说「表里的脚本被 sha256 钉住，红了删行或改脚本归实验那一边」，钉住就不能改脚本 | grep | 同 | 同（文字没动） | 没动 | 「`表里的脚本被 sha256 钉住，红了删行或改脚本归实验那一边`」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/gate.d/stage-owners.tsv:48`）；A 里红了的新出路是撤回改动或问用户改 sha256，「改脚本」那一半与 L7 冲突 |
| 正推 EX-3 与 V8 专问：E171 六份按现算、派发提示必须写规范句，两边读法不一致 | hd-header、x6-sweep | — | 表内脚本 header_hosts 判 multi；同内容拷到表外判 single；规范句头的拷贝判 multi | 没动 | 提示一侧仍无豁免；文件头一侧有表。MA-9 的文字已改成只讲规范句 |

### 7.1 X7 专问：E171 下一次修订、换行符、执行位各会怎样（`out/x7-sha.txt`，A 的拷贝里改 `run_segment_one_revision_11_multi_host.sh`）

| 改动 | exempt-check | lib_long_run script | detached-run.sh long-run-check |
|---|---|---|---|
| 原样（0） | 0 | 0 | 0 |
| 只改执行位（1、2） | 0 | 0 | 0 |
| 文件末尾多一个换行（3）；CRLF（4）；去掉末尾换行（5） | 1 | 1 | 5 |
| 改脚本并同步改表（6、7） | 0 | 0 | 0 |
| 同内容拷到表外（8） | — | — | 5 |
| 改名（9） | 1 | 1 | 5 |
| 表里 sha256 写成全 0（10）、大写（11） | 1 | 1 | 5 |
| 符号链接指到表内脚本（12） | — | — | 0 |
| 基准表用现表（13） | 0 | — | — |

- 执行位变化无影响（sha256 只看内容）；A 的根下没有 `.gitattributes`（`ls -a` 现查），git 不会替你换行符，只有编辑器或同步工具改了才红；红的时候 detached-run.sh 起长活直接退 5，也就是 E171 六份入口只要被任何工具动过一个字节就起不来。
- E171 下一次修订的新入口：不进表（表的第 5 行写明），文件头必须规范句，c 份可过；入口之外的编排（segment_one_units_revision_11.py、rows_orchestrator.py）不在 sha256 里，改它们不触发任何红。

## 8 第一轮判对的历史重喂：有没有改坏

不是挑着喂，是把两腿报告里判「一致」「没打中」的喂法连同打中的一起重喂，A 与 B 并排（`out/` 里每条都在）。数（命令：`python3` 读 `out/fm-*.txt`、`out/v6-matrix.txt`，逐行比 A 与 B 的退出码）：

| 喂法族 | 例数 | A=B | A≠B | A≠B 里属于 L 的本意（修到） | A≠B 里改坏第一轮判对的 |
|---|---|---|---|---|---|
| ⑮⑭ 派发闸 `fm-*.txt`（195 个标签） | 195 | 153 | 42 | 38 例 B 放 A 拒（括注 15+5+1、规范句化 17）+ 1 例 B 拒 A 放（规范句后接「；不看忙闲」，定义原话） | 3 例：B 拒 A 放——规范句后接 host-resources 现算、available_parallelism、按核数现算（O-2，见 2.4） |
| 叫停轮登记 `v6-matrix.txt`（47 例，库 + ⑥） | 47 | 38 | 9 | 4 例：有材料的别的轮名不收（m3-close-b1 三种材料）、名单内旧行收 | 3 例 B 不收 A 收：`<名>-r<数字>` 形无材料、other-r2 无材料、已归档旧轮（O-1，见 5.2）；另 2 例（占名单名、名单名带空白）是名单设计，非回归 |
| 重型测试闸 `v5-copy-*` | 7 | 7 | 0 | — | 0 |
| 重型测试闸 `v5-chdir-*` | 11 | 9 | 2 | C2、C8 | 0 |
| 重型测试闸 env 矩阵 | 48 | A 全判对 | — | B 判错 16 | 0 |
| heavy-thread-defaults v3 样本 | 14 份装置 | 4（tests.rs、baseline_read.py、if_default.sh 两版同点名；printenv_read.sh 两版同不点名） | 10 份 B 不点名而 A 点名 | 九种写法 + lib.rs | 0（但 x2 合成样本上 A 点名 8 种不读环境变量的写法，B 一种不点，见 3.2：不属于第一轮判对的，是 L2 回到零轮之前的老误判） |
| private-values 形状 | 6 | 0（B 泄值，A 不泄） | 6 | 全部 | 0 |
| E171 入口 / 豁免表 | 3 份 × 4 道 + 13 种改动 | — | — | — | 见 7（V8-甲 c 份没修到；改一个字节起不来是 L7 的本意） |

- 改坏的三类都是同一种：L 的改法把「认得的写法」收窄成「只认一种/一份」，没被收进来的原来拦得住的写法就漏了——O-1 漏 `-r` 形、O-2 漏现算词、O-3（只缩不涨）则是新加两行表。
- L2 退回宽判法后，第一轮之前那批误判回来了几处、排除表兜没兜住：见 3.2，真树 2 处，2 处都兜住，但表涨了 2 行，格自己的「只缩不涨」会红（基准里没有这两行时）。

## 9 第一轮判决哪一格判重、判轻（逐格，依据文件:行）

判重 = 判得比事实重（站不住/部分站不住的范围或后果过大）；判轻 = 判得比事实轻（站得住、记账、漏记了改法的后果）。

| 格 | 第一轮判 | 本腿判 | 依据 |
|---|---|---|---|
| V1 | 站不住 | 判对 | 误拒与漏放两层都有喂法：括注误拒（统一、万一、十分）与漏放（按核数取）B 都在；但第一轮把「一次被拒的误伤」和「违反固定不看忙闲的漏放」合进同一个「站不住」，后者才是承重的。两层 L1 都修到 |
| V2 | 站得住 | 判轻 | 判决理由「两句都是给主 agent 的规矩，钩子判不了」（`research/prompts/m3-defs-b6-r1-main-verification.md:21`）本身对；漏了 L1 之后的承重：clerk、forward、defense 多数派发要靠已判行顶缺行（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agents/clerk.md:24`），已判行的理由钩子一概不看内容（`out/fm-V2.txt`：写「机器忙，负载高，让出 CPU」或写一个字 x，A=B=C=0；判法「`valve = valve_reason(prompt, "线程上限已判") is not None and not break_switch_is("thread-valve-ignored")`」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/hooks/runner-dispatch-guard.sh:1081`），所以「N 取登记值」对这三种类型事实上只剩主 agent 自觉 |
| V3 | 站不住（零轮更糟） | 方向判对；L2 的代价判轻 | 判决只写「W01 那一处（dispatch-prompt.py 读派发闸表的那一行）进 .claude/heavy-thread-exempt 一行」（`research/prompts/m3-defs-b6-r1-main-verification.md:34`）。实际：真树回来 2 处（W01 与 L2 自己新形 `&&` 带出的 replay.sh），表涨 2 行，与格的「只缩不涨」冲突（3.2）；还漏记 printenv 读法两版都漏（3.1） |
| V4 | 站得住 | 判对 | A 与 B 的 `diff -rq` 里 verification.md、capped.sh、crash 脚本全不在（本腿现查，`diff -rq … \| grep -c` 输出 0），V4 这一格在 L1–L7 里无改动 |
| V5 | 部分站不住 | 甲判轻，乙丙判对 | 判决把 V5-甲「另立账」（`research/prompts/m3-defs-b6-r1-main-verification.md:41`），但 A 的共用约束已把它写成规则：「入口脚本 import 的库改过的同样不起，闸只看入口那一份的正文」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2/repo/.claude/agent-common.md:81`），钩子 S1、S1b、S4 仍放（`out/v5-copy-after.txt`）。按正文第五节「定义说拒而钩子不拒」一条，应列「站不住」而不是记账。乙（说明）、丙（C2、C8）判对且修到 |
| V6 | 站不住 | 日期根判对；「别的轮名」根判重 | 判决写「别的轮名只认 -r<数字> 形与登记过的名字」（`research/prompts/m3-defs-b6-r1-main-verification.md:25`）并把 m3-close-b1 当打中。真仓 310 个认得的轮名里只有 1 个不是 -r 形（`out/` 里没有，本腿在 A 的 `recognised_round_names` 上现数），B 的认法在真仓上几乎不漏，m3-close-b1 是编的名字；而 L4 换法带出反方向的漏（5.2）。日期根（门禁收 ⑥ 拒）真实、已修到 |
| V7 | 站不住（零轮更糟） | 判对；少记一条通道 | 判决只讲逐段打码（`/.claude/gate.d/doc-text.sh:440`）。同一个格里 `fail(f"取不到 {path} …` 的路径也不经 shown_path（6.1），L5 没碰 |
| V8 | 部分站不住 | V8-乙判对；V8-甲的改法判轻 | 判决 L7 栏写它修 V8 甲（`research/prompts/m3-defs-b6-r1-main-verification.md:39`）；同一份判决 V8 行承认「文件头写的读法与脚本做的事是否一致，静态判法看不到」（`research/prompts/m3-defs-b6-r1-main-verification.md:27`）。L7 只能把 `# long-run-hosts:` 逼成规范句，规范句头 + 现算正文 A 三道全过（7，c 份），所以 L7 并没有修 V8-甲，只是把「必须改头」的门槛从任意一句换成逐字一句；判决应写「V8-甲 不修，靠写的人」，别记成已修 |

### 9.1 每条结论什么现象会推翻

| 结论 | 推翻它的现象 |
|---|---|
| ⑮ 括注一概拒、说明行不判（X1 修到） | 派发闸对「线程上限：31（本机）」退 0；或「线程上限说明：…」那一行被当成第二个「线程上限」退 2 |
| 改过头 O-1（`<名>-r<数字>` 无材料的名字改后收） | A 的库对 other-r2（无材料、无登记）退「不收」；或规则文字改成「只认全集」使两边一致（则降为文字问题，行为仍变） |
| 改过头 O-2（规范句 + 现算词放行） | A 对「规范句；host-resources 现算」「；available_parallelism」「；按核数现算」判拒 |
| 改过头 O-3（只缩不涨） | 主工作区 HEAD 那一版 `.claude/heavy-thread-exempt` 已含这两行，格不红；或格改成按「路径」而不是「路径:锚串」比基准 |
| V8-甲没修到 | 把规范句头 + 现算正文的 c 份放进 A 的拷贝，`long-run-check` 退非 0，或 `heavy-thread-defaults` 点名它 |
| 表自证（sha256 同步改） | x7 第 6、7 例 exempt-check 退 1（例如基准比较把 sha256 也比进去） |
| V5-甲没修到 | S1、S1b、S4 任一退 2 |
| printenv 读法没修到 | `out/v3-after.txt` 点名 `printenv_read.sh` |
| DP-1 没修到 | `dispatch-prompt.py:20` 不再写「书记员」 |
| 修到的各行 | 同一条喂法在 A 上退出码回到 B 的（见各表 B/A 两栏） |

## 10 要主 agent 跑的命令（门禁格，子 agent 起不了 --check）

完整写法、预计和树的造法在 `research/prompts/m3-defs-b6-r2-sonnet-model/gate-samples/要主agent跑.txt`（34 行）。摘要（预计都是拿格里的判法函数在小树上跑出来的推测，没起过格）：

1. heavy-thread-defaults「排除表涨了」：在带 git 基准的树里，基准用 B 的 `.claude/heavy-thread-exempt`、现表用 A 的，跑 `bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults <树>`。预计退 1，点名第 82、83 行。**这条最要紧**：真仓里红不红取决于 HEAD 那一版表里有没有这两行。
2. heavy-thread-defaults 两份样本：`cp -a research/prompts/m3-defs-b6-r2-sonnet-model/feeds/v3/sample T/htd`、`cp -a research/prompts/m3-defs-b6-r2-sonnet-model/feeds/x2/sample T/x2`，各 `--check heavy-thread-defaults`。预计 A：v3 点名九种写法与 lib.rs:8，不点名 printenv_read.sh；x2 点名 8 种不读环境变量的写法。
3. long-run-header-exempt：`bash research/prompts/m3-defs-b6-r2-sonnet-model/tools/make_gate_trees.sh T/trees` 后跑 `--check long-run-header-exempt T/trees/exempt-table`。预计退 1，四处（缺路径、已按登记值写齐、sha256 对不上、写法不对）；另把 `research/prompts/m3-defs-b6-r2-sonnet-model/feeds/v8/run_segment_one_revision_12_multi_host.sh` 放进拷贝里，预计 long-run-check 退 5，换成规范句头则三条都退 0。
4. abandoned-rounds：对 T/trees 下 6 棵逐个 `bash .claude/gate.d/doc-process-records.sh --check abandoned-rounds T/trees/<树>`。预计 legacy-sourceless 红、other-name-recognised 红、symlink-source 红，legacy-listed、other-name-unrecognised、other-form-nomaterial 绿；格里另有的检查（有判决、发过腿）库判不到，以格为准。真表预计绿。
5. private-values：`cp -a research/prompts/m3-defs-b6-r2-sonnet-model/gate-samples/doc-text/pv-tar-mirror-red T/pv && (cd T/pv && bash setup.sh) && bash .claude/gate.d/doc-text.sh --check private-values T/pv`。预计 A 退 1，红行里没有 `srv/sample-private`。

## 11 复跑

命令（整条经内存包装，上限 2G，分两趟，工作目录随意；每趟 `--work` 要新目录）：

```
cd /home/fy5090/code/singlefs
BEFORE_FROZEN=~/.local/share/singlefs/work/m3-defs-b6-r1/repo PRE0_FROZEN=~/.local/share/singlefs/work/m3-defs-b3-r2/repo \
  RERUN_PARTS="fm feed v5" bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b6-r2-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b6-r2/repo --work <新目录1>
BEFORE_FROZEN=… PRE0_FROZEN=… RERUN_PARTS="v6 htd x6x7 misc" bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b6-r2-sonnet-model/tools/rerun.sh … --work <新目录2>
```

结果（最终一趟，工具冻结之后）：第一趟 22 行 `result=same`、0 行别的；第二趟 23 行 `result=same`、0 行别的；共 45 份，与 `research/prompts/m3-defs-b6-r2-sonnet-model/out/` 的 45 份一一对应（`ls out | wc -l` 输出 45；`cd out && sha256sum * | sha256sum` 输出 e86773840c82a66125c7c35815102770b2088a1999bb641b9231569482ca05b9）。两趟都带 `nice -n 19`，每趟前台跑完（约 2–3 分钟）。跑完后三份冻结副本 `find … -newer <开工时间戳> -type f` 无输出。
`--frozen` 必须是有 `multi-host.env` 的那一份第二轮冻结副本才会跑 `x5-path-probe`；没有就跳过那一份。

## 12 没做什么

- 没起门禁格（`--check`）、没跑任何 `--selftest`、没跑 gate.sh、没编译、没跑重型测试；门禁格的预计是读格的代码加判法函数得出的，标「要主 agent 跑」。
- 没读禁读清单里的文件：主工作区被判的定义、规则、钩子、脚本、门禁；调度记录；`m3-tooling-*` 实现员自报；攻方 R2 腿的报告与模型目录（`ls` 只看过草稿目录名里有 `m3-defs-b6-r2-opus` 这一个字，没进去）。开工先读的 `.claude/agent-common.md` 与 `.claude/rules/three-way-inference.md` 读的是主工作区那两份，没有拿它们当证据，所有行号都来自冻结副本。
- 主工作区 `HEAD` 那一版 `.claude/heavy-thread-exempt` 里有没有第 82、83 行那两行，没现查（读不了主工作区）；O-3 是否红由主 agent 现查。
- 没替主 agent 撤销或维持判决：本报告只交「修到 / 没修到 / 改过头」与「判重 / 判轻」的复核结果；O-1、O-2、O-3 的改法（文字改成全集，还是代码把 `-r` 形补回来；规范句后的说明是否也过现算词表；表里那两行要不要问用户）不在本腿。
- `tools/rerun.sh --selftest` 退 1：模板的自证测的是模板自带的那一段 `rerun_steps`，这一份换了步骤，自证对它没有意义；复跑以第 11 节的两趟为准。
- 没造新的收严方案；没给 `doc-text.sh:473`、printenv 读法、DP-1「书记员」、stage-owners.tsv:48 出改法，只记了去向。
- 攻方腿一次抽样的「没打中」本腿没再抽第二次；本腿的「没改坏」是对第一轮判对的喂法重喂一遍（单次观测），遇到判错才算数。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r2-sonnet` 下留着 27 MB 的输入与日志（喂法的小树、各趟 `.log`、`out/`）；开工时拷的三份冻结副本拷贝（每份约 0.5 GB）、各趟 `--work` 里的仓拷贝（`xtree`、`x6rev12`、`x7`、`x5`）都已删掉，没有留编译目录与工作树。
