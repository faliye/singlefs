"""只看子 agent 真的停下来等、又被计时器叫醒的那几段（设计用法：run_in_background 起计时器、结束这一轮）。

等待段：一次调用以 end_turn 结束，下一次调用的输入只有计时器的返回或到点通知（被计时器叫醒）。从那次 end_turn 开始，
  每次被叫醒后若又以 end_turn 结束、下一次又被计时器叫醒，就接着算同一段；到下一次不是被计时器叫醒的调用、或被叫醒后
  没再以 end_turn 结束（它发现等的活完了、接着干）为止。
开销：被叫醒之后、又回去 end_turn 的那几轮里的全部调用（没有计时器时一次都不会发生）；最后一次被叫醒后接着干的那一轮不算
  （没有计时器时，后台活跑完的通知同样会叫醒它一次）。
节省：等待段总长 > 300 秒、结束等待那次调用缓存读 > 0 的，没有计时器时那次要重写读到的 token：读 × (1.25 − 0.1)。
  结束等待那次缓存读为 0 的记失效。
到点通知有两种形态：`attachment` 里的 `queued_command`，与字符串内容的 `user` 消息（「[SYSTEM NOTIFICATION …] <task-notification>」），按任务号认。
前台用法：计时器命令在前台跑、卡满 Bash 单次上限被转到后台（返回里有「moved to the background」）的次数与它之后那次调用的当量另报，
  不算进节省——那种用法下没有计时器 agent 会换别的办法等，省没省看不出来。
当量：新输入 ×1、5 分钟缓存写 ×1.25、缓存读 ×0.1、输出 ×5（基础输入 token）。
"""
import datetime, glob, json, os, re, sys

WEIGHTS = {"input_tokens": 1.0, "cache_creation_input_tokens": 1.25, "cache_read_input_tokens": 0.1, "output_tokens": 5.0}


def weighted(usage):
    return sum(usage.get(key, 0) * weight for key, weight in WEIGHTS.items())


def parse(path):
    records = []
    for line in open(path, errors="replace"):
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("timestamp"):
            records.append(record)
    keepalive_tool_ids, keepalive_task_ids = set(), set()
    calls, seen, inputs, foreground = [], {}, set(), []
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
                            if "moved to the background" in body:
                                foreground.append(len(calls))
                            inputs.add("keepalive")
                        else:
                            inputs.add("own")
                    elif block.get("type") == "text":
                        inputs.add(classify_text(block.get("text", ""), keepalive_task_ids))
            elif content:
                inputs.add(classify_text(str(content), keepalive_task_ids))
            continue
        if kind == "attachment":
            attachment = record.get("attachment") or {}
            if attachment.get("type") == "queued_command":
                notified = re.findall(r"<task-id>(b[a-z0-9]+)</task-id>", str(attachment.get("prompt", "")))
                inputs.add("keepalive" if notified and all(t in keepalive_task_ids for t in notified) else "external")
            continue
        if kind != "assistant":
            continue
        message = record.get("message", {})
        content = message.get("content")
        if isinstance(content, list):
            for block in content:
                if block.get("type") == "tool_use" and "cache-keepalive" in json.dumps(block.get("input", {})):
                    keepalive_tool_ids.add(block.get("id"))
        message_id = message.get("id")
        if message_id in seen:
            if message.get("stop_reason"):
                calls[seen[message_id]]["stop"] = message.get("stop_reason")
            continue
        seen[message_id] = len(calls)
        calls.append({"at": datetime.datetime.fromisoformat(record["timestamp"].replace("Z", "+00:00")),
                      "usage": message.get("usage") or {}, "inputs": inputs, "stop": message.get("stop_reason")})
        inputs = set()
    return calls, foreground


def classify_text(text, keepalive_task_ids):
    """字符串形态的输入：任务完成通知里的任务号全是计时器的，算计时器叫醒；其余（别的任务的通知、主 agent 的消息）算外部。"""
    notified = re.findall(r"<task-id>(b[a-z0-9]+)</task-id>", text)
    if notified and all(task_id in keepalive_task_ids for task_id in notified):
        return "keepalive"
    return "external"


def woken_by_keepalive(call):
    return call["inputs"] == {"keepalive"}


def waits(calls):
    found, index = [], 0
    while index < len(calls) - 1:
        if not (calls[index]["stop"] == "end_turn" and woken_by_keepalive(calls[index + 1])):
            index += 1
            continue
        start = index
        overhead, wakes, cursor = 0.0, 0, index + 1
        end = None
        while cursor < len(calls):
            if not woken_by_keepalive(calls[cursor]):
                end = cursor  # 不是被计时器叫醒的：等待结束于这次
                break
            wakes += 1
            round_end = cursor
            while round_end < len(calls) and calls[round_end]["stop"] != "end_turn":
                round_end += 1
            if round_end >= len(calls):
                end = cursor  # 被叫醒后一直干到会话末尾，没再睡
                break
            if round_end + 1 < len(calls) and woken_by_keepalive(calls[round_end + 1]):
                overhead += sum(weighted(call["usage"]) for call in calls[cursor:round_end + 1])
                cursor = round_end + 1
                continue
            if round_end + 1 < len(calls) and not woken_by_keepalive(calls[round_end + 1]):
                overhead += sum(weighted(call["usage"]) for call in calls[cursor:round_end + 1])
                end = round_end + 1
                break
            end = cursor
            break
        if end is None:
            end = len(calls) - 1
        span = (calls[end]["at"] - calls[start]["at"]).total_seconds()
        read = calls[end]["usage"].get("cache_read_input_tokens", 0)
        write = calls[end]["usage"].get("cache_creation_input_tokens", 0)
        saved = read * 1.15 if span > 300 and read > 0 else 0.0
        failed = span > 300 and read == 0
        found.append({"start": calls[start]["at"], "minutes": round(span / 60, 1), "wakes": wakes, "overhead": overhead,
                      "saved": saved, "failed": failed, "write": write})
        index = max(end, start + 1)
    return found


def main(directories):
    total = {"waits": 0, "long": 0, "overhead": 0.0, "saved": 0.0, "failed": 0, "failed_cost": 0.0,
             "foreground": 0, "foreground_cost": 0.0}
    print("agent\t等待开始(UTC)\t等了(分)\t被叫醒\t开销\t省下\t备注")
    for directory in directories:
        for path in sorted(glob.glob(os.path.join(directory, "*.jsonl"))):
            calls, foreground = parse(path)
            total["foreground"] += len(foreground)
            total["foreground_cost"] += sum(weighted(calls[i]["usage"]) for i in foreground if i < len(calls))
            for wait in waits(calls):
                total["waits"] += 1
                total["long"] += 1 if wait["minutes"] > 5 else 0
                total["overhead"] += wait["overhead"]
                total["saved"] += wait["saved"]
                note = ""
                if wait["failed"]:
                    total["failed"] += 1
                    total["failed_cost"] += wait["write"] * 1.25
                    note = f"结束等待那次仍整份重写，写 {wait['write']}"
                print(f"{os.path.basename(path)[-23:-6]}\t{wait['start']:%m-%d %H:%M}\t{wait['minutes']}\t{wait['wakes']}\t{wait['overhead']:.0f}\t{wait['saved']:.0f}\t{note}")
    print(f"设计用法：{total['waits']} 段被计时器叫醒过的等待（长于 5 分钟 {total['long']} 段）；叫醒开销 {total['overhead']:.0f}；"
          f"省下 {total['saved']:.0f}；结束等待仍整份重写 {total['failed']} 次（{total['failed_cost']:.0f}）；净 {total['saved'] - total['overhead']:+.0f}")
    print(f"前台用法：计时器在前台卡满上限 {total['foreground']} 次，之后那次调用合计 {total['foreground_cost']:.0f}（不算进节省）")


if __name__ == "__main__":
    main(sys.argv[1:])
