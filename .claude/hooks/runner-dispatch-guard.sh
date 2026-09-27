#!/usr/bin/env bash
# PreToolUse hook（Agent）：派发那一刻查派发提示齐不齐、写得对不对，缺一样当场拒（退出 2，stderr 写原因与出路）。
# hook-events: PreToolUse:Agent PreToolUse:Task
# admission: always Claude Code 在主 agent 每次派发之前调它，判的是这一次的派发提示与此刻会话里在跑的子 agent
# run-condition: command python3
#
# 判法，按次序，第一条不成立就拒：
#   ① 重型测试一行：派 crash-verifier、gate-triage 之外的任何类型，提示里要有一行「重型测试：不跑」（行首可带 - * > 与粗体）。
#      提示正文里提到重型阶段的句子不再逐句判；执行时 heavy-test-guard.sh 与看门狗的进程一层照旧兜底。
#   ② 不读共用约束的类型（general-purpose，或不给 subagent_type）：提示里要有「开工先读」与 `.claude/agent-common.md`。
#   ③ 输入齐不齐：`.claude/agents/<类型>.md` 的 frontmatter 有 `required-inputs:` 的，逐组查——组之间逗号分隔，组内 `|` 分隔的同义词在提示里出现一个就算；
#      定义不在、没写这一行的不查。
#   ④ 写范围：类型在 `.claude/hooks/agent-write-scope.tsv` 里的，提示里「报告路径 / 报告写进 / 写进 / 写到 / 更新」后面紧跟的路径
#      （/tmp/claude-<uid>/、research/、.claude/、crates/、litmus/、records/ 起头）逐个按它那几行模式判，不在里面就拒；前面六个字以内有否定词的不判。
#   ⑤ 实现员：要有一行「要动的 crates 文件：…」，里面的 crates/ 路径不超过 IMPLEMENTATION_WRITER_FILE_LIMIT 个（8，推的；拆不开的交用户定，放行行「文件上限已判：<理由>」）；
#      与本会话还在跑的实现员登记的文件有交集就拒。登记在 <DISPATCH_GUARD_STATE_ROOT>/<会话 id>/implementation-writers.tsv，一行一次派发
#      （时刻、提示的 sha256、文件）；每次判之前先删掉已结束的：主会话记录里那次派发（toolUseResult 的 prompt 同 sha256）的 agent
#      交回了（`<agent-message from="<id>">` 后跟 `[Subagent hand-back]`）、任务通知是 failed / killed / stopped、或被 TaskStop 停了；
#      派发之后 10 分钟主会话记录里还没有它的，按没派出去删。
#   ⑥ 快照冲突：派 kb-scribe、implementation-writer 时要改的文件在还没判完的三方轮开工快照里，拒（判法见 snapshot_conflict_verdict 上面那段）。
#   ⑦ 模型：新派的是 opus（tool_input.model，没给取定义的 model；inherit、没有定义的按 opus 算，推的）时，本会话还在跑的 opus 子 agent
#      （主会话记录里 resolvedModel 含 opus、没结束、会话记录 3 小时内动过或 3 小时内派的）已有 OPUS_CONCURRENCY_LIMIT 个（8，用户 2026-09-27 定）就拒；
#      放行行「opus 并发已判：<理由>」。另外，主会话记录里 8 天内有 failed 任务通知写着「resets <时刻> (UTC)」、它的
#      「model sent to the API」与新派的同一族（opus / sonnet / haiku），而现在还没到那个时刻，拒；那次失败之后同一族又派出去、没再撞限额的
#      算窗口已解开（换了账号或提前恢复），不拒；放行行「限额窗口已判：<理由>」。
#   ⑧ 书记员：提示里点名的 /tmp/claude-<uid>/ 下规格文件逐份交给 research/scripts/kb-spec-check.py，红了拒；一份认得出的规格都没有也拒
#      （规格写在提示正文里的，把提示正文当规格判）；放行行「规格检查已判：<理由>」。
#   ⑨ 设计员写重跑登记（提示里有 research/prompts/e<号>-r<n>-prereg.md）：`research/scripts/admission.py experiment <根> E<号>` 退 77
#      （输入自上次产物以来没变）就拒；没登记的实验（退 2）放行；放行行「准入已判：<理由>」。
#   ⑩ 变异分诊员：提示里要有 `research/mutations/…tsv` 或 `crates/mutations.tsv`——不接没有表的广谱活动。
#   ⑪ 执行员：提示里写着「只修锚点」或「只补落产物」的放行这一条；否则要有「这一段回答的岔路：<非空>」；登记对应的实验已经有实验页的算续做，
#      还要有「上一段岔路表里还差：<非空>」，写「无 / 没有」的拒（一行都不差就该交岔路表，不续派）。
# 放行行都写在行首（可带 - * > 与粗体），冒号后面是理由，理由不能空、不能是占位（<理由>）。
# 这一道自己出错（读不了表、主会话记录、规格，JSON 解析不了、脚本跑不起来）不拒：放行，错误写进 stderr。
#
#   runner-dispatch-guard.sh             # 从 stdin 读 hook 的 JSON
#   runner-dispatch-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝；RUNNER_DISPATCH_GUARD_DISABLE_CHECK=1（全关）、
#                                        # RUNNER_DISPATCH_GUARD_DISABLE_HEAVY_TEST=1（只关重型测试一行）或 RUNNER_DISPATCH_GUARD_DISABLE_SNAPSHOT=1（只关快照）时
#                                        # 自检必须判红；RUNNER_DISPATCH_GUARD_BREAK=snapshot-no-normalize / snapshot-spec-unread /
#                                        # snapshot-closed-round-open / snapshot-body-optional / snapshot-valve-ignored / snapshot-valve-loose /
#                                        # snapshot-any-agent / snapshot-loose-lines / snapshot-no-edges / snapshot-unreadable-denies /
#                                        # snapshot-unreadable-aborts / snapshot-missing-spec-denies / json-error-silent / json-error-denies /
#                                        # inputs-ignored / general-purpose-free / scope-ignored / scope-negation-ignored / crates-no-limit /
#                                        # registry-ignored / registry-no-prune / opus-no-limit / window-ignored / spec-check-skipped /
#                                        # admission-ignored / mutation-table-free / runner-backfill-refused / valve-loose-all / window-never-lifted 各自也必须让它红
# gate-similar: heavy-test-guard.sh 管同一条「子 agent 不跑重型测试」，但挂 Bash、在执行那一刻判真要跑的命令与它执行的脚本；这里只在派发那一刻查一行结构化声明，判法没有能共用的
# gate-similar: continuation-guard.sh 同样读会话记录认子 agent 交回没有，但它挂 SendMessage、按子 agent 自己的会话记录判收件人；这里挂 Agent|Task、按主会话记录里的派发回执、交回消息、任务通知与 TaskStop 判在跑的实现员与 opus 数，读的记录种类不同
# gate-similar: ask-user-claim-guard.sh 也判一段给人看的中文，但挂 AskUserQuestion、判断言词加出处；这里判的是固定标签行与路径，不逐句理解
# gate-similar: write-guard.sh 读同一张写范围表，但在写的那一刻判目标路径；这里在派发那一刻判提示里点名的路径，模式匹配两边各一份写法（它逐字符拼正则，这里按 ** 与 * 切段）
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$HOOK_DIR/../singlefs-ai-sop/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import glob, hashlib, json, os, re, shutil, subprocess, sys, tempfile, time
from datetime import datetime, timedelta, timezone

NONE_WORDS = {"", "无", "没有", "无。", "-", "—", "空", "none"}

def labelled_value(prompt, label):
    match = re.search(label + r"[：:]\s*(.*)", prompt)
    return None if match is None else match.group(1).strip()


HEAVY_TEST_OWNERS = {"crash-verifier", "gate-triage"}
LINE_START = r"^[ \t>*_\-]*"
HEAVY_TEST_LINE = re.compile(LINE_START + r"重型测试[* \t]*[：:][* \t]*不跑", re.M)
COMMON_CONSTRAINTS_READ = re.compile(r"开工先读[^\n]*\.claude/agent-common\.md")
IMPLEMENTATION_WRITER_FILE_LIMIT = 8        # 推的，没量过：965k 那几次实现员各动了 8 个以上 crates 文件，倒推的上限
OPUS_CONCURRENCY_LIMIT = 8                  # 用户 2026-09-27 JST 02:2x 定（弹窗原话「8」）
INHERITED_MODEL_FAMILY = "opus"             # inherit 与没有定义的类型按主 agent 的模型算；这个项目的主 agent 跑 opus（推的）
RUNNING_RECENT_SECONDS = 3 * 3600
UNLAUNCHED_GRACE_SECONDS = 600
LIMIT_WINDOW_LOOKBACK_SECONDS = 8 * 86400
CRATES_FILES_LINE = re.compile(LINE_START + r"要动的\s*crates\s*文件[* \t]*[：:](.*)$", re.M)
CRATES_PATH = re.compile(r"crates/[^\s`'\"，。；：、（）()「」,;]+")
SCOPE_MENTION = re.compile(r"(?:报告路径|报告写进|报告写到|写进|写到|更新)[*\s]*[：:]?[*\s]*`?(?P<path>(?:/tmp/claude-\d+/|research/|\.claude/|crates/|litmus/|records/)[^\s`'\"，。；：、（）()「」,;]*)")
SCOPE_NEGATION = re.compile(r"不|别|禁止|勿")
MUTATION_TABLE_MENTION = re.compile(r"research/mutations/[^\s`'\"]+\.tsv|crates/mutations\.tsv")
RERUN_REGISTRATION = re.compile(r"research/prompts/e(\d+)-r\d+-prereg\.md")
RESETS = re.compile(r"resets (?:(?P<month>[A-Z][a-z]{2}) (?P<day>\d{1,2}),? )?(?P<hour>\d{1,2})(?::(?P<minute>\d{2}))?(?P<half>am|pm) \(UTC\)")
MODEL_SENT = re.compile(r"model sent to the API: ([\w.\-\[\]]+)")
HANDBACK_FROM = re.compile(r'agent-message from=\\?"(a[0-9a-f]{6,})\\?">(?:\\n|\s)*\[Subagent hand-back\]')
NOTIFICATION_BLOCK = re.compile(r"<task-notification>(.*?)</task-notification>", re.S)
MONTHS = {name: index for index, name in enumerate(["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"], 1)}


def valve_reason(prompt, label):
    """行首「<label>：<理由>」的理由；没有、空、占位都返回 None。"""
    match = re.search(LINE_START + re.escape(label) + r"[* \t]*[：:][* \t]*(.*?)[ \t]*$", prompt, re.M)
    if match is None or break_switch_is("valve-loose-all"):
        return None
    reason = match.group(1).strip()
    return None if not reason or reason.startswith("<") or reason in NONE_WORDS else reason


def agent_definition_fields(project_root, subagent_type):
    """定义的 frontmatter → {键: 值}（值去掉行尾 # 注释）；定义不在返回 None。"""
    path = os.path.join(project_root, ".claude", "agents", f"{subagent_type}.md")
    try:
        text = open(path, encoding="utf-8").read()
    except OSError:
        return None
    fields = {}
    if text.startswith("---"):
        for line in text.split("\n---", 1)[0].splitlines()[1:]:
            if ":" in line:
                key, value = line.split(":", 1)
                fields[key.strip()] = value.split("#", 1)[0].strip()
    return fields


def missing_required_inputs(fields, prompt):
    if break_switch_is("inputs-ignored"):
        return []
    groups = [group.strip() for group in (fields.get("required-inputs") or "").split(",") if group.strip()]
    return [group for group in groups if not any(alternative.strip() and alternative.strip() in prompt for alternative in group.split("|"))]


def scope_pattern_regex(pattern):
    pieces = re.split(r"(\*\*|\*)", pattern)
    return re.compile("".join(".*" if piece == "**" else "[^/]*" if piece == "*" else re.escape(piece) for piece in pieces) + r"\Z")


def scope_patterns(project_root, subagent_type):
    patterns = []
    table_path = os.path.join(project_root, ".claude", "hooks", "agent-write-scope.tsv")
    if not os.path.isfile(table_path):
        return patterns
    try:
        for line in open(table_path, encoding="utf-8"):
            fields = line.rstrip("\n").split("\t")
            if not line.startswith("#") and len(fields) >= 2 and fields[0].strip() == subagent_type:
                patterns.append(fields[1].strip())
    except OSError as error:
        print(f"! 派发闸读不了写范围表，写范围这一条没判：{error}", file=sys.stderr)
    return patterns


def out_of_scope_mentions(project_root, subagent_type, prompt):
    patterns = scope_patterns(project_root, subagent_type)
    if not patterns or break_switch_is("scope-ignored"):
        return []
    outside = []
    for match in SCOPE_MENTION.finditer(prompt):
        before = prompt[max(0, match.start() - 6):match.start()]
        if SCOPE_NEGATION.search(before) and not break_switch_is("scope-negation-ignored"):
            continue
        path = match.group("path").rstrip("/.")
        root_prefix = os.path.normpath(project_root) + "/"
        relative = path[len(root_prefix):] if path.startswith(root_prefix) else path
        if not any(scope_pattern_regex(pattern).match(relative) for pattern in patterns) and relative not in outside:
            outside.append(relative)
    return outside


def model_family(model_text):
    lowered = (model_text or "").lower()
    for family in ("opus", "sonnet", "haiku", "fable"):
        if family in lowered:
            return family
    return INHERITED_MODEL_FAMILY if lowered in ("", "inherit") else lowered


def parse_moment(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00")).timestamp()


def session_dispatch_events(transcript_path):
    """主会话记录 → (launches{agent 号: (派发时刻, 模型, 提示 sha256)}, ended{agent 号: 最晚结束时刻}, windows[(到期时刻, 模型族, 原句)])。
    先用 grep 挑出带这几种记号的行，免得逐行解析上百兆的记录；读不了返回三个空的。"""
    launches, ended, windows = {}, {}, []
    if not transcript_path or not os.path.isfile(transcript_path):
        return launches, ended, windows
    needles = ['"async_launched"', "[Subagent hand-back]", "<task-notification>", "Successfully stopped task"]
    try:
        grep = subprocess.run(["grep", "-F", *sum((["-e", needle] for needle in needles), []), transcript_path],
                              capture_output=True, text=True, timeout=50)
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"! 派发闸读不了主会话记录，按会话判的几条没判：{error}", file=sys.stderr)
        return launches, ended, windows
    now = time.time()
    for line in grep.stdout.splitlines():
        try:
            record = json.loads(line)
            moment = parse_moment(record.get("timestamp") or "")
        except (ValueError, TypeError):
            continue
        result = record.get("toolUseResult")
        if isinstance(result, dict) and result.get("status") == "async_launched" and result.get("agentId"):
            digest = hashlib.sha256(str(result.get("prompt") or "").encode("utf-8")).hexdigest()
            launches[result["agentId"]] = (moment, result.get("resolvedModel") or "", digest)
        if isinstance(result, dict) and "Successfully stopped task" in str(result.get("message", "")) and result.get("task_id"):
            ended[result["task_id"]] = max(moment, ended.get(result["task_id"], 0))
        for agent_id in HANDBACK_FROM.findall(line):
            ended[agent_id] = max(moment, ended.get(agent_id, 0))
        for block in NOTIFICATION_BLOCK.findall(line.replace("\\n", "\n").replace('\\"', '"')):
            status = re.search(r"<status>(\w+)</status>", block)
            for agent_id in re.findall(r"<task-id>(\w+)</task-id>", block):
                if status and status.group(1) in ("failed", "killed", "stopped"):
                    ended[agent_id] = max(moment, ended.get(agent_id, 0))
            resets, sent = RESETS.search(block), MODEL_SENT.search(block)
            if status and status.group(1) == "failed" and resets and now - moment <= LIMIT_WINDOW_LOOKBACK_SECONDS:
                windows.append((reset_moment(resets, moment), model_family(sent.group(1) if sent else ""), resets.group(0), moment,
                                re.findall(r"<task-id>(\w+)</task-id>", block)))
    return launches, ended, windows


def reset_moment(match, notified_at):
    """「resets 3am (UTC)」→ 通知之后第一次到那个时刻；写了日期的取那一天（早于通知就算明年）。"""
    hour = int(match.group("hour")) % 12 + (12 if match.group("half") == "pm" else 0)
    minute = int(match.group("minute") or 0)
    base = datetime.fromtimestamp(notified_at, timezone.utc)
    if match.group("month"):
        candidate = base.replace(month=MONTHS.get(match.group("month"), base.month), day=int(match.group("day")), hour=hour, minute=minute, second=0, microsecond=0)
        if candidate.timestamp() < notified_at:
            candidate = candidate.replace(year=candidate.year + 1)
        return candidate.timestamp()
    candidate = base.replace(hour=hour, minute=minute, second=0, microsecond=0)
    if candidate.timestamp() <= notified_at:
        candidate = datetime.fromtimestamp(candidate.timestamp() + 86400, timezone.utc)
    return candidate.timestamp()


def is_running(agent_id, launches, ended, transcript_path):
    launched_at = launches[agent_id][0]
    if ended.get(agent_id, 0) >= launched_at:
        return False
    own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl") if transcript_path.endswith(".jsonl") else ""
    last_seen = max(launched_at, os.path.getmtime(own) if own and os.path.isfile(own) else 0)
    return time.time() - last_seen <= RUNNING_RECENT_SECONDS


def registry_path(hook_input):
    root = os.environ.get("DISPATCH_GUARD_STATE_ROOT") or f"/tmp/claude-{os.getuid()}/dispatch-guard-state"
    session = re.sub(r"[^\w.-]", "_", str(hook_input.get("session_id") or "unknown-session"))
    return os.path.join(root, session, "implementation-writers.tsv")


def live_registry_rows(hook_input, launches, ended):
    """登记里还在跑的那几行（顺手把结束了的、派发之后没出现的删掉）。→ [(时刻, 提示 sha256, [文件])]。"""
    path = registry_path(hook_input)
    try:
        rows = [line.rstrip("\n").split("\t") for line in open(path, encoding="utf-8") if line.strip()]
    except OSError:
        return []
    by_digest = {digest: agent_id for agent_id, (_, _, digest) in launches.items()}
    transcript_path = hook_input.get("transcript_path") or ""
    live = []
    for row in rows:
        if len(row) != 3:
            continue
        registered_at, digest, files = float(row[0]), row[1], row[2].split()
        agent_id = by_digest.get(digest)
        if not break_switch_is("registry-no-prune"):
            if agent_id is None and time.time() - registered_at > UNLAUNCHED_GRACE_SECONDS:
                continue
            if agent_id is not None and not is_running(agent_id, launches, ended, transcript_path):
                continue
        live.append((registered_at, digest, files))
    rewrite_registry(path, live)
    return live


def rewrite_registry(path, rows):
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        temporary = f"{path}.{os.getpid()}.tmp"
        with open(temporary, "w", encoding="utf-8") as handle:
            for registered_at, digest, files in rows:
                handle.write(f"{registered_at}\t{digest}\t{' '.join(files)}\n")
        os.replace(temporary, path)
    except OSError as error:
        print(f"! 派发闸写不了实现员登记 {path}：{error}", file=sys.stderr)


def register_implementation_writer(hook_input, prompt, files, live_rows):
    digest = hashlib.sha256(prompt.encode("utf-8")).hexdigest()
    rewrite_registry(registry_path(hook_input), live_rows + [(time.time(), digest, files)])


def implementation_writer_verdict(hook_input, prompt, launches, ended):
    """→ (拒绝说明或 None, 要登记的文件, 还在跑的登记行)。"""
    line = CRATES_FILES_LINE.search(prompt)
    if line is None:
        return ("✗ 派 implementation-writer 的提示里没有「要动的 crates 文件：…」一行。\n"
                "→ 怎么办：把这一件要动的 crates/ 文件逐个写进这一行（路径从仓根起）；说不全要动哪些文件，这件活还没切小，先切。"), [], []
    files = sorted(set(CRATES_PATH.findall(line.group(1))))
    if (len(files) > IMPLEMENTATION_WRITER_FILE_LIMIT and not break_switch_is("crates-no-limit")
            and valve_reason(prompt, "文件上限已判") is None):
        return (f"✗ 这一件要动 {len(files)} 个 crates 文件，超过 {IMPLEMENTATION_WRITER_FILE_LIMIT} 个（推的上限）。\n"
                "→ 怎么办：拆成几件分开派，一件不超过上限；拆不开的交用户定，定了在提示里写一行「文件上限已判：<用户怎么定的>」。"), files, []
    live = live_registry_rows(hook_input, launches, ended)
    if break_switch_is("registry-ignored"):
        return None, files, live
    overlapping = sorted({path for _, _, registered in live for path in registered} & set(files))
    if overlapping:
        return (f"✗ 这一件要动的文件与还在跑的实现员撞了：{'、'.join(overlapping)}。\n"
                "→ 怎么办：排在那个实现员交回之后再派，或把两件并给同一个实现员（.claude/main-agent.md「一轮怎么开、怎么收」第 4 条）。"), files, live
    return None, files, live


def model_verdict(hook_input, prompt, subagent_type, fields, launches, ended, windows):
    requested = (hook_input.get("tool_input") or {}).get("model") or (fields or {}).get("model") or ""
    family = model_family(requested)
    transcript_path = hook_input.get("transcript_path") or ""
    now = time.time()
    limit_failed = {agent_id for window in windows for agent_id in window[4]}
    for until, window_family, original, notified_at, _ in windows:
        # 撞限额之后同一族又派出去、而且没再撞限额的：窗口已经解开了（换了账号或限额提前恢复）
        lifted = any(model_family(model) == window_family and launched_at > notified_at and agent_id not in limit_failed
                     for agent_id, (launched_at, model, _) in launches.items()) and not break_switch_is("window-never-lifted")
        if (window_family == family and now < until and not lifted and not break_switch_is("window-ignored")
                and valve_reason(prompt, "限额窗口已判") is None):
            tokyo = datetime.fromtimestamp(until, timezone(timedelta(hours=9))).strftime("%m-%d %H:%M")
            return (f"✗ {family} 这一族撞过限额（「{original}」），到 {tokyo} JST 之前不派同一族的 agent。\n"
                    f"→ 怎么办：等到 {tokyo} JST，或换模型（派发参数 model 写别的一族）——换不换交用户定；用户定了照派，提示里写一行「限额窗口已判：<用户怎么定的>」。")
    if family != "opus" or break_switch_is("opus-no-limit") or valve_reason(prompt, "opus 并发已判") is not None:
        return None
    running = [agent_id for agent_id in launches if model_family(launches[agent_id][1]) == "opus" and is_running(agent_id, launches, ended, transcript_path)]
    if len(running) >= OPUS_CONCURRENCY_LIMIT:
        return (f"✗ 本会话已有 {len(running)} 个 opus 子 agent 在跑（{'、'.join(running[:10])}），上限 {OPUS_CONCURRENCY_LIMIT}（用户 2026-09-27 定）。\n"
                "→ 怎么办：等其中一个交回再派，或这一件换 sonnet（派发参数 model）；确要超过就交用户定，定了在提示里写一行「opus 并发已判：<用户怎么定的>」。")
    return None


def kb_scribe_spec_verdict(prompt, project_root, hook_dir):
    if valve_reason(prompt, "规格检查已判") is not None or break_switch_is("spec-check-skipped"):
        return None
    checker = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "research", "scripts", "kb-spec-check.py")
    candidates = []
    for mention in sorted(set(SCRATCH_FILE_MENTION.findall(prompt))):
        path = mention.rstrip("。，；：、.")
        if os.path.isfile(path) and path.endswith((".json", ".md", ".txt")):
            candidates.append(path)
    inline = None
    if not candidates:
        inline = tempfile.NamedTemporaryFile("w", suffix=".md", delete=False, encoding="utf-8")
        inline.write(prompt)
        inline.close()
        candidates = [inline.name]
    try:
        recognized, problems = 0, []
        for path in candidates:
            try:
                result = subprocess.run([sys.executable, checker, path, "--root", project_root], capture_output=True, text=True, timeout=50)
            except (OSError, subprocess.TimeoutExpired) as error:
                print(f"! 派发闸跑不了 kb-spec-check.py，规格这一条没判：{error}", file=sys.stderr)
                return None
            if result.returncode in (0, 1):
                recognized += 1
            if result.returncode == 1:
                problems += [line.strip() for line in result.stdout.splitlines() if line.strip().startswith("✗")]
            elif result.returncode not in (0, 2):
                print(f"! kb-spec-check.py 退出码 {result.returncode}，规格这一条没判：{result.stdout[-200:]}{result.stderr[-200:]}", file=sys.stderr)
                return None
        if problems:
            return ("✗ 给书记员的规格会让门禁红（research/scripts/kb-spec-check.py）：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                    "\n→ 怎么办：改规格原文再派；判断不了的交用户；要 kb-spec-drafter 起草就先派它。确要照这份派，提示里写一行「规格检查已判：<理由>」。")
        if recognized == 0:
            return ("✗ 派 kb-scribe 的提示里认不出一份规格（提示正文与点名的 /tmp/claude-<uid>/ 下文件都不是 kb-spec-check.py 认的写法）。\n"
                    "→ 怎么办：照 research/scripts/kb-spec-check.py 文件头「规格的两种写法」写，或派 kb-spec-drafter 起草；确要照原样派，提示里写一行「规格检查已判：<理由>」。")
        return None
    finally:
        if inline is not None:
            os.unlink(inline.name)


def designer_admission_verdict(prompt, project_root, hook_dir):
    match = RERUN_REGISTRATION.search(prompt)
    if match is None or break_switch_is("admission-ignored") or valve_reason(prompt, "准入已判") is not None:
        return None
    admission = (os.environ.get("RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT")
                 or os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "research", "scripts", "admission.py"))
    try:
        result = subprocess.run([sys.executable, admission, "experiment", project_root, f"E{int(match.group(1))}"], capture_output=True, text=True, timeout=50)
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"! 派发闸跑不了 admission.py，准入这一条没判：{error}", file=sys.stderr)
        return None
    if result.returncode == 77:
        return (f"✗ E{int(match.group(1))} 的输入自上次产物以来没变（research/scripts/admission.py experiment 退 77），不写它的重跑登记。\n"
                "→ 怎么办：沿用上次产物与结论；要为别的原因重跑（换臂、换判据），在提示里写一行「准入已判：<哪一样变了>」。")
    return None

SNAPSHOT_GUARDED_AGENTS = {"kb-scribe", "implementation-writer"}
SNAPSHOT_DIRECTORY_SUFFIX = "-snapshot"
SNAPSHOT_LIST_NAME_MARK = "sha256"
# 快照清单的一行：64 位十六进制哈希、空白、路径（sha256sum 二进制模式在路径前加 *）
SNAPSHOT_LIST_LINE = re.compile(r"^\s*[0-9a-fA-F]{64}\s+\*?(\S.*?)\s*$")
# 清单里的路径从第一段 .claude/ 或 crates/ 起取（./.claude/…、tree/crates/…、defs/.claude/… 都归一成仓库根相对路径）
REPOSITORY_PATH_START = re.compile(r"(?:^|/)((?:\.claude|crates)/)")
# 一条路径在文字里「出现」：前后不连着路径字符（后面的 . 只在它后面还跟着路径字符时才算连着，句末的点不算）
PATH_LEFT_EDGE = r"(?<![A-Za-z0-9_\-./~])"
PATH_RIGHT_EDGE = r"(?![A-Za-z0-9_\-/]|\.[A-Za-z0-9_])"
DOT_SLASH_BEFORE_REPOSITORY_PATH = re.compile(PATH_LEFT_EDGE + r"\./(?=\.claude/|crates/)")
# 提示里点名的规格文件：/tmp/claude-<uid>/ 下的路径，到空白、引号、括号或中文标点为止
SCRATCH_FILE_MENTION = re.compile(r"/tmp/claude-\d+/[^\s`'\"，。；：、（）()「」『』“”‘’《》<>\[\]{}|,;*]+")
SPECIFICATION_READ_LIMIT_BYTES = 4 * 1024 * 1024
# 放行口：一行以「快照冲突已判：<轮名> <理由>」开头（行首可以有列表记号与粗体星号）；只在这一行里找，不跨到下一行
SNAPSHOT_CONFLICT_RULED_LINE = re.compile(r"^[ \t>*_\-]*快照冲突已判[* \t]*[：:][* \t]*([^\s，,：:；;。]+)[ \t，,：:；;。]+(.*?)[ \t]*$", re.M)
SNAPSHOT_CONFLICT_RULED_ANYWHERE = re.compile(r"快照冲突已判[*\s]*[：:][*\s]*([^\s，,：:；;。]*)")
SNAPSHOT_CONFLICTS_SHOWN_LIMIT = 20

def break_switch_is(name):
    """自检用的弄坏开关：RUNNER_DISPATCH_GUARD_BREAK 等于 name 时，那一处判法故意走错，自检必须判红。"""
    return os.environ.get("RUNNER_DISPATCH_GUARD_BREAK") == name

def open_three_way_rounds(project_root):
    """还没判完的三方轮：[(轮名, 快照目录)]。research/prompts/<轮>-snapshot/ 在、_<轮>-body.md 在、<轮>-main-verification.md 不在。"""
    prompts_directory = os.path.join(project_root, "research", "prompts")
    if not os.path.isdir(prompts_directory):
        return []
    rounds = []
    for entry_name in sorted(os.listdir(prompts_directory)):
        snapshot_directory = os.path.join(prompts_directory, entry_name)
        if not entry_name.endswith(SNAPSHOT_DIRECTORY_SUFFIX) or not os.path.isdir(snapshot_directory):
            continue
        round_name = entry_name[:-len(SNAPSHOT_DIRECTORY_SUFFIX)]
        has_body = os.path.isfile(os.path.join(prompts_directory, f"_{round_name}-body.md"))
        has_verdict = os.path.exists(os.path.join(prompts_directory, f"{round_name}-main-verification.md"))
        if round_name and (has_body or break_switch_is("snapshot-body-optional")) and (not has_verdict or break_switch_is("snapshot-closed-round-open")):
            rounds.append((round_name, snapshot_directory))
    return rounds

def repository_path_of_snapshot_line(line):
    """快照清单的一行归一成的仓库根相对路径；不是「<哈希>  <路径>」的形状、或路径里没有 .claude/ 与 crates/ 起头的一段，返回 None。"""
    if break_switch_is("snapshot-loose-lines"):
        fields = line.split()
        return fields[-1] if fields else None
    match = SNAPSHOT_LIST_LINE.match(line)
    if match is None:
        return None
    listed_path = match.group(1)
    if break_switch_is("snapshot-no-normalize"):
        return listed_path
    start = REPOSITORY_PATH_START.search(listed_path)
    if start is None:
        return None
    normalized = os.path.normpath(listed_path[start.start(1):])
    return None if normalized.startswith("..") else normalized

def snapshot_listed_paths(snapshot_directory, notes):
    """快照目录里文件名含 sha256 的每份清单读出来的路径：{仓库根相对路径: 清单文件}。认不出的行跳过；读不了的清单记进 notes、跳过。"""
    listed = {}
    for list_name in sorted(os.listdir(snapshot_directory)):
        if SNAPSHOT_LIST_NAME_MARK not in list_name:
            continue
        list_path = os.path.join(snapshot_directory, list_name)
        try:
            with open(list_path, encoding="utf-8", errors="replace") as list_file:
                lines = list_file.read().splitlines()
        except OSError as error:
            if break_switch_is("snapshot-unreadable-aborts"):
                raise
            if break_switch_is("snapshot-unreadable-denies"):
                listed[f"<读不了的清单 {list_name}>"] = list_path
            notes.append(f"! 派发闸（快照那一条）读不了快照清单 {list_path}：{error}；这一份跳过，没当成撞快照")
            continue
        for line in lines:
            repository_path = repository_path_of_snapshot_line(line)
            if repository_path is not None:
                listed.setdefault(repository_path, list_path)
    return listed

def specification_texts(prompt, notes):
    """提示里点名的 /tmp/claude-<uid>/ 下的规格文件：[(路径, 内容)]。不在的、是目录的跳过；在而读不了的记进 notes、跳过。"""
    texts = []
    seen = set()
    for mention in SCRATCH_FILE_MENTION.finditer(prompt):
        candidate = mention.group().rstrip(".:")
        if not os.path.isfile(candidate):
            # 路径后面紧跟中文（「spec.md里」）：去掉尾巴上的非 ASCII 字再看一次
            candidate = re.sub(r"[^\x00-\x7f]+$", "", candidate).rstrip(".:")
        if candidate in seen:
            continue
        seen.add(candidate)
        if not os.path.isfile(candidate):
            if break_switch_is("snapshot-missing-spec-denies") and not os.path.isdir(candidate):
                texts.append((candidate, "<不在的规格>"))
            continue
        try:
            with open(candidate, "rb") as specification_file:
                texts.append((candidate, specification_file.read(SPECIFICATION_READ_LIMIT_BYTES).decode("utf-8", errors="replace")))
        except OSError as error:
            notes.append(f"! 派发闸（快照那一条）读不了派发提示点名的规格 {candidate}：{error}；这一份没收进要改的文件")
    return texts

def mentioned_repository_paths(text, candidate_paths, project_root):
    """candidate_paths 里哪几条在 text 里出现：前后不连着路径字符；项目根起的绝对路径与 ./ 前缀先去掉。"""
    root_prefix = project_root.rstrip("/")
    if root_prefix:
        text = text.replace(root_prefix + "/", "")
    text = DOT_SLASH_BEFORE_REPOSITORY_PATH.sub("", text)
    if break_switch_is("snapshot-no-edges"):
        return {path for path in candidate_paths if path in text}
    return {path for path in candidate_paths if re.search(PATH_LEFT_EDGE + re.escape(path) + PATH_RIGHT_EDGE, text)}

def ruled_snapshot_conflicts(prompt):
    """派发提示里写了「快照冲突已判：<轮名> <理由>」的轮名（理由非空、不是 <理由> 这种占位）。"""
    if break_switch_is("snapshot-valve-ignored"):
        return set()
    if break_switch_is("snapshot-valve-loose"):
        return {"*"} if SNAPSHOT_CONFLICT_RULED_ANYWHERE.search(prompt) else set()
    ruled = set()
    for match in SNAPSHOT_CONFLICT_RULED_LINE.finditer(prompt):
        reason = match.group(2).strip()
        if reason not in NONE_WORDS and not (reason.startswith("<") and reason.endswith(">")):
            ruled.add(match.group(1))
    return ruled

def snapshot_conflict_verdict(subagent_type, prompt, project_root):
    """第三条。返回 (0, 给 stderr 的提醒或 None) 放行；(2, 说明) 拒绝。这一条自己出错一律放行，错误写进说明。"""
    if os.environ.get("RUNNER_DISPATCH_GUARD_DISABLE_SNAPSHOT") == "1":
        return 0, None
    if subagent_type not in SNAPSHOT_GUARDED_AGENTS and not break_switch_is("snapshot-any-agent"):
        return 0, None
    notes = []
    try:
        rounds = open_three_way_rounds(project_root)
        if not rounds:
            return 0, None
        sources = [("派发提示", prompt)]
        if not break_switch_is("snapshot-spec-unread"):
            sources += [(f"规格 {path}", text) for path, text in specification_texts(prompt, notes)]
        ruled = ruled_snapshot_conflicts(prompt)
        conflicts = []  # [(轮名, 快照目录, [(路径, 出现在哪, 清单文件)])]
        for round_name, snapshot_directory in rounds:
            listed = snapshot_listed_paths(snapshot_directory, notes)
            hits = {}
            for source_name, text in sources:
                for path in sorted(mentioned_repository_paths(text, listed, project_root)):
                    hits.setdefault(path, (source_name, listed[path]))
            if break_switch_is("snapshot-unreadable-denies"):
                hits.update({path: ("快照清单", list_path) for path, list_path in listed.items() if path.startswith("<读不了的清单")})
            if break_switch_is("snapshot-missing-spec-denies"):
                hits.update({f"<不在的规格 {path}>": ("派发提示", "-") for path, text in sources[1:] if text == "<不在的规格>"})
            if hits and round_name not in ruled and "*" not in ruled:
                conflicts.append((round_name, snapshot_directory, [(path, *hits[path]) for path in sorted(hits)]))
    except Exception as error:  # 挂在每一次派发上的闸，坏了不能把所有派发都挡住
        notes.append(f"! 派发闸（快照那一条）自己出错，这一次放行：{type(error).__name__}: {error}")
        return 0, "\n".join(notes)
    if not conflicts:
        return 0, ("\n".join(notes) or None)
    listing = []
    for round_name, snapshot_directory, hits in conflicts:
        listing.append(f"  三方轮 {round_name}（开工快照 {os.path.relpath(snapshot_directory, project_root)}/，"
                       f"research/prompts/{round_name}-main-verification.md 还不在）：")
        for path, source_name, list_path in hits[:SNAPSHOT_CONFLICTS_SHOWN_LIMIT]:
            listing.append(f"    {path}（出现在{source_name}；快照清单 {os.path.basename(list_path)}）")
        if len(hits) > SNAPSHOT_CONFLICTS_SHOWN_LIMIT:
            listing.append(f"    （另有 {len(hits) - SNAPSHOT_CONFLICTS_SHOWN_LIMIT} 个）")
    round_names = "、".join(round_name for round_name, _, _ in conflicts)
    valve_lines = "；".join(f"「快照冲突已判：{round_name} <理由>」" for round_name, _, _ in conflicts)
    return 2, (f"✗ 派 {subagent_type} 要改的文件落在还没判完的三方轮 {round_names} 的开工快照里：\n" + "\n".join(listing) + "\n"
               "→ 规矩：.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」：腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。\n"
               f"→ 怎么办：等 {round_names} 的判决写完再派，或把这几处改动并进判决；判过这次派发不碰那一轮的证据（例如只读不改），"
               f"在派发提示里单起一行写{valve_lines}（理由不许空）。"
               + ("\n" + "\n".join(notes) if notes else ""))

def decide(hook_input, project_root, hook_dir=None):
    """返回 (0, None 或给 stderr 的提醒) 放行；(2, 说明) 拒绝。"""
    if os.environ.get("RUNNER_DISPATCH_GUARD_DISABLE_CHECK") == "1":
        return 0, None
    hook_dir = hook_dir or os.path.join(project_root, ".claude", "hooks")
    tool_input = hook_input.get("tool_input") or {}
    subagent_type = tool_input.get("subagent_type") or "general-purpose"
    prompt = tool_input.get("prompt") or ""
    if (subagent_type not in HEAVY_TEST_OWNERS and os.environ.get("RUNNER_DISPATCH_GUARD_DISABLE_HEAVY_TEST") != "1"
            and not HEAVY_TEST_LINE.search(prompt)):
        return 2, (f"✗ 派 {subagent_type} 的提示里没有「重型测试：不跑」一行。\n"
                   "→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时、或用户要求时跑，"
                   "只由 crash-verifier（54、55、57、59 号）与 gate-triage（gate.sh 整轮与 87 号）跑；执行时 .claude/hooks/heavy-test-guard.sh 也拒。\n"
                   "→ 怎么办：提示里单独写一行「重型测试：不跑」；提交时的重阶段派 crash-verifier、整轮门禁派 gate-triage；提交之外确实要跑，先弹窗问用户，"
                   "用户同意了由主 agent 带 SINGLEFS_HEAVY_TESTS=user-request 跑。")
    fields = agent_definition_fields(project_root, subagent_type)
    if fields is None and not COMMON_CONSTRAINTS_READ.search(prompt) and not break_switch_is("general-purpose-free"):
        return 2, (f"✗ {subagent_type} 不是项目定义的 agent，不读共用约束，提示里却没写「开工先读 `.claude/agent-common.md`」。\n"
                   "→ 怎么办：先看 .claude/main-agent.md「什么时候派哪个 agent」那张表有没有对应的定义（改门禁、钩子、脚本、定义派 tooling-writer，"
                   "查真假与机理派 investigator）；确要派它，提示里写一行「开工先读 `.claude/agent-common.md`」，长活照那份的「长活可以等」写。")
    missing = missing_required_inputs(fields or {}, prompt)
    if missing:
        return 2, (f"✗ 派 {subagent_type} 的提示缺它定义要的输入：{'、'.join(missing)}（.claude/agents/{subagent_type}.md 的 required-inputs）。\n"
                   f"→ 怎么办：照那份定义「输入（主 agent 必须给）」一节补齐，每一样写出它的名字（同义词用 | 分隔，写一个就算）。")
    outside = out_of_scope_mentions(project_root, subagent_type, prompt)
    if outside:
        return 2, (f"✗ 提示要 {subagent_type} 写进它写范围之外的路径：{'、'.join(outside)}（.claude/hooks/agent-write-scope.tsv）。\n"
                   "→ 怎么办：改成它写范围里的路径（报告与草稿放 /tmp/claude-<uid>/ 下）；范围外的改动交该写的 agent 或主 agent 自己改。")
    snapshot_code, snapshot_message = snapshot_conflict_verdict(subagent_type, prompt, project_root)
    if snapshot_code:
        return snapshot_code, snapshot_message
    transcript_path = hook_input.get("transcript_path") or ""
    launches, ended, windows = session_dispatch_events(transcript_path)
    model_message = model_verdict(hook_input, prompt, subagent_type, fields, launches, ended, windows)
    if model_message:
        return 2, model_message
    if subagent_type == "kb-scribe":
        spec_message = kb_scribe_spec_verdict(prompt, project_root, hook_dir)
        if spec_message:
            return 2, spec_message
    if subagent_type == "experiment-designer":
        admission_message = designer_admission_verdict(prompt, project_root, hook_dir)
        if admission_message:
            return 2, admission_message
    if subagent_type == "mutation-triage" and not MUTATION_TABLE_MENTION.search(prompt) and not break_switch_is("mutation-table-free"):
        return 2, ("✗ 派 mutation-triage 的提示里没有变异表（research/mutations/<名>.tsv 或 crates/mutations.tsv 的条目名）。\n"
                   "→ 怎么办：给它表名与条目；没有表的广谱变异是实验，照 .claude/main-agent.md 那一行走 experiment-designer 登记。")
    if subagent_type == "implementation-writer":
        writer_message, files, live_rows = implementation_writer_verdict(hook_input, prompt, launches, ended)
        if writer_message:
            return 2, writer_message
        register_implementation_writer(hook_input, prompt, files, live_rows)
        return 0, snapshot_message
    if subagent_type != "experiment-runner":
        return 0, snapshot_message
    if "只修锚点" in prompt or ("只补落产物" in prompt and not break_switch_is("runner-backfill-refused")):
        return 0, None
    answered = labelled_value(prompt, "这一段回答的岔路")
    if answered is None or answered in NONE_WORDS:
        return 2, ("✗ 派 experiment-runner 的提示里没写「这一段回答的岔路：…」。\n"
                   "→ 怎么办：对着岔路单（research/prompts/<轮>-forks.md）写明这一段的量让哪几行够判；"
                   "说不出是哪一行，这一段就不该派（.claude/rules/three-way-inference.md「交岔路时写岔路单，派实验时带上它」）；"
                   "只补落产物、没有岔路可答的，写「只补落产物」。")
    number_match = re.search(r"research/prompts/e(\d+)-(?:preregistration|r\d+-prereg)\.md", prompt)
    if number_match is None:
        return 0, None
    pages = glob.glob(os.path.join(project_root, ".claude", "kb", "experiments", f"{int(number_match.group(1))}-*.md"))
    if not pages:
        return 0, None
    remaining = labelled_value(prompt, "上一段岔路表里还差")
    if remaining is None:
        return 2, (f"✗ E{number_match.group(1)} 已经有实验页，这是续做，派发提示里却没写「上一段岔路表里还差：…」。\n"
                   "→ 怎么办：先读上一段交回的岔路表，把还开着的行点名写进提示；一行都不差就停下交岔路表，不续派。")
    if remaining in NONE_WORDS:
        return 2, (f"✗ E{number_match.group(1)} 上一段岔路表里一行都不差了，不续派。\n"
                   "→ 怎么办：按岔路单交岔路表给用户；剩下的量在实验页标「够判后未跑」，状态写「已跑（够判）」。")
    return 0, None

# 2026-09-24 至 09-26 派发闸按旧的逐句判法拒掉的 14 句原句（主会话记录里拒绝消息引的那一句，逐字；截断处照原样）：有了「重型测试：不跑」一行都该放行
OLD_HEAVY_TEST_REFUSALS = [
    "**抓到的实例**：实现员把「库单测 → 动到的测试二进制 → 新层 0 七条流全量 → 变异」写进 `/tmp/claude-1000/impl-m2-treesplit/run-chain.sh`，再用后台 Bash 起它",
    "理由照实写：59 号是重型阶段，抽成共用库之后没法在提交之外跑它来证明没改坏",
    "理由照实写：59 号属于只在提交时才执行的重型阶段，改它的实现在提交之前没有办法证明没改坏，所以两边各留一份",
    "还有一处旧判法要一起看：实二四的报告 `research/prompts/m2-vm-raise-floor-implementer-report.md`「要你定的」第 3 条说，55 号还按「宿主只重跑第一个事务」判 `second-tr",
    "1. 「理由照实写：59 号属于只在提交时才执行的重型阶段，改它的实现在提交之前没有办法证明没改坏，所以两边各留一份」——这是在写一行说明的理由，没有要谁去跑",
    "- `diff <(bash .claude/scripts/check.sh) x`：括号里真在执行重型测试，对 implementation-writer 照拒",
    "跑门禁 59 号样本、73（gate-lint / shell-lint）、doc-lint、62、63，逐道报原样判定行",
    "前任有一次想在副本上跑全量测试被重型测试闸拒了——你只跑自己写的那几个测试二进制",
    "- **不许跑重型测试**：54 的 `--full`、55（QEMU）、57（herd7 真跑）、59（crates 变异表）、`gate.sh`、`cargo test --workspace` 都不许跑，钩子会拒",
    "- 59 号的红绿样本：重型测试闸按文件名拒绝子 agent 跑 59 号",
    "- **禁跑**：名字带 `layer0` 的测试目标、54 号",
    "- 撤回 F9：门禁分诊员定义里□那几处删掉，恢复直接起 `gate.sh`",
    "第二轮你这一侧的探针与复跑在 `research/prompts/m2-layer0-scale-r2-opus-model/`，可以接着用",
    "只在临时拷贝上跑 `admission.py`、54 号与 hook 的自证",
]

FOOTER_WITHOUT_HEAVY_LINE = "开工先读 `.claude/agent-common.md`\n规格检查已判：自检样本不带规格\n"
FOOTER = "重型测试：不跑\n" + FOOTER_WITHOUT_HEAVY_LINE

def selftest(hook_dir):
    work = tempfile.mkdtemp()
    other_directories = []  # 快照那一条另起的临时目录，finally 里一起删
    state_root = tempfile.mkdtemp()  # 实现员登记放自检自己的状态目录，不碰真会话的
    other_directories.append(state_root)
    os.environ["DISPATCH_GUARD_STATE_ROOT"] = state_root
    try:
        os.makedirs(os.path.join(work, ".claude", "kb", "experiments"))
        open(os.path.join(work, ".claude", "kb", "experiments", "153-样本.md"), "w").write("## E153 样本 —— 部分已跑\n")
        first = "跑前登记：research/prompts/e160-preregistration.md\n"
        again = "跑前登记：research/prompts/e153-preregistration.md\n"
        # 默认给每个样本提示补上「重型测试：不跑」、开工先读共用约束与规格检查放行行，实现员再补一行各不相同的要动文件：
        # 这样各条样本只判它自己要判的那一条；要判那几条本身的样本用 footer 换掉或关掉
        writer_case_counter = [0]
        def case_in(project_root, label, subagent_type, prompt, want, footer=True):
            footer_text = FOOTER if footer is True else (footer or "")
            text = prompt + ("\n" + footer_text if footer_text else "")
            if subagent_type == "implementation-writer" and footer_text and "要动的 crates 文件" not in text:
                writer_case_counter[0] += 1
                text += f"要动的 crates 文件：crates/selftest/src/case{writer_case_counter[0]}.rs\n"
            hook_input = {"tool_name": "Agent", "tool_input": {"subagent_type": subagent_type, "prompt": text}}
            return (label, want, decide(hook_input, project_root, hook_dir)[0])
        def case(label, subagent_type, prompt, want, footer=True):
            return case_in(work, label, subagent_type, prompt, want, footer)
        # 快照那一条的样本仓：r-open 开着（有正文、没判决）；r-closed 已判完；r-nobody 有快照没正文（普查一类）；r-odd 开着，清单里全是认不出的行
        def write_sample(project_root, relative_path, content):
            path = os.path.join(project_root, relative_path)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as sample_file:
                sample_file.write(content)
        digest = "0123456789abcdef" * 4
        write_sample(work, "research/prompts/_r-open-body.md", "# r-open 正文\n")
        write_sample(work, "research/prompts/r-open-snapshot/kb-sha256.txt",
                     f"{digest}  ./.claude/kb/checks-owed.md\n{digest}  ./.claude/kb/decisions/18-块里携带什么信息.md\n")
        write_sample(work, "research/prompts/r-open-snapshot/crates-sha256.txt",
                     f"{digest}  tree/crates/singlefs-core/src/mount.rs\n{digest}  defs/.claude/agents/crash-verifier.md\n")
        write_sample(work, "research/prompts/_r-closed-body.md", "# r-closed 正文\n")
        write_sample(work, "research/prompts/r-closed-snapshot/sha256sums.txt", f"{digest}  .claude/kb/decisions/03-空间分配.md\n")
        write_sample(work, "research/prompts/r-closed-main-verification.md", "# r-closed 判决\n")
        write_sample(work, "research/prompts/r-nobody-snapshot/sources.sha256", f"{digest}  crates/singlefs-core/src/allocator.rs\n")
        write_sample(work, "research/prompts/_r-odd-body.md", "# r-odd 正文\n")
        write_sample(work, "research/prompts/r-odd-snapshot/odd-sha256.txt",
                     f"快照时刻 14:09:00 UTC\n# 注释\n{digest}  README.md\n{digest}  research/prompts/r-odd-model.md\nnot-a-hash  .claude/kb/pitfalls.md\n")
        # 另一个样本仓：一份清单读不了（悬空的符号链接），另一份读得了
        unreadable_work = tempfile.mkdtemp()
        other_directories.append(unreadable_work)
        write_sample(unreadable_work, "research/prompts/_r-broken-body.md", "# r-broken 正文\n")
        write_sample(unreadable_work, "research/prompts/r-broken-snapshot/kb-sha256.txt", f"{digest}  .claude/kb/checks-owed.md\n")
        os.symlink(os.path.join(unreadable_work, "不在的文件"), os.path.join(unreadable_work, "research/prompts/r-broken-snapshot/gone-sha256.txt"))
        # 规格文件放在 /tmp/claude-<uid>/ 下（派发闸只认那里）
        scratch_root = f"/tmp/claude-{os.getuid()}"
        os.makedirs(scratch_root, exist_ok=True)
        specification_directory = tempfile.mkdtemp(prefix="runner-dispatch-guard-selftest-", dir=scratch_root)
        other_directories.append(specification_directory)
        specification_path = os.path.join(specification_directory, "spec.md")
        write_sample(specification_directory, "spec.md", "1. 文件：.claude/kb/checks-owed.md\n   旧串：| C120 | …\n   新串：| C120 | …\n")
        missing_specification_path = os.path.join(specification_directory, "missing-spec.md")
        collided = "规格：改 `.claude/kb/checks-owed.md` 里 C120 那一行。\n" + FOOTER
        forwarded_refusal = decide({"tool_input": {"subagent_type": "kb-scribe", "prompt": collided}}, work, hook_dir)[1] or ""
        cases = [
            case("不是执行员", "experiment-designer", "随便", 0),
            case("只修锚点", "experiment-runner", "只修锚点：表 e153", 0),
            case("第一段点名了岔路", "experiment-runner", first + "这一段回答的岔路：岔路 1、岔路 4\n", 0),
            case("第一段没点名岔路", "experiment-runner", first, 2),
            case("第一段岔路写无", "experiment-runner", first + "这一段回答的岔路：无\n", 2),
            case("续做点名了还差的行", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：岔路 1 的崩溃支线\n", 0),
            case("续做没写还差", "experiment-runner", again + "这一段回答的岔路：岔路 1\n", 2),
            case("续做还差写无", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：无\n", 2),
            case("重跑登记也算续做", "experiment-runner", "research/prompts/e153-r2-prereg.md\n这一段回答的岔路：岔路 3\n", 2),
        ]
        # ── 派发提示的结构：重型测试一行、共用约束、定义要的输入、写范围、实现员文件与登记、模型并发与限额窗口、书记员规格、设计员准入、变异表 ──
        defs_work = tempfile.mkdtemp()
        other_directories.append(defs_work)
        write_sample(defs_work, ".claude/agents/three-way-local-attack.md", "---\nname: three-way-local-attack\nmodel: sonnet\nrequired-inputs: 禁读清单, 草稿目录, 提示文件\n---\n")
        write_sample(defs_work, ".claude/agents/three-way-verifier.md", "---\nname: three-way-verifier\nmodel: sonnet  # 注释\nrequired-inputs: 快照, 草稿目录, 报告路径|-verifier-output.md\n---\n")
        write_sample(defs_work, ".claude/agents/experiment-runner.md", "---\nname: experiment-runner\nmodel: opus\nrequired-inputs: 草稿目录\n---\n")
        write_sample(defs_work, ".claude/agents/implementation-writer.md", "---\nname: implementation-writer\nmodel: opus\n---\n")
        write_sample(defs_work, ".claude/agents/mutation-triage.md", "---\nname: mutation-triage\nmodel: sonnet\n---\n")
        write_sample(defs_work, ".claude/hooks/agent-write-scope.tsv",
                     "# 样本\nexperiment-runner\tresearch/results/**\t样本\nexperiment-runner\tresearch/prompts/e*-r*-prereg.md\t样本\nexperiment-runner\t/tmp/claude-1000/**\t样本\n"
                     "implementation-writer\tcrates/**\t样本\nimplementation-writer\t/tmp/claude-1000/**\t样本\n")
        heavy_line = "重型测试：不跑\n"
        defs_case = lambda label, subagent_type, prompt, want: case_in(defs_work, label, subagent_type, heavy_line + prompt, want, footer=False)
        for index, sentence in enumerate(OLD_HEAVY_TEST_REFUSALS, 1):
            cases.append(case(f"重型:旧判法误拦的原句 {index} 带上那一行就放行", "general-purpose", sentence, 0))
        cases += [
            case("重型:没写那一行（实现员）", "implementation-writer", "改完交回。", 2, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case("重型:没写那一行（通用 agent）", "general-purpose", "查一个文件。", 2, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case("重型:崩溃验证员不要那一行", "crash-verifier", "提交流程里跑 54 号 --full。", 0, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case("重型:门禁分诊员不要那一行", "gate-triage", "带 SINGLEFS_HEAVY_TESTS=commit 跑 gate.sh --staged。", 0, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case("重型:那一行带粗体与列表记号也认", "implementation-writer", "改完交回。\n- **重型测试**：不跑\n", 0, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case("重型:写在句子中间不算那一行", "implementation-writer", "这一件的重型测试：不跑也行吧", 2, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case_in(defs_work, "共用约束:通用 agent 没写开工先读", "general-purpose", heavy_line + "查一个文件。", 2, footer=False),
            case_in(defs_work, "共用约束:通用 agent 写了开工先读", "general-purpose", heavy_line + "开工先读 `.claude/agent-common.md`。查一个文件。", 0, footer=False),
            defs_case("输入:本地攻方缺禁读清单（09-23 12:48 那一次）", "three-way-local-attack", "提示文件 research/prompts/r-local-attack.md，草稿目录 /tmp/claude-1000/r/", 2),
            defs_case("输入:本地攻方齐了", "three-way-local-attack", "提示文件 research/prompts/r-local-attack.md，草稿目录 /tmp/claude-1000/r/，禁读清单：别的腿的产出", 0),
            defs_case("输入:核查员缺草稿目录（09-24 23:44 那一次）", "three-way-verifier", "快照 research/prompts/r-snapshot/，报告路径 research/prompts/r-verifier-output.md", 2),
            defs_case("输入:核查员缺快照（09-25 02:53 那一次）", "three-way-verifier", "草稿目录 /tmp/claude-1000/v/，报告 research/prompts/r-verifier-output.md", 2),
            defs_case("输入:核查员同义词认得", "three-way-verifier", "快照 x，草稿目录 /tmp/claude-1000/v/，写 research/prompts/r-verifier-output.md", 0),
            defs_case("写范围:执行员报告写进 research/prompts（09-26 那一次）", "experiment-runner",
                      "草稿目录 /tmp/claude-1000/e/\n报告路径：research/prompts/e142-r15-runner-report.md\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:执行员去更新问题单", "experiment-runner",
                      "草稿目录 /tmp/claude-1000/e/\n更新 research/prompts/m2-rbf1-e142-questions.md 的状态\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:实现员写进 invariants.md", "implementation-writer",
                      "要动的 crates 文件：crates/a/src/one.rs\n顺手写进 .claude/kb/invariants.md 的状态列\n", 2),
            defs_case("写范围:报告写进草稿目录放行", "experiment-runner",
                      "草稿目录 /tmp/claude-1000/e/\n报告写进 /tmp/claude-1000/e/report.md\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:否定句里的路径不判", "experiment-runner",
                      "草稿目录 /tmp/claude-1000/e/\n不要写进 research/prompts/x-report.md\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("变异分诊:没给表", "mutation-triage", "跑 2697 条广谱变异。", 2),
            defs_case("变异分诊:给了表", "mutation-triage", "跑 crates/mutations.tsv 里 m2-rollback 那几条。", 0),
            defs_case("执行员:只补落产物放行", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物：E142 那份。", 0),
        ]
        # 实现员：要动的 crates 文件一行、件数上限、与在跑的实现员撞文件（登记在自检自己的状态目录里）
        state_root = tempfile.mkdtemp()
        other_directories.append(state_root)
        os.environ["DISPATCH_GUARD_STATE_ROOT"] = state_root
        session_work = tempfile.mkdtemp()
        other_directories.append(session_work)
        def session_case(label, subagent_type, prompt, want, transcript_name, model=None, project_root=defs_work):
            tool_input = {"subagent_type": subagent_type, "prompt": heavy_line + prompt}
            if model:
                tool_input["model"] = model
            hook_input = {"tool_name": "Agent", "tool_input": tool_input, "session_id": transcript_name,
                          "transcript_path": os.path.join(session_work, transcript_name + ".jsonl")}
            return (label, want, decide(hook_input, project_root)[0])
        def write_session(transcript_name, records):
            with open(os.path.join(session_work, transcript_name + ".jsonl"), "a", encoding="utf-8") as handle:
                for record in records:
                    handle.write(json.dumps(record, ensure_ascii=False) + "\n")
        def stamp(seconds_ago):
            return datetime.fromtimestamp(time.time() - seconds_ago, timezone.utc).isoformat().replace("+00:00", "Z")
        def launch(agent_id, model, prompt, seconds_ago=600):
            return {"timestamp": stamp(seconds_ago), "toolUseResult": {"status": "async_launched", "agentId": agent_id, "resolvedModel": model, "prompt": prompt}}
        def handed_back(agent_id, seconds_ago=60):
            return {"timestamp": stamp(seconds_ago), "content": f'<agent-message from="{agent_id}">\n[Subagent hand-back] 报告'}
        def notification(agent_id, status, summary="", seconds_ago=60):
            return {"timestamp": stamp(seconds_ago), "content": f"<task-notification>\n<task-id>{agent_id}</task-id>\n<status>{status}</status>\n<summary>{summary}</summary>\n</task-notification>"}
        writer_one = "要动的 crates 文件：crates/a/src/one.rs crates/a/src/two.rs\n"
        write_session("registry", [])
        cases.append(session_case("实现员:第一件登记下来", "implementation-writer", writer_one, 0, "registry"))
        cases.append(session_case("实现员:第二件撞了在跑的那件", "implementation-writer", "要动的 crates 文件：crates/a/src/two.rs、crates/a/src/three.rs\n", 2, "registry"))
        cases.append(session_case("实现员:第三件不撞", "implementation-writer", "要动的 crates 文件：crates/a/src/three.rs\n", 0, "registry"))
        write_session("registry", [launch("a00000000000000a1", "claude-opus-5", heavy_line + writer_one), handed_back("a00000000000000a1")])
        cases.append(session_case("实现员:第一件交回之后不再撞", "implementation-writer", "要动的 crates 文件：crates/a/src/two.rs\n", 0, "registry"))
        cases.append(session_case("实现员:没写要动的文件", "implementation-writer", "改挂载准入。", 2, "registry"))
        cases.append(session_case("实现员:文件超过上限", "implementation-writer", "要动的 crates 文件：" + " ".join(f"crates/b/src/f{n}.rs" for n in range(IMPLEMENTATION_WRITER_FILE_LIMIT + 1)) + "\n", 2, "registry"))
        cases.append(session_case("实现员:文件超过上限、用户定了放宽", "implementation-writer", "要动的 crates 文件：" + " ".join(f"crates/c/src/f{n}.rs" for n in range(IMPLEMENTATION_WRITER_FILE_LIMIT + 1)) + "\n文件上限已判：用户定这一件放宽到 12\n", 0, "registry"))
        cases.append(session_case("实现员:文件超过上限、放行行理由空", "implementation-writer", "要动的 crates 文件：" + " ".join(f"crates/d/src/f{n}.rs" for n in range(IMPLEMENTATION_WRITER_FILE_LIMIT + 1)) + "\n文件上限已判：\n", 2, "registry"))
        # opus 并发：4 个在跑（1 个交回、1 个失败的不算）；限额窗口：失败通知写着还没到的 resets 时刻
        write_session("busy", [launch(f"a000000000000b{n:02d}", "claude-opus-5-5[1m]", f"活 {n}") for n in range(OPUS_CONCURRENCY_LIMIT + 2)]
                      + [handed_back("a000000000000b00"), notification("a000000000000b01", "failed")])
        write_session("light", [launch(f"a000000000000c{n:02d}", "claude-opus-5", f"活 {n}") for n in range(OPUS_CONCURRENCY_LIMIT - 1)]
                      + [launch("a000000000000c99", "claude-sonnet-5", "别的")])
        future = datetime.fromtimestamp(time.time() + 7200, timezone.utc)
        future_text = f"{future.hour % 12 or 12}{'pm' if future.hour >= 12 else 'am'}"
        write_session("window", [launch("a0000000000000d1", "claude-opus-5", "撞限额的", 1200),
                                 notification("a0000000000000d1", "failed", f"You've hit your session limit · resets {future_text} (UTC) (error type rate_limit, model sent to the API: claude-opus-5)", 600)])
        write_session("windowlifted", [launch("a0000000000000f1", "claude-opus-5", "撞限额的", 1200),
                                       notification("a0000000000000f1", "failed", f"resets {future_text} (UTC) (model sent to the API: claude-opus-5)", 900),
                                       launch("a0000000000000f2", "claude-opus-5-5[1m]", "换号之后派的", 300)])
        past = datetime.fromtimestamp(time.time() - 30 * 3600, timezone.utc)
        past_reset = datetime.fromtimestamp(time.time() - 29 * 3600, timezone.utc)
        write_session("windowpast", [notification("a0000000000000e1", "failed", f"resets {past_reset.hour % 12 or 12}{'pm' if past_reset.hour >= 12 else 'am'} (UTC) (model sent to the API: claude-opus-5)", 30 * 3600)])
        cases += [
            session_case(f"opus:已有 {OPUS_CONCURRENCY_LIMIT} 个在跑再派 opus", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物", 2, "busy"),
            session_case("opus:用户定了超上限", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物\nopus 并发已判：用户 23:10 定这一件照派", 0, "busy"),
            session_case("opus:派 sonnet 不算", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物", 0, "busy", model="sonnet"),
            session_case(f"opus:{OPUS_CONCURRENCY_LIMIT - 1} 个在跑放行", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物", 0, "light"),
            session_case("opus:没有定义的类型按 opus 算", "general-purpose", "开工先读 `.claude/agent-common.md`。", 2, "busy"),
            session_case("窗口:还没到 resets 时刻不派同一族", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物", 2, "window"),
            session_case("窗口:换一族放行", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物", 0, "window", model="sonnet"),
            session_case("窗口:用户定了照派", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物\n限额窗口已判：用户说换号了", 0, "window"),
            session_case("窗口:resets 时刻已过", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物", 0, "windowpast"),
            session_case("窗口:撞限额之后同一族又派出去没再撞，已解开", "experiment-runner", "草稿目录 /tmp/claude-1000/e/\n只补落产物", 0, "windowlifted"),
        ]
        # 书记员的规格：点名的规格文件交给 kb-spec-check.py（仓根是样本仓 work）
        write_sample(work, ".claude/kb/tooling.md", "# 工具\n\n| 甲 | 旧值 |\n\n## 历史版本\n")
        good_spec = os.path.join(specification_directory, "good-spec.json")
        write_sample(specification_directory, "good-spec.json", json.dumps([{"file": ".claude/kb/tooling.md", "old": "| 甲 | 旧值 |", "new": "| 甲 | 新值 |", "basis": "样本"}], ensure_ascii=False))
        bad_spec = os.path.join(specification_directory, "bad-spec.json")
        write_sample(specification_directory, "bad-spec.json", json.dumps([{"file": ".claude/kb/tooling.md", "old": "| 乙 | 不在的行 |", "new": "| 乙 | 新 |", "basis": "样本"}], ensure_ascii=False))
        no_valve = heavy_line + "开工先读 `.claude/agent-common.md`\n"
        cases += [
            case("规格:点名的规格检查过了", "kb-scribe", f"逐条规格在 {good_spec}，照写。", 0, footer=no_valve),
            case("规格:点名的规格旧串不在", "kb-scribe", f"逐条规格在 {bad_spec}，照写。", 2, footer=no_valve),
            case("规格:一份认得出的规格都没有", "kb-scribe", "照判决写回 C120。", 2, footer=no_valve),
            case("规格:写了放行行", "kb-scribe", "照判决写回 C120。\n规格检查已判：用户口述的定案，没有规格文件\n", 0, footer=no_valve),
        ]
        # 设计员的重跑登记：准入判输入没变（假的 admission.py 退 77）就拒
        for name, code in (("admission-unchanged.py", 77), ("admission-changed.py", 0)):
            write_sample(work, name, f"import sys\nsys.exit({code})\n")
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-unchanged.py")
        cases += [
            case("准入:输入没变不写重跑登记", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 2),
            case("准入:写了放行行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。\n准入已判：换了一条臂\n", 0),
            case("准入:不是重跑登记不判", "experiment-designer", "写跑前登记 research/prompts/e170-preregistration.md。", 0),
        ]
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-changed.py")
        cases.append(case("准入:输入变了放行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 0))
        cases += [
            # 快照：派书记员 / 实现员时要改的文件落在还没判完的三方轮的开工快照里，拒绝
            case("快照:书记员撞开着的轮（清单写 ./.claude/）", "kb-scribe", collided, 2),
            case("快照:实现员撞开着的轮（清单写 tree/crates/）", "implementation-writer", "改 crates/singlefs-core/src/mount.rs 的挂载准入，带测试。", 2),
            case("快照:提示写项目根起的绝对路径、中文文件名", "kb-scribe", f"改 {work}/.claude/kb/decisions/18-块里携带什么信息.md 已定项 11 的射程。", 2),
            case("快照:清单写 defs/.claude/ 也归一", "kb-scribe", "规格：改 `.claude/agents/crash-verifier.md` 的写范围一节。", 2),
            case("快照:规格文件里点名的照拒（路径后面紧跟中文）", "kb-scribe", f"逐条规格在 {specification_path}里，照做。报告写到同一目录。", 2),
            case("快照:路径后面连着别的路径字符不算", "implementation-writer", "对照 crates/singlefs-core/src/mount.rs.orig 那份旧拷贝。", 0),
            case("快照:撞的是已判完的轮，放行", "kb-scribe", "规格：改 `.claude/kb/decisions/03-空间分配.md` 已定项 2。", 0),
            case("快照:有快照没正文的不是三方轮，放行", "implementation-writer", "改 crates/singlefs-core/src/allocator.rs 的分配顺序。", 0),
            case("快照:有放行行，放行", "kb-scribe", collided + "快照冲突已判：r-open 只读它核 C120 的行号，不改那一行\n", 0),
            case("快照:放行行写的是别的轮，照拒", "kb-scribe", collided + "快照冲突已判：r-closed 只读\n", 2),
            case("快照:放行行没写理由，照拒", "kb-scribe", collided + "快照冲突已判：r-open\n", 2),
            case("快照:放行行没写理由、下一行不算它的理由，照拒", "kb-scribe", collided + "快照冲突已判：r-open\n报告写到草稿目录。\n", 2),
            case("快照:放行行理由是占位，照拒", "kb-scribe", collided + "- 快照冲突已判：r-open <理由>\n", 2),
            case("快照:拒绝信息原样转贴，句中的放行模板不算", "kb-scribe", forwarded_refusal, 2),
            case("快照:派别的 agent 放行（通用）", "general-purpose", collided, 0),
            case("快照:派别的 agent 放行（实验设计员）", "experiment-designer", collided, 0),
            case("快照:清单里认不出的行不拒", "kb-scribe", "规格：改 README.md、research/prompts/r-odd-model.md 与 `.claude/kb/pitfalls.md`。", 0),
            case("快照:规格文件不存在不拒", "kb-scribe", f"逐条规格在 {missing_specification_path}，草稿目录 {specification_directory}/，报告写 {specification_directory}/report.md。", 0),
            case_in(unreadable_work, "快照:一份清单读不了，不拒", "kb-scribe", "规格：改 `.claude/kb/pitfalls.md`。", 0),
            case_in(unreadable_work, "快照:一份清单读不了，读得了的那份撞了照拒", "kb-scribe", collided, 2),
        ]
        script = os.path.join(hook_dir, "runner-dispatch-guard.sh")
        def run_hook(project_root, stdin_text):
            return subprocess.run(["bash", script], input=stdin_text, capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=project_root))
        snapshot_refusal = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": collided}}))
        cases.append(("stdin:快照撞了拒绝（退出码 2）", 2, snapshot_refusal.returncode))
        cases.append(("stdin:快照拒绝的 stderr 写了轮名、文件与出路", 1,
                      int(all(needle in snapshot_refusal.stderr for needle in ("r-open", ".claude/kb/checks-owed.md", "等 r-open 的判决写完再派", "快照冲突已判：r-open")))))
        broken_json = run_hook(work, "{不是 JSON")
        cases.append(("stdin:JSON 解析不了放行", 0, broken_json.returncode))
        cases.append(("stdin:JSON 解析不了时 stderr 写了错误", 1, int("JSON" in broken_json.stderr)))
        unreadable_list = run_hook(unreadable_work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": "规格：改 `.claude/kb/pitfalls.md`。\n" + FOOTER}}))
        cases.append(("stdin:快照清单读不了放行", 0, unreadable_list.returncode))
        cases.append(("stdin:快照清单读不了时 stderr 写了是哪一份", 1, int("读不了快照清单" in unreadable_list.stderr and "gone-sha256.txt" in unreadable_list.stderr)))
        for label, prompt, want in (("stdin:续做没写还差", again + "这一段回答的岔路：岔路 1\n", 2),
                                    ("stdin:第一段点名了岔路", first + "这一段回答的岔路：岔路 1\n", 0)):
            completed = subprocess.run(["bash", script],
                                       input=json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "experiment-runner", "prompt": prompt + FOOTER}}),
                                       capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=work))
            cases.append((label, want, completed.returncode))
        heavy = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "implementation-writer",
                                                                            "prompt": "改完交回。\n" + FOOTER_WITHOUT_HEAVY_LINE}}))
        cases.append(("stdin:派实现员没写重型测试一行拒绝（退出码 2）", 2, heavy.returncode))
        cases.append(("stdin:拒绝时 stderr 写了出路", 1, int("→ 怎么办" in heavy.stderr and "弹窗" in heavy.stderr)))
    finally:
        os.environ.pop("DISPATCH_GUARD_STATE_ROOT", None)
        os.environ.pop("RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT", None)
        shutil.rmtree(work)
        for directory in other_directories:
            shutil.rmtree(directory, ignore_errors=True)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide() 与它调的各条判法（重型测试一行、agent_definition_fields、out_of_scope_mentions、implementation_writer_verdict、model_verdict、"
              "kb_scribe_spec_verdict、designer_admission_verdict、snapshot_conflict_verdict）；RUNNER_DISPATCH_GUARD_DISABLE_CHECK、_DISABLE_HEAVY_TEST、"
              "_DISABLE_SNAPSHOT 或 _BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种）：没写「重型测试：不跑」一行的拒（崩溃验证员、门禁分诊员不要，旧判法误拦的 {len(OLD_HEAVY_TEST_REFUSALS)} 句原句带上那一行放行）；"
          "通用 agent 没写开工先读共用约束的拒；定义 required-inputs 缺一组的拒、同义词认得；提示点名写范围外的路径拒、否定句里的不判；"
          "实现员没写要动的 crates 文件、超上限、与在跑的实现员撞文件的拒，交回之后不再撞；opus 在跑满上限的拒、sonnet 与用户放行行放行，"
          "限额窗口没到的同一族拒、换一族或时刻已过放行；书记员规格检查红或一份规格都认不出的拒；设计员重跑登记准入判输入没变的拒；"
          "变异分诊员没给表的拒；执行员没点名岔路、续做没写还差或还差写无的拒，只修锚点与只补落产物放行；"
          "派书记员或实现员时要改的文件在没判完的三方快照里就拒，已判完的轮、没正文的快照、写了放行行的、派别的 agent 的、认不出的清单行、不在的规格与读不了的清单放行")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except ValueError as error:
        if not break_switch_is("json-error-silent"):
            print(f"! 派发闸读不懂 hook 的 JSON，这一次放行：{error}", file=sys.stderr)
        return 2 if break_switch_is("json-error-denies") else 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    code, message = decide(hook_input, project_root)
    if message:
        print(message, file=sys.stderr)
    return code

sys.exit(main())
PY
