"""草稿目录写者锁：同一个草稿目录同一时刻只许持锁的那一个子 agent 写。

两处用这一份：
  .claude/hooks/runner-dispatch-guard.sh   派发那一刻在提示里「草稿目录：<路径>」那一行点名的目录里排他建锁，持锁的还在跑就拒（⑯）
  .claude/hooks/write-guard.sh             子 agent 用 Write / Edit / MultiEdit 写草稿根底下的文件时，认锁，不是持锁的那一个就拒（第九道）

锁与认领（都在草稿目录里，不进仓）：
  .writer-lock            JSON：session_id（派发它的会话）、dispatched_at（派发时刻，秒）、dispatched_date（同一时刻的日期时间文字，给人看）、
                          prompt_sha256（派发提示的 sha256）、subagent_type、transcript_path（派发它的会话的记录）、nonce（这一次派发独有的随机串）
  .writer-claim-<nonce>   JSON：agent_id、claimed_at。第一个写进这个目录、又被认作持锁者的子 agent 排他建它；锁被新一次派发接手（nonce 换了）后旧认领不再算数

谁是持锁者（writer_verdict，写那一刻判）：
  1. 这把锁已经有认领：认领里的 agent_id 是持锁者，是它放行（同一个子 agent 续写、被 SendMessage 续做都是这一种），不是它拒。
  2. 还没有认领：在主会话记录（hook 输入的 transcript_path）里找这个子 agent 的派发回执（async_launched，agentId 对得上）。
     找到了：回执里的提示 sha256 要等于锁里的 prompt_sha256，回执时刻不早于锁的派发时刻（LAUNCH_SLACK_SECONDS 以内的误差不计），两样都对才认领；
     不对的是别的派发（重启之后留下的旧进程、同目录里另一件活的子 agent）拒，不认领。
     找不到（回执还没落进记录、记录读不了）：先到先得，排他建认领文件，建成了它就是持锁者，没建成读认领文件看是谁。
  3. 子 agent 的 agent_id 在 hook 输入里没有（主 agent 的写、没带身份的调用）：不判。

嵌套的草稿目录（目标往上走经过几处有 .writer-lock 的目录，find_lock_directory）：只认最近一层的锁，认领文件也只建在最近一层。父目录的持锁者 A 写子目录，
子目录另派给 E 持锁时，A 写 E 的子目录被拒；E 写父目录，最近一层就是父目录的锁，同样被拒。

回执对不上时点名持锁者（expected_holders）：最近一层没有认领文件、写的人的回执又对不上，持锁者要么还没写过、要么不在这份记录里。到锁里登记的派发会话记录（transcript_path）
与 hook 输入的记录里，找提示 sha256 对得上这把锁、回执时刻不早于锁的派发时刻的 async_launched 回执，拒绝说明写出它们的 agent_id；找不到写锁里登记的
提示 sha256 前 12 位、派发时刻与会话，让主 agent 能查。

派发接手（takeover）：同一个目录已有锁、持锁的已经交回 / 失效 / 从没起来过，或提示里写了「写者锁已判：<理由>」，新一次派发用 os.replace 换上新锁（nonce 换新，旧认领作废），
旧写者之后再写这个目录的 Write / Edit 被第九道拒。

Bash 里的写这里看不见（write-guard.sh 只挂 Write / Edit / MultiEdit）：靠子 agent 守，定义与共用约束里「开工先认锁」那一句管它。

用法：python 里把 .claude/hooks 放进 sys.path 再 import lib_writer_lock。

弄坏开关 LIB_WRITER_LOCK_BREAK=<项>（一次设一个，认不出的值抛 ValueError；打开后红在哪几格写在两个用它的钩子的文件头）：
  claim-not-exclusive      认领不排他：每次写都把认领文件改成自己，后写的抢走持锁者
  holder-ignored           不比 agent_id，谁写都放行
  launch-ignored           认领前不看派发回执，先到先得（旧进程抢在新持锁者前认领）
  stale-launch-accepted    认领前只比提示 sha256，不比回执时刻（同一份提示重派之后，旧进程照样认作持锁者）
  no-walk-up               只看目标所在的那一层目录有没有锁，不往上走（子目录里的写漏过去）
  foreign-holder-unnamed   回执对不上的拒绝不去记录里找持锁者的 agent_id（改前的说明：只写「派发提示对得上这把锁的那一个子 agent」）
  replace-temp-kept        接手换锁写临时文件之后、换上之前抛异常，不删自己建的临时文件（草稿目录里留下 .tmp）
"""
import hashlib
import json
import os
import secrets
import subprocess
import sys
import time
from datetime import datetime, timezone

sys.dont_write_bytecode = True

LOCK_NAME = ".writer-lock"
CLAIM_PREFIX = ".writer-claim-"
BREAK_VARIABLE = "LIB_WRITER_LOCK_BREAK"
KNOWN_BREAK_MODES = ("claim-not-exclusive", "holder-ignored", "launch-ignored", "stale-launch-accepted", "no-walk-up", "foreign-holder-unnamed",
                     "replace-temp-kept")
EXPECTED_HOLDER_LIMIT = 3
PROMPT_DIGEST_SHOWN = 12
LAUNCH_SLACK_SECONDS = 2
TRANSCRIPT_GREP_TIMEOUT_SECONDS = 50


def break_mode():
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAK_MODES:
        raise ValueError(f"{BREAK_VARIABLE}={mode} 不认识，只认 {'、'.join(KNOWN_BREAK_MODES)}")
    return mode


def prompt_digest(prompt):
    return hashlib.sha256(str(prompt or "").encode("utf-8")).hexdigest()


def lock_path(scratch):
    return os.path.join(scratch, LOCK_NAME)


def claim_path(scratch, lock):
    return os.path.join(scratch, CLAIM_PREFIX + str(lock["nonce"]))


def new_lock(session_id, prompt, subagent_type, transcript_path, now=None):
    moment = time.time() if now is None else now
    return {"session_id": session_id or "", "dispatched_at": moment,
            "dispatched_date": datetime.fromtimestamp(moment, timezone.utc).isoformat(),
            "prompt_sha256": prompt_digest(prompt), "subagent_type": subagent_type or "",
            "transcript_path": transcript_path or "", "nonce": secrets.token_hex(8)}


def read_json(path):
    """JSON 文件 → dict；不在、读不了、不是 JSON 对象都返回 None。"""
    try:
        with open(path, encoding="utf-8") as handle:
            value = json.load(handle)
    except (OSError, ValueError):
        return None
    return value if isinstance(value, dict) else None


def read_lock(scratch):
    lock = read_json(lock_path(scratch))
    if lock is None or not isinstance(lock.get("nonce"), str) or not isinstance(lock.get("dispatched_at"), (int, float)):
        return None
    return lock


def read_claim(scratch, lock):
    claim = read_json(claim_path(scratch, lock))
    return claim if claim is not None and isinstance(claim.get("agent_id"), str) else None


def write_exclusively(path, value):
    """把 value 写成 path：先写同目录临时文件，再 os.link 到 path（已存在就是 FileExistsError）；别人读到的要么没有这个文件、要么是写完整的。"""
    temporary = f"{path}.{os.getpid()}.{secrets.token_hex(4)}.tmp"
    with open(temporary, "x", encoding="utf-8") as handle:
        json.dump(value, handle, ensure_ascii=False)
    try:
        os.link(temporary, path)
    finally:
        os.unlink(temporary)


def create_lock(scratch, lock):
    """排他建锁。→ True 建成；False 目录里已经有锁。目录不在先建。"""
    os.makedirs(scratch, exist_ok=True)
    try:
        write_exclusively(lock_path(scratch), lock)
    except FileExistsError:
        return False
    return True


def replace_lock(scratch, lock):
    """接手：同目录临时文件写完整再 os.replace 换上锁；写或换失败时删掉自己建的临时文件再把异常抛出去；旧认领文件顺手删（nonce 换了本来也不算数）。"""
    temporary = f"{lock_path(scratch)}.{os.getpid()}.{secrets.token_hex(4)}.tmp"
    handle = open(temporary, "x", encoding="utf-8")   # 建不成（同名已在）就不是自己的文件，不删
    try:
        with handle:
            json.dump(lock, handle, ensure_ascii=False)
        os.replace(temporary, lock_path(scratch))
    except BaseException:
        if break_mode() != "replace-temp-kept":
            try:
                os.unlink(temporary)
            except FileNotFoundError:
                pass
        raise
    keep = CLAIM_PREFIX + lock["nonce"]
    for name in os.listdir(scratch):
        if name.startswith(CLAIM_PREFIX) and name != keep:
            try:
                os.unlink(os.path.join(scratch, name))
            except OSError:
                pass


def find_lock_directory(target, roots):
    """target 所在的目录往上走到某个草稿根（不含根本身）为止，第一处有 .writer-lock 的目录；没有返回 None。"""
    absolute = os.path.normpath(os.path.abspath(target))
    for root in roots:
        root = root.rstrip("/")
        if not absolute.startswith(root + "/"):
            continue
        directory = os.path.dirname(absolute)
        while directory.startswith(root + "/"):
            if os.path.isfile(lock_path(directory)):
                return directory
            if break_mode() == "no-walk-up":
                break
            directory = os.path.dirname(directory)
    return None


def launch_receipt(agent_id, transcript_path):
    """主会话记录里这个子 agent 的派发回执 → (回执时刻（秒）, 提示 sha256)；没找到、记录读不了返回 None。"""
    if not agent_id or not transcript_path or not os.path.isfile(transcript_path):
        return None
    try:
        found = subprocess.run(["grep", "-F", "-e", agent_id, transcript_path], capture_output=True, text=True, timeout=TRANSCRIPT_GREP_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired):
        return None
    for line in found.stdout.splitlines():
        if '"async_launched"' not in line:
            continue
        try:
            record = json.loads(line)
            result = record.get("toolUseResult")
            if isinstance(result, dict) and result.get("status") == "async_launched" and result.get("agentId") == agent_id:
                moment = datetime.fromisoformat(str(record.get("timestamp") or "").replace("Z", "+00:00")).timestamp()
                return moment, prompt_digest(result.get("prompt"))
        except (ValueError, TypeError):
            continue
    return None


def try_claim(scratch, lock, agent_id):
    """排他建认领文件。→ 认领文件最终写的 agent_id（建成的是 agent_id 自己，被别人抢先的是别人的）。"""
    claim = {"agent_id": agent_id, "claimed_at": time.time()}
    path = claim_path(scratch, lock)
    if break_mode() == "claim-not-exclusive":
        with open(path, "w", encoding="utf-8") as handle:
            json.dump(claim, handle, ensure_ascii=False)
        return agent_id
    try:
        write_exclusively(path, claim)
        return agent_id
    except FileExistsError:
        existing = read_json(path)
        return existing.get("agent_id") if existing else None


def writer_verdict(scratch, lock, agent_id, transcript_path):
    """这个子 agent 写 scratch 这一刻认不认作持锁者。→ (结论, 持锁者 agent_id 或 None)：
    "holder" 是持锁者（本来就是，或这一次认领成了）；"other" 别的子 agent 持锁（第二项是它）；
    "foreign" 它的派发回执对不上这把锁（别的派发、或早于这把锁的旧进程），没人认领也不认领给它（第二项是 None）。"""
    mode = break_mode()
    if mode == "holder-ignored":
        return "holder", agent_id
    if mode != "claim-not-exclusive":
        claim = read_claim(scratch, lock)
        if claim is not None:
            return ("holder", agent_id) if claim["agent_id"] == agent_id else ("other", claim["agent_id"])
    if mode != "launch-ignored":
        receipt = launch_receipt(agent_id, transcript_path)
        if receipt is not None:
            moment, digest = receipt
            late_enough = moment >= lock["dispatched_at"] - LAUNCH_SLACK_SECONDS or mode == "stale-launch-accepted"
            if digest != lock.get("prompt_sha256") or not late_enough:
                return "foreign", None
    owner = try_claim(scratch, lock, agent_id)
    if owner == agent_id:
        return "holder", agent_id
    return "other", owner


def expected_holders(lock, requester, transcript_path):
    """最近一层没有认领文件、写的人的回执又对不上时，这把锁该有的持锁者：在锁里登记的派发会话记录与 hook 输入的记录里，找提示 sha256 对得上这把锁、
    回执时刻不早于锁的派发时刻的 async_launched 回执，交回它们的 agent_id（不含 requester，最多 EXPECTED_HOLDER_LIMIT 个，按回执时刻先后）。
    找不到、记录读不了交空列表。弄坏开关 foreign-holder-unnamed 下一律交空列表。"""
    if break_mode() == "foreign-holder-unnamed":
        return []
    found = {}
    for path in dict.fromkeys(str(item) for item in (lock.get("transcript_path"), transcript_path) if item):
        if not os.path.isfile(path):
            continue
        try:
            listing = subprocess.run(["grep", "-F", "-e", '"async_launched"', path], capture_output=True, text=True, timeout=TRANSCRIPT_GREP_TIMEOUT_SECONDS)
        except (OSError, subprocess.TimeoutExpired):
            continue
        for line in listing.stdout.splitlines():
            try:
                record = json.loads(line)
                result = record.get("toolUseResult")
                if not (isinstance(result, dict) and result.get("status") == "async_launched" and isinstance(result.get("agentId"), str)):
                    continue
                moment = datetime.fromisoformat(str(record.get("timestamp") or "").replace("Z", "+00:00")).timestamp()
            except (ValueError, TypeError, AttributeError):
                continue
            if (result["agentId"] != requester and prompt_digest(result.get("prompt")) == lock.get("prompt_sha256")
                    and moment >= lock["dispatched_at"] - LAUNCH_SLACK_SECONDS):
                found.setdefault(result["agentId"], moment)
    return [agent for agent, _ in sorted(found.items(), key=lambda item: item[1])][:EXPECTED_HOLDER_LIMIT]


def describe_unknown_holder(lock, requester, transcript_path):
    """回执对不上、还没有认领文件时拒绝说明里的「持锁者」一句：点名找到的 agent_id；找不到写锁里登记的提示 sha256 前几位、派发时刻与会话。"""
    holders = expected_holders(lock, requester, transcript_path)
    registered = (f"锁登记的派发提示 sha256 前 {PROMPT_DIGEST_SHOWN} 位 {str(lock.get('prompt_sha256') or '?')[:PROMPT_DIGEST_SHOWN]}，"
                  f"派发于 {str(lock.get('dispatched_date'))[:10]}，派发会话 {lock.get('session_id') or '?'}")
    if holders:
        return (f"子 agent {'、'.join(holders)}（派发回执的提示对得上这把锁、还没有认领文件，多半还没写过这个目录；{registered}）")
    return (f"派发提示对得上这把锁的那一个子 agent（主会话记录里没找到它的回执；{registered}）")
