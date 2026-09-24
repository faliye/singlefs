**出处 `.claude/singlefs-ai-sop/rules/show-me-test.md:60-87`（整段抄，未转述）**

```markdown
## 踩过的坑要做成会失败的检查，不要做成提醒句

写一句「注意别在 X 的时候做 Y」，拦不住手敲命令的人。
**能拦住的是一个当场拒绝执行的检查。**
踩到新坑，第一反应该是「这条怎么做成门禁里会红的一项」，不是「这条写进哪个文档」。

### 但要先问这个坑在哪一层：装置里，还是口径上

**立在一个装置里的检查，管不住下一个装置。** 断言与变异测试都只在它所在的那套装置里生效；
换一套装置从头把同一件事再建一遍模型，那条检查一个字都不会说。


⇒ **判据不是「这个坑做没做成会红的检查」，是「这个坑在哪一层」**：

| 坑在哪 | 检查立在哪 |
|---|---|
| 一段代码的行为里 | 那段代码旁边就够 |
| **建模口径上**（同一个量该怎么算、边界算不算、单位是什么） | **立在装置之间**：两套装置算同一个量，就要有一条检查逼它们落到同一个数；对不上判红 |

**快速判据**：换一套装置、从头把同一件事再做一遍，还会不会再踩一次？
会 ⇒ 它在口径层，检查跨装置才有用。
⚠️ 修坑的人最容易在这里收手：他确实把坑做成了会红的检查，**证据齐全、门禁全绿**，
所以不会再想一遍这条检查的射程有多远。

⚠️ **跨装置的检查也可能只钉住了值，没钉住量。**
⇒ **跨装置的检查要钉住「这个名字指的是哪个量」**：两个量就登记两个名字，各钉各的值；
只比「值对不对」的检查，在一个名字被当成两个量用的时候一声不吭（`code-discipline.md`「一个语义概念全仓一个名字」的另一面）。

```

**出处 `.claude/singlefs-ai-sop/rules/show-me-test.md:97-121`（整段抄，未转述）**

```markdown
## 门禁不许假装通过

没实现的门禁阶段要**明说没实现**，不许悄悄跳过。
一个偷偷没跑崩溃测试的绿门禁，比红门禁危险得多。

**项目本地阶段这一轮无对象可判，退出码写 77**：`gate.sh` 记「本次未跑」，不记通过，也不算覆盖。
`exit 0` 的跳过在汇总里与「判过了」一模一样，而门禁分不出这两种——这一半靠写阶段的人。

同理，**跑批脚本不许把单轮的失败吞掉**（`|| true` 那种），输出路径也不许跨轮复用。
这两样凑一起，失败的那轮会安静地拿上一轮的输出顶上——看着一切正常，只是数字不动。
判据是「这份输出能不能证明是这一轮产生的」：跑之前删掉旧输出，跑完认一个本轮才会出现的完成标记。

**扫到 0 项也不是通过。** 判据的搜索范围写窄一点，对象就会全被第一步跳过——
既不算通过也不算失败，末尾照样报绿，而没人看得出来。
⇒ 扫一批对象的检查，成功那句里要报出**检查了多少项**；`gate-lint.sh` 判这一条。

**报了「查了多少」还不够，「没查的是哪些」也要逐个列出，而且清单要现算。**
一个阶段可以老老实实报出检查了多少项，同时把另一半对象整批漏掉：两句话都为真，
而读的人看不出后一句存在。
`gate-lint` 全绿，而那张表里 9 个计时行一个都没跑，其中 6 个既没跑、也没出现在任何一行里。
被掩住的是两处真问题：一个实验的留存产物早就对不上源码，另一个在 release 下直接 panic 跑不起来。
根因是跳过清单手抄——写死成 4 个编号，而真正没跑的是「表里全部计时行」加上那 4 个。
**所以跳过清单要与被扫集合出自同一份数据、现算**；成功那句同时报「跑了 N 个；没跑 M 个：逐个列名」。
这一条还没做成检查：`gate-lint` 现在不看报了数的脚本有没有同时列出跳过项，靠写阶段的人自己守。

```

**出处 `.claude/singlefs-ai-sop/rules/show-me-test.md:122-146`（整段抄，未转述）**

```markdown
# 一条改动不同步铺满全仓时，排除的每一份都要登记

「跳过清单要与被扫集合出自同一份数据、现算」那一条说的是**一道检查**的跳过清单。同一件事对**一次改动**同样成立，而且更容易被绕过去：
一个规范、一次操作、一次改动只铺了一部分文件时，**没铺到的要逐条登记进一张排除表**，
一行一条、`#` 后面写为什么不铺。**没有排除表就意味着全仓都要改。**

判据不是「改起来麻烦」，是「改了这句话就成假的，或者改了某道闸就失效」：
别家项目的术语与原文引文、某道检查自己的输入、工具自己的模式表、说这次改动本身的那句话——
这几类改了会出错，登记；其余一律改。

排除表照跟检查一样办：**指向不存在的路径判红**——不起作用的排除项会让人以为那批文件已经被绕开了。
表本身要能被一条命令读，别让「排除了什么」只活在做这件事的人的记忆里。

**成功那句里报的数，本身也要有东西钉住。** 它是成功行，永远不会红；它报了数，
满足「扫一批对象的检查要报出检查了多少项」那一条；它不是拒绝，`gate-lint` 另外三条都够不着它。
三样加起来，一个算错的统计量可以在门禁里一直绿着，而它正是给人看的那个结论。

**项目本地阶段与共享阶段同规矩。** 它们一样会拒绝提交者，一样受 `gate-lint`
与 `shell-lint` 管——`gate.sh` 把 `.claude/gate.d/` 一并交给这两个 lint。

⚠️ **射程只到 `.claude/gate.d/`。** 项目别处的脚本（研究脚本、hook）一样会拒绝人，却不在这两个 lint 的射程里。
⇒ 项目有这类目录，就在 `.claude/gate.d/` 里接一个本地阶段，把它们交给这两个 lint：调用时分别设 `GATE_LINT_DIR`、`SHELL_LINT_DIR` 指到目标目录——
不设的话共享脚本默认还会扫 SOP 自己的包，拿样本目录判红绿时就混进了真仓的脚本。
只接 gate-lint 等于只补了一半

```

**出处 `.claude/singlefs-ai-sop/rules/command-safety.md:160-170`（整段抄，未转述）**

```markdown
## 子 shell 里的赋值传不回父进程

`rc="$(run_one ...)"` 这种写法里，`run_one` 内部给变量的赋值**在父进程里是空的**。
如果抓结果要用到那个变量（比如日志路径），就会永远抓到零条，而退出码是 0——
又一次绿着灯出错。**要传值就落到文件或者走参数，别靠变量。**

在 `set -u` 下更糟：引用的地方拿到的不是空值，是当场报「unbound variable」，
**脚本还没来得及打印诊断就被带走了**。
`howto` 一句都没打出来，而装置说谎的时候最该看的就是那几句。
写条注释提醒拦不住这种事，它是 `scripts/shell-lint.sh` 里会红的一项。

```

**出处 `.claude/singlefs-ai-sop/rules/command-safety.md:171-181`（整段抄，未转述）**

```markdown
## 脚本改文件之后要回读确认，警告是免费的信号

用脚本做字符串替换来改代码或文档，**匹配不上是不会报错的，它就什么都不做**。
缩进差一个空格、引号换了个体、行尾多个空格，替换就悄悄失效了，而退出码是 0。

**两条做法**：
1. **替换必须断言命中**：替换数为 0 就报错退出，不许「改完就当改了」。
2. **改完回读**：grep 一下新内容在不在，或者直接跑一遍看行为变没变。

⚠️ **编译器和 linter 的警告是发现这类失效最便宜的信号，不许略过。**

```

**出处 `.claude/singlefs-ai-sop/rules/command-safety.md:182-193`（整段抄，未转述）**

```markdown
## 进程边界上的三种静默失效

`set -e`、`pipefail` 和环境变量在**跨进程**的地方一起出问题，形态都是「退出码对，判据没跑」：

| 形态 | 实测 | 怎么写 |
|---|---|---|
| `set -e` 下写 `里层; rc=$?` | 里层一判红，外层在这一行就退出，后面的清理走不到——而判红正是最要看结果的时候 | 退出码在 `if` 里取：`if 里层; then rc=0; else rc=$?; fi` |
| `export X="$(cmd)"` / `local x="$(cmd)"` | `export` 与 `local` 是命令，它们自己的退出码把命令替换的盖掉了；`cmd` 失败也当成功 | 先赋值再 `export`，两行写 |
| 握手用的环境变量漏给子进程 | 外层设给里层的变量还在环境里，里层起的第三层拿它当自己的输入——实测：门禁自检嵌套跑时反红两例 | 读完就 `unset`；起子进程时 `env -u` 清掉 |

**所以判据是「这个值跨了几层进程」**：跨一层就要问它在下一层还在不在、该不该在。

```

**出处 `.claude/singlefs-ai-sop/rules/kb-discipline.md:73-90`（整段抄，未转述）**

```markdown
## 2. 每条带出处与状态

来源、是实测还是推理、什么口径。日期只跟着事件走：实测、读来的结论写明是哪天做的；
说现状的句子不带日期（见第 8 条下的「带日期的现状快照也是历史，会停在那一天」）。

这不是学究气：**模型只能靠这个来分辨「这条已经定了」和「这条当时只是猜的」。**
线索没了，两者在检索结果里长得一模一样。

- 数字必须带口径：是块数还是字节数、含不含元数据、什么硬件什么负载下测的。
- 从别处读来的结论要标来源，并注明「未在本项目验证」。
- **引用别处的实测数字，要写清它能证明什么、不能证明什么。**
  光有口径不够。口径回答的是「这个数怎么来的」，
  而检索到它的人真正想问的是「这个数能不能用到我这儿」。
  中间隔着一步外推，**这步外推得由写的人做完并写下来**。
  留给下一个人，他多半不会做，会直接拿去用。
- **所有旧数据都只是参考。** 每条实测都绑在当时那个构建上，
  想拿它支撑新结论，先复跑一遍，确认今天还成立。

```

**出处 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:229-280`（整段抄，未转述）**

```markdown
## 新立一条判据，当场拿它回扫已有的条目

**一份判据只执行一半，下一个人会照松的那一半用。**

⇒ 立完判据先问：**它对已经在册的条目判什么？** 答不上来就是还没立完。

### 撤回一个数或一条结论，同样要当场回扫谁在引它

同一条纪律，对象换成**被撤回或被改写的数值与条款**。它比「新立一条判据，当场拿它回扫已有的条目」更隐蔽：
撤回的人通常确实扫了几处，于是他有理由认为自己扫完了，而漏掉的那处要过几天才发作。

**判据**：撤回之后问一句「**全仓还有谁在用这个数**」，答得出一份清单才算撤完。
扫的范围是全仓，不是「我记得的那几处」——记忆挑出来的正是好找的那几处。


⇒ 这与「喂给多方论证的背景材料，本身要先核」是同一个洞的两端：
一端是引的人没现查，另一端是撤的人没扫干净。**两端都堵，才堵得住。**

⚠️ **扫出来的清单不能一把全替换：每一处先分清它说的是现状，还是那一次发生的事。**
同一个旧值在仓里常常一半是「现在是多少」，一半是「那一次改成了多少」「从第几次跑起按什么写」——后一种一直是真的，替换掉反而成了假话。

**判据：把这句话里的旧值换成新值，它还是不是真话。** 是，就说明它说的是现状，改；不是，就说明它说的是那一次，留着。

⚠️ **回扫要按旧说法搜，不能只按新名字搜。** 过时的句子写的是旧说法（「只有第一个事务」「还没有第二个实例」），
拿这一阶段新做出来的东西的名字去搜，搜到的多半是已经改好的句子。
现成的旧说法清单就是这一阶段自己的 diff：它在某处删掉或改写的现状句，别处往往原样还活着。

⇒ 回扫之前先写出「这件事做成之前仓里会怎么说它」（没有 X、X 还没、只有 Y、N 条），拿这些说法全仓搜；
这一阶段的 diff 里删掉的现状句，也逐句搜一遍别处还有没有。

### 撤回的理由失效，不等于被撤回的结论自动回来

这是「撤回一个数或一条结论，同样要当场回扫谁在引它」的反面，而且更容易犯——它看起来像是在纠错。

一条结论被撤回时，落款的理由往往只是当时最好写的那一条。过些天那条理由自己塌了
（前提变了、被它依赖的决策改了），很容易顺手推出「那撤回就不作数了，原结论该回来」。
**推不出来。** 撤回是一次判决；理由塌了只说明这次判决的依据没了，不说明反面成立。
要让原结论回来得**重新论证**，而重新论证可能得出相反的答案。

**还有更值钱的一步**：撤回之后，那一格通常已经有另一条依据顶上来了。
只盯着被撤回的那条，会把顶上来的那条整个漏掉——而后者才是今天真正承重的东西。

⇒ **发现撤回的理由失效时，按顺序问三句**：

| 问 | 答不出就别往下走 |
|---|---|
| 今天顶在这一格上的依据是哪一条 | 说得出它住在哪个文件的哪一段 |
| 那条依据自己站不站得住 | 逐个支点去查，别只读它的结论句 |
| 要让被撤回的那条复活，重新论证做了没有 | 没做就是没做，不许拿「理由塌了」代替 |

⇒ 三条里没有一条与「互斥还在不在」有关。**顺着「前提没了所以撤回作废」走，一条都碰不到。**

```

**出处 `.claude/rules/three-way-inference.md:147-162`（整段抄，未转述）**

```markdown
## 一条腿只抽一次样不算一次观测——否定结论尤其不算

模型的答复是**有变化的观测**，因此 `.claude/singlefs-ai-sop/rules/test-discipline.md`
「单次观测不算数」那条对它成立：同一份提示、同一个模型，两次可以给出方向相反的答案。

⚠️ **两个方向的门槛不一样**，与那条规则同形：

| 结论 | 采信条件 |
|---|---|
| **打中了**（给出反例、指出矛盾） | 一次就值得去核。它是个线索，真伪由主 agent 现查坐实，抽样次数不改变这一步 |
| **没打中**（「构造不出反例」「没发现问题」） | **一次不算**。它与「这一轮它没想到」分不开，而两者在答复里长得一模一样 |

⇒ **做法**：拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次；两次都没打中才记「没打中」，
两次不一致就照 `.claude/singlefs-ai-sop/rules/test-discipline.md` 记「不稳定」，不下结论。
云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的。

```

**出处 `.claude/singlefs-ai-sop/scripts/lib.sh:88-140`（整段抄，未转述）**

```markdown
# ── 日期能不能是真的：唯一定义 ──────────────────────────
# 编出来的日期看不出是编的，除非它落在不可能的区间里。两端都可机检：
# 比这个仓第一个提交早一个月以上、或者比今天还晚，都不可能是真发生过的事。
# 实测：三份样本和一处 skill 示例里写着 2026-01-01，比这个仓的第一个提交早了大半年，而门禁一直是绿的。
# 这个包自己的起点：`git log --reverse` 现查，本仓第一个提交是这一天。
# 样本、模板、规则里的日期说的都是这个包的事，下界就用它，不随包被拷到哪里而变。
SOP_START_DATE=2026-08-26

# 被检查的那个项目自己的起点。**只在 <目录> 本身就是 git 仓的顶层时**才问 git：
# 不这么限的话，`git -C` 会一路往上找——SOP 副本放在项目的 .claude/ 下，找到的是项目的历史，
# 拿项目的第一个提交去判这个包的样本日期，量的就不是同一件事。
# 也不拿 CHANGELOG 最早那一节兜底：它从 0.0.22 才开始逐节记，比真起点晚一周，
# 拿它当下界会把 2026-08-29 这类真日期判成不可能（实测）。拿不到就返回空，由调用方决定退到哪。
project_start_date() { # project_start_date <目录> → YYYY-MM-DD 或空
  local dir toplevel
  dir="$(cd "$1" 2>/dev/null && pwd -P)" || return 0
  toplevel="$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null || true)"
  [[ -n "$toplevel" && "$(cd "$toplevel" && pwd -P)" == "$dir" ]] || return 0
  # 浅克隆里「第一个提交」是截断处，不是项目的起点：拿它当下界，项目越老误判越多（审核实测）。这时也返回空。
  [[ "$(git -C "$dir" rev-parse --is-shallow-repository 2>/dev/null || true)" != true ]] || return 0
  git -C "$dir" log --reverse --format=%ad --date=short 2>/dev/null | head -1 || true
}
# 下界从第一个提交往前放宽 DATE_GRACE_DAYS 天：git init 之前做的工作，会带着当时的日期进第一个提交。
# 实测于使用者项目：第一个提交的当天，那次提交里就有 5 条前一天的历史条目，按第一个提交卡死全被判成不可能。
# 实测只早一天，放宽一周；宽得越多，放过的编造日期越多。2026-01-01 这种早了大半年的照样拦得住。
DATE_GRACE_DAYS=7
date_lower_bound() { # date_lower_bound <起点 YYYY-MM-DD 或空> → 下界或空
  [[ -n "${1:-}" ]] || return 0
  date -d "$1 - $DATE_GRACE_DAYS days" +%F
}
# 下界要做日期减法，靠 GNU date 的 -d。不认 -d 的 date（busybox、BSD）算不出下界，
# 而 date_out_of_range 是在 if 条件里调的，set -e 不管：下界静默变空，2026-01-01 照样放行（审核实测）。
# 所以查日期的脚本开头先试一次，不行就停。
require_date_arithmetic() {
  [[ "$(date -d '2026-01-02 - 1 day' +%F 2>/dev/null || true)" == 2026-01-01 ]] || die "这台机器的 date 不认 -d，算不出日期下界" \
    "日期检查要 GNU date（coreutils）：装上它，或把它放到 PATH 前面，再重跑。env.sh 也查这一项。"
}
# 「今天」按地球上最晚的那个时区（UTC+14）算：比它还晚的日期，在哪儿都还没到。
# 只取本机时钟的日期不行：本机时钟是 UTC、人在东京时，东京 00:00–09:00 写下的当天日期比 UTC 的今天晚一天，
# 会被判成「晚于今天」（审核实测于使用者项目：194 个提交里有 8 个在这个时段按东京日期写了历史条目）。
# 用 POSIX 写法 UTC-14（符号与直觉相反，表示 UTC+14），不依赖系统装没装时区数据。
latest_today() { TZ=UTC-14 date +%F; }
# 在范围内时不输出、返回 1；不在范围内时打印「为什么不可能」、返回 0。
date_out_of_range() { # date_out_of_range <YYYY-MM-DD> [项目起点]
  local checked_date="$1" start_date="${2:-}" today lower_bound
  today="$(latest_today)"
  if [[ "$checked_date" > "$today" ]]; then printf '晚于今天（%s，按最晚的时区算）' "$today"; return 0; fi
  lower_bound="$(date_lower_bound "$start_date")"
  if [[ -n "$lower_bound" && "$checked_date" < "$lower_bound" ]]; then
    printf '早于这个仓第一个提交（%s）%s 天以上' "$start_date" "$DATE_GRACE_DAYS"; return 0
  fi
  return 1
}
```

**出处 `.claude/singlefs-ai-sop/scripts/doc-lint.sh:606-623`（整段抄，未转述）**

```markdown
  # ── M. 日期不许是编的 ───────────────────────────────────
  # 历史条目的 `### 日期` 与正文里的 `实测（日期）` 都是在说「这件事哪天发生的」。
  # 编出来的日期看不出是编的，除非它落在不可能的区间里：比这个仓第一个提交早出宽限、或者比最晚时区的今天还晚。
  # 实测：三份样本与一处 skill 示例里写着 2026-01-01，比第一个提交早了大半年，而门禁一直是绿的。
  # 三种语言的「实测（日期）」写法都认：en 仓写 Measured (，ja 仓写 実測（（只认中文那种时，en / ja 的这一半一条都没查，审核实测）。
  while IFS=: read -r date_line_no date_rest; do
    [[ -n "$date_line_no" ]] || continue
    date_value="$(printf '%s' "$date_rest" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' || true)"
    date_value="${date_value%%$'\n'*}"
    [[ -n "$date_value" ]] || continue
    if date_why="$(date_out_of_range "$date_value" "$PROJECT_START_DATE")"; then
      bad "$rel:$date_line_no  日期 $date_value 不可能：$date_why"
      howto "写真实发生的那一天：历史条目写这一条是哪天记下的，实测写它是哪天跑的。" \
            "不知道是哪天就去查（git log 那个文件、或者当时的产物），别填一个占位日期——" \
            "占位日期读起来和真日期一模一样，而后面每一个引用它的人都会当真。"
      filefail=1
    fi
  done < <(grep -nE '(^###[[:space:]]+|实测（|実測（|Measured \()[0-9]{4}-[0-9]{2}-[0-9]{2}' "$f" || true)
```

**出处 `.claude/gate.d/95-fixture-claims.sh:120-193`（整段抄，未转述）**

```markdown
# ── 第 ④ 条：日期不许是编的 ────────────────────────────
# 判据 source 上游 lib.sh，不另写一份：两套装置算同一个量，就要落到同一个数。
# lib.sh 自带 set -euo pipefail，所以整段关在子 shell 里，判定经文件带出来
# （command-safety.md：子 shell 里的赋值传不回父进程）。
date_report="$(mktemp)"
date_rc=0
(
  source "$LIB"
  require_date_arithmetic
  git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'SKIP\t不是 git 仓，列不出要扫的文件\n'; exit 0; }
  start_date="$(project_start_date "$(pwd)" || true)"; start_date="${start_date:-$SOP_START_DATE}"
  date_re='[0-9]{4}-[0-9]{2}-[0-9]{2}'
  scanned=0; dated=0; bodies=0; body_dates=0
  # 路径走 NUL：git 默认把非 ASCII 路径整条引起来并转义成八进制，报出来的那一行既搜不到也对不上 want
  while IFS= read -r -d '' path; do
    scanned=$((scanned + 1))
    [[ "$path" =~ $date_re ]] || continue
    dated=$((dated + 1))
    while IFS= read -r found; do
      [[ -n "$found" ]] || continue
      if why="$(date_out_of_range "$found" "$start_date")"; then
        printf 'BADNAME\t%s\t%s\t%s\n' "$path" "$found" "$why"
      fi
    done < <(grep -oE "$date_re" <<<"$path" | sort -u)
  done < <(git ls-files -z --cached --others --exclude-standard | sort -z -u)
  # 正文只扫 fixtures：真 kb 正文里的日期常常说的是外部的事（别家项目、RFC 历史、引编造日期当例子），
  # 而下界假设「本仓的事不可能早于本仓第一个提交」对它们不成立。
  while IFS= read -r -d '' body; do
    bodies=$((bodies + 1))
    while IFS= read -r hit; do
      [[ -n "$hit" ]] || continue
      line_number="${hit%%:*}"; found="${hit##*:}"
      body_dates=$((body_dates + 1))
      if why="$(date_out_of_range "$found" "$start_date")"; then
        printf 'BADBODY\t%s:%s\t%s\t%s\n' "$body" "$line_number" "$found" "$why"
      fi
    done < <(grep -noE "$date_re" "$body" 2>/dev/null | sort -u -t: -k1,1n)
  done < <(git ls-files -z --cached --others --exclude-standard '.claude/gate.d/fixtures/*.md' | sort -z -u)
  printf 'COUNT\t%s\t%s\t%s\t%s\t%s\n' "$scanned" "$dated" "$bodies" "$body_dates" "$start_date"
) > "$date_report" 2>&1 || date_rc=$?

if grep -qE '^BAD(NAME|BODY)' "$date_report"; then
  date_rc=1
  if grep -q '^BADNAME' "$date_report"; then
    echo "  ✗ $(grep -c '^BADNAME' "$date_report") 个文件名里的日期不可能是真的："   # gate-lint:summary
    while IFS=$'\t' read -r _ where found why; do
      printf '      %s：%s %s\n' "$where" "$found" "$why"   # gate-lint:detail
    done < <(grep '^BADNAME' "$date_report")
  fi
  if grep -q '^BADBODY' "$date_report"; then
    echo "  ✗ $(grep -c '^BADBODY' "$date_report") 处样本正文里的日期不可能是真的："   # gate-lint:summary
    while IFS=$'\t' read -r _ where found why; do
      printf '      %s：%s %s\n' "$where" "$found" "$why"   # gate-lint:detail
    done < <(grep '^BADBODY' "$date_report")
  fi
  echo "     → 怎么办：写真实发生的那一天——记录写它是哪天记下的，产物写它是哪天跑的，"
  echo "       样本写它所属 fixture 的创建日：git log --reverse --format=%ad --date=short -- <fixture 目录>；"
  echo "       样本里要表示先后两次事件的，就用创建日和它的次日，别用 2026-01-01 / 2026-01-02 这种占位。"
  echo "       查不出来也别填占位日期：占位日期读起来和真日期一模一样，后面每一个引用它的人都会当真。"
  echo "       红样本非要一个编造日期不可的，让 fixtures/<阶段>/red/setup.sh 在临时仓里现造，别摆进仓里——"
  echo "       摆进来它会被这一条自己扫到，红的是样本而不是被测的东西。"
elif grep -q '^SKIP' "$date_report"; then
  printf '  ⊘ 日期这一条未跑：%s\n' "$(grep '^SKIP' "$date_report" | cut -f2)"
elif ! grep -q '^COUNT' "$date_report"; then
  date_rc=1
  echo "  ✗ 日期这一条没跑完，报告里没有计数行："
  sed 's/^/      /' "$date_report"   # gate-lint:detail
  echo "     → 怎么办：上面是子 shell 的原样输出；多半是 lib.sh 取不到或 git 列不出文件，按它说的修。"
else
  read -r _ scanned dated bodies body_dates start_date < <(grep '^COUNT' "$date_report")
  printf '  ✓ 日期都可能是真的（文件名：%s 个文件里 %s 条带日期；样本正文：%s 份 .md 里 %s 个日期；下界 %s 往前宽 7 天）\n' \
    "$scanned" "$dated" "$bodies" "$body_dates" "$start_date"
fi
rm -f "$date_report"
```

**出处 `.claude/gate.d/fixtures/95-fixture-claims.sh/red/setup.sh:1-19`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# 红样本要判的第三条是「样本目录是空壳」：目录在，里面一个 red / green 都没有。
# 这个空目录只能在这里建——git 不跟踪空目录，放在样本的文件树里带不进 `gate.sh --staged`
# 的临时 worktree，那一格就判不出来（stage-selftest.sh 先把样本拷进临时目录再跑这个脚本）。
set -euo pipefail
mkdir -p .claude/gate.d/fixtures/51-hollow.sh

# 第 ④ 条（日期不许是编的）的坏样本也在这里现造，不摆进仓里——摆进来会被这一条在真仓跑时自己扫到（C441）。
# 先做一个真实日期的提交：project_start_date 取不到值时下界为空，红样本就只证了上界，
# 而这次踩的坑在下界（2026-01-01 比第一个提交早大半年）。
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
GIT_COMMITTER_DATE="2026-09-01T00:00:00" GIT_AUTHOR_DATE="2026-09-01T00:00:00" sh -c 'git add -A && git commit -qm base'
# 文件名那一维
mkdir -p records
printf '# 样本\n' > records/2026-01-01-样本.md
# 正文那一维：挂在已有的孤儿目录下，不新建目录——新建会把第 ① 条的红（声称了而目录不存在）消掉
printf '## D99 样本决策 —— 已定\n\n| # | 名字 | 状态 |\n|---|---|---|\n| 1 | 甲项 | **已定（2026-01-02）：取甲** |\n' \
  > .claude/gate.d/fixtures/99-orphan.sh/red/样本决策.md
```

**出处 `.claude/gate.d/fixtures/95-fixture-claims.sh/red/expect:1-11`（整段抄，未转述）**

```markdown
exit=1
want=1 个阶段的头部声称有判别力样本，而目录不存在
want=50-claims-but-missing.sh
want=1 个孤儿样本目录，没有对应的阶段
want=99-orphan.sh
want=1 个样本目录是空壳
want=51-hollow.sh
want=1 个文件名里的日期不可能是真的
want=records/2026-01-01-样本.md
want=1 处样本正文里的日期不可能是真的
want=早于这个仓第一个提交
```

**出处 `.claude/gate.d/fixtures/95-fixture-claims.sh/green/setup.sh:1-12`（整段抄，未转述）**

```markdown
#!/usr/bin/env bash
# 绿样本也要建 git 仓：不是 git 仓时第 ④ 条报「未跑」，那一格就没验到。
# 两维各放一个日期落在可能区间里的对象，证明这一条在绿的一侧真的在看——
# 扫到 0 项而报绿，与判过了一模一样（show-me-test.md）。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
GIT_COMMITTER_DATE="2026-09-01T00:00:00" GIT_AUTHOR_DATE="2026-09-01T00:00:00" sh -c 'git add -A && git commit -qm base'
mkdir -p records
printf '# 样本\n' > records/2026-09-02-样本.md
printf '## D99 样本决策 —— 已定\n\n| # | 名字 | 状态 |\n|---|---|---|\n| 1 | 甲项 | **已定（2026-09-02）：取甲** |\n' \
  > .claude/gate.d/fixtures/50-good.sh/green/样本决策.md
```

**出处 `.claude/gate.d/fixtures/95-fixture-claims.sh/green/expect:1-6`（整段抄，未转述）**

```markdown
exit=0
want=阶段头部声称的判别力样本都在
want=没有孤儿目录
want=日期都可能是真的
want=1 条带日期
want=1 份 .md 里 1 个日期
```

**出处 `.claude/kb/checks-owed.md:392-392`（整段抄，未转述）**

```markdown
| C441 | 全仓清扫工具会吃掉自己的判别力样本 | 术语清扫按登记表全仓替换时，把门禁 90 号自己的红绿两份样本也扫了：`fixtures/90-term-renames.sh/{red,green}/.claude/kb/term-renames.md` 的**旧名那一列**被换成新名，红绿两份内容变得一模一样，判别力当场归零——而 90 号本身照样报绿，只有阶段自检报「green 期望退出 0，实测 1」时才露出来（2026-09-21 实测）。同一形态对任何「按表替换全仓」的工具都成立：它的样本里必须留着被替换的那一侧 | 判据：一道检查的判别力样本被它自己扫过之后，红样本还红不红。做法是让清扫工具默认把 `fixtures/<自己的阶段名>/` 排除掉，排不掉就在自检里造一次「样本被自己扫过」的场景、要求红样本仍然红 | 无 | 2026-09-21 archive-rename 轮：`.claude/term-rename-exempt` 里那条 `fixtures/90-term-renames.sh/` 的登记 |
```

**出处 `.claude/kb/checks-owed.md:455-455`（整段抄，未转述）**

```markdown
| C510 | 编造的日期没人拦 | **「日期不许是编的」这条禁令住在上游**（`.claude/singlefs-ai-sop/scripts/doc-lint.sh` 的 M 段，判据是 `lib.sh` 的 `date_out_of_range`：早于本仓第一个提交宽限 7 天、或晚于最晚时区的今天，都不可能是真发生过的事），**而它只认正文里的两种形态**：历史条目的 `### 日期`、正文里的「实测（日期）」。2026-09-23 实测：门禁样本里 8 个文件名与 32 处正文写着 2026-01-01 / 2026-01-02，比本仓第一个提交（2026-08-26）早大半年；带一个 `records/2026-01-01-临时实测.md` 跑 64 道阶段加 doc-lint，**0 道点名它**。一个编造日期读起来和真日期一模一样，而门禁样本正是后来人照抄形态的样板 | **已还大半（2026-09-23，门禁 95 号第 ④ 条）**：判据 source 上游 `lib.sh` 的 `date_out_of_range`，不另写一份；两维各判各的、各报各的数——文件名扫全仓路径（现查 1651 个文件、90 条带日期），正文只扫 `.claude/gate.d/fixtures/` 下的 `.md`（现查 173 份、80 个日期）。当天把那 8 个文件名与 32 处正文改成各自 fixture 的创建日（样本里要表先后两次事件的用创建日与次日）。判别力在 `fixtures/95-fixture-claims.sh/` 双向证过：red 的 `setup.sh` 在临时仓里现造一个坏文件名与一处坏正文（**不摆进仓里**，摆进来会被这一条自己扫到，C441），并先做一个真实日期的提交让 `project_start_date` 有值、下界那一支真的走到；green 两维各放一个合法日期，成功句报「1 条带日期」「1 份 .md 里 1 个日期」，防「扫到 0 项也报绿」。**仍欠三样**：① `.claude/doc-lint-exclude` 里每行要写清它关掉了哪几类检查——只为绕开登记冲突而整份排除，等于把日期、指代、编号简称几条一起关掉，排除项该按检查分格、不按文件分格；② 运行时才生成的编造日期 95 号扫不到（`fixtures/60-stale-open-items.sh/*/setup.sh` 的 `GIT_COMMITTER_DATE="2026-01-01"`、`fixtures/91-archive-past-rounds.sh/red/setup.sh` 造的 `e1-old-round-2026-01-01.out`、`research/scripts/stale-candidates.py` 自检里的 `records/2026-01-01-log.md`）；③ 真 kb 正文里的 `已定（日期）`、`—— 已跑（日期）` 形态**明确不做**，依据见下一格 | **射程不许再放大，这条是实测定的**：2026-09-23 扫全仓正文得 52 处越界，其中 20 处全部合法——`prior-art.md` 9 处（别家项目的真实日期：bcachefs 手册 2026-04-16、`k1024.org/posts/2019/2019-02-08`、Greg KH 2026-07-15 表态）、`decisions-history/2026-09.md` 3 处（RFC 演进史 2015-08-07 → 2016-11-14 → 2017-01-30）、`checks-owed.md` 8 处（这一行自己引 2026-01-01 当例子）。`date_out_of_range` 的下界假设是「本仓的事不可能早于本仓第一个提交」，**这只对说本仓自己事的日期成立**；上游 M 段只认那两种形态不是射程不够，是有意收窄。⚠️ 本条 2026-09-23 早先的版本提议「上游把 M 段射程扩到表格单元格与标题括注」，**当天被这次实测推翻、已撤回** | 2026-09-23 用户看到 `fixtures/81-audit-contradictions.sh/red/records/2026-01-01-总审核.md` 问「我们禁止了这样的命名，为什么还出现」，逐处现查后立 |
```
