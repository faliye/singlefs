"""G8：整理挪出调用线程之后，今天调用线程上那一趟给的两条保证还剩不剩。
照 crates/singlefs-core/src/mounted_session.rs 的 run_compaction_on 那一圈（预算 max(1, 3S-4-已过去的发布数)、
「上一条完成的意图」是一趟里的局部变量、用户改动把已过去的发布数清零、用户改动之后那一趟至多一批）
与 compaction_plan.rs 的 publish_budget_of_a_compaction_run 抄成状态机；整理的活写成一串意图，每条签发一次、搬 b 批（最后一批完成意图），
每条完成之后都不增加全空聚簇段（最坏：签了也腾不出段）。
起跑办法三种：今天（只在用户改动之后起、至多一批）；后台按定时兜底每一拍起一趟显式入口（run_compaction，无一批之限）；
后台每次发布之后查水位就起（D26 已定项 13 第 5 条「每次发布之后」，整理自己的发布也算发布）。
用户改动的间隔是用户定的，放开扫：只在开头一次、每 1、5、50 拍一次。
报：两次用户改动之间连着的整理发布最多几次（与 3S-4、3S 比）；完成了一条不增全空段的意图之后又签了几条。"""
import sys

S = 8
LIMIT = 3 * S - 4

def budget(passed):
    return max(1, 3 * S - 4 - passed)

class Pool:
    def __init__(self, batches_per_intent):
        self.b = batches_per_intent
        self.live = None          # 活意图里还剩几批
        self.passed = 0           # 已过去的发布数（用户改动清零）
        self.run_of_gc = 0        # 当前连着的整理发布数
        self.max_run_of_gc = 0
        self.issued_after_a_no_yield = 0
        self.completed_without_yield = 0
        self.publishes = 0
    def gc_publish(self):
        self.passed += 1; self.run_of_gc += 1; self.publishes += 1
        self.max_run_of_gc = max(self.max_run_of_gc, self.run_of_gc)
    def user_change(self):
        self.passed = 0; self.run_of_gc = 0; self.publishes += 1

def run(pool, one_batch):
    """一趟：照 run_compaction_on 的循环。返回这一趟发了几次。"""
    last_completed = None           # 一趟里的局部变量（mounted_session.rs 那一行 let mut last_completed_intent = None）
    published = batches = 0
    limit = budget(pool.passed)
    while True:
        if published >= limit:
            return published
        if one_batch and batches == 1:
            return published
        if pool.live is None:
            if last_completed is not None and last_completed == "no_yield":
                return published    # NoYieldFromTheLastIntent
            if pool.completed_without_yield > 0:
                pool.issued_after_a_no_yield += 1
            pool.live = pool.b
            pool.gc_publish(); published += 1
            continue
        pool.live -= 1
        pool.gc_publish(); published += 1; batches += 1
        if pool.live == 0:
            pool.live = None
            last_completed = "no_yield"
            pool.completed_without_yield += 1

def simulate(policy, user_interval, ticks, b):
    pool = Pool(b)
    for tick in range(ticks):
        user_now = (tick == 0) if user_interval is None else (tick % user_interval == 0)
        if user_now:
            pool.user_change()
            if policy == "today":
                run(pool, one_batch=True)
        if policy == "background_timer":
            run(pool, one_batch=False)
        elif policy == "background_after_every_publish":
            # 每发成一次就查一次水位、越线就再起一趟；一拍里至多推 1000 次，免得模型自己不停
            steps = 0
            while steps < 1000:
                n = run(pool, one_batch=False)
                steps += 1
                if n == 0:
                    break
    return pool

def main():
    ticks = int(sys.argv[1]) if len(sys.argv) > 1 else 200
    for b in (2, 6):
        for policy in ("today", "background_timer", "background_after_every_publish"):
            for interval in (None, 1, 5, 50):
                p = simulate(policy, interval, ticks, b)
                print("name=g8_loop batches_per_intent=%d policy=%s user_change_every=%s ticks=%d publishes=%d max_consecutive_gc_publishes=%d budget_line=%d root_ring_slots=%d over_budget_line=%s intents_issued_after_an_intent_without_yield=%d" % (
                    b, policy, "once" if interval is None else interval, ticks, p.publishes, p.max_run_of_gc, LIMIT, 3 * S,
                    "true" if p.max_run_of_gc > LIMIT else "false", p.issued_after_a_no_yield))

main()
