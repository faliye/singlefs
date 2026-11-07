#!/usr/bin/env python3
# admission: always 每一次调都按此刻的定义打骨架或核一份提示，上一次的结论不替这一次作保
# run-condition: command bash
"""派发提示的骨架：按定义的 required-inputs 与共用约束要的固定行打出来，主 agent 只填内容；核一份提示还剩不剩占位。

用法：
    dispatch-prompt.py <类型> [--set 名=值 …] [--batch <调度记录>#<批号>] [--out 文件]   # 打骨架；--set 直接填某一行
    dispatch-prompt.py --check <提示文件> [--type <类型>] [--model <模型>] [--root <仓根>]  # 派之前一次列全派发闸会拒的地方、退 3
    dispatch-prompt.py --selftest                                                      # 下面的弄坏开关设了哪一个都必须判红

--check 的判法不在这里另写：知道类型（--type，或骨架第一行「# 派发：<类型>」）就起派发闸自己的
`bash .claude/hooks/runner-dispatch-guard.sh --check-prompt <提示文件> --type <类型> --root <仓根>`，同一份判法各条都判完，
不成立的一次列全（占位、定义要的输入、写范围、快照、批次一行、书记员规格……）；⑤ 与在跑的实现员撞文件、⑦ 并发
要会话状态，写成「派的那一刻另判」。不知道类型、或派发闸跑不起来的，只查占位（<填 …>），并写明别几条没核。

为什么：派发闸拒的原因多半是缺一行（岔路行、要改的文件、出口、还差）；主 agent 派完再给子 agent
发的纠正或追加也多是补缺项。缺项该在写的时候补，不该在派的时候拒：骨架把定义要的每一样都列成一行占位，占位没填完就不派。
骨架里的固定行与派发闸（.claude/hooks/runner-dispatch-guard.sh）判的是同一套：开工先读（没有定义的类型）、
定义的 required-inputs 每组一行、实现员的「要动的 crates 文件」、写回员的「规格检查已判」、执行员的「这一段回答的岔路」、每种类型的「批次」一行。
「批次：」一行派发闸 ⑱ 判形如 records/<文件>.md#<批号>、文件在本仓或登记的工作树里，十种类型缺这一行拒（判法只在派发闸文件头 ⑱ 条）。
「自证在哪跑」一行派发闸不单判，没填时按占位拒。
类型在写范围表（.claude/hooks/agent-write-scope.tsv，读法 lib_write_scope.py 的 load_scopes，这里不另抄）里有行的，骨架在类型专属的几行后面、「## 关的是哪几条」前面打一行「写范围：<表里这个类型的路径模式，用「、」隔开>」
（clerk 列 records/**、research/prompts/**、research/results/** 与 $SINGLEFS_WORK_ROOT/**，不列 .claude/kb/）：主 agent 填「要改的文件」「报告」时对着它写，派发闸 ④ 就不会拒「提示要子 agent 写进写范围之外的路径」。
表里没有行的类型、没有这张表的仓（自检的样本仓）不打这一行；--set 写范围=… 可以改写这一行。
带「草稿目录：」一行的类型，骨架在它后面多一行「认领：bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose '<用途一句>'」
（欠账 C792：草稿目录要有归属文件；--set 草稿目录=… 与 --set 认领用途=… 填进去）。--check 提示里有「草稿目录：」一行时再判这一行：
在不在、claim 后面的目录与「草稿目录：」是不是同一个（~ 展开、末尾斜杠不计）、用途是不是空的；占位没填的归占位那一判，不在这里重复报。
派发闸不判「线程上限」「预计挂钟」「分片」「续跑」「多机」「复用」这几行（records/2026-10-30-编译绕过内存总上限.md 批 3b），骨架一种类型都不打；
自检核每种类型的骨架里都没有这几行，带这几行的提示 --check 照样退 0。
「报告：」一行默认填 <草稿目录>/result.md（--set 报告=… 填别的）：Claude Code 自带的拦截不让子 agent 用 Write 建名字为 report.md 的文件，写 result.md 写得进去；
派发闸 ⑰ 判报告行（「报告」「报告路径」「报告写进」「报告写到」四种标签，与 ④ 写范围共用一份解析）点名的文件名里带 report（不分大小写）的、以 SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 .md 拒，
--check 经 --check-prompt 带出这一条。
三方腿（类型表只登记在定义 frontmatter 的 leg: three-way 一行，按 <仓根>/.claude/agents 经 .claude/hooks/lib_three_way_legs.py 读，three_way_report_types；
派发闸、交回闸与账表读同一处）的「报告：」一行填占位 research/prompts/<填 轮>-<填 模型>-output.md，
轮与模型两处主 agent 填（不填 --check 照占位拒）；定义的 required-inputs 没要「报告」的也打这一行，要了的只打一行。派发闸 ⑰ 对这几种另判形态（判法只在派发闸文件头 ⑰ 条）：
本腿的报告只认报告行点名的 research/prompts/<名>-output.md，或报告行里没有时整份提示里不在引号里、不在禁读清单里、名字以本腿轮名起头的那一处；
一处都没有、报告行点名的文件落在草稿目录下或是裸文件名又不是这一形态（例如草稿目录下的 result.md）、禁读清单点到本腿自己的报告、本腿报告路径的名字里带 report 都拒，
交回闸只认这种形态。骨架照填：「报告：」一行写本腿的，「禁读清单：」一行只写别的腿的报告与模型目录。
「草稿目录：」一行怎么认（认领行那一判用它）只在 .claude/hooks/lib_prompt_scratch_line.py 的 scratch_line_values（与派发闸 ⑯ ⑰、完工清理闸、交回闸 ⑥、看门狗共用；
拆分提案第九十节 ㉚），这里不另写；读不到那份库时认领行那一判不做，输出写一句。
退出码：0 打出骨架，或 --check 判下来派发闸放行（不知道类型时：没有占位）；2 用法错、定义不在、提示文件读不了；3 --check 判下来派发闸会拒（或有占位）。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句的开头，括号里是那条「✗ 自检：」的开头）；
逐格列全的几条（draft-line-own-regex、retired-lines-printed）每一项是打开开关后一格失败行「✗ 自检：」起到「实际」为止的原文
（没有「实际」的到第一处不在「」里的全角冒号为止），项之间用「；」隔开，与失败的格逐格相同：
  DISPATCH_PROMPT_BREAK=ignore-placeholders  placeholders() 一处占位都不认（不知道类型、只查占位的那一路）
      --check 带占位、不知道类型的骨架（「✗ 自检：--check 对带占位、不知道类型的骨架应当退 3 并逐行列出」）
  DISPATCH_PROMPT_BREAK=guard-skipped  check_prompt() 知道类型也不起派发闸的 --check-prompt，只查占位
      --check 带类型一次列全（「✗ 自检：--check 带类型应当一次列全派发闸会拒的两处」）
      「✗ 自检：三方腿骨架报告填成草稿目录下的 result.md（three-way-forward），--check 应当退 3并点名「research/prompts/<轮>-<模型>-output.md」，实际」
      「✗ 自检：三方腿骨架报告填成草稿目录下的 result.md（three-way-attack），--check 应当退 3并点名「research/prompts/<轮>-<模型>-output.md」，实际」
      「✗ 自检：三方腿骨架报告填成草稿目录下的 result.md（three-way-defense），--check 应当退 3并点名「research/prompts/<轮>-<模型>-output.md」，实际」
      「✗ 自检：--check 对「报告：」写 report.md 的提示应当退 3 并点名文件名、给 result.md 的写法，实际」
      「✗ 自检：工具员骨架批次一行点名不在的调度记录，--check 应当退 3 并点名那份记录，实际」
  DISPATCH_PROMPT_BREAK=claim-line-dropped  skeleton() 不在「草稿目录：」后面打「认领：」一行
      带草稿目录的类型骨架在草稿目录那一行后面紧跟认领行（「✗ 自检：带草稿目录的类型骨架应当在草稿目录那一行后紧跟一行认领」）、
      --set 认领用途=… 填得进（「✗ 自检：--set 草稿目录、认领用途」）
      「✗ 自检：clerk 的骨架填完，--check 应当退 0，实际」
      「✗ 认领行：有「草稿目录：…」却没有「认领：…」一行」
      「✗ 自检：experiment-designer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：experiment-runner 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：implementation-writer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：investigator 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：mutation-triage 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：three-way-attack 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：three-way-defense 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：three-way-forward 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：tooling-writer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（three-way-forward），--check 应当退 0，实际」
      「✗ 自检：三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（three-way-attack），--check 应当退 0，实际」
      「✗ 自检：三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（three-way-defense），--check 应当退 0，实际」
      「✗ 自检：--check 对填完的提示应当退 0（骨架第一行认出类型、派发闸放行），实际」
      「✗ 自检：工具员骨架填完、另加撤掉的线程上限与长活几行，--check 应当退 0（派发闸不再判这几行），实际」
      「✗ 自检：写范围一行：clerk 的骨架带着「写范围：」一行填完，--check 应当退 0（派发闸不把这一行当写进范围外的路径），实际」
      「✗ 自检：骨架里要临时服务的自证那一行点名了登记的脚本、拼了整条包装写法，填完过派发闸 --check 应当退 0，实际」
  DISPATCH_PROMPT_BREAK=claim-check-skipped  --check 不判认领行
      认领行缺、目录对不上、用途为空各判红（「✗ 自检：认领行」）、--check 端到端缺认领行退 3（「✗ 自检：--check 对缺认领行的提示」）
  DISPATCH_PROMPT_BREAK=selftest-location-dropped  skeleton() 不打「自证在哪跑」一行
      会跑自证的类型骨架带这一行（「✗ 自检：会跑自证的类型骨架应当带一行「自证在哪跑：<填 …>」」）、
      --set 填得进这一行（「✗ 自检：--set 自证在哪跑=…」）、--check 没填时点名这一行（「✗ 自检：--check 对没填自证在哪跑的提示」）
  DISPATCH_PROMPT_BREAK=selftest-location-handback-hint-dropped  SELFTEST_LOCATION_LINE 的占位里不写这些命令由主 agent 起、子 agent 写进报告自己不起（SELFTEST_LOCATION_HANDBACK_HINT）
      「✗ 自检：会跑自证的类型骨架的「自证在哪跑」占位应当写明命令由主 agent 起、子 agent 自己不起（tooling-writer）」；「✗ 自检：会跑自证的类型骨架的「自证在哪跑」占位应当写明命令由主 agent 起、子 agent 自己不起（implementation-writer）」
      「✗ 自检：会跑自证的类型骨架的「自证在哪跑」占位应当写明命令由主 agent 起、子 agent 自己不起（investigator）」；「✗ 自检：会跑自证的类型骨架的「自证在哪跑」占位应当写明命令由主 agent 起、子 agent 自己不起（experiment-runner）」
  DISPATCH_PROMPT_BREAK=selftest-location-hint-bare  SELFTEST_LOCATION_HANDBACK_HINT 不分表内表外，一律写 `task-server/task submit -- <脚本> --selftest` 这一形（SELFTEST_LOCATION_REQUIRED_CLAUSE 交空；表里 required 的脚本裸写会被 project-code-guard.sh 拒）
      「✗ 自检：会跑自证的类型骨架的「自证在哪跑」占位应当点到「要临时服务的自证」那一行（表里 required 的脚本裸写会被拒）（tooling-writer）」；「✗ 自检：会跑自证的类型骨架的「自证在哪跑」占位应当点到「要临时服务的自证」那一行（表里 required 的脚本裸写会被拒）（implementation-writer）」
      「✗ 自检：会跑自证的类型骨架的「自证在哪跑」占位应当点到「要临时服务的自证」那一行（表里 required 的脚本裸写会被拒）（investigator）」；「✗ 自检：会跑自证的类型骨架的「自证在哪跑」占位应当点到「要临时服务的自证」那一行（表里 required 的脚本裸写会被拒）（experiment-runner）」
  DISPATCH_PROMPT_BREAK=temporary-service-line-dropped  skeleton() 不打「要临时服务的自证」一行
      「✗ 自检：会跑自证的类型骨架应当带一行「要临时服务的自证：…」（tooling-writer）」；「✗ 自检：会跑自证的类型骨架应当带一行「要临时服务的自证：…」（implementation-writer）」
      「✗ 自检：会跑自证的类型骨架应当带一行「要临时服务的自证：…」（investigator）」；「✗ 自检：会跑自证的类型骨架应当带一行「要临时服务的自证：…」（experiment-runner）」
      「✗ 自检：骨架点名了要临时服务的自证，那一行应当拼出整条包装写法」；「✗ 自检：--set 要临时服务的自证=… 应当把这一行换成主 agent 写的」
  DISPATCH_PROMPT_BREAK=temporary-service-named-dropped  这一行不拼点名了的脚本的整条命令（只写规则）
      「✗ 自检：骨架点名了要临时服务的自证，那一行应当拼出整条包装写法」
  DISPATCH_PROMPT_BREAK=temporary-service-command-bare  拼出来的整条命令是裸写的样子（没有包装）
      「✗ 自检：骨架点名了要临时服务的自证，那一行应当拼出整条包装写法」
  DISPATCH_PROMPT_BREAK=temporary-service-table-unread  读不到要临时服务的自证登记表（名单当空）
      「✗ 自检：要临时服务的自证登记表应当读得到 scripts/mutation/crates-mutation-rows.py（任务 2731 退 2 的那一份）」、「✗ 自检：骨架点名了要临时服务的自证，那一行应当拼出整条包装写法」
  DISPATCH_PROMPT_BREAK=temporary-service-line-unquoted  「要临时服务的自证」那一行的规则一句退回会被两道闸拒的写法：借别处做法（「照登记表那一份的写法」）却不带原文，
      整条命令放在反引号里、句子里不写写进报告与由主 agent 交（点名了的脚本那一截照旧）
      「✗ 自检：骨架照抄进提示的每一行都要过借用写法闸与派发闸（tooling-writer）」；「✗ 自检：骨架照抄进提示的每一行都要过借用写法闸与派发闸（implementation-writer）」
      「✗ 自检：骨架照抄进提示的每一行都要过借用写法闸与派发闸（investigator）」；「✗ 自检：骨架照抄进提示的每一行都要过借用写法闸与派发闸（experiment-runner）」
      「✗ 自检：experiment-runner 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：implementation-writer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：investigator 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：tooling-writer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：工具员骨架填完、另加撤掉的线程上限与长活几行，--check 应当退 0（派发闸不再判这几行），实际」
      点名了登记脚本、拼了整条包装写法的那一格（骨架里要临时服务的自证那一行填完过派发闸 --check）不红：点名时规则一句后面接着带
      写进报告、由主 agent 交的整条命令，整句因提到主 agent、有交回说法被派发闸 ㉒ 放过，又因带了直角引号括起的原文过借用写法闸，规则一句写错这一格看不出来；规则一句由上面不点名的几格盯

  DISPATCH_PROMPT_BREAK=scratch-expanded-from-home  selftest_work_root()：自检的草稿目录与报告路径写成 ~/ 起头、照家目录展开的样子（~/ 加工作根相对本机家目录的路径）
      （本机家目录下展开回工作根；自证把 HOME 换成别的目录再判同一份提示，展开就落到工作根外面，--check 的结局跟着变，第二台的家目录是另一个，平常的趟里也一样靠工作根取）
      「✗ 自检：clerk 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 自检：experiment-designer 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 自检：experiment-runner 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 自检：implementation-writer 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 自检：investigator 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 自检：mutation-triage 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 自检：three-way-attack 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 自检：three-way-defense 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 自检：three-way-forward 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 自检：tooling-writer 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际」
      「✗ 派 … 的提示报告行点名的 ~/」
      本机家目录下展开回工作根，哪一台上跑红的都是这十格；十格的失败信息嵌着派发闸打的 ✗ 行，其中报告行点名的 ~/ 那一行在六种类型（clerk、experiment-runner、implementation-writer、investigator、mutation-triage、tooling-writer）里单独成行，上面最后一项点名它；别的格不红

  DISPATCH_PROMPT_BREAK=report-default-dropped  skeleton() 的「报告：」一行退回 <填 报告|报告路径> 占位，不填 <草稿目录>/result.md
      骨架的「报告：」一行默认填草稿目录下的 result.md（「✗ 自检：骨架的「报告：」一行默认应当填 <草稿目录>/result.md」；有定义的类型骨架那一格「✗ 自检：有定义的类型：骨架缺 …」同样红）
  DISPATCH_PROMPT_BREAK=three-way-report-default-dropped  三方腿的「报告：」一行只按定义写：定义要了「报告」的填 <草稿目录>/result.md，没要的不打这一行
      三方腿的骨架只打一行占位（「✗ 自检：三方腿的骨架应当只打一行「报告：research/prompts/<填 轮>-<填 模型>-output.md」」，三种各一句）、
      只填别的占位时点名报告行（「✗ 自检：三方腿骨架只填别的占位、不动报告行」，三种各一句）、
      骨架读到的三方腿类型与 frontmatter 逐个相同（「✗ 自检：三方腿类型表：样本仓骨架按三方腿打报告占位的类型」「✗ 自检：三方腿类型表：本仓骨架按三方腿打报告占位的类型」）
      「✗ 自检：three-way-attack 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：three-way-defense 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：three-way-forward 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（three-way-forward），--check 应当退 0，实际」
      「✗ 自检：三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（three-way-attack），--check 应当退 0，实际」
      「✗ 自检：三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（three-way-defense），--check 应当退 0，实际」
      「✗ 派 three-way-attack 的提示里没有行首「报告：」一行（句中写的「报告写进 …」不算这一行）。」
  DISPATCH_PROMPT_BREAK=leg-types-hardcoded  three_way_report_types() 不读定义 frontmatter 的 leg: three-way，改用写死的 three-way-forward、three-way-attack、three-way-defense
      骨架读到的三方腿类型与 frontmatter 逐个相同（「✗ 自检：三方腿类型表：样本仓骨架按三方腿打报告占位的类型应当与 frontmatter 逐个相同」）
  DISPATCH_PROMPT_BREAK=draft-line-own-regex  claim_problems() 不调 lib_prompt_scratch_line.py，改用这里自己的 DRAFT_LINE（第 0 列、全角冒号，围栏代码块里的也认）
      「✗ 自检：认领行 绿：闭合了的围栏代码块里的草稿目录一行不算，不要求认领：应当 0 个问题，实际」；「✗ 自检：认领行 红：半角冒号的草稿目录一行同样要认领：应当 1 个问题，实际」；「✗ 自检：认领行调共用库：「草稿目录：」一行应当经 lib_prompt_scratch_line.py 的 scratch_line_values 认，实际」
  DISPATCH_PROMPT_BREAK=untouched-line-dropped  skeleton() 不给实现员、工具员打「不碰：<填 …>」一行（常设要求 D3）
      「✗ 自检：实现员与工具员的骨架应当带一行「不碰：<填 …>」（implementation-writer）」；「✗ 自检：实现员与工具员的骨架应当带一行「不碰：<填 …>」（tooling-writer）」
      「✗ 自检：implementation-writer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：tooling-writer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：工具员骨架填完、另加撤掉的线程上限与长活几行，--check 应当退 0（派发闸不再判这几行），实际」
      「✗ 自检：骨架里要临时服务的自证那一行点名了登记的脚本、拼了整条包装写法，填完过派发闸 --check 应当退 0，实际」
  DISPATCH_PROMPT_BREAK=scope-line-omitted  skeleton() 不打「写范围：…」一行（拆分提案第一百六十一节骨架那一半：按类型从写范围表取模式，主 agent 填要改的文件与报告时对着它写）
      「✗ 自检：写范围一行：clerk 的骨架应当有一行「写范围：」列 records/**、research/prompts/**、research/results/** 与 $SINGLEFS_WORK_ROOT/**、不列 .claude/kb/，实际」
      「✗ 自检：写范围一行：kb-scribe 的骨架应当有一行「写范围：」列 .claude/kb/**、不列 records/，实际」
  DISPATCH_PROMPT_BREAK=handover-line-dropped  skeleton() 不给执行员打「承接：<填 …>」一行（常设要求 B8）
      「✗ 自检：执行员骨架应当在「上一段岔路表里还差」后面紧跟一行「承接：<填 …>」」
  DISPATCH_PROMPT_BREAK=addressed-section-dropped  skeleton() 不打「## 关的是哪几条」一节（常设要求 A5）
      「✗ 自检：骨架应当带「## 关的是哪几条」一节（缺的类型：…）」
      「✗ 自检：clerk 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：experiment-designer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：experiment-runner 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：implementation-writer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：investigator 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：mutation-triage 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：three-way-attack 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：three-way-defense 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：three-way-forward 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：tooling-writer 的骨架填完，--check 应当退 0，实际」
      「✗ 自检：三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（three-way-forward），--check 应当退 0，实际」
      「✗ 自检：三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（three-way-attack），--check 应当退 0，实际」
      「✗ 自检：三方腿骨架报告填成 research/prompts/x-r1-opus-output.md（three-way-defense），--check 应当退 0，实际」
      「✗ 自检：--check 对填完的提示应当退 0（骨架第一行认出类型、派发闸放行），实际」
      「✗ 自检：工具员骨架填完、另加撤掉的线程上限与长活几行，--check 应当退 0（派发闸不再判这几行），实际」
      「✗ 自检：写范围一行：clerk 的骨架带着「写范围：」一行填完，--check 应当退 0（派发闸不把这一行当写进范围外的路径），实际」
      「✗ 自检：骨架里要临时服务的自证那一行点名了登记的脚本、拼了整条包装写法，填完过派发闸 --check 应当退 0，实际」
  DISPATCH_PROMPT_BREAK=retired-lines-printed  skeleton() 退回撤掉之前的样子：每种类型都在批次一行后面打线程上限、预计挂钟、分片、续跑、多机、复用几行占位
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（clerk）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（experiment-designer）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（experiment-runner）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（implementation-writer）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（investigator）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（mutation-triage）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（three-way-attack）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（three-way-defense）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（three-way-forward）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（tooling-writer）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（kb-scribe）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（sweep）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（Explore）」
      「✗ 自检：骨架不该带撤掉的线程上限与长活几行（sample-writer）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（clerk）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（experiment-designer）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（experiment-runner）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（implementation-writer）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（investigator）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（mutation-triage）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（three-way-attack）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（three-way-defense）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（three-way-forward）」
      「✗ 自检：真定义打出的骨架不该带撤掉的线程上限与长活几行（tooling-writer）」
派发闸那一边的开关（经 --check 起的派发闸照样带着）打开后这里红的格：
  RUNNER_DISPATCH_GUARD_BREAK=three-way-report-unchecked  派发闸不判三方腿的报告形态
      填成草稿目录下的 result.md 退 3（「✗ 自检：三方腿骨架报告填成草稿目录下的 result.md」，三种各一句）
  RUNNER_DISPATCH_GUARD_BREAK=placeholders-ignored  派发闸不认 <填 …> 占位
      只填别的占位退 3（「✗ 自检：三方腿骨架只填别的占位、不动报告行」，三种各一句）
      「✗ 自检：--check 带类型应当一次列全派发闸会拒的两处（['有 2 处不成立', '占位没填', '缺它定义要的输入：禁读清单', '禁读清单：<填 禁读清单>']），实际」
      「✗ 派 sample-attack 的提示缺它定义要的输入：禁读清单（.claude/agents/sample-attack.md 的 required-inputs）。」
      「✗ 自检：--check 对没填自证在哪跑的提示应当退 3 并点名那一行，实际」
  RUNNER_DISPATCH_GUARD_BREAK=retired-lines-refused  派发闸退回撤掉之前的一部分样子：提示里有线程上限、预计挂钟、多机这几种标签行就拒
      「✗ 自检：工具员骨架填完、另加撤掉的线程上限与长活几行，--check 应当退 0（派发闸不再判这几行），实际」

会跑自证的类型（SELFTEST_RUNNER_TYPES，按各定义「输入」一节判）的骨架另带一行「自证在哪跑：<填 …>」：
自证登记 .claude/hooks/selftest-registry.tsv 按本仓路径认，副本、草稿里的拷贝跑 --selftest 被 project-code-guard.sh 拒，派的时候就写明在哪跑。
占位里另写一句这些命令由主 agent 经 task-server 交、子 agent 写进报告自己不起（SELFTEST_LOCATION_HANDBACK_HINT）：主 agent 照它填，不写内存包装与绕开包装的起法；
表外的脚本写 `task-server/task submit -- <脚本> --selftest` 这一形，表里 required 的脚本裸写会被 project-code-guard.sh 拒、整条写在「要临时服务的自证」那一行（SELFTEST_LOCATION_REQUIRED_CLAUSE 点到它）。
同样这几种类型的骨架再带一行「要临时服务的自证：…」（TEMPORARY_SERVICE_LABEL）：.claude/hooks/selftest-temporary-service.tsv 里判定是 required 的自证只在临时 prune-server 底下跑，
裸写 --selftest 退 2，执行前也被 project-code-guard.sh 拒；这一行写明整条写法（task-server/task submit -- 加包装 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- 再加解释器、脚本与 --selftest，包装不带 --server-binary），
并把 --set 的值里点名了的 required 脚本各拼成整条（temporary_service_line）。这一行不是占位，原样留在提示里。
骨架里的每一行都要能原样照抄进派发提示：整条命令放在「」里当原文（借用写法闸 .claude/singlefs-ai-sop/scripts/claude-hooks/borrowed-convention-guard.sh 认「」里的原文），
同一句写明「写进报告「要主 agent 跑的命令」一节，由主 agent 交」（派发闸 ㉒ 认交回的说法，「」里的命令不当许可子 agent 交任务）；
自检把会跑自证的几种类型的骨架逐行喂借用写法闸、整份填完喂派发闸，被拒就红。
"""
import glob
import json
import os
import re
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "hooks"))
try:   # 三方腿类型表只登记在定义 frontmatter 的 leg: three-way 一行，读法在 .claude/hooks/lib_three_way_legs.py（与派发闸、交回闸、账表共用；读不到时骨架不打三方腿的报告占位，stderr 写一句）
    import lib_three_way_legs as three_way_legs  # noqa: E402
    three_way_legs_error = None
except Exception as error:   # 文件不在、语法错都算读不到
    three_way_legs, three_way_legs_error = None, error
try:   # 写范围表的读法只在 .claude/hooks/lib_write_scope.py（write-guard.sh、bash-command-detector.sh、派发闸共用；读不到时骨架不打「写范围：」一行，stderr 写一句）
    import lib_write_scope as write_scope  # noqa: E402
    write_scope_error = None
except Exception as error:   # 文件不在、语法错都算读不到
    write_scope, write_scope_error = None, error
try:   # 工作根的定义只在 .claude/hooks/lib_work_root.py：自检的草稿目录与报告路径从这里取，不经 ~ 与 $HOME 展开（第二台的家目录是另一个，展开就落到工作根外面）
    import lib_work_root as work_root_library  # noqa: E402
    work_root_library_error = None
except Exception as error:   # 文件不在、语法错都算读不到
    work_root_library, work_root_library_error = None, error
BROKEN = os.environ.get("DISPATCH_PROMPT_BREAK", "")
PLACEHOLDER = re.compile(r"<填[^>]*>")
REPORT_LINE_NAME, REPORT_FILE_NAME = "报告", "result.md"   # 骨架的「报告：」一行默认填 <草稿目录>/result.md（派发闸 ⑰ 拒文件名里带 report 的）
# 三方腿的报告就是 research/prompts/<轮>-<模型>-output.md（交回闸只认这种形态；派发闸 ⑰ 读同一处类型表）：骨架填这个占位，轮与模型两处主 agent 填
THREE_WAY_REPORT_TYPES_BEFORE_REGISTRY = ("three-way-forward", "three-way-attack", "three-way-defense")   # 写死的那一份：只有弄坏开关 leg-types-hardcoded 下还在用


def three_way_report_types(agents_dir):
    """骨架打三方腿报告占位的类型：agents_dir（<仓根>/.claude/agents）里定义 frontmatter 写了 leg: three-way 的（lib_three_way_legs.py 的 three_way_leg_types）。
    读不到那份库交空集、stderr 写一句；弄坏开关 leg-types-hardcoded 下用写死的三种。"""
    if BROKEN == "leg-types-hardcoded":
        return frozenset(THREE_WAY_REPORT_TYPES_BEFORE_REGISTRY)
    if three_way_legs is None:
        print(f"  ! 读不到 .claude/hooks/lib_three_way_legs.py（{three_way_legs_error!r}），三方腿的「报告：」一行这一次没按三方腿打", file=sys.stderr)
        return frozenset()
    return three_way_legs.three_way_leg_types(os.path.dirname(os.path.dirname(os.path.normpath(agents_dir))))


THREE_WAY_REPORT_PLACEHOLDER = "research/prompts/<填 轮>-<填 模型>-output.md"
# --check 起的派发闸（判法只在它那里）与骨架第一行写的类型
DISPATCH_GUARD = os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "runner-dispatch-guard.sh")
DISPATCH_HEADER = re.compile(r"\A# 派发：(\S+)")
GUARD_TIMEOUT_SECONDS = 120
# 定义之外、派发闸另判的行：类型 → 那一行的骨架
# 派发闸不判的几种标签行（records/2026-10-30-编译绕过内存总上限.md 批 3b）：骨架不打，自检核每种类型的骨架里都没有
RETIRED_SKELETON_LABELS = ("线程上限：", "线程上限已判：", "预计挂钟：", "分片：", "续跑：", "多机：", "多机已判：", "分不了：", "复用：")
RETIRED_LINES_BEFORE = ["线程上限：<填 登记的每台默认重活线程数>", "预计挂钟：<填 2 小时内 / 超过 2 小时>", "分片：<填 超过 2 小时才写>", "续跑：<填 超过 2 小时才写>",
                        "多机：<填 超过 2 小时才写>", "复用：<填 超过 2 小时才写>"]   # 只有弄坏开关 retired-lines-printed 下还在用：撤掉之前骨架打的那几行
# 常设要求 D3（实现员、工具员：这一轮别的会话在改、这一件不碰的文件）与 B8（执行员续做：上一段的报告），派发闸 ⑱ ⑪ 核这两行（拆分提案第一百二十二节 ⑨）
UNTOUCHED_LINE = "不碰：<填 这一轮别的会话在改、这一件不碰的文件，逐个列；没有写「无」>"
HANDOVER_LINE = "承接：<填 续做时写上一段执行员的报告路径（派发闸核它在）；第一段删掉这一行>"
ADDRESSED_SECTION_TITLE = "## 关的是哪几条"   # 常设要求 A5：派发闸 ⑱ 核这一节在、节下不空
EXTRA_LINES = {
    "implementation-writer": ["要动的 crates 文件：<填 从仓根起逐个列全，不与在跑的实现员撞>"] + ([] if BROKEN == "untouched-line-dropped" else [UNTOUCHED_LINE]),
    "tooling-writer": [] if BROKEN == "untouched-line-dropped" else [UNTOUCHED_LINE],
    "kb-scribe": ["规格检查已判：写回员自己起草并跑 kb-spec-check.py"],
    "experiment-runner": ["这一段回答的岔路：<填 岔路单里的哪几行>", "上一段岔路表里还差：<填 续做时写还差的行；第一段删掉这一行>",
                          *([] if BROKEN == "handover-line-dropped" else [HANDOVER_LINE])],
}
# 会跑自证的类型要的一行（派发提示写明自证在哪跑）：按各定义「输入」一节判——
# 工具员要给判红样本与弄坏开关，实现员与执行员会改脚本、装置或变异表锚点，调查员在仓副本里改坏、二分
# 占位里另提一句这些命令谁起、怎么起：一切跑项目代码的命令只认经 task-server/task 起（records/2026-10-30-编译绕过内存总上限.md 批 3b），
# 子 agent 一律不往统一任务服务交任务（records/2026-10-29-subagent拆分提案.md 第一百七十四节），所以这一行写的是主 agent 起交回命令的那棵树；
# 内存与排队归服务，提示里不写包装（不写「排不上带 ulimit -v 与 nice 直跑」这类绕开服务的做法）
# 表里 required 的自证裸写 --selftest 在执行前被 project-code-guard.sh 拒（一百九十七节 ②）：占位里表外的写这一形，required 的指到下面「要临时服务的自证」那一行的整条；
# 弄坏开关 selftest-location-hint-bare 下不分表内表外，一律写这一形
TEMPORARY_SERVICE_LABEL = "要临时服务的自证"
SELFTEST_LOCATION_REQUIRED_CLAUSE = ("" if BROKEN == "selftest-location-hint-bare" else
                                     f"（表外的脚本写这一形，.claude/hooks/selftest-temporary-service.tsv 里 required 的脚本裸写会被 project-code-guard.sh 拒，照骨架「{TEMPORARY_SERVICE_LABEL}」那一行的整条写）")
SELFTEST_LOCATION_HANDBACK_HINT = ("子 agent 照它填命令里的路径、自己不起：自证、样本档与弄坏开关拼成 `task-server/task submit -- <脚本> --selftest` 这样的整条" + SELFTEST_LOCATION_REQUIRED_CLAUSE + "，"
                                   "写进报告「要主 agent 跑的命令」一节，由主 agent 在这棵树里交服务；命令里不写内存包装、host-dispatch.py 与绕开包装的起法（ulimit -v、nice 直跑、改小上限）")
SELFTEST_LOCATION_LINE = ("自证在哪跑：<填 子 agent 交回的自证与样本档命令由主 agent 在哪棵树里起：本仓主工作区那一份（自证登记按本仓路径认，副本与草稿里的拷贝被 project-code-guard.sh 拒）；"
                          + ("" if BROKEN == "selftest-location-handback-hint-dropped" else SELFTEST_LOCATION_HANDBACK_HINT + "；")
                          + "这一轮没有自证要交的写「不跑」>")
SELFTEST_RUNNER_TYPES = ("tooling-writer", "implementation-writer", "investigator", "experiment-runner")
for selftest_runner_type in SELFTEST_RUNNER_TYPES:
    EXTRA_LINES.setdefault(selftest_runner_type, []).append(SELFTEST_LOCATION_LINE)

# 要临时 prune-server 的自证（登记在 .claude/hooks/selftest-temporary-service.tsv，判定写 required 的：--selftest 开跑先核临时服务，裸跑退 2，执行前也被 project-code-guard.sh 拒）：
# 骨架给整条写法，--set 的值里点名了的脚本各拼一条。这一行不是占位（不带 <填 …>），原样留在提示里；句子里写明由主 agent 交、子 agent 写进报告（派发闸 ㉒ 认交回的说法）
TEMPORARY_SERVICE_TABLE = os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "selftest-temporary-service.tsv")
TEMPORARY_SERVICE_WRAPPER = "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --"
TEMPORARY_SERVICE_REQUIRED = "required"


def temporary_service_scripts(table=TEMPORARY_SERVICE_TABLE):
    """要临时服务的自证登记表里判定写 required 的脚本（相对仓根的路径，按表里次序）。一行 `<路径> <判定> # <理由>`，空行与 # 起头的行不算；
    读不到交空、stderr 写一句；弄坏开关 temporary-service-table-unread 下交空。"""
    if BROKEN == "temporary-service-table-unread":
        return []
    try:
        with open(table, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        print(f"  ! 读不到 {table}（{error!r}），骨架里这一次没拼要临时服务的自证", file=sys.stderr)
        return []
    rows = [line.split(None, 2) for line in lines if line.strip() and not line.lstrip().startswith("#")]
    return [fields[0] for fields in rows if len(fields) > 1 and fields[1] == TEMPORARY_SERVICE_REQUIRED]


def temporary_service_command(script):
    """一份 required 脚本的整条写法：task-server/task submit -- 包装 解释器 脚本 --selftest（.py 用 python3，别的用 bash）。弄坏开关 temporary-service-command-bare 下没有包装。"""
    bare = f"{'python3' if script.endswith('.py') else 'bash'} {script} --selftest"
    return f"task-server/task submit -- {bare if BROKEN == 'temporary-service-command-bare' else TEMPORARY_SERVICE_WRAPPER + ' ' + bare}"


def temporary_service_line(settings):
    """骨架里「要临时服务的自证」那一行：规则一句，加上 --set 的值里点名了的 required 脚本各拼出来的整条。一句话写完（句中不用句号与分号：派发闸 ㉒ 按句判，
    每一处点到 task-server/task submit 的句子都要有写进报告、由主 agent 交的说法；整条命令放在「」里当原文，借用写法闸与派发闸都不拒）。
    弄坏开关 temporary-service-named-dropped 下只写规则；temporary-service-line-unquoted 下规则一句退回会被两道闸拒的写法。"""
    if BROKEN == "temporary-service-line-unquoted":   # 会被两道闸拒的写法：借别处做法不带原文，命令在反引号里、不写交回
        text = (f"{TEMPORARY_SERVICE_LABEL}：这些脚本的自证照登记表那一份的写法拼成 "
                f"`task-server/task submit -- {TEMPORARY_SERVICE_WRAPPER} <解释器> <脚本> --selftest` 交服务")
    else:
        text = (f"{TEMPORARY_SERVICE_LABEL}：名单在 .claude/hooks/selftest-temporary-service.tsv（判定是 required 的脚本，--selftest 开跑先核临时 prune-server，裸写退 2，执行前也被拒），"
                f"这些脚本的自证整条拼成「task-server/task submit -- {TEMPORARY_SERVICE_WRAPPER} <解释器> <脚本> --selftest」这一形态"
                "（整条不带 --server-binary，交给服务的这一条由服务认成会编的、给编译目录，包装现编一遍 prune-server，自己编了、两台都读得到的二进制才另加 --server-binary，写在整条之外），"
                "写进报告「要主 agent 跑的命令」一节，由主 agent 交，子 agent 自己不起")
    named = [script for script in temporary_service_scripts() if any(script in value for value in settings.values())]
    if named and BROKEN != "temporary-service-named-dropped":
        text += "，这一件点名的要临时服务的自证：" + "、".join(f"「{temporary_service_command(script)}」" for script in named) + "，都写进报告「要主 agent 跑的命令」一节，由主 agent 交"
    return text


def frontmatter_fields(definition_path):
    fields = {}
    try:
        lines = open(definition_path, encoding="utf-8").read().split("\n")
    except OSError:
        return None
    if not lines or lines[0].strip() != "---":
        return None
    for line in lines[1:]:
        if line.strip() == "---":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            fields[key.strip()] = value.split("#", 1)[0].strip()
    return fields


CLAIM_COMMAND = "bash scripts/hosts/work-directory-ownership.sh claim"
DRAFT_LINE = re.compile(r"^草稿目录：\s*(\S+)", re.M)   # 这里自己的认法：只有弄坏开关 draft-line-own-regex 下还在用
try:   # 「草稿目录：」一行的认法只在 .claude/hooks/lib_prompt_scratch_line.py（与派发闸 ⑯ ⑰、完工清理闸、交回闸 ⑥、看门狗共用；读不到时认领行不核，输出写一句）
    import lib_prompt_scratch_line as prompt_scratch_line  # noqa: E402
    prompt_scratch_line_error = None
except Exception as error:   # 文件不在、语法错都算读不到
    prompt_scratch_line, prompt_scratch_line_error = None, error
CLAIM_LINE = re.compile(r"^认领：.*?work-directory-ownership\.sh\s+claim\s+(\S+)\s+--purpose(?:\s+(.*\S))?\s*$", re.M)


def claim_line(settings):
    if "认领" in settings:
        return f"认领：{settings['认领']}"
    directory = settings.get("草稿目录", "<填 草稿目录，与上一行一致>")
    purpose = settings.get("认领用途")
    return f"认领：{CLAIM_COMMAND} {directory} --purpose " + (f"'{purpose}'" if purpose else "'<填 这个目录的用途一句>'")


def normalized_directory(value):
    return os.path.expanduser(value.strip("`'\"")).rstrip("/")


def draft_line_value(text):
    """提示里「草稿目录：」一行点名的路径（照写的样子，没展开）→ (路径或 None, 读不到库的原因或 None)：认法是 lib_prompt_scratch_line.py 的 scratch_line_values
    （行首只许空白、全角半角冒号都认、闭合了的围栏代码块里的不认、<填 …> 占位不认），几行的取第一行。弄坏开关 draft-line-own-regex 下用这里自己的 DRAFT_LINE。"""
    if BROKEN == "draft-line-own-regex":
        draft = DRAFT_LINE.search(text)
        return (draft.group(1) if draft else None), None
    if prompt_scratch_line is None:
        return None, repr(prompt_scratch_line_error)
    try:
        values = prompt_scratch_line.scratch_line_values(text)
    except Exception as error:   # 那份库的弄坏开关写错这类
        return None, f"{type(error).__name__}: {error}"
    return (values[0] if values else None), None


def claim_problems(text):
    """提示里有「草稿目录：」一行时，认领行在不在、目录对不对得上、用途空不空：[问题一句]；占位没填的不报（占位那一判报）。
    那一行怎么认在 draft_line_value；读不到 lib_prompt_scratch_line.py 时不判，stderr 写一句。"""
    if BROKEN == "claim-check-skipped":
        return []
    draft, library_error = draft_line_value(text)
    if library_error:
        print(f"  ! 读不到 .claude/hooks/lib_prompt_scratch_line.py（{library_error}），「草稿目录：」一行认不出，认领行这一次没核", file=sys.stderr)
        return []
    if draft is None:
        return []
    claim = CLAIM_LINE.search(text)
    if claim is None:
        return [f"有「草稿目录：{draft}」却没有「认领：{CLAIM_COMMAND} <草稿目录> --purpose '<用途一句>'」一行"]
    problems = []
    claimed, purpose = claim.group(1), (claim.group(2) or "").strip("`'\" ")
    if "<填" not in claimed and "<填" not in draft and normalized_directory(claimed) != normalized_directory(draft):
        problems.append(f"认领的目录 {claimed} 与「草稿目录：」的 {draft} 不是同一个")
    if not purpose:
        problems.append("认领行的 --purpose 后面是空的")
    return problems


WRITE_SCOPE_LABEL = "写范围"


def write_scope_line(agent_type, agents_dir):
    """骨架里的「写范围：…」一行：类型在写范围表（<仓根>/.claude/hooks/agent-write-scope.tsv，仓根取 agents_dir 往上两层）里的路径模式，原样列出（表的写法与读法在
    lib_write_scope.py 的 load_scopes，不另抄）。主 agent 填「要改的文件」「报告」时对着它写，免得派发闸 ④ 拒「提示要子 agent 写进写范围之外的路径」。
    类型不在表里（没有 Write / Edit 的）、表不在、读不了的不打这一行。弄坏开关 scope-line-omitted 下不打。"""
    if BROKEN == "scope-line-omitted":
        return None
    table_path = os.path.join(os.path.dirname(os.path.dirname(os.path.normpath(agents_dir))), ".claude", "hooks", "agent-write-scope.tsv")
    if not os.path.isfile(table_path):
        return None
    if write_scope is None:
        print(f"  ! 读不到 .claude/hooks/lib_write_scope.py（{write_scope_error!r}），骨架这一次没打「{WRITE_SCOPE_LABEL}：」一行", file=sys.stderr)
        return None
    try:
        patterns = write_scope.load_scopes(table_path).get(agent_type, [])
    except OSError as error:
        print(f"  ! 读不了写范围表 {table_path}（{error}），骨架这一次没打「{WRITE_SCOPE_LABEL}：」一行", file=sys.stderr)
        return None
    return f"{WRITE_SCOPE_LABEL}：" + "、".join(patterns) if patterns else None


def skeleton(agent_type, agents_dir, settings, batch):
    fields = frontmatter_fields(os.path.join(agents_dir, f"{agent_type}.md"))
    three_way = agent_type in three_way_report_types(agents_dir)
    lines = [f"# 派发：{agent_type}", ""]
    if fields is None:
        lines.append("开工先读 `.claude/agent-common.md`")
    lines.append(f"批次：{batch or '<填 调度记录#批号>'}")
    if BROKEN == "retired-lines-printed":   # 退回撤掉之前的样子：每种类型都打那几行占位
        lines += RETIRED_LINES_BEFORE
    required = (fields or {}).get("required-inputs", "")
    emitted = set()
    for group in [piece.strip() for piece in required.split(",") if piece.strip()]:
        name = group.split("|", 1)[0].strip()
        emitted.add(name)
        default = f"<填 {group}>"
        if name == REPORT_LINE_NAME and three_way and BROKEN != "three-way-report-default-dropped":
            default = THREE_WAY_REPORT_PLACEHOLDER
        elif name == REPORT_LINE_NAME and BROKEN != "report-default-dropped":
            draft = settings.get("草稿目录")
            default = f"{draft.rstrip('/')}/{REPORT_FILE_NAME}" if draft else f"<填 草稿目录，与上一行一致>/{REPORT_FILE_NAME}"
        lines.append(f"{name}：{settings.get(name, default)}")
        if name == "草稿目录" and BROKEN != "claim-line-dropped":
            lines.append(claim_line(settings))
    # 三方腿：定义的 required-inputs 没要「报告」时骨架也打这一行（排在类型专属的几行前面）；要了的，上面按定义打过就不重复
    three_way_lines = [f"{REPORT_LINE_NAME}：{THREE_WAY_REPORT_PLACEHOLDER}"] if three_way and BROKEN != "three-way-report-default-dropped" else []
    for extra in three_way_lines + EXTRA_LINES.get(agent_type, []):
        if BROKEN == "selftest-location-dropped" and extra == SELFTEST_LOCATION_LINE:
            continue
        key = extra.split("：", 1)[0]
        if key in emitted:
            continue   # 定义的 required-inputs 已经要了这一行，不重复
        lines.append(f"{key}：{settings[key]}" if key in settings else extra)
    if agent_type in SELFTEST_RUNNER_TYPES and BROKEN != "temporary-service-line-dropped":
        lines.append(f"{TEMPORARY_SERVICE_LABEL}：{settings[TEMPORARY_SERVICE_LABEL]}" if TEMPORARY_SERVICE_LABEL in settings else temporary_service_line(settings))   # --set 要临时服务的自证=… 换成主 agent 写的
    scope_line = f"{WRITE_SCOPE_LABEL}：{settings[WRITE_SCOPE_LABEL]}" if WRITE_SCOPE_LABEL in settings else write_scope_line(agent_type, agents_dir)
    if scope_line:
        lines.append(scope_line)
    if BROKEN != "addressed-section-dropped":
        lines += ["", ADDRESSED_SECTION_TITLE, "", settings.get("关的是哪几条", "<填 逐条列已经抓到的问题，每条各自的验收标准>")]
    lines += ["", "## 出口", "", settings.get("出口", "<填 做到什么算完、交什么到哪>"), ""]
    return "\n".join(lines) + "\n"


def placeholders(text):
    if BROKEN == "ignore-placeholders":
        return []
    return [(number, line) for number, line in enumerate(text.split("\n"), 1) if PLACEHOLDER.search(line)]


def guard_check(path, agent_type, root, model):
    """起派发闸的 --check-prompt：→ (退出码 0 / 3, None) 判成了（输出已经原样打出）；(None, 没判成的原因) 起不来或输出认不出。"""
    command = ["bash", DISPATCH_GUARD, "--check-prompt", path, "--type", agent_type, "--root", root or REPOSITORY_ROOT]
    if model:
        command += ["--model", model]
    try:
        completed = subprocess.run(command, capture_output=True, text=True, timeout=GUARD_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired) as error:
        return None, repr(error)
    if completed.returncode not in (0, 2) or not completed.stdout.strip() or "用法：runner-dispatch-guard.sh" in completed.stdout:
        return None, f"退出码 {completed.returncode}：{(completed.stdout + completed.stderr).strip()[-300:]}"
    sys.stdout.write(completed.stdout)
    if completed.stderr.strip():
        sys.stdout.write(completed.stderr)
    return (3 if completed.returncode == 2 else 0), None


def print_claim_problems(text):
    problems = claim_problems(text)
    for problem in problems:
        print(f"  ✗ 认领行：{problem}")   # gate-lint:detail
    if problems:
        print(f"  → 怎么办：「草稿目录：」那一行之后写一行「认领：{CLAIM_COMMAND} <草稿目录> --purpose '<用途一句>'」，目录与上一行一致（骨架本来就带这一行）")
    return len(problems)


def check_prompt(path, agent_type=None, root=None, model=None):
    try:
        text = open(path, encoding="utf-8").read()
    except OSError as error:
        print(f"  ✗ 读不了提示文件 {path}：{error}\n  → 怎么办：给一个存在的提示文件路径")
        return 2
    header = DISPATCH_HEADER.match(text)
    agent_type = agent_type or (header.group(1) if header else None)
    if agent_type and BROKEN != "guard-skipped":
        code, failure = guard_check(path, agent_type, root, model)
        if code is not None:
            return 3 if print_claim_problems(text) else code
        print(f"  ! 派发闸的 --check-prompt 没跑成（{failure}），这一次只查了占位，别几条派的那一刻才判")
    elif not agent_type:
        print("  ! 没给类型（--type <类型>，或骨架第一行「# 派发：<类型>」），派发闸的别几条没核，这一次只查了占位")
    found = placeholders(text)
    for number, line in found:
        print(f"  ✗ 第 {number} 行还是占位：{line.strip()}")   # gate-lint:detail
    if found:
        print(f"  → 怎么办：把这 {len(found)} 处 <填 …> 换成实际内容再派；确实不适用的整行删掉")
    claim_found = print_claim_problems(text)
    if found or claim_found:
        return 3
    print(f"  ✓ 提示里没有占位（查了 {len(text.split(chr(10)))} 行）")
    return 0


SELFTEST_BATCH_RECORD = os.path.join("records", "dispatch-prompt-selftest.md")   # 自检填进批次一行的调度记录：--check 的样本仓里都建这一份（派发闸 ⑱ 核它在不在）
SELFTEST_SCRATCH_NAME = "dispatch-prompt-selftest"   # 自检填进草稿目录一行的目录名，落在工作根下（派发闸 ⑱ 要它在工作根下；--check 不建锁，目录不用在）


def selftest_work_root():
    """自检造草稿目录、报告路径用的工作根：lib_work_root.py 的 configured_work_root（绝对路径，两台同一个，不经 ~ 与 $HOME）；读不到那份库、取不到工作根抛 RuntimeError 或 ValueError。
    弄坏开关 scratch-expanded-from-home 下经家目录展开：~/ 加工作根相对本机家目录的路径（本机家目录下展开回工作根，HOME 换成别的目录就落到工作根外面；
    按各台家目录算、不写死 ~/.local/share/singlefs/work，让这个开关在哪台跑红的格都一样）。"""
    if work_root_library is None:
        raise RuntimeError(f"读不到 .claude/hooks/lib_work_root.py（{work_root_library_error!r}）")
    work_root = work_root_library.configured_work_root()
    if BROKEN == "scratch-expanded-from-home":
        return "~/" + os.path.relpath(work_root, os.path.expanduser("~"))
    return work_root


def selftest_scratch():
    """自检填进草稿目录一行的目录（工作根下的 SELFTEST_SCRATCH_NAME）。"""
    return os.path.join(selftest_work_root(), SELFTEST_SCRATCH_NAME)


def with_home_directory(directory, function):
    """HOME 换成 directory 之后调 function()，调完还原（不管抛不抛）；自证拿它判「--check 的结局不随家目录变」。"""
    saved = os.environ.get("HOME")
    os.environ["HOME"] = directory
    try:
        return function()
    finally:
        if saved is None:
            os.environ.pop("HOME", None)
        else:
            os.environ["HOME"] = saved


SCRATCH_PLACEHOLDER = "<填 草稿目录，与上一行一致>"   # 认领行与报告行里跟着草稿目录走的那一处占位（claim_line、skeleton 打的）


def write_batch_record(root):
    """自检的样本仓里建批次一行点名的那份调度记录（已在就不动）。"""
    path = os.path.join(root, SELFTEST_BATCH_RECORD)
    if not os.path.isfile(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("# 自检样本的调度记录\n")


def fill_for_guard(skeleton_text, keep_prefix=None):
    """骨架里的占位填成能过派发闸的样子：批次一行的占位填成 SELFTEST_BATCH_RECORD（调用方在 --check 的样本仓里建它），
    草稿目录一行与跟着它的认领行、报告行填成工作根下的 selftest_scratch()，别的占位填「已填」；keep_prefix 开头的行原样留着。"""
    scratch = selftest_scratch()
    filled = []
    for line in skeleton_text.split("\n"):
        if keep_prefix and line.startswith(keep_prefix):
            filled.append(line)
            continue
        line = line.replace(SCRATCH_PLACEHOLDER, scratch)
        if line.startswith("草稿目录：") and PLACEHOLDER.search(line):
            filled.append(f"草稿目录：{scratch}")
        elif line.startswith("批次：") and PLACEHOLDER.search(line):
            filled.append(f"批次：{SELFTEST_BATCH_RECORD}#1")
        else:
            filled.append(PLACEHOLDER.sub("已填", line))
    return "\n".join(filled)


GUARD_BATCH_TABLE = re.compile(r"^BATCH_LINE_AGENTS = \{([^}]*)\}", re.M)


def guard_batch_types():
    """派发闸（runner-dispatch-guard.sh）里 BATCH_LINE_AGENTS 那张类型表（没写批次一行也拒的十种）→ (类型集, None)；读不出 → (None, 原因)。"""
    try:
        guard_text = open(DISPATCH_GUARD, encoding="utf-8").read()
    except OSError as error:
        return None, f"读不了派发闸 {DISPATCH_GUARD}：{error!r}"
    table = GUARD_BATCH_TABLE.search(guard_text)
    if table is None:
        return None, "派发闸里认不出 BATCH_LINE_AGENTS = {…} 那一行（表改了写法，这里的解析要跟着改）"
    return set(re.findall(r'"([^"]+)"', table.group(1))), None


def selftest():
    # 自证起的派发闸子进程（--check-prompt，环境原样继承）靠派发闸的标记认：RUNNER_DISPATCH_GUARD_BREAK 只在自证里认，这一趟是自证起的
    os.environ["RUNNER_DISPATCH_GUARD_SELFTEST_MARK"] = "runner-dispatch-guard.sh --selftest 与 dispatch-prompt.py --selftest 起的子进程"
    try:
        work_root_text = selftest_work_root()
    except (RuntimeError, ValueError) as error:
        print(f"  ✗ 自检：取不到工作根，自检的草稿目录没处落：{error}")   # gate-lint:detail
        print("    → 怎么办：工作根只由 .claude/hooks/lib_work_root.py 定（数据根取 SINGLEFS_DATA_ROOT 或多机配置 multi-host.env 的 SHARED_DATA_ROOT），照那份库的说明配好再跑")
        return 1
    work = tempfile.mkdtemp(prefix="dispatch-prompt-selftest-")
    failures, checked = [], 0
    try:
        write_batch_record(work)
        agents = os.path.join(work, ".claude", "agents")
        os.makedirs(agents)
        open(os.path.join(agents, "sample-writer.md"), "w", encoding="utf-8").write(
            "---\nname: sample-writer\nmodel: sonnet\nrequired-inputs: 草稿目录, 报告|报告路径, 条款\n---\n")
        text = skeleton("sample-writer", agents, {"草稿目录": f"{work_root_text}/x/"}, None)
        checked += 1
        wanted = ["批次：<填 调度记录#批号>", f"草稿目录：{work_root_text}/x/", f"报告：{work_root_text}/x/result.md", "条款：<填 条款>", "## 出口"]
        missing = [piece for piece in wanted if piece not in text]
        if missing or "开工先读" in text:
            failures.append(f"有定义的类型：骨架缺 {missing}，或多了没定义类型才有的开工先读行：\n{text}")
        text_builtin = skeleton("Explore", agents, {}, "records/x.md#1")
        checked += 1
        if "开工先读 `.claude/agent-common.md`" not in text_builtin or "批次：records/x.md#1" not in text_builtin:
            failures.append(f"没有定义的类型应当带开工先读一行、--batch 填进批次行：\n{text_builtin}")
        open(os.path.join(agents, "implementation-writer.md"), "w", encoding="utf-8").write(
            "---\nname: implementation-writer\nmodel: opus\nrequired-inputs: 草稿目录, 报告, 条款, 要动的 crates 文件\n---\n")
        text_writer = skeleton("implementation-writer", agents, {}, None)
        checked += 1
        if text_writer.count("要动的 crates 文件：") != 1:
            failures.append(f"实现员骨架「要动的 crates 文件」应当只一行（定义已经要了就不再加）：\n{text_writer}")
        # 派发闸不判的那几种标签行，骨架一种类型都不打（开关 retired-lines-printed 下这几格红）；类型表读派发闸的 BATCH_LINE_AGENTS
        guard_types, tables_problem = guard_batch_types()
        checked += 1
        if tables_problem:
            failures.append(tables_problem)
        for retired_type in (*sorted(guard_types or ()), "kb-scribe", "sweep", "Explore", "sample-writer"):
            checked += 1
            printed = [line for line in skeleton(retired_type, agents, {}, None).split("\n") if line.startswith(RETIRED_SKELETON_LABELS)]
            if printed:
                failures.append(f"骨架不该带撤掉的线程上限与长活几行（{retired_type}）：{printed}")
        import io, contextlib
        real_work = os.path.join(work, "real-definitions")
        real_agents = os.path.join(real_work, ".claude", "agents")
        os.makedirs(real_agents)
        foreign_home = os.path.join(work, "foreign-home")   # 换了家目录那一格的 HOME：不是 $HOME，里面什么都没有
        os.makedirs(foreign_home)
        write_batch_record(real_work)
        for guard_type in sorted(guard_types or set()):
            real_definition = os.path.join(REPOSITORY_ROOT, ".claude", "agents", f"{guard_type}.md")
            if os.path.isfile(real_definition):
                with open(real_definition, encoding="utf-8") as source, open(os.path.join(real_agents, f"{guard_type}.md"), "w", encoding="utf-8") as target:
                    target.write(source.read())
        for guard_type in sorted(guard_types or set()):
            text_real = skeleton(guard_type, real_agents, {}, None)
            checked += 1
            printed = [line for line in text_real.split("\n") if line.startswith(RETIRED_SKELETON_LABELS)]
            if printed:
                failures.append(f"真定义打出的骨架不该带撤掉的线程上限与长活几行（{guard_type}）：{printed}\n{text_real}")
            table_text = "变异表 crates/mutations.tsv。\n" if guard_type == "mutation-triage" else ""   # 变异分诊员的变异表由派发闸 ⑩ 另判
            real_path = os.path.join(work, f"real-{guard_type}.md")
            open(real_path, "w", encoding="utf-8").write(fill_for_guard(text_real) + table_text)
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code_real = check_prompt(real_path, guard_type, real_work)
            checked += 1
            if code_real != 0:
                failures.append(f"{guard_type} 的骨架填完，--check 应当退 0，实际 {code_real}：{buffer.getvalue()[-600:]}")
            # 家目录换成别的目录（第二台的家目录就是另一个）再判同一份提示：--check 的结局（退出码与输出）不随家目录变。
            # 草稿目录从工作根取，所以换了家目录结局相同；写成 ~ 起头的（弄坏开关 scratch-expanded-from-home）在本机家目录下过、换了家目录就落到工作根外面
            buffer_foreign = io.StringIO()
            with contextlib.redirect_stdout(buffer_foreign):
                code_foreign_home = with_home_directory(foreign_home, lambda: check_prompt(real_path, guard_type, real_work))
            checked += 1
            if code_foreign_home != code_real or buffer_foreign.getvalue() != buffer.getvalue():
                failures.append(f"{guard_type} 的骨架换了家目录（HOME 指到别的目录），--check 的结局应与本机家目录下相同，实际 本机家目录下退 {code_real}、换了之后退 {code_foreign_home}："
                                f"{buffer_foreign.getvalue()[-600:]}")
        # 骨架照抄进提示的每一行都要过借用写法闸与派发闸（「要临时服务的自证」那一行照抄进派发提示被两道拒过）：会跑自证的类型，骨架带上点名了登记脚本的那一截，
        # 逐行喂借用写法闸（.claude/singlefs-ai-sop/scripts/borrowed-convention.py --hook，挂 Agent），整份填完喂派发闸的 --check-prompt，看有没有许可跑项目代码与叫子 agent 交任务两种拒绝
        # （弄坏开关 temporary-service-line-unquoted 下这四格红：红在不点名脚本的那一种骨架上）
        borrow_judge = os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts", "borrowed-convention.py")
        for runner_type in SELFTEST_RUNNER_TYPES:
            checked += 1
            refused_lines = []
            # 骨架分两种各喂一遍：不点名脚本的（那一行只有规则一句，没有交回说法的整条命令跟在后面）与点名了登记脚本的（那一行后面接着带「写进报告……由主 agent 交」的整条命令，
            # 整句因提到主 agent 与有交回说法而不被派发闸判，也因带了「」原文而过借用写法闸）；只喂点名的那种，规则一句退回错写法也看不出来
            for copy_name, copy_settings in (("不点名脚本", {}), ("点名了登记脚本", {"出口": "改 scripts/mutation/crates-mutation-rows.py"})):
                copied = skeleton(runner_type, real_agents, copy_settings, None)
                for line in (line for line in copied.split("\n") if line.strip()):
                    hook_input = json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": runner_type, "prompt": line}}, ensure_ascii=False)
                    try:
                        judged = subprocess.run([sys.executable, borrow_judge, "--hook"], input=hook_input, capture_output=True, text=True, timeout=GUARD_TIMEOUT_SECONDS)
                        if judged.returncode == 2:
                            refused_lines.append(f"{copy_name}：借用写法闸拒：{line[:120]}")
                    except (OSError, subprocess.TimeoutExpired) as error:
                        refused_lines.append(f"{copy_name}：借用写法闸起不来（{error!r}）：{line[:120]}")
                copied_path = os.path.join(work, f"copied-{runner_type}-{'named' if copy_settings else 'unnamed'}.md")
                open(copied_path, "w", encoding="utf-8").write(fill_for_guard(copied))
                buffer = io.StringIO()
                with contextlib.redirect_stdout(buffer):
                    check_prompt(copied_path, runner_type, real_work)
                refused_lines += [f"{copy_name}：派发闸拒：{piece}" for piece in ("许可或要求它跑项目代码", "许可或要求它自己往统一任务服务交任务") if piece in buffer.getvalue()]
            if refused_lines:
                failures.append(f"骨架照抄进提示的每一行都要过借用写法闸与派发闸（{runner_type}）：" + "；".join(refused_lines))
        # 写范围一行（拆分提案第一百六十一节骨架那一半）：骨架按类型从写范围表取模式打一行「写范围：…」（读法 lib_write_scope.py）；表里没有行的类型、没有表的仓不打。
        # 样本仓里放本仓的 clerk、kb-scribe 定义与写范围表，端到端过一遍派发闸（这一行不能被当成写进范围外的路径）；弄坏开关 scope-line-omitted 下前两格红
        scope_work = os.path.join(work, "scope-line")
        scope_agents = os.path.join(scope_work, ".claude", "agents")
        os.makedirs(scope_agents)
        os.makedirs(os.path.join(scope_work, ".claude", "hooks"))
        write_batch_record(scope_work)
        for relative_path in (os.path.join(".claude", "agents", "clerk.md"), os.path.join(".claude", "agents", "kb-scribe.md"), os.path.join(".claude", "hooks", "agent-write-scope.tsv")):
            with open(os.path.join(REPOSITORY_ROOT, relative_path), encoding="utf-8") as source, open(os.path.join(scope_work, relative_path), "w", encoding="utf-8") as target:
                target.write(source.read())
        clerk_text = skeleton("clerk", scope_agents, {}, None)
        clerk_scope_lines = [line for line in clerk_text.split("\n") if line.startswith(f"{WRITE_SCOPE_LABEL}：")]
        checked += 1
        if len(clerk_scope_lines) != 1 or not all(pattern in clerk_scope_lines[0] for pattern in ("records/**", "research/prompts/**", "research/results/**", "$SINGLEFS_WORK_ROOT/**")) \
                or ".claude/kb/" in clerk_scope_lines[0]:
            failures.append(f"写范围一行：clerk 的骨架应当有一行「写范围：」列 records/**、research/prompts/**、research/results/** 与 $SINGLEFS_WORK_ROOT/**、不列 .claude/kb/，实际 {clerk_scope_lines}")
        kb_scope_lines = [line for line in skeleton("kb-scribe", scope_agents, {}, None).split("\n") if line.startswith(f"{WRITE_SCOPE_LABEL}：")]
        checked += 1
        if len(kb_scope_lines) != 1 or ".claude/kb/**" not in kb_scope_lines[0] or "records/" in kb_scope_lines[0]:
            failures.append(f"写范围一行：kb-scribe 的骨架应当有一行「写范围：」列 .claude/kb/**、不列 records/，实际 {kb_scope_lines}")
        unlisted_scope_lines = [line for line in skeleton("sweep", scope_agents, {}, None).split("\n") if line.startswith(f"{WRITE_SCOPE_LABEL}：")] \
            + [line for line in text.split("\n") if line.startswith(f"{WRITE_SCOPE_LABEL}：")]
        checked += 1
        if unlisted_scope_lines:
            failures.append(f"写范围一行：写范围表里没有行的类型（sweep）与没有表的样本仓里的骨架不该打「写范围：」一行，实际 {unlisted_scope_lines}")
        scope_prompt = os.path.join(work, "scope-clerk.md")
        with open(scope_prompt, "w", encoding="utf-8") as handle:
            handle.write(fill_for_guard(clerk_text).replace(selftest_scratch(), "$SINGLEFS_WORK_ROOT/dispatch-prompt-selftest"))   # 草稿目录写工作根记号：这一格有表，派发闸 ④ 判报告行的落点
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_scope = check_prompt(scope_prompt, "clerk", scope_work)
        checked += 1
        if code_scope != 0:
            failures.append(f"写范围一行：clerk 的骨架带着「写范围：」一行填完，--check 应当退 0（派发闸不把这一行当写进范围外的路径），实际 {code_scope}：{buffer.getvalue()[-600:]}")
        # 三方腿的报告行（派发闸 ⑰ 判形态）：骨架打 research/prompts/<填 轮>-<填 模型>-output.md 占位（定义的 required-inputs 要不要「报告」都打、只打一行）；
        # 只填别的占位、不动这一行 --check 退 3 并点名这一行；填成 research/prompts/x-r1-opus-output.md 退 0；填成草稿目录下的 result.md 退 3 并给出形态
        three_way_inputs = {"three-way-forward": "禁读清单, 草稿目录, 背景材料, 报告", "three-way-attack": "禁读清单, 草稿目录, 背景材料, 攻击面",
                            "three-way-defense": "禁读清单, 草稿目录, 背景材料, 判决, 报告|报告路径"}
        for three_way_type, three_way_required in three_way_inputs.items():
            open(os.path.join(agents, f"{three_way_type}.md"), "w", encoding="utf-8").write(
                f"---\nname: {three_way_type}\nmodel: sonnet\nrequired-inputs: {three_way_required}\nleg: three-way\n---\n")
            three_way_text = skeleton(three_way_type, agents, {"草稿目录": f"{work_root_text}/x-r1/"}, None)
            report_lines_found = [line for line in three_way_text.split("\n") if line.startswith(f"{REPORT_LINE_NAME}：")]
            checked += 1
            if report_lines_found != [f"{REPORT_LINE_NAME}：{THREE_WAY_REPORT_PLACEHOLDER}"]:
                failures.append(f"三方腿的骨架应当只打一行「报告：{THREE_WAY_REPORT_PLACEHOLDER}」（{three_way_type}），实际 {report_lines_found}")
            for three_way_label, report_value, three_way_want, three_way_needle in (
                    ("只填别的占位、不动报告行", None, 3, f"报告：{THREE_WAY_REPORT_PLACEHOLDER}"),
                    ("报告填成 research/prompts/x-r1-opus-output.md", "research/prompts/x-r1-opus-output.md", 0, None),
                    ("报告填成草稿目录下的 result.md", f"{work_root_text}/x-r1/result.md", 3, "research/prompts/<轮>-<模型>-output.md")):
                three_way_filled = fill_for_guard(three_way_text, keep_prefix=f"{REPORT_LINE_NAME}：")
                if report_value is not None:
                    three_way_filled = three_way_filled.replace(f"{REPORT_LINE_NAME}：{THREE_WAY_REPORT_PLACEHOLDER}", f"{REPORT_LINE_NAME}：{report_value}")
                three_way_path = os.path.join(work, f"{three_way_type}-{three_way_want}-{len(three_way_label)}.md")
                open(three_way_path, "w", encoding="utf-8").write(three_way_filled)
                buffer = io.StringIO()
                with contextlib.redirect_stdout(buffer):
                    code_three_way = check_prompt(three_way_path, three_way_type, work)
                checked += 1
                if code_three_way != three_way_want or (three_way_needle and three_way_needle not in buffer.getvalue()):
                    failures.append(f"三方腿骨架{three_way_label}（{three_way_type}），--check 应当退 {three_way_want}"
                                    + (f"并点名「{three_way_needle}」" if three_way_needle else "") + f"，实际 {code_three_way}：{buffer.getvalue()[-600:]}")
        # 三方腿类型表（拆分提案第九十节 ⑯）：骨架读到的类型集合（按哪几种打报告占位）与登记处（定义 frontmatter 的 leg: three-way 一行）逐个相同。
        # 样本仓里的登记故意与写死的三种不同：攻方没登记、辩方的值带行尾注释、另登记一种不以 three-way 起头的 sample-leg、材料员没登记、正文里写的不算；
        # 本仓按 frontmatter 现扫，至少要有一种（开关 leg-types-hardcoded 下样本仓那一格红）
        legs_agents = os.path.join(work, "legs-sample", ".claude", "agents")
        os.makedirs(legs_agents)
        for leg_name, leg_line in (("three-way-forward", "leg: three-way\n"), ("three-way-attack", ""), ("three-way-defense", "leg: three-way  # 注释\n"),
                                   ("three-way-materials", ""), ("sample-leg", "leg: three-way\n")):
            open(os.path.join(legs_agents, f"{leg_name}.md"), "w", encoding="utf-8").write(f"---\nname: {leg_name}\n{leg_line}---\n正文里写的\nleg: three-way\n不算\n")

        def frontmatter_scanned_legs(agents_directory):   # 「frontmatter 里写着什么」：lib_three_way_legs.py 里不拆键值的另一种读法（读不到那份库就是空表，下面两项红）
            return three_way_legs.three_way_leg_types_scanned(os.path.dirname(os.path.dirname(agents_directory))) if three_way_legs else []

        def skeleton_legs(agents_directory):
            names = [os.path.basename(path)[: -len(".md")] for path in glob.glob(os.path.join(agents_directory, "*.md"))]
            return sorted(name for name in set(names) | set(THREE_WAY_REPORT_TYPES_BEFORE_REGISTRY)
                          if f"\n{REPORT_LINE_NAME}：{THREE_WAY_REPORT_PLACEHOLDER}\n" in skeleton(name, agents_directory, {}, None))
        repository_agents = os.path.join(REPOSITORY_ROOT, ".claude", "agents")
        repository_legs = frontmatter_scanned_legs(repository_agents)
        checked += 1
        if skeleton_legs(legs_agents) != frontmatter_scanned_legs(legs_agents):
            failures.append(f"三方腿类型表：样本仓骨架按三方腿打报告占位的类型应当与 frontmatter 逐个相同 {frontmatter_scanned_legs(legs_agents)}，实际 {skeleton_legs(legs_agents)}")
        checked += 1
        if not repository_legs or skeleton_legs(repository_agents) != repository_legs:
            failures.append(f"三方腿类型表：本仓骨架按三方腿打报告占位的类型应当与 frontmatter 逐个相同、至少登记了一种 {repository_legs}，实际 {skeleton_legs(repository_agents)}")
        # 常设要求 D3、B8、A5（拆分提案第一百二十二节 ⑨）：实现员、工具员带一行「不碰：<填 …>」，执行员在「上一段岔路表里还差」后面带一行「承接：<填 …>」，
        # 每种类型都带「## 关的是哪几条」一节；开关 untouched-line-dropped、handover-line-dropped、addressed-section-dropped 下各自那几格红
        for untouched_type in ("implementation-writer", "tooling-writer"):
            checked += 1
            if skeleton(untouched_type, agents, {}, None).count(f"\n{UNTOUCHED_LINE}\n") != 1:
                failures.append(f"实现员与工具员的骨架应当带一行「不碰：<填 …>」（{untouched_type}）")
        checked += 1
        if "不碰：" in skeleton("kb-scribe", agents, {}, None):
            failures.append("不是实现员、工具员的类型（kb-scribe）骨架不该带「不碰：」一行")
        runner_lines = skeleton("experiment-runner", agents, {}, None).split("\n")
        checked += 1
        remaining_at = next((index for index, line in enumerate(runner_lines) if line.startswith("上一段岔路表里还差：")), None)
        if remaining_at is None or runner_lines[remaining_at + 1:remaining_at + 2] != [HANDOVER_LINE]:
            failures.append("执行员骨架应当在「上一段岔路表里还差」后面紧跟一行「承接：<填 …>」：\n" + "\n".join(runner_lines))
        missing_section = [section_type for section_type in (*sorted(guard_types or ()), "kb-scribe", "sweep", "Explore")
                           if f"\n{ADDRESSED_SECTION_TITLE}\n" not in skeleton(section_type, agents, {}, None)]
        checked += 1
        if missing_section:
            failures.append(f"骨架应当带「{ADDRESSED_SECTION_TITLE}」一节（缺的类型：{'、'.join(missing_section)}）")
        # 会跑自证的类型带一行「自证在哪跑」，别的类型不带；--set 填得进去
        for selftest_runner_type in SELFTEST_RUNNER_TYPES:
            text_selftest = skeleton(selftest_runner_type, agents, {}, None)
            checked += 3
            if text_selftest.count("自证在哪跑：<填") != 1:
                failures.append(f"会跑自证的类型骨架应当带一行「自证在哪跑：<填 …>」（{selftest_runner_type}）：\n{text_selftest}")
            elif f"{SELFTEST_LOCATION_HANDBACK_HINT}；这一轮没有自证要交的写「不跑」>" not in text_selftest:   # 没有这一行的归上一格报，这一格不重复红
                failures.append(f"会跑自证的类型骨架的「自证在哪跑」占位应当写明命令由主 agent 起、子 agent 自己不起（{selftest_runner_type}）：\n{text_selftest}")
            elif f"「{TEMPORARY_SERVICE_LABEL}」" not in next(line for line in text_selftest.split("\n") if line.startswith("自证在哪跑：<填")):   # 表里 required 的裸写会被拒，占位要点到整条写法在哪一行
                failures.append(f"会跑自证的类型骨架的「自证在哪跑」占位应当点到「{TEMPORARY_SERVICE_LABEL}」那一行（表里 required 的脚本裸写会被拒）（{selftest_runner_type}）：\n{text_selftest}")
        checked += 1
        if "自证在哪跑" in skeleton("kb-scribe", agents, {}, None):
            failures.append("不跑自证的类型（kb-scribe）骨架不该带「自证在哪跑」一行")
        text_selftest_set = skeleton("investigator", agents, {"自证在哪跑": "本仓主工作区那一份"}, None)
        checked += 1
        if "自证在哪跑：本仓主工作区那一份" not in text_selftest_set or "自证在哪跑：<填" in text_selftest_set:
            failures.append(f"--set 自证在哪跑=… 应当把这一行填上、不再留占位：\n{text_selftest_set}")
        # 要临时 prune-server 的自证（.claude/hooks/selftest-temporary-service.tsv 里 required 的）：会跑自证的类型带一行「要临时服务的自证」，别的类型不带；
        # --set 的值里点名了登记的脚本，那一行拼出它的整条包装写法，没登记的不拼（DISPATCH_PROMPT_BREAK=temporary-service-line-dropped、temporary-service-named-dropped、
        # temporary-service-command-bare、temporary-service-table-unread 下红的格写在文件头）
        for selftest_runner_type in SELFTEST_RUNNER_TYPES:
            checked += 1
            if skeleton(selftest_runner_type, agents, {}, None).count(f"\n{TEMPORARY_SERVICE_LABEL}：") != 1:
                failures.append(f"会跑自证的类型骨架应当带一行「{TEMPORARY_SERVICE_LABEL}：…」（{selftest_runner_type}）")
        checked += 1
        if f"{TEMPORARY_SERVICE_LABEL}：" in skeleton("kb-scribe", agents, {}, None):
            failures.append(f"不跑自证的类型（kb-scribe）骨架不该带「{TEMPORARY_SERVICE_LABEL}」一行")
        incident_script = "scripts/mutation/crates-mutation-rows.py"
        checked += 1
        if incident_script not in temporary_service_scripts():
            failures.append(f"要临时服务的自证登记表应当读得到 {incident_script}（任务 2731 退 2 的那一份），实际 {temporary_service_scripts()}")
        text_named = skeleton("tooling-writer", agents, {"出口": f"改 {incident_script} 与 scripts/kb/replace-once.py"}, None)
        incident_command = f"task-server/task submit -- {TEMPORARY_SERVICE_WRAPPER} python3 {incident_script} --selftest"
        checked += 2
        if incident_command not in text_named or "replace-once.py --selftest" in text_named:
            failures.append(f"骨架点名了要临时服务的自证，那一行应当拼出整条包装写法（{incident_command}），没登记的脚本（replace-once.py）不拼：\n{text_named}")
        if f"{TEMPORARY_SERVICE_LABEL}：自己写的" not in skeleton("tooling-writer", agents, {TEMPORARY_SERVICE_LABEL: "自己写的"}, None):
            failures.append(f"--set {TEMPORARY_SERVICE_LABEL}=… 应当把这一行换成主 agent 写的")
        # 认领行（欠账 C792）：带「草稿目录：」的类型骨架在它后面紧跟一行，没有草稿目录的类型不带；--set 填得进；--check 判在不在、目录对不对得上、用途空不空
        open(os.path.join(agents, "sample-nodir.md"), "w", encoding="utf-8").write("---\nname: sample-nodir\nmodel: sonnet\nrequired-inputs: -body.md\n---\n")
        lines_writer = skeleton("sample-writer", agents, {"草稿目录": f"{work_root_text}/x/"}, None).split("\n")
        checked += 1
        position = next((index for index, line in enumerate(lines_writer) if line.startswith("草稿目录：")), None)
        want_claim = f"认领：{CLAIM_COMMAND} {work_root_text}/x/ --purpose '<填 这个目录的用途一句>'"
        if position is None or lines_writer[position + 1:position + 2] != [want_claim]:
            failures.append(f"带草稿目录的类型骨架应当在草稿目录那一行后紧跟一行认领（{want_claim}）：\n" + "\n".join(lines_writer))
        checked += 1
        text_claim_set = skeleton("sample-writer", agents, {"草稿目录": "~/x/", "认领用途": "测试用"}, None)
        text_claim_empty = skeleton("sample-writer", agents, {}, None)
        text_nodir = skeleton("sample-nodir", agents, {}, None)
        if (f"认领：{CLAIM_COMMAND} ~/x/ --purpose '测试用'" not in text_claim_set or "认领：" in text_nodir
                or f"认领：{CLAIM_COMMAND} <填 草稿目录，与上一行一致> --purpose '<填" not in text_claim_empty):
            failures.append(f"--set 草稿目录、认领用途：要填进认领行，没填留占位，没有草稿目录的类型不带：\n{text_claim_set}\n{text_claim_empty}\n{text_nodir}")
        good_claim = f"草稿目录：~/x/\n认领：{CLAIM_COMMAND} {os.path.expanduser('~')}/x --purpose '一句'\n"   # 判 ~ 与展开写法是同一个目录（只过认领行那一判，不进派发闸，不涉工作根）
        for claim_label, claim_text, claim_want in (
                ("绿：目录写法不同（~ 与展开、末尾斜杠）但是同一个", good_claim, 0),
                ("红：没有认领行", "草稿目录：~/x/\n报告：y\n", 1),
                ("红：目录对不上", f"草稿目录：~/x/\n认领：{CLAIM_COMMAND} ~/y/ --purpose '一句'\n", 1),
                ("红：用途是空的", f"草稿目录：~/x/\n认领：{CLAIM_COMMAND} ~/x/ --purpose\n", 1),
                ("绿：没有草稿目录一行就不要求认领", "报告：y\n", 0),
                ("绿：闭合了的围栏代码块里的草稿目录一行不算，不要求认领", "示例：\n```\n草稿目录：~/x/\n```\n报告：y\n", 0),
                ("红：半角冒号的草稿目录一行同样要认领", "草稿目录: ~/x/\n报告：y\n", 1)):
            checked += 1
            got_claim = claim_problems(claim_text)
            if len(got_claim) != claim_want:
                failures.append(f"认领行 {claim_label}：应当 {claim_want} 个问题，实际 {got_claim}")
        # 「草稿目录：」一行调的是 lib_prompt_scratch_line.py 的 scratch_line_values（拆分提案第九十节 ㉚）：换成替身，认领行那一判交回的是替身给的目录
        checked += 1
        if prompt_scratch_line is None:
            failures.append(f"认领行调共用库：读不到 .claude/hooks/lib_prompt_scratch_line.py（{prompt_scratch_line_error!r}）")
        else:
            library_values = prompt_scratch_line.scratch_line_values
            prompt_scratch_line.scratch_line_values = lambda prompt: ["~/sentinel-from-library/"]
            try:
                routed_claim = claim_problems("没有草稿目录一行\n")
            finally:
                prompt_scratch_line.scratch_line_values = library_values
            if routed_claim != ["有「草稿目录：~/sentinel-from-library/」却没有「认领：" + CLAIM_COMMAND + " <草稿目录> --purpose '<用途一句>'」一行"]:
                failures.append(f"认领行调共用库：「草稿目录：」一行应当经 lib_prompt_scratch_line.py 的 scratch_line_values 认，实际 {routed_claim}")
        import io, contextlib
        no_claim = os.path.join(work, "no-claim.md")
        open(no_claim, "w", encoding="utf-8").write(
            "\n".join(line for line in PLACEHOLDER.sub("已填", text).split("\n") if not line.startswith("认领：")))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_no_claim = check_prompt(no_claim, "sample-writer", work)
        checked += 1
        if code_no_claim != 3 or "认领行：有「草稿目录：" not in buffer.getvalue():
            failures.append(f"--check 对缺认领行的提示应当退 3 并点名认领行，实际 {code_no_claim}：{buffer.getvalue()[-600:]}")
        # 报告文件名：「报告：」一行默认填 <草稿目录>/result.md（末尾斜杠有没有都只一个）；没给草稿目录留占位；--set 报告=… 填什么是什么；
        # --check 点名的文件名带 report 的，经派发闸 ⑰ 退 3
        def report_line_of(settings):
            return next((line for line in skeleton("sample-writer", agents, settings, None).split("\n") if line.startswith("报告：")), "")
        report_lines = [report_line_of({"草稿目录": "~/x"}), report_line_of({"草稿目录": "~/x/"}), report_line_of({}), report_line_of({"草稿目录": "~/x/", "报告": "~/y/final.md"})]
        checked += 1
        if report_lines != ["报告：~/x/result.md", "报告：~/x/result.md", "报告：<填 草稿目录，与上一行一致>/result.md", "报告：~/y/final.md"]:
            failures.append(f"骨架的「报告：」一行默认应当填 <草稿目录>/result.md（末尾斜杠只一个、没给草稿目录留占位、--set 报告=… 照填）：{report_lines}")
        report_red = os.path.join(work, "report-name-red.md")
        open(report_red, "w", encoding="utf-8").write(PLACEHOLDER.sub("已填", text).replace("/result.md", "/report.md"))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_report_red = check_prompt(report_red, "sample-writer", work)
        checked += 1
        if code_report_red != 3 or "报告行点名的文件名里带 report" not in buffer.getvalue() or "result.md" not in buffer.getvalue():
            failures.append(f"--check 对「报告：」写 report.md 的提示应当退 3 并点名文件名、给 result.md 的写法，实际 {code_report_red}：{buffer.getvalue()[-600:]}")
        text_gate = skeleton("gate-triage", agents, {}, None)
        checked += 1
        draft = os.path.join(work, "draft.md")
        open(draft, "w", encoding="utf-8").write(text.split("\n", 1)[1])   # 去掉「# 派发：」那一行：不知道类型，只查占位
        import io, contextlib
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_draft = check_prompt(draft)
        checked += 1
        if code_draft != 3 or "条款：<填" not in buffer.getvalue() or "没给类型" not in buffer.getvalue():
            failures.append(f"--check 对带占位、不知道类型的骨架应当退 3 并逐行列出、写明没给类型，实际 {code_draft}：{buffer.getvalue()[-300:]}")
        filled = os.path.join(work, "filled.md")
        open(filled, "w", encoding="utf-8").write(fill_for_guard(text))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_filled = check_prompt(filled, root=work)
        checked += 1
        if code_filled != 0:
            failures.append(f"--check 对填完的提示应当退 0（骨架第一行认出类型、派发闸放行），实际 {code_filled}：{buffer.getvalue()[-300:]}")
        # 带类型时起派发闸的 --check-prompt，同一份判法一次列全：有占位、缺禁读清单（写的是「禁读：」）两处
        open(os.path.join(agents, "sample-attack.md"), "w", encoding="utf-8").write(
            "---\nname: sample-attack\nmodel: sonnet\nrequired-inputs: 禁读清单, 草稿目录, 攻击面\n---\n")
        attack = os.path.join(work, "attack.md")
        open(attack, "w", encoding="utf-8").write(f"草稿目录 {work_root_text}/t/\n禁读：别的腿的产出\n攻击面：<填 攻击面>\n"
                                                  "\n## 关的是哪几条\n\n1. 自检样本\n")   # 派发闸 ⑱ 要有定义的类型带这一节；这一格只判上面三处
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_attack = check_prompt(attack, "sample-attack", work)
        checked += 1
        wanted_attack = ["有 2 处不成立", "占位没填", "缺它定义要的输入：禁读清单", "禁读清单：<填 禁读清单>"]
        if code_attack != 3 or any(piece not in buffer.getvalue() for piece in wanted_attack):
            failures.append(f"--check 带类型应当一次列全派发闸会拒的两处（{wanted_attack}），实际 {code_attack}：{buffer.getvalue()[-600:]}")
        # 工具员的骨架别的都填了、只留「自证在哪跑」：--check 起派发闸，退 3 并点名这一行
        open(os.path.join(agents, "tooling-writer.md"), "w", encoding="utf-8").write(
            "---\nname: tooling-writer\nmodel: opus\nrequired-inputs: 草稿目录, 报告, 出口, 要改的文件\n---\n")
        location_left = os.path.join(work, "selftest-location-left.md")
        open(location_left, "w", encoding="utf-8").write(fill_for_guard(skeleton("tooling-writer", agents, {}, None), keep_prefix="自证在哪跑："))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_location = check_prompt(location_left, root=work)
        checked += 1
        if code_location != 3 or "自证在哪跑：<填" not in buffer.getvalue():
            failures.append(f"--check 对没填自证在哪跑的提示应当退 3 并点名那一行，实际 {code_location}：{buffer.getvalue()[-600:]}")
        # 撤掉的那几行写进提示，派发闸不再因为它们拒：工具员骨架填完、另加线程上限（公式、括注）、预计挂钟超过 2 小时与写现算的多机一行，--check 退 0
        # （派发闸那一边的开关 retired-lines-refused 下这一格红）
        retired_filled = os.path.join(work, "retired-lines-filled.md")
        open(retired_filled, "w", encoding="utf-8").write(fill_for_guard(skeleton("tooling-writer", agents, {}, None))
                                                          + "\n线程上限：7 ÷ 2（本机）\n预计挂钟：超过 2 小时\n多机：每台线程按此刻空闲现算\n")
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_retired = check_prompt(retired_filled, root=work)
        checked += 1
        if code_retired != 0:
            failures.append(f"工具员骨架填完、另加撤掉的线程上限与长活几行，--check 应当退 0（派发闸不再判这几行），实际 {code_retired}：{buffer.getvalue()[-600:]}")
        # 「要临时服务的自证」那一行（点名了登记脚本、拼了整条包装写法的样子）填完过派发闸：每一处点到 task-server/task submit 的句子都带写进报告、由主 agent 交的说法（派发闸 ㉒）
        named_filled = os.path.join(work, "temporary-service-named-filled.md")
        open(named_filled, "w", encoding="utf-8").write(fill_for_guard(skeleton("tooling-writer", agents, {"出口": "改 scripts/mutation/crates-mutation-rows.py"}, None)))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_named = check_prompt(named_filled, root=work)
        checked += 1
        if code_named != 0:
            failures.append(f"骨架里要临时服务的自证那一行点名了登记的脚本、拼了整条包装写法，填完过派发闸 --check 应当退 0，实际 {code_named}：{buffer.getvalue()[-600:]}")
        batch_missing_record = os.path.join(work, "batch-missing-record.md")
        open(batch_missing_record, "w", encoding="utf-8").write(
            fill_for_guard(skeleton("tooling-writer", agents, {}, "records/dispatch-prompt-selftest-not-there.md#1")))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_batch = check_prompt(batch_missing_record, root=work)
        checked += 1
        if code_batch != 3 or "records/dispatch-prompt-selftest-not-there.md 在本仓仓根与各棵工作树里都不在" not in buffer.getvalue():
            failures.append(f"工具员骨架批次一行点名不在的调度记录，--check 应当退 3 并点名那份记录，实际 {code_batch}：{buffer.getvalue()[-600:]}")
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 skeleton()、check_prompt() 与派发闸的 --check-prompt；DISPATCH_PROMPT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {checked} 项）：有定义的类型按 required-inputs 逐组一行、--set 填进去；没有定义的带开工先读；实现员带要动的文件；"
          "每种类型的骨架都不打线程上限、预计挂钟、分片、续跑、多机、复用几行，派发闸十种类型的真定义骨架填完过派发闸，另加那几行照样过；"
          f"会跑自证的 {len(SELFTEST_RUNNER_TYPES)} 种类型（{'、'.join(SELFTEST_RUNNER_TYPES)}）带「自证在哪跑」一行、别的不带、--set 填得进；"
          "同样带「要临时服务的自证」一行（名单取 .claude/hooks/selftest-temporary-service.tsv，点名了登记脚本的拼出整条包装写法、填完过派发闸），骨架逐行照抄过借用写法闸与派发闸；"
          "带草稿目录的类型在草稿目录那一行后多一行认领（--set 草稿目录、认领用途填得进），--check 判认领行在不在、目录对得上、用途非空（「草稿目录：」一行经 lib_prompt_scratch_line.py 认）；"
          "「报告：」一行默认填 <草稿目录>/result.md、--check 写 report.md 的退 3；三方腿三种的「报告：」一行是 research/prompts/<填 轮>-<填 模型>-output.md 占位（只打一行），"
          "不填 --check 退 3、填成 research/prompts/x-r1-opus-output.md 退 0、填成草稿目录下的 result.md 退 3；"
          "--check 不知道类型时只查占位（有占位退 3、写明没给类型），"
          "知道类型时起派发闸的 --check-prompt、会拒的几处一次列全退 3（只留「自证在哪跑」没填的点名这一行），填完的退 0；"
          "实现员、工具员带「不碰：」一行，执行员在「上一段岔路表里还差」后面带「承接：」一行，每种类型都带「## 关的是哪几条」一节；"
          "自检的草稿目录从工作根取（lib_work_root.py），HOME 换成别的目录再判派发闸十种类型的真定义骨架，--check 的结局与本机家目录下相同")
    return 0


def refuse_break_outside_selftest(argv):
    """弄坏开关只在自证里认：命令行不是单独一个 --selftest 而 DISPATCH_PROMPT_BREAK 非空（不管值是什么），参数解析之前就拒，交回退出码 2；放行交回 None。
    自证不起自己的子进程（只起派发闸，派发闸的开关另有它自己的标记），所以不需要认本脚本的标记。"""
    if not BROKEN or argv == ["--selftest"]:
        return None
    print(f"  ✗ DISPATCH_PROMPT_BREAK={BROKEN} 是弄坏开关，只在自证（dispatch-prompt.py --selftest）里认，平常跑设了就拒，这一趟什么都没做", file=sys.stderr)
    print("     → 怎么办：去掉 DISPATCH_PROMPT_BREAK 再跑；要证红就跑 DISPATCH_PROMPT_BREAK=<项> python3 scripts/agents/dispatch-prompt.py --selftest", file=sys.stderr)
    return 2


def main(argv):
    refused = refuse_break_outside_selftest(argv)
    if refused is not None:
        return refused
    if argv[:1] == ["--selftest"]:
        return selftest()
    if argv[:1] == ["--check"]:
        options, rest = {}, argv[2:]
        while len(rest) >= 2 and rest[0] in ("--type", "--model", "--root"):
            options[rest[0]] = rest[1]
            rest = rest[2:]
        if len(argv) < 2 or argv[1].startswith("--") or rest:
            print("  ✗ --check 要一个提示文件，后面只认 --type <类型>、--model <模型>、--root <仓根>\n"
                  "  → 怎么办：dispatch-prompt.py --check <提示文件> --type <类型>（骨架第一行写了「# 派发：<类型>」的可以不给 --type）")
            return 2
        return check_prompt(argv[1], options.get("--type"), options.get("--root"), options.get("--model"))
    if not argv or argv[0].startswith("--"):
        print("  ✗ 用法：dispatch-prompt.py <类型> [--set 名=值 …] [--batch <调度记录>#<批号>] [--out 文件] | --check <提示文件> | --selftest\n"
              "  → 怎么办：第一个参数写 .claude/agents/ 里的类型名（clerk、implementation-writer …）")
        return 2
    agent_type, settings, batch, out = argv[0], {}, None, None
    rest = argv[1:]
    while rest:
        flag = rest.pop(0)
        if flag == "--set" and rest and "=" in rest[0]:
            name, value = rest.pop(0).split("=", 1)
            settings[name.strip()] = value.strip()
        elif flag == "--batch" and rest:
            batch = rest.pop(0)
        elif flag == "--out" and rest:
            out = rest.pop(0)
        else:
            print(f"  ✗ 认不出参数 {flag}\n  → 怎么办：只认 --set 名=值、--batch <记录>#<号>、--out 文件")
            return 2
    agents_dir = os.path.join(REPOSITORY_ROOT, ".claude", "agents")
    if agent_type not in ("Explore",) and not os.path.isfile(os.path.join(agents_dir, f"{agent_type}.md")):
        print(f"  ✗ .claude/agents/{agent_type}.md 不在：这个类型没有项目定义\n  → 怎么办：一次性的活派 clerk；只读搜索派 Explore；别的类型先写定义")
        return 2
    text = skeleton(agent_type, agents_dir, settings, batch)
    if out:
        with open(out, "x", encoding="utf-8") as handle:
            handle.write(text)
        print(f"  ✓ 骨架写进 {out}（{len(placeholders(text))} 处占位待填）")
    else:
        sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
