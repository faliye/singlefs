"""按「等待链」统计计时器的花费与节省（换一种切法，与 analyze.py 并列）。

每次模型调用（message.id 相同的 assistant 记录合成一次）记下它之前进来的输入属于哪几类：
  keepalive = 计时器命令的返回或它的到点通知；own = agent 自己别的工具的返回；external = 别的后台任务的通知、主 agent 的消息。
链：从一次起计时器的调用（链头）开始，到之后第一次输入里带 external 的调用（链尾）为止；链里再起的计时器并进同一条链。
  没有计时器时，agent 在链头结束这一轮、到链尾才被外部输入叫醒，中间一次模型调用都没有。
链的花费：链头与链尾之间的全部调用（叫醒、查进度、续起计时器），按基础输入 token 当量（新输入 ×1、5 分钟缓存写 ×1.25、缓存读 ×0.1、输出 ×5）。
链的节省：链长 > 300 秒、链尾缓存读 > 0 的，没有计时器时链尾要把读到的 token 重写一遍：读 × (1.25 − 0.1)。
  链长 ≤ 300 秒的，没有计时器缓存也不会过期，节省记 0。链长 > 300 秒、链尾缓存读仍为 0 的，记一次失效。
  链头之后没有任何 external 输入就结束的（交回、被停），链尾取最后一次调用，节省记 0。
"""
import datetime, glob, json, os, re, sys

WEIGHTS = {"input_tokens": 1.0, "cache_creation_input_tokens": 1.25, "cache_read_input_tokens": 0.1, "output_tokens": 5.0}


def weighted(usage):
    return sum(usage.get(key, 0) * weight for key, weight in WEIGHTS.items())


def parse_calls(path):
    records = []
    for line in open(path, errors="replace"):
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("timestamp"):
            records.append(record)
    keepalive_tool_ids, keepalive_task_ids = set(), set()
    calls, seen, inputs = [], {}, set()
    for record in records:
        kind = record.get("type")
        if kind == "user":
            content = record.get("message", {}).get("content")
            if isinstance(content, list):
                for block in content:
                    if block.get("type") == "tool_result":
                        if block.get("tool_use_id") in keepalive_tool_ids:
                            found = re.search(r"ID: (b[a-z0-9]+)", json.dumps(block.get("content"), ensure_ascii=False))
                            if found:
                                keepalive_task_ids.add(found.group(1))
                            inputs.add("keepalive")
                        else:
                            inputs.add("own")
                    elif block.get("type") == "text":
                        inputs.add("external")
            elif content:
                inputs.add("external")
            continue
        if kind == "attachment":
            attachment = record.get("attachment") or {}
            if attachment.get("type") == "queued_command":
                notified = re.findall(r"<task-id>(b[a-z0-9]+)</task-id>", str(attachment.get("prompt", "")))
                if notified and all(task_id in keepalive_task_ids for task_id in notified):
                    inputs.add("keepalive")
                else:
                    inputs.add("external")
            continue
        if kind != "assistant":
            continue
        message = record.get("message", {})
        content = message.get("content")
        arms = False
        if isinstance(content, list):
            for block in content:
                if block.get("type") == "tool_use" and "cache-keepalive" in json.dumps(block.get("input", {})):
                    keepalive_tool_ids.add(block.get("id"))
                    arms = True
        message_id = message.get("id")
        if message_id in seen:
            calls[seen[message_id]]["arms"] = calls[seen[message_id]]["arms"] or arms
            continue
        seen[message_id] = len(calls)
        calls.append({"at": datetime.datetime.fromisoformat(record["timestamp"].replace("Z", "+00:00")),
                      "usage": message.get("usage") or {}, "inputs": inputs, "arms": arms})
        inputs = set()
    return calls


def chains(calls):
    found = []
    index = 0
    while index < len(calls):
        if not calls[index]["arms"]:
            index += 1
            continue
        head = index
        tail = None
        for cursor in range(head + 1, len(calls)):
            if "external" in calls[cursor]["inputs"]:
                tail = cursor
                break
        ended_without_external = tail is None
        if tail is None:
            tail = len(calls) - 1
        inside = calls[head + 1:tail] if not ended_without_external else calls[head + 1:]
        cost = sum(weighted(call["usage"]) for call in inside)
        span = (calls[tail]["at"] - calls[head]["at"]).total_seconds()
        read = calls[tail]["usage"].get("cache_read_input_tokens", 0)
        write = calls[tail]["usage"].get("cache_creation_input_tokens", 0)
        arms = sum(1 for call in calls[head:tail + 1] if call["arms"])
        if ended_without_external or span <= 300:
            saved, failed = 0.0, False
        else:
            saved, failed = (read * 1.15, False) if read > 0 else (0.0, True)
        found.append({"start": calls[head]["at"], "minutes": round(span / 60, 1), "arms": arms, "inside": len(inside),
                      "cost": cost, "saved": saved, "failed": failed, "write": write, "open_end": ended_without_external})
        index = tail if tail > head else head + 1
    return found


def main(directories):
    total = {"chains": 0, "long": 0, "cost": 0.0, "saved": 0.0, "failed": 0, "failed_cost": 0.0, "agents": 0}
    print("agent\t链开始(UTC)\t链长(分)\t起计时器\t链里调用\t花掉\t省下上界\t备注")
    for directory in directories:
        for path in sorted(glob.glob(os.path.join(directory, "*.jsonl"))):
            found = chains(parse_calls(path))
            if found:
                total["agents"] += 1
            for chain in found:
                total["chains"] += 1
                total["cost"] += chain["cost"]
                total["saved"] += chain["saved"]
                total["long"] += 1 if chain["minutes"] > 5 else 0
                note = ""
                if chain["failed"]:
                    total["failed"] += 1
                    total["failed_cost"] += chain["write"] * 1.25
                    note = f"链尾仍整份重写，写 {chain['write']}"
                elif chain["open_end"]:
                    note = "链头之后没有外部输入（交回或被停）"
                print(f"{os.path.basename(path)[-23:-6]}\t{chain['start']:%m-%d %H:%M}\t{chain['minutes']}\t{chain['arms']}\t{chain['inside']}\t{chain['cost']:.0f}\t{chain['saved']:.0f}\t{note}")
    print(f"合计：{total['agents']} 个 agent 起过计时器，{total['chains']} 条链（长于 5 分钟 {total['long']} 条）；"
          f"链里调用花掉 {total['cost']:.0f}；省下至多 {total['saved']:.0f}；链尾仍整份重写 {total['failed']} 次（{total['failed_cost']:.0f}）；"
          f"净 {total['saved'] - total['cost']:+.0f}（基础输入 token 当量）")


if __name__ == "__main__":
    main(sys.argv[1:])
