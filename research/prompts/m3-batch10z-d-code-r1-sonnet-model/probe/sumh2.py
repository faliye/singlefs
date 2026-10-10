import sys,re
for line in open(sys.argv[1]):
    if not line.startswith("RESULT") or "超出" in line: continue
    m=re.match(r"RESULT (\S+)\[组(\d+)/(\d+)组,R单元(\d+)个,不在C里(\d+)\]: 择P=(\w+) 表记被抛弃落点=(\S+) C借来=(\S+) 挂后R单元空闲=(\d+) \| 会话写(\d+)/(\d+) 被拒=(.*?) 推抬F串=(\d+) R在环里时空闲/被覆盖的步=(\[.*?\]) 逐步首违例=(.*?) 写满=(.*?) R在环里\(写后\)=(\w+) 被覆盖\(写后\)=(\d+) 违例\(写后\)=(\[.*?\]) \| 卸载=(.*?) R在环里=(\w+) 空闲\(卸载后\)=(\S+) 被覆盖=(\d+) 违例\(卸载后\)=(.*)$",line.strip())
    if not m: print("RAW",line[:300]); continue
    g=m.groups()
    ref=re.search(r"(RollbackFloorCeiling\w+|NewerState\w+|NoSpace\w+|\w+Refused\w*)",g[11]); 
    print(f"{g[0]} g{g[1]:>2}/{g[2]} 择P={g[5]:5} 记被抛弃={g[6]:5} C借={g[7]:9} 不在C={g[4]:>2} 挂后空闲={g[8]:>2} 写{g[9]}/{g[10]} 拒={ref.group(1) if ref else g[11][:20]} 抬F={g[12]} 险步={g[13]} 首违例={g[14][:60]} R环内(写后)={g[16]} 违例写后={g[18]} 卸载={g[19][:30]} 违例卸载后={g[23][:80]}")
