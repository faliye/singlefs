S=16384; R=2
base_after_write_bytes=3357786112   # run-final-d-lines.txt 第 13 行：n3k1 sequential_write 之后 baseline_available_per_device（标签 4、6 取今天的 0）
D=[14,13,12,11]                     # S1 每次删除类发布每盘新写槽：意图 11+3、批1 13、批2 12、批3 12-2+1
names=["顺序写 3 单元之后","意图之后","批 1（留 2）之后","批 2（留 1）之后","批 3（留 0，删记录）之后"]
def table(W, label):
    P=sum(W)
    rows=[]
    base=base_after_write_bytes
    t6=P; t4=0
    rows.append((names[0], t6, t4, base - (t6+t4)*S))
    for j,d in enumerate(D):
        base-=d*S
        if j==0:
            t6=0; t4=sum(W[1:])
        else:
            t4-=W[j]
        rows.append((names[j+1], t6, t4, base-(t6+t4)*S))
    print(label, "P(每盘槽)=",P)
    for n,a,b,av in rows:
        print(f"  {n}\t标签6={a*S*R}\t标签4={b*S*R}\t每盘可用={av}\t(每盘 6/4 份额={a},{b} 槽)")
table([14,13,12,11],"策略 X（逐发布实需）")
table([14,14,14,14],"策略 Y（常数 W=14）")
print("紧池：策略 X，写之后可用=0。每次删除类发布 j 的准入：朴素=按发布前读数（标签已扣）判需求；抵扣=按本次发布之后的标签判")
av=0; t=[14,13,12,11]
for j in range(4):
    before=av
    naive = before >= t[j]
    offset = before + t[j] >= t[j]
    av = before + t[j] - t[j]
    print(f"  发布 {j}: 发布前可用={before} 槽，需求={t[j]} 槽；朴素{'放行' if naive else '拒'}；抵扣{'放行' if offset else '拒'}；发布后可用={av}")
for label,ns in (("E-c(deleted_inodes 零条目根)",[24,24,22,22]),("E-b(deleted_inodes 根指针置零)",[24,24,22,20])):
    inc=[2**n+11 for n in ns]
    print(label, inc, sum(inc))
print("r1 正推 F-1 估", 3*(2**22+11)+(2**24+11))
# 预付对数据的比（每盘槽）：W 取本文第 3.1 节量到的各发布需求，S1 另加：意图 +3、末批 −2 +1
print("N=3 K=1:", 14+13+12+11, "/", 3*2, "=", (14+13+12+11)/(3*2))
print("N=300 K=144 连续:", 15+18+15+13, "/", 300*2, "=", (15+18+15+13)/(300*2))
print("N=144 K=144 散落:", 18+263, "/", 144*2, "=", (18+263)/(144*2))
print("判决 K-U4 行的 13 槽 / (144 单元 × 2 槽) =", 13/(144*2), "；K=144 连续一批 18 槽 / 288 =", 18/288)
print("字面最坏上界（算的）：分配记录树 2*min(K+26,262)+5；K=1:", 2*min(1+26,262)+5, "K=144:", 2*min(144+26,262)+5)
