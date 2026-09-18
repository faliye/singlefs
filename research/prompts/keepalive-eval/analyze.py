"""统计子 agent 的 4 分钟计时器（research/scripts/cache-keepalive.sh）花了多少、省了多少。

一次模型调用 = 会话记录里 message.id 相同的 assistant 记录合成一条（取一份 usage）。
计时器带出的调用：它与上一次调用之间进来的输入，除了余量提醒，只有计时器那条命令的返回（前台跑卡满 120 秒被转后台，
  或后台起的回执）与计时器的到点通知（`attachment` 里 `queued_command` 的 <task-notification>）。其余调用算真干活。
口径（基础输入 token 当量，按官方价目倍数折算）：新输入 ×1，5 分钟缓存写 ×1.25，缓存读 ×0.1，输出 ×5。
花掉的：计时器带出的调用的当量之和（只算直接带出的那一次，被叫醒后顺手查进度的调用算真干活，所以是下界；另报上界：
  叫醒之后到下一次起计时器或这一轮结束为止的全部调用）。
省下的：两次真干活的调用隔了超过 300 秒、中间有计时器带出的调用、之后那次缓存读 > 0 的，
  没有计时器时那次要把读到的 token 重新写一遍：省 = 读 × (1.25 − 0.1)。这是上界：它假设没有计时器时中间不会有别的调用。
失效的：两次真干活的调用隔了超过 300 秒、之后那次缓存读为 0 的（整份重写）。
用法：python3 analyze.py <会话目录>/subagents …
"""
import datetime, glob, json, os, re, sys

WEIGHT_INPUT, WEIGHT_WRITE, WEIGHT_READ, WEIGHT_OUTPUT = 1.0, 1.25, 0.1, 5.0


def weighted(usage):
    return (usage.get("input_tokens", 0) * WEIGHT_INPUT + usage.get("cache_creation_input_tokens", 0) * WEIGHT_WRITE
            + usage.get("cache_read_input_tokens", 0) * WEIGHT_READ + usage.get("output_tokens", 0) * WEIGHT_OUTPUT)


def analyze(path):
    records = []
    for line in open(path, errors="replace"):
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("timestamp"):
            records.append(record)
    keepalive_tool_ids, keepalive_task_ids = set(), set()
    calls = []  # 每次调用：[时刻, usage, 是否计时器带出, 是否起了计时器, stop_reason]
    seen = {}
    inputs_since_call = []  # 上一次调用之后进来的输入种类：keepalive / other
    for record in records:
        kind = record.get("type")
        if kind == "user":
            content = record.get("message", {}).get("content")
            if isinstance(content, list):
                for block in content:
                    if block.get("type") == "tool_result":
                        body = json.dumps(block.get("content"), ensure_ascii=False)
                        if block.get("tool_use_id") in keepalive_tool_ids:
                            found = re.search(r"ID: (b[a-z0-9]+)", body)
                            if found:
                                keepalive_task_ids.add(found.group(1))
                            inputs_since_call.append("keepalive")
                        else:
                            inputs_since_call.append("other")
                    elif block.get("type") == "text":
                        inputs_since_call.append("other")
            elif content:
                inputs_since_call.append("other")
            continue
        if kind == "attachment":
            attachment = record.get("attachment") or {}
            if attachment.get("type") == "queued_command":
                prompt = str(attachment.get("prompt", ""))
                notified = re.findall(r"<task-id>(b[a-z0-9]+)</task-id>", prompt)
                if notified and all(task_id in keepalive_task_ids for task_id in notified):
                    inputs_since_call.append("keepalive")
                else:
                    inputs_since_call.append("other")
            continue
        if kind != "assistant":
            continue
        message = record.get("message", {})
        content = message.get("content")
        rearms = isinstance(content, list) and any(
            block.get("type") == "tool_use" and "cache-keepalive" in json.dumps(block.get("input", {})) for block in content)
        if isinstance(content, list):
            for block in content:
                if block.get("type") == "tool_use" and "cache-keepalive" in json.dumps(block.get("input", {})):
                    keepalive_tool_ids.add(block.get("id"))
        message_id = message.get("id")
        if message_id in seen:
            call = calls[seen[message_id]]
            call[3] = call[3] or rearms
            call[4] = message.get("stop_reason") or call[4]
            continue
        moment = datetime.datetime.fromisoformat(record["timestamp"].replace("Z", "+00:00"))
        caused = bool(inputs_since_call) and all(item == "keepalive" for item in inputs_since_call)
        seen[message_id] = len(calls)
        calls.append([moment, message.get("usage") or {}, caused, rearms, message.get("stop_reason")])
        inputs_since_call = []
    lower = sum(weighted(call[1]) for call in calls if call[2])
    upper, in_wake = 0.0, False
    for call in calls:
        if call[2]:
            in_wake = True
        if in_wake:
            upper += weighted(call[1])
            if call[3] or call[4] == "end_turn":
                in_wake = False
    real = [call for call in calls if not call[2]]
    caused = [call for call in calls if call[2]]
    saved, bridged, failures = 0.0, 0, []
    for previous, following in zip(real, real[1:]):
        gap = (following[0] - previous[0]).total_seconds()
        if gap <= 300:
            continue
        had_keepalive = any(previous[0] < call[0] < following[0] for call in caused)
        read = following[1].get("cache_read_input_tokens", 0)
        if read > 0 and had_keepalive:
            bridged += 1
            saved += read * (WEIGHT_WRITE - WEIGHT_READ)
        elif read == 0:
            failures.append((round(gap / 60, 1), following[1].get("cache_creation_input_tokens", 0), had_keepalive))
    armed = sum(1 for call in calls if call[3])
    return len(calls), armed, len(caused), lower, upper, bridged, saved, failures


def main(directories):
    total = {"calls": 0, "armed": 0, "caused": 0, "lower": 0.0, "upper": 0.0, "bridged": 0, "saved": 0.0, "rewrites": 0, "rewrite_cost": 0.0, "agents": 0, "agents_armed": 0}
    print("agent\t调用\t起计时器\t计时器带出的调用\t花掉下界\t花掉上界\t接住的长等待\t省下上界\t整份重写(分钟, 写入, 中间有计时器)")
    for directory in directories:
        for path in sorted(glob.glob(os.path.join(directory, "*.jsonl"))):
            calls, armed, caused, lower, upper, bridged, saved, failures = analyze(path)
            total["agents"] += 1
            total["calls"] += calls
            total["rewrites"] += len(failures)
            total["rewrite_cost"] += sum(item[1] * WEIGHT_WRITE for item in failures)
            if armed == 0 and not failures:
                continue
            total["agents_armed"] += 1 if armed else 0
            for key, value in (("armed", armed), ("caused", caused), ("lower", lower), ("upper", upper), ("bridged", bridged), ("saved", saved)):
                total[key] += value
            print(f"{os.path.basename(path)[-23:-6]}\t{calls}\t{armed}\t{caused}\t{lower:.0f}\t{upper:.0f}\t{bridged}\t{saved:.0f}\t{failures}")
    print(f"合计：{total['agents']} 个子 agent、{total['calls']} 次调用；起过计时器的 {total['agents_armed']} 个，共起 {total['armed']} 次，"
          f"带出调用 {total['caused']} 次；花掉 {total['lower']:.0f}–{total['upper']:.0f}；接住 {total['bridged']} 段长等待、省下至多 {total['saved']:.0f}；"
          f"整份重写 {total['rewrites']} 次、花 {total['rewrite_cost']:.0f}（当量，基础输入 token）")


if __name__ == "__main__":
    main(sys.argv[1:])
