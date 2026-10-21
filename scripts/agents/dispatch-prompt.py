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
不成立的一次列全（占位、重型测试一行、定义要的输入、写范围、快照、长活声明、书记员规格……）；⑤ 与在跑的实现员撞文件、⑦ 并发与限额窗口
要会话状态，写成「派的那一刻另判」。不知道类型、或派发闸跑不起来的，只查占位（<填 …>），并写明别几条没核。

为什么：2026-09-20 起 8 天里派发闸拒了 187 次，前几种原因全是缺一行（岔路行、要改的文件、出口、还差）；主 agent 派完再给子 agent
发了 573 条纠正或追加。缺项该在写的时候补，不该在派的时候拒：骨架把定义要的每一样都列成一行占位，占位没填完就不派。
骨架里的固定行与派发闸（.claude/hooks/runner-dispatch-guard.sh）判的是同一套：重型测试一行、开工先读（没有定义的类型）、
定义的 required-inputs 每组一行、实现员的「要动的 crates 文件」、写回员的「规格检查已判」、执行员的「这一段回答的岔路」、
设计员与带重活的几种类型（线程上限那张类型表，九种：实现员、调查员、工具员、执行员、变异分诊员、攻方腿、clerk、前向腿、防方腿）的「预计挂钟」与超过 2 小时时的「分片」「续跑」「多机」三行
（派发闸的判法在 .claude/hooks/lib_long_run.py；「多机」一行只认那份库的 CANONICAL_HOSTS_SENTENCE 那一句规范写法，逐字（可带句末句号），后面不接任何说明（说明另起一行写），别的写法一律拒，判在派发闸 ⑭；骨架的占位引那一句，不另抄一份）。
「多机」一行的占位里另写放行行「多机已判：」的写法（HOSTS_VALVE_HINT）：用户给了逐台不同的台数或线程时另起一行写它与理由、逐台参数再另起一行，派发闸 ⑭ 判这一行理由不空、只写一行；
子 agent 有这一行时照逐台写的参数跑，没有就只照规范句。--check 认它（起派发闸，同一份判法）。
「自证在哪跑」一行派发闸不单判，没填时按占位拒。
带「草稿目录：」一行的类型，骨架在它后面多一行「认领：bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose '<用途一句>'」
（欠账 C792：草稿目录要有归属文件；--set 草稿目录=… 与 --set 认领用途=… 填进去）。--check 提示里有「草稿目录：」一行时再判这一行：
在不在、claim 后面的目录与「草稿目录：」是不是同一个（~ 展开、末尾斜杠不计）、用途是不是空的；占位没填的归占位那一判，不在这里重复报。
「线程上限」一行默认填登记的每台默认重活线程数（.claude/heavy-thread-default，经 scripts/hosts/heavy_threads.py 的 default_heavy_threads 取），
不再是「整机核数 ÷ 同时在跑的重活数」的占位（用户 2026-10-12 定：没给参数每台默认登记值）；派发闸 ⑮ 判提示里的 N 等于它，不等于要另写「线程上限已判：<理由>」。
登记读不到时这一行留占位，--check 拒。派发闸 ⑮ 这一行只认一个十进制整数，后面什么都不接，括注也不行（不写 ÷、/、一半、第二个数）；要说明的另起一行照普通正文写，那一行不判；闭合了的围栏代码块（照 CommonMark 认）里的行派发闸 ⑮ 同样不判（标签行两种认法遮不遮不一致的拒，派发闸文件头 ⑮ 条）。
带重活的九种类型缺这一行也拒；九种里多数派发并不带重活（clerk 的定义许编译、跑编译出来的代码，three-way-forward、three-way-defense 的定义只跑模型目录里的 tools/rerun.sh、不编译），不带重活的写一行「线程上限已判：<理由>」顶缺行（要登记值以外的数的，写「线程上限：N」再加这一行）。
骨架的两张类型表 HEAVY_WORK_TYPES、LONG_RUN_TYPES 与派发闸的 THREAD_LIMIT_REQUIRED_AGENTS、LONG_RUN_AGENTS 逐个相同，自检解析派发闸那两行逐个比。
退出码：0 打出骨架，或 --check 判下来派发闸放行（不知道类型时：没有占位）；2 用法错、定义不在、提示文件读不了；3 --check 判下来派发闸会拒（或有占位）。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句的开头，括号里是那条「✗ 自检：」的开头）：
  DISPATCH_PROMPT_BREAK=ignore-placeholders  placeholders() 一处占位都不认（不知道类型、只查占位的那一路）
      --check 带占位、不知道类型的骨架（「✗ 自检：--check 对带占位、不知道类型的骨架应当退 3 并逐行列出」）
  DISPATCH_PROMPT_BREAK=guard-skipped  check_prompt() 知道类型也不起派发闸的 --check-prompt，只查占位
      --check 带类型一次列全（「✗ 自检：--check 带类型应当一次列全派发闸会拒的三处」）
  DISPATCH_PROMPT_BREAK=long-run-lines-dropped  skeleton() 不打预计挂钟与分片、续跑、多机那四行
      设计员与带重活的九种类型的骨架带预计挂钟四行（「✗ 自检：设计员与带重活的类型的骨架应当带预计挂钟与分片、续跑、多机四行（<类型>）」，十种类型各一句；
      「✗ 自检：派发闸要预计挂钟的类型，真定义打出的骨架应当带线程上限…与预计挂钟、分片、续跑、多机各一行（<类型>）」，十种类型各一句）、
      骨架的多机一行点名登记值（「✗ 自检：骨架的多机一行应当点名 --emit-hosts 与 heavy-thread-default」「✗ 自检：骨架的多机一行应当写上放行行「多机已判：」的写法」）、
      填完的工具员骨架过派发闸（「✗ 自检：工具员骨架不动线程上限，--check 应当退 0」「✗ 自检：工具员骨架超过 2 小时、多机一行取登记值，--check 应当退 0」「…多机一行写现算，--check 应当退 3」）
  DISPATCH_PROMPT_BREAK=claim-line-dropped  skeleton() 不在「草稿目录：」后面打「认领：」一行
      带草稿目录的类型骨架在草稿目录那一行后面紧跟认领行（「✗ 自检：带草稿目录的类型骨架应当在草稿目录那一行后紧跟一行认领」）、
      --set 认领用途=… 填得进（「✗ 自检：--set 草稿目录、认领用途」）
  DISPATCH_PROMPT_BREAK=claim-check-skipped  --check 不判认领行
      认领行缺、目录对不上、用途为空各判红（「✗ 自检：认领行」）、--check 端到端缺认领行退 3（「✗ 自检：--check 对缺认领行的提示」）
  DISPATCH_PROMPT_BREAK=long-run-types-narrowed  skeleton() 只给设计员、执行员打那四行（退回改前的类型）
      设计员与带重活的类型的骨架带预计挂钟四行（实现员、调查员、工具员、变异分诊员、攻方腿、clerk、前向腿、防方腿各一句；真定义那一句同样各红）、
      填完的工具员骨架过派发闸（工具员骨架不动线程上限、超过 2 小时取登记值各一句）
  DISPATCH_PROMPT_BREAK=types-before-nine  两张类型表退回补 clerk、three-way-forward、three-way-defense 之前的（线程上限表少前向腿、防方腿，预计挂钟表少 clerk、前向腿、防方腿）
      骨架的类型表与派发闸的表逐个相同（「✗ 自检：骨架的类型表与派发闸的表应当逐个相同：线程上限 骨架多 [] 少 ['three-way-defense', 'three-way-forward']…」）、
      真定义打出的骨架带线程上限与预计挂钟四行（clerk、three-way-forward、three-way-defense 三句）、这三种的骨架填完 --check 应当退 0（三句，实际 3）
  DISPATCH_PROMPT_BREAK=long-run-hosts-live-restored  skeleton() 的「多机」一行退回现算的写法（点名 host-resources.py 现算每台线程与内存）
      骨架的多机一行引规范句、不写现算（「✗ 自检：骨架的多机一行应当点名 --emit-hosts 与 heavy-thread-default、引 CANONICAL_HOSTS_SENTENCE、不出现现算词」；
      现算词是 LIVE_HOSTS_WORDS 那一张：host-resources、available_threads、available_parallelism、nproc、cpu_count、sched_getaffinity、核数、忙闲、空闲、负载、除以、÷）、
      骨架的多机一行写上多机已判（「✗ 自检：骨架的多机一行应当写上放行行「多机已判：」的写法」）
  DISPATCH_PROMPT_BREAK=hosts-valve-hint-dropped  skeleton() 的「多机」一行占位里不写放行行「多机已判：」的写法（HOSTS_VALVE_HINT）
      骨架的多机一行写上多机已判（「✗ 自检：骨架的多机一行应当写上放行行「多机已判：」的写法」）
  DISPATCH_PROMPT_BREAK=live-words-narrowed  live_hosts_words_in() 退回只认 host-resources 与 available_threads 两个现算词（改前的判法）
      现算词表认全（「✗ 自检：现算词表应当认出每一个现算词：nproc …」，没认出的那几个逐个列出）
  DISPATCH_PROMPT_BREAK=selftest-location-dropped  skeleton() 不打「自证在哪跑」一行
      会跑自证的类型骨架带这一行（「✗ 自检：会跑自证的类型骨架应当带一行「自证在哪跑：<填 …>」」）、
      --set 填得进这一行（「✗ 自检：--set 自证在哪跑=…」）、--check 没填时点名这一行（「✗ 自检：--check 对没填自证在哪跑的提示」）
  DISPATCH_PROMPT_BREAK=thread-formula-restored  skeleton() 的线程上限默认退回「<填 整机核数 ÷ 同时在跑的重活数>」占位，不填登记值
      实现员骨架应当带登记的线程上限（「✗ 自检：实现员骨架应当带登记的线程上限」）、
      线程上限默认填的数应当过派发闸（「✗ 自检：工具员骨架不动线程上限，--check 应当退 0」）

会跑自证的类型（SELFTEST_RUNNER_TYPES，按各定义「输入」一节判）的骨架另带一行「自证在哪跑：<填 …>」：
自证登记 .claude/hooks/selftest-registry.tsv 按本仓路径认，副本、草稿里的拷贝跑 --selftest 被重型闸拒，派的时候就写明在哪跑。
"""
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
try:   # 多机一行的规范句只写在 .claude/hooks/lib_long_run.py 一处，骨架引它（读不到时占位里只指过去，派发闸 ⑭ 照样判）
    from lib_long_run import CANONICAL_HOSTS_SENTENCE  # noqa: E402
except Exception:   # 文件不在、语法错都算读不到
    CANONICAL_HOSTS_SENTENCE = "<规范句见 .claude/hooks/lib_long_run.py 的 CANONICAL_HOSTS_SENTENCE，读不到时先恢复那份库>"
BROKEN = os.environ.get("DISPATCH_PROMPT_BREAK", "")
# 骨架的「多机」一行里不许出现的现算词（台数与线程照登记值读，不现算）：现查余量的脚本与字段、各种取核数的写法、按忙闲算的中文说法；
# 「不现算」是规范句自己的话，不在表里
LIVE_HOSTS_WORDS = ("host-resources", "available_threads", "available_parallelism", "nproc", "cpu_count", "sched_getaffinity", "核数", "忙闲", "空闲", "负载", "除以", "÷")
PLACEHOLDER = re.compile(r"<填[^>]*>")
# --check 起的派发闸（判法只在它那里）与骨架第一行写的类型
DISPATCH_GUARD = os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "runner-dispatch-guard.sh")
DISPATCH_HEADER = re.compile(r"\A# 派发：(\S+)")
GUARD_TIMEOUT_SECONDS = 120
HEAVY_TEST_OWNERS = set()   # 子 agent 一个都不跑重型测试，骨架一律带「重型测试：不跑」
# 派发闸 ⑮ 要线程上限一行的九种类型（与 .claude/hooks/runner-dispatch-guard.sh 的 THREAD_LIMIT_REQUIRED_AGENTS 逐个相同；自检把两边的表逐个比）
HEAVY_WORK_TYPES = {"implementation-writer", "experiment-runner", "three-way-attack", "investigator", "tooling-writer", "clerk", "mutation-triage",
                    "three-way-forward", "three-way-defense"}
# 派发闸 ⑭ 判预计挂钟的类型：设计员，加上 ⑮ 的九种（与 .claude/hooks/runner-dispatch-guard.sh 的 LONG_RUN_AGENTS 逐个相同）
LONG_RUN_TYPES = ("experiment-designer", "implementation-writer", "investigator", "tooling-writer", "experiment-runner", "mutation-triage", "three-way-attack",
                  "clerk", "three-way-forward", "three-way-defense")
LONG_RUN_TYPES_BEFORE_WIDENING = ("experiment-designer", "experiment-runner")
if BROKEN == "types-before-nine":   # 补 clerk、three-way-forward、three-way-defense 之前的两张表（线程上限行有 clerk、没有预计挂钟四行，后两种两样都没有）
    HEAVY_WORK_TYPES = {"implementation-writer", "experiment-runner", "three-way-attack", "investigator", "tooling-writer", "clerk", "mutation-triage"}
    LONG_RUN_TYPES = ("experiment-designer", "implementation-writer", "investigator", "tooling-writer", "experiment-runner", "mutation-triage", "three-way-attack")
# 定义之外、派发闸另判的行：类型 → 那一行的骨架
# 派设计员、执行员要的预计挂钟一行与超过 2 小时时的三行（派发闸 ⑭；只写 2 小时内的，后三行删掉）
# 多机一行的占位里写上放行行「多机已判：<理由>」的写法（派发闸 ⑭ 认它）：用户给了逐台不同的台数或线程才写，子 agent 有它时照逐台写的参数、没有就只照规范句
HOSTS_VALVE_HINT = "用户给了逐台不同的台数或线程的，另起一行「多机已判：」后面写理由，逐台参数再另起一行照普通正文写（没有这一行，子 agent 只照规范句）；"   # 占位里不再套尖括号：骨架占位只认到第一个 >
LONG_RUN_LINES = ["预计挂钟：<填 2 小时内 / 超过 2 小时，后面接推的依据>",
                  "分片：<填 超过 2 小时才写：分片单元，几片怎么定；2 小时内删掉这一行>",
                  "续跑：<填 超过 2 小时才写：同一条命令重起只补没做完的，怎么认做完的；2 小时内删掉这一行>",
                  "多机：<填 超过 2 小时才写：逐字写这一句「" + CANONICAL_HOSTS_SENTENCE + "」（可带句末句号，后面不接任何说明，要说明另起一行写；别的写法派发闸 ⑭ 一律拒）；"
                  + ("" if BROKEN == "hosts-valve-hint-dropped" else HOSTS_VALVE_HINT) +
                  "分不了的这一行改写成「分不了：」点名缺的分片、续跑、多机与为什么；2 小时内删掉这一行>"]
LONG_RUN_HOSTS_LINE_BEFORE_REGISTERED = ("多机：<填 超过 2 小时才写：台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读、每台的线程与内存从 host-resources.py --json 现算，"
                                         "不写常量；分不了的这一行改写成「分不了：」点名缺的分片、续跑、多机与为什么；2 小时内删掉这一行>")   # 只有 long-run-hosts-live-restored 开关下还会打出来
EXTRA_LINES = {
    "implementation-writer": ["要动的 crates 文件：<填 从仓根起逐个列全，不与在跑的实现员撞>"],
    "kb-scribe": ["规格检查已判：写回员自己起草并跑 kb-spec-check.py"],
    "experiment-runner": ["这一段回答的岔路：<填 岔路单里的哪几行>", "上一段岔路表里还差：<填 续做时写还差的行；第一段删掉这一行>", *LONG_RUN_LINES],
    "experiment-designer": [*LONG_RUN_LINES],
}
for long_run_type in LONG_RUN_TYPES:   # 执行员、设计员上面已经带了；别的几种在这里补（要动的 crates 文件之类的类型专属行排在前面）
    if long_run_type not in ("experiment-designer", "experiment-runner"):
        EXTRA_LINES.setdefault(long_run_type, []).extend(LONG_RUN_LINES)
# 会跑自证的类型要的一行（用户 2026-10-11 定「派发提示写明自证在哪跑」）：按各定义「输入」一节判——
# 工具员要给判红样本与弄坏开关，实现员与执行员会改脚本、装置或变异表锚点，调查员在仓副本里改坏、二分
SELFTEST_LOCATION_LINE = ("自证在哪跑：<填 自证与样本档在哪个路径起：本仓主工作区那一份（自证登记按本仓路径认，副本与草稿里的拷贝被重型闸拒）；"
                          "这一轮不跑自证的写「不跑」>")
SELFTEST_RUNNER_TYPES = ("tooling-writer", "implementation-writer", "investigator", "experiment-runner")
for selftest_runner_type in SELFTEST_RUNNER_TYPES:
    EXTRA_LINES.setdefault(selftest_runner_type, []).append(SELFTEST_LOCATION_LINE)
HEAVY_THREADS_DIRECTORY = os.path.join(REPOSITORY_ROOT, "scripts", "hosts")
OLD_THREAD_FORMULA_PLACEHOLDER = "<填 整机核数 ÷ 同时在跑的重活数>"   # 用户 2026-10-11 否掉的公式，只有 thread-formula-restored 开关下还会打出来
THREAD_OVERRIDE_VARIABLE = "SINGLEFS_THREAD_CAP"


def registered_heavy_threads():
    """登记的每台默认重活线程数（.claude/heavy-thread-default，经 scripts/hosts/heavy_threads.py 的 default_heavy_threads 取）；读不到返回 None。
    临时覆盖（SINGLEFS_THREAD_CAP）不算：骨架填的是登记值本身。"""
    try:
        sys.path.insert(0, HEAVY_THREADS_DIRECTORY)
        import heavy_threads
        environment = {name: value for name, value in os.environ.items() if name != THREAD_OVERRIDE_VARIABLE}
        return int(heavy_threads.default_heavy_threads(environment=environment))
    except Exception:   # 文件不在、语法错、接口对不上、登记文件读不出数都算读不到
        return None


def thread_limit_default():
    """骨架里「线程上限」一行没用 --set 给时填什么：登记值；读不到留占位（--check 拒）。"""
    if BROKEN == "thread-formula-restored":
        return OLD_THREAD_FORMULA_PLACEHOLDER
    registered = registered_heavy_threads()
    return str(registered) if registered is not None else "<填 登记的每台默认重活线程数读不到，照 .claude/heavy-thread-default 填>"


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
DRAFT_LINE = re.compile(r"^草稿目录：\s*(\S+)", re.M)
CLAIM_LINE = re.compile(r"^认领：.*?work-directory-ownership\.sh\s+claim\s+(\S+)\s+--purpose(?:\s+(.*\S))?\s*$", re.M)


def claim_line(settings):
    if "认领" in settings:
        return f"认领：{settings['认领']}"
    directory = settings.get("草稿目录", "<填 草稿目录，与上一行一致>")
    purpose = settings.get("认领用途")
    return f"认领：{CLAIM_COMMAND} {directory} --purpose " + (f"'{purpose}'" if purpose else "'<填 这个目录的用途一句>'")


def normalized_directory(value):
    return os.path.expanduser(value.strip("`'\"")).rstrip("/")


def claim_problems(text):
    """提示里有「草稿目录：」一行时，认领行在不在、目录对不对得上、用途空不空：[问题一句]；占位没填的不报（占位那一判报）。"""
    if BROKEN == "claim-check-skipped":
        return []
    draft = DRAFT_LINE.search(text)
    if draft is None:
        return []
    claim = CLAIM_LINE.search(text)
    if claim is None:
        return [f"有「草稿目录：{draft.group(1)}」却没有「认领：{CLAIM_COMMAND} <草稿目录> --purpose '<用途一句>'」一行"]
    problems = []
    claimed, purpose = claim.group(1), (claim.group(2) or "").strip("`'\" ")
    if "<填" not in claimed and "<填" not in draft.group(1) and normalized_directory(claimed) != normalized_directory(draft.group(1)):
        problems.append(f"认领的目录 {claimed} 与「草稿目录：」的 {draft.group(1)} 不是同一个")
    if not purpose:
        problems.append("认领行的 --purpose 后面是空的")
    return problems


def skeleton(agent_type, agents_dir, settings, batch):
    fields = frontmatter_fields(os.path.join(agents_dir, f"{agent_type}.md"))
    lines = [f"# 派发：{agent_type}", ""]
    if fields is None:
        lines.append("开工先读 `.claude/agent-common.md`")
    if agent_type not in HEAVY_TEST_OWNERS:
        lines.append("重型测试：不跑")
    lines.append(f"批次：{batch or '<填 调度记录#批号>'}")
    if agent_type in HEAVY_WORK_TYPES:
        lines.append(f"线程上限：{settings.get('线程上限') or thread_limit_default()}")
    required = (fields or {}).get("required-inputs", "")
    emitted = set()
    for group in [piece.strip() for piece in required.split(",") if piece.strip()]:
        name = group.split("|", 1)[0].strip()
        emitted.add(name)
        lines.append(f"{name}：{settings.get(name, '<填 ' + group + '>')}")
        if name == "草稿目录" and BROKEN != "claim-line-dropped":
            lines.append(claim_line(settings))
    for extra in EXTRA_LINES.get(agent_type, []):
        if BROKEN == "long-run-lines-dropped" and extra in LONG_RUN_LINES:
            continue
        if BROKEN == "long-run-types-narrowed" and extra in LONG_RUN_LINES and agent_type not in LONG_RUN_TYPES_BEFORE_WIDENING:
            continue
        if BROKEN == "long-run-hosts-live-restored" and extra.startswith("多机："):
            extra = LONG_RUN_HOSTS_LINE_BEFORE_REGISTERED
        if BROKEN == "selftest-location-dropped" and extra == SELFTEST_LOCATION_LINE:
            continue
        key = extra.split("：", 1)[0]
        if key in emitted:
            continue   # 定义的 required-inputs 已经要了这一行，不重复
        lines.append(f"{key}：{settings[key]}" if key in settings else extra)
    lines += ["", "## 关的是哪几条", "", settings.get("关的是哪几条", "<填 逐条列已经抓到的问题，每条各自的验收标准>"), "",
              "## 出口", "", settings.get("出口", "<填 做到什么算完、交什么到哪>"), ""]
    return "\n".join(lines) + "\n"


def live_hosts_words_in(text):
    """text 里出现的现算词（LIVE_HOSTS_WORDS，按子串找），按表里的次序交回。弄坏开关 live-words-narrowed 下只认 host-resources 与 available_threads。"""
    words = LIVE_HOSTS_WORDS[:2] if BROKEN == "live-words-narrowed" else LIVE_HOSTS_WORDS
    return [word for word in words if word in text]


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


def fill_for_guard(skeleton_text, keep_prefix=None):
    """骨架里的占位填成能过派发闸的样子：预计挂钟填 2 小时内、分片 / 续跑 / 多机三行删掉（骨架写明 2 小时内删掉这三行），别的占位填「已填」；keep_prefix 开头的行原样留着。"""
    filled = []
    for line in skeleton_text.split("\n"):
        if keep_prefix and line.startswith(keep_prefix):
            filled.append(line)
        elif line.startswith("预计挂钟："):
            filled.append("预计挂钟：2 小时内")
        elif not line.startswith(("分片：", "续跑：", "多机：")):
            filled.append(PLACEHOLDER.sub("已填", line))
    return "\n".join(filled)


GUARD_THREAD_TABLE = re.compile(r"^THREAD_LIMIT_REQUIRED_AGENTS = \{(.*?)\}", re.M | re.S)
GUARD_LONG_RUN_TABLE = re.compile(r"^LONG_RUN_AGENTS = \{([^}]*)\}([^\n]*)", re.M)


def guard_type_tables():
    """派发闸（runner-dispatch-guard.sh）里 THREAD_LIMIT_REQUIRED_AGENTS 与 LONG_RUN_AGENTS 两张类型表 → (线程上限的类型集, 预计挂钟的类型集, None)；读不出 → (None, None, 原因)。"""
    try:
        guard_text = open(DISPATCH_GUARD, encoding="utf-8").read()
    except OSError as error:
        return None, None, f"读不了派发闸 {DISPATCH_GUARD}：{error!r}"
    thread_table, long_run_table = GUARD_THREAD_TABLE.search(guard_text), GUARD_LONG_RUN_TABLE.search(guard_text)
    if thread_table is None or long_run_table is None:
        return None, None, "派发闸里认不出 THREAD_LIMIT_REQUIRED_AGENTS = {…} 或 LONG_RUN_AGENTS = {…} 那两行（表改了写法，这里的解析要跟着改）"
    thread_types = set(re.findall(r'"([^"]+)"', thread_table.group(1)))
    long_run_types = set(re.findall(r'"([^"]+)"', long_run_table.group(1)))
    if "THREAD_LIMIT_REQUIRED_AGENTS" in long_run_table.group(2):
        long_run_types |= thread_types
    return thread_types, long_run_types, None


def selftest():
    work = tempfile.mkdtemp(prefix="dispatch-prompt-selftest-")
    failures, checked = [], 0
    try:
        agents = os.path.join(work, ".claude", "agents")
        os.makedirs(agents)
        open(os.path.join(agents, "sample-writer.md"), "w", encoding="utf-8").write(
            "---\nname: sample-writer\nmodel: sonnet\nrequired-inputs: 草稿目录, 报告|报告路径, 条款\n---\n")
        text = skeleton("sample-writer", agents, {"草稿目录": "~/.local/share/singlefs/work/x/"}, None)
        checked += 1
        wanted = ["重型测试：不跑", "批次：<填 调度记录#批号>", "草稿目录：~/.local/share/singlefs/work/x/", "报告：<填 报告|报告路径>", "条款：<填 条款>", "## 出口"]
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
        registered_threads = registered_heavy_threads()
        if text_writer.count("要动的 crates 文件：") != 1:
            failures.append(f"实现员骨架「要动的 crates 文件」应当只一行（定义已经要了就不再加）：\n{text_writer}")
        checked += 1
        if registered_threads is None or f"\n线程上限：{registered_threads}\n" not in text_writer or "整机核数" in text_writer:
            failures.append(f"实现员骨架应当带登记的线程上限（{registered_threads}，.claude/heavy-thread-default 经 scripts/hosts/heavy_threads.py 取），不再是整机核数除以重活数的占位：\n{text_writer}")
        checked += 1
        if "线程上限：16" not in skeleton("implementation-writer", agents, {"线程上限": "16"}, None):
            failures.append("--set 线程上限=16 应当把这一行填成 16，不被默认值盖掉")
        long_run_keys = ["预计挂钟：<填", "分片：<填", "续跑：<填", "多机：<填"]
        for long_run_type in LONG_RUN_TYPES:
            text_long_run = skeleton(long_run_type, agents, {}, None)
            checked += 1
            if any(text_long_run.count(key) != 1 for key in long_run_keys):
                failures.append(f"设计员与带重活的类型的骨架应当带预计挂钟与分片、续跑、多机四行（{long_run_type}）：\n{text_long_run}")
        checked += 1
        if any(key in skeleton("kb-scribe", agents, {}, None) for key in long_run_keys):
            failures.append("不在类型表里的类型（kb-scribe）骨架不该带预计挂钟与分片、续跑、多机四行")
        hosts_line = next((line for line in skeleton("tooling-writer", agents, {}, None).split("\n") if line.startswith("多机：")), "")
        checked += 1
        if ("--emit-hosts" not in hosts_line or "heavy-thread-default" not in hosts_line or live_hosts_words_in(hosts_line)
                or CANONICAL_HOSTS_SENTENCE not in hosts_line or CANONICAL_HOSTS_SENTENCE.startswith("<")):
            failures.append(f"骨架的多机一行应当点名 --emit-hosts 与 heavy-thread-default、引 CANONICAL_HOSTS_SENTENCE、不出现现算词"
                            f"（出现了 {live_hosts_words_in(hosts_line)}）：{hosts_line}")
        # 现算词表自己认得全：每个词各造一句，都要认出来（开关 live-words-narrowed 下只认两个，这一项红）
        checked += 1
        missed = [word for word in LIVE_HOSTS_WORDS if word not in live_hosts_words_in(f"多机：每台线程按 {word} 取")]
        if missed:
            failures.append(f"现算词表应当认出每一个现算词：{' '.join(missed)}")
        checked += 1
        if "「多机已判：」后面写理由" not in hosts_line:
            failures.append(f"骨架的多机一行应当写上放行行「多机已判：」的写法（用户给了逐台参数时用）：{hosts_line}")
        # 骨架的两张类型表与派发闸 ⑮ ⑭ 的表逐个相同；表里每一种用真定义打骨架：带线程上限与预计挂钟四行，填完过派发闸
        guard_thread_types, guard_long_run_types, tables_problem = guard_type_tables()
        checked += 1
        if tables_problem:
            failures.append(tables_problem)
        elif HEAVY_WORK_TYPES != guard_thread_types or set(LONG_RUN_TYPES) != guard_long_run_types:
            failures.append("骨架的类型表与派发闸的表应当逐个相同："
                            f"线程上限 骨架多 {sorted(HEAVY_WORK_TYPES - guard_thread_types)} 少 {sorted(guard_thread_types - HEAVY_WORK_TYPES)}；"
                            f"预计挂钟 骨架多 {sorted(set(LONG_RUN_TYPES) - guard_long_run_types)} 少 {sorted(guard_long_run_types - set(LONG_RUN_TYPES))}")
        import io, contextlib
        real_work = os.path.join(work, "real-definitions")
        real_agents = os.path.join(real_work, ".claude", "agents")
        os.makedirs(real_agents)
        for guard_type in sorted((guard_thread_types or set()) | (guard_long_run_types or set())):
            real_definition = os.path.join(REPOSITORY_ROOT, ".claude", "agents", f"{guard_type}.md")
            if os.path.isfile(real_definition):
                with open(real_definition, encoding="utf-8") as source, open(os.path.join(real_agents, f"{guard_type}.md"), "w", encoding="utf-8") as target:
                    target.write(source.read())
        for guard_type in sorted(guard_long_run_types or set()):
            text_real = skeleton(guard_type, real_agents, {}, None)
            checked += 1
            wanted_lines = (["预计挂钟：<填", "分片：<填", "续跑：<填", "多机：<填"]
                            + ([f"\n线程上限：{registered_threads}\n"] if guard_type in (guard_thread_types or set()) else []))
            if registered_threads is None or any(text_real.count(key) != 1 for key in wanted_lines):
                failures.append(f"派发闸要预计挂钟的类型，真定义打出的骨架应当带线程上限（{registered_threads}，在线程上限表里的）与预计挂钟、分片、续跑、多机各一行（{guard_type}）：\n{text_real}")
            table_text = "变异表 crates/mutations.tsv。\n" if guard_type == "mutation-triage" else ""   # 变异分诊员的变异表由派发闸 ⑩ 另判
            real_path = os.path.join(work, f"real-{guard_type}.md")
            open(real_path, "w", encoding="utf-8").write(fill_for_guard(text_real) + table_text)
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code_real = check_prompt(real_path, guard_type, real_work)
            checked += 1
            if code_real != 0:
                failures.append(f"{guard_type} 的骨架填完（预计挂钟 2 小时内）--check 应当退 0，实际 {code_real}：{buffer.getvalue()[-600:]}")
        # 会跑自证的类型带一行「自证在哪跑」，别的类型不带；--set 填得进去
        for selftest_runner_type in SELFTEST_RUNNER_TYPES:
            text_selftest = skeleton(selftest_runner_type, agents, {}, None)
            checked += 1
            if text_selftest.count("自证在哪跑：<填") != 1:
                failures.append(f"会跑自证的类型骨架应当带一行「自证在哪跑：<填 …>」（{selftest_runner_type}）：\n{text_selftest}")
        checked += 1
        if "自证在哪跑" in skeleton("kb-scribe", agents, {}, None):
            failures.append("不跑自证的类型（kb-scribe）骨架不该带「自证在哪跑」一行")
        text_selftest_set = skeleton("investigator", agents, {"自证在哪跑": "本仓主工作区那一份"}, None)
        checked += 1
        if "自证在哪跑：本仓主工作区那一份" not in text_selftest_set or "自证在哪跑：<填" in text_selftest_set:
            failures.append(f"--set 自证在哪跑=… 应当把这一行填上、不再留占位：\n{text_selftest_set}")
        # 认领行（欠账 C792）：带「草稿目录：」的类型骨架在它后面紧跟一行，没有草稿目录的类型不带；--set 填得进；--check 判在不在、目录对不对得上、用途空不空
        open(os.path.join(agents, "sample-nodir.md"), "w", encoding="utf-8").write("---\nname: sample-nodir\nmodel: sonnet\nrequired-inputs: -body.md\n---\n")
        lines_writer = skeleton("sample-writer", agents, {"草稿目录": "~/.local/share/singlefs/work/x/"}, None).split("\n")
        checked += 1
        position = next((index for index, line in enumerate(lines_writer) if line.startswith("草稿目录：")), None)
        want_claim = f"认领：{CLAIM_COMMAND} ~/.local/share/singlefs/work/x/ --purpose '<填 这个目录的用途一句>'"
        if position is None or lines_writer[position + 1:position + 2] != [want_claim]:
            failures.append(f"带草稿目录的类型骨架应当在草稿目录那一行后紧跟一行认领（{want_claim}）：\n" + "\n".join(lines_writer))
        checked += 1
        text_claim_set = skeleton("sample-writer", agents, {"草稿目录": "~/x/", "认领用途": "测试用"}, None)
        text_claim_empty = skeleton("sample-writer", agents, {}, None)
        text_nodir = skeleton("sample-nodir", agents, {}, None)
        if (f"认领：{CLAIM_COMMAND} ~/x/ --purpose '测试用'" not in text_claim_set or "认领：" in text_nodir
                or f"认领：{CLAIM_COMMAND} <填 草稿目录，与上一行一致> --purpose '<填" not in text_claim_empty):
            failures.append(f"--set 草稿目录、认领用途：要填进认领行，没填留占位，没有草稿目录的类型不带：\n{text_claim_set}\n{text_claim_empty}\n{text_nodir}")
        good_claim = f"草稿目录：~/.local/share/singlefs/work/x/\n认领：{CLAIM_COMMAND} {os.path.expanduser('~')}/.local/share/singlefs/work/x --purpose '一句'\n"
        for claim_label, claim_text, claim_want in (
                ("绿：目录写法不同（~ 与展开、末尾斜杠）但是同一个", good_claim, 0),
                ("红：没有认领行", "草稿目录：~/x/\n报告：y\n", 1),
                ("红：目录对不上", f"草稿目录：~/x/\n认领：{CLAIM_COMMAND} ~/y/ --purpose '一句'\n", 1),
                ("红：用途是空的", f"草稿目录：~/x/\n认领：{CLAIM_COMMAND} ~/x/ --purpose\n", 1),
                ("绿：没有草稿目录一行就不要求认领", "报告：y\n", 0)):
            checked += 1
            got_claim = claim_problems(claim_text)
            if len(got_claim) != claim_want:
                failures.append(f"认领行 {claim_label}：应当 {claim_want} 个问题，实际 {got_claim}")
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
        text_gate = skeleton("gate-triage", agents, {}, None)
        checked += 1
        if "重型测试：不跑" not in text_gate:
            failures.append("原来跑重型测试的类型名，骨架也要带「重型测试：不跑」")
        draft = os.path.join(work, "draft.md")
        open(draft, "w", encoding="utf-8").write(text.split("\n", 1)[1])   # 去掉「# 派发：」那一行：不知道类型，只查占位
        import io, contextlib
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_draft = check_prompt(draft)
        checked += 1
        if code_draft != 3 or "报告：<填" not in buffer.getvalue() or "没给类型" not in buffer.getvalue():
            failures.append(f"--check 对带占位、不知道类型的骨架应当退 3 并逐行列出、写明没给类型，实际 {code_draft}：{buffer.getvalue()[-300:]}")
        filled = os.path.join(work, "filled.md")
        open(filled, "w", encoding="utf-8").write(PLACEHOLDER.sub("已填", text))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_filled = check_prompt(filled, root=work)
        checked += 1
        if code_filled != 0:
            failures.append(f"--check 对填完的提示应当退 0（骨架第一行认出类型、派发闸放行），实际 {code_filled}：{buffer.getvalue()[-300:]}")
        # 带类型时起派发闸的 --check-prompt，同一份判法一次列全：缺重型测试一行、有占位、缺禁读清单（写的是「禁读：」）三处
        open(os.path.join(agents, "sample-attack.md"), "w", encoding="utf-8").write(
            "---\nname: sample-attack\nmodel: sonnet\nrequired-inputs: 禁读清单, 草稿目录, 攻击面\n---\n")
        attack = os.path.join(work, "attack.md")
        open(attack, "w", encoding="utf-8").write("草稿目录 ~/.local/share/singlefs/work/t/\n禁读：别的腿的产出\n攻击面：<填 攻击面>\n")
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_attack = check_prompt(attack, "sample-attack", work)
        checked += 1
        wanted_attack = ["有 3 处不成立", "没有「重型测试：不跑」一行", "占位没填", "缺它定义要的输入：禁读清单", "禁读清单：<填 禁读清单>"]
        if code_attack != 3 or any(piece not in buffer.getvalue() for piece in wanted_attack):
            failures.append(f"--check 带类型应当一次列全派发闸会拒的三处（{wanted_attack}），实际 {code_attack}：{buffer.getvalue()[-600:]}")
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
        # 工具员骨架里线程上限一行不动（默认填的登记值），别的占位都填上：派发闸 ⑮ 要放行（骨架默认与闸判的是同一个数）
        thread_default_filled = os.path.join(work, "thread-default-filled.md")
        open(thread_default_filled, "w", encoding="utf-8").write(fill_for_guard(skeleton("tooling-writer", agents, {}, None)))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_thread_default = check_prompt(thread_default_filled, root=work)
        checked += 1
        if code_thread_default != 0 or "线程上限" in buffer.getvalue():
            failures.append(f"工具员骨架不动线程上限，--check 应当退 0（默认填的登记值过派发闸 ⑮），实际 {code_thread_default}：{buffer.getvalue()[-600:]}")
        # 预计挂钟超过 2 小时：骨架的分片、续跑、多机三行填成「台数取多机配置、每台线程取登记值」的写法过派发闸 ⑭，多机写现算的不过
        def over_two_hours_prompt(hosts_line):
            lines = ["预计挂钟：超过 2 小时，推的：一千行 × 每行 2 分钟" if line.startswith("预计挂钟：") else line
                     for line in fill_for_guard(skeleton("tooling-writer", agents, {}, None)).split("\n")]
            return "\n".join(lines + ["分片：按行号 mod 片数切", "续跑：结果表里已有的行跳过，重起同一条命令只补没做完的", hosts_line, ""])
        for over_index, (over_label, hosts_text, over_want) in enumerate((
                ("取规范句", "多机：" + CANONICAL_HOSTS_SENTENCE, 0),
                ("取规范句、带句末句号", "多机：" + CANONICAL_HOSTS_SENTENCE + "。", 0),
                ("取规范句、说明另起一行写", "多机：" + CANONICAL_HOSTS_SENTENCE + "\n说明：双机时两台各取这一个数", 0),
                ("规范句后接「；」与说明", "多机：" + CANONICAL_HOSTS_SENTENCE + "；双机时两台各取这一个数", 3),
                ("规范句后接按此刻空闲现算", "多机：" + CANONICAL_HOSTS_SENTENCE + "；按此刻空闲现算每台槽数与线程", 3),
                ("规范句后接机器忙时按负载降", "多机：" + CANONICAL_HOSTS_SENTENCE + "；机器忙时按负载降", 3),
                ("规范句后接 host-resources 的 available_threads", "多机：" + CANONICAL_HOSTS_SENTENCE + "；每台线程按 host-resources.py 的 available_threads 取", 3),
                ("写现算", "多机：台数取 crash-point-replay-shard-configuration-check.sh --emit-hosts，每台线程用 host-resources.py --json 现算", 3),
                ("换了词的取法（不看忙闲）", "多机：台数取多机配置（--emit-hosts），每台线程取登记值 .claude/heavy-thread-default，不看忙闲", 3),
                ("规范句后的说明里有数字", "多机：" + CANONICAL_HOSTS_SENTENCE + "；见第 2 节", 3),
                ("规范句加多机已判与逐台参数一行", "多机：" + CANONICAL_HOSTS_SENTENCE + "\n多机已判：用户给了逐台参数，第二台上在跑用户自己的基准\n逐台参数：本机照登记值，第二台 16 线程", 0),
                ("规范句加多机已判而没写理由", "多机：" + CANONICAL_HOSTS_SENTENCE + "\n多机已判：\n逐台参数：第二台 16 线程", 3))):
            over_path = os.path.join(work, f"over-two-hours-{over_index}.md")
            open(over_path, "w", encoding="utf-8").write(over_two_hours_prompt(hosts_text))
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code_over = check_prompt(over_path, root=work)
            checked += 1
            if code_over != over_want or (over_want == 3 and "多机" not in buffer.getvalue()):
                failures.append(f"工具员骨架超过 2 小时、多机一行{over_label}，--check 应当退 {over_want}，实际 {code_over}：{buffer.getvalue()[-600:]}")
        # 线程上限一行：只认一个整数，括注一概拒（说明另起一行照普通正文写，那一行不判；代码块里的示例行不判）
        for thread_label, thread_text, thread_want in (("后接括注", "线程上限：{n}（本机）", 3), ("后接按负载降的括注", "线程上限：{n}（机器忙时按负载降）", 3),
                                                       ("另起一行照普通正文写说明", "线程上限：{n}\n本机，双机时两台各取这一个数（{n} 与 {n}）", 0),
                                                       ("代码块里的示例行", "线程上限：{n}\n喂法示例：\n```\n线程上限：{n}（本机）\n```", 0)):
            thread_path = os.path.join(work, f"thread-note-{thread_want}-{len(thread_label)}.md")
            filled = fill_for_guard(skeleton("tooling-writer", agents, {}, None)).replace(f"线程上限：{registered_threads}", thread_text.format(n=registered_threads))
            open(thread_path, "w", encoding="utf-8").write(filled)
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code_thread_note = check_prompt(thread_path, root=work)
            checked += 1
            if code_thread_note != thread_want or (thread_want == 3 and "括注一概不认" not in buffer.getvalue()):
                failures.append(f"工具员骨架的线程上限一行{thread_label}，--check 应当退 {thread_want}（括注一概拒、说明另起一行不判），实际 {code_thread_note}：{buffer.getvalue()[-600:]}")
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 skeleton()、check_prompt() 与派发闸的 --check-prompt；DISPATCH_PROMPT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {checked} 项）：有定义的类型按 required-inputs 逐组一行、--set 填进去；没有定义的带开工先读；实现员带要动的文件与线程上限；"
          "每种类型都带重型测试行；设计员与带重活的九种类型带预计挂钟与分片、续跑、多机四行（多机一行取登记值、不写现算），别的类型不带；骨架的类型表与派发闸的表逐个相同，九种类型的真定义骨架填完过派发闸；"
          f"会跑自证的 {len(SELFTEST_RUNNER_TYPES)} 种类型（{'、'.join(SELFTEST_RUNNER_TYPES)}）带「自证在哪跑」一行、别的不带、--set 填得进；"
          "带草稿目录的类型在草稿目录那一行后多一行认领（--set 草稿目录、认领用途填得进），--check 判认领行在不在、目录对得上、用途非空；"
          "线程上限一行默认填登记的每台默认重活线程数（--set 给了的用给的），该默认过派发闸 ⑮；"
          "--check 不知道类型时只查占位（有占位退 3、写明没给类型），"
          "知道类型时起派发闸的 --check-prompt、会拒的几处一次列全退 3（只留「自证在哪跑」没填的点名这一行），填完的退 0")
    return 0


def main(argv):
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
