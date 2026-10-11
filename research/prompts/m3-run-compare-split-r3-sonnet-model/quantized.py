import sys
sys.path.insert(0,'.')
from weighted_shares_drift import cycle, owner_of_block
def quant(w, total):
    s=sum(w); raw=[x*total/s for x in w]; q=[int(r) for r in raw]
    rem=total-sum(q)
    for i in sorted(range(len(w)), key=lambda i: raw[i]-q[i], reverse=True)[:rem]: q[i]+=1
    return q
pairs=[((1393344,988839),(1427848,980112)),((35453,20156),(52822,20838)),((52937,20951),(119814,130951)),((4544035,5880702),(611669,813458)),((136700,45275),(140427,62390))]
for total in (8,16,32,64):
    for a,b in pairs:
        qa,qb=quant(a,total),quant(b,total)
        ca,cb=cycle(qa),cycle(qb)
        n=39917
        moved=sum(1 for k in range(n) if owner_of_block(ca,k)!=owner_of_block(cb,k))
        print(f"QUANT total={total} {a}->{qa} {b}->{qb} moved_fraction={moved/n:.4f}")
