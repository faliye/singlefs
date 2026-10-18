#!/usr/bin/env python3
"""乙（中央领块）的时序模型：本机当发块方，块按 segment 块一段切开，两台各自做完一段再领下一段，发块方记领用账。

账的三种状态：free、leased（给谁、什么时候）、done。「做完」照候选正文的读法由下一次领块请求带回（第二台每做完一段经 ssh 向本机领下一段）。
故障：领块的回话丢了（发块方记了 leased、第二台没收到，重发请求领到下一段）、第二台被停（term：取回了库；offline：库取不回）、
发块方（本机）重起（账在内存里就丢；账落盘就接着用）。超时 timeout 秒没回来的 leased 段收回重发（none 表示不收回）。
只算段的去向：几段两台都做了（重复）、几段谁都没做或做了的结果取不回（洞，导入那一趟对账为红、要再跑一趟）。
只用 python3 标准库；输出一行一格，键=值。
"""
import itertools


def simulate(segments, seconds_per_segment, request_seconds, timeout, fault, fault_at, ledger_on_disk):
    ledger = {s: ("free", None, None) for s in range(segments)}
    done_in = {"local": set(), "peer": set()}
    lost = set()           # 做完了但结果取不回（第二台掉线）
    hosts = {"local": {"segment": None, "until": 0.0, "alive": True, "awaiting_reply": False},
             "peer": {"segment": None, "until": 0.0, "alive": True, "awaiting_reply": False}}
    now = 0.0
    dispatcher_restarted = False
    lost_reply_used = False
    step = 1.0

    def lease(host, now):
        nonlocal lost_reply_used
        for s in range(segments):
            state, holder, since = ledger[s]
            expired = state == "leased" and timeout is not None and now - since > timeout
            if state == "free" or expired:
                ledger[s] = ("leased", host, now)
                if fault == "reply-lost" and host == "peer" and not lost_reply_used and now >= fault_at:
                    lost_reply_used = True
                    return None      # 账上记了给第二台，回话没到；第二台下一步重发
                return s
        return "nothing-left"

    while now < 10**6:
        if fault in ("peer-term", "peer-offline") and now >= fault_at and hosts["peer"]["alive"]:
            hosts["peer"]["alive"] = False
            if hosts["peer"]["segment"] is not None:
                hosts["peer"]["segment"] = None   # 手里那一段丢掉
            if fault == "peer-offline":
                lost |= done_in["peer"]
        if fault == "dispatcher-restart" and now >= fault_at and not dispatcher_restarted:
            dispatcher_restarted = True
            if not ledger_on_disk:
                # 账丢了：照本机库里有的重建，第二台做完的与在做的都当 free
                ledger = {s: (("done", "local", None) if s in done_in["local"] else ("free", None, None)) for s in range(segments)}
        for name, host in hosts.items():
            if not host["alive"]:
                continue
            if host["segment"] is not None and now >= host["until"]:
                done_in[name].add(host["segment"])
                ledger[host["segment"]] = ("done", name, now)
                host["segment"] = None
            if host["segment"] is None and now >= host["until"]:
                got = lease(name, now)
                if got == "nothing-left":
                    host["alive"] = False
                    host["finished"] = now
                    continue
                if got is None:
                    host["until"] = now + request_seconds
                    continue
                host["segment"] = got
                host["until"] = now + request_seconds + seconds_per_segment[name]
        if not any(h["alive"] for h in hosts.values()):
            break
        now += step
    reachable = done_in["local"] | (done_in["peer"] - lost)
    duplicated = len(done_in["local"] & done_in["peer"])
    holes = segments - len(reachable)
    ledger_says_done_but_unreachable = sum(1 for s, (state, _, _) in ledger.items() if state == "done" and s not in reachable)
    stuck_leases = sum(1 for s, (state, _, _) in ledger.items() if state == "leased")
    return {"duplicated_segments": duplicated, "holes": holes, "ledger_done_but_unreachable": ledger_says_done_but_unreachable,
            "stuck_leases": stuck_leases, "end": now}


def main():
    segments = 120
    # 一段的秒数：两台判速 8.8 万 / 17.5 万态每秒（记录 2026-10-17 那一条，量的），一段按 64 块 × 65536 态折
    per_segment = {"local": 64 * 65536 / 88000, "peer": 64 * 65536 / 175000}
    print(f"name=yi_parameters segments={segments} local_seconds_per_segment={per_segment['local']:.0f} peer_seconds_per_segment={per_segment['peer']:.0f}")
    for fault, timeout, ledger_on_disk in itertools.product(
            ("none", "reply-lost", "peer-term", "peer-offline", "dispatcher-restart"), (None, 30, 600), (False, True)):
        for fault_at in (200, 1500, 3000):
            row = simulate(segments, per_segment, 5, timeout, fault, fault_at, ledger_on_disk)
            print(f"name=yi_lease fault={fault} fault_at={fault_at} timeout={timeout if timeout is not None else 'none'} ledger_on_disk={ledger_on_disk} "
                  + " ".join(f"{key}={value:.0f}" if isinstance(value, float) else f"{key}={value}" for key, value in row.items()))


if __name__ == "__main__":
    main()
