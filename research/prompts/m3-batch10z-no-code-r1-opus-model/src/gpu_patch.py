#!/usr/bin/env python3
"""m3-batch10z-no-code-r1 攻方：给一份 crates 副本的 gpu_memory_ledger.rs 打 GPU 一侧的探针。用法：gpu_patch.py <副本根> <模型目录>。
① 借之前那一步（take_in_the_devices_kept_before_booking）加只看 OPUS_NO_GPU_FN_ARM 的七条臂（listed_only 第十五块之前的走法、skip_last、first_only、
lost_only_listed、break_after_lost、lost_then_poll、poll_all），不设就是入库原样；② 测试模块里加两条攻方用例（gpu_model.rs、gpu_stress.rs）。
每一处替换都核命中次数，对不上退出 3。"""
import sys
root, model = sys.argv[1], sys.argv[2]
p = root + '/crates/singlefs-harness/src/gpu_memory_ledger.rs'
s = open(p).read()
def rep(old, new):
    global s
    n = s.count(old)
    if n != 1:
        print("命中次数不对", n, repr(old[:80])); sys.exit(3)
    s = s.replace(old, new)
rep('''        let listed_cards = self.leases_kept_after_a_failed_wait_on_each_card();
        let card_count = self.books_locked().len();
        for card_position in 0..card_count {
            let Some(kept) = device_kept_on_the_card(card_position) else {
                continue;
            };''', '''        let listed_cards = self.leases_kept_after_a_failed_wait_on_each_card();
        let card_count = self.books_locked().len();
        let opus_arm = std::env::var("OPUS_NO_GPU_FN_ARM").unwrap_or_default();
        let positions: Vec<usize> = match opus_arm.as_str() {
            "listed_only" => listed_cards.iter().map(|l| l.card_position).collect(),
            "skip_last" => (0..card_count.saturating_sub(1)).collect(),
            "first_only" => (0..card_count.min(1)).collect(),
            _ => (0..card_count).collect(),
        };
        for card_position in positions {
            let Some(kept) = device_kept_on_the_card(card_position) else {
                continue;
            };
            if opus_arm == "lost_only_listed" && !listed_cards.iter().any(|l| l.card_position == card_position) {
                continue;
            }''')
rep('''            if is_lost(&kept) {
                kept_on_the_books.take_in_that_the_device_was_found_lost();
                continue;
            }
            let Some(listed) = listed_cards''', '''            if is_lost(&kept) {
                kept_on_the_books.take_in_that_the_device_was_found_lost();
                if opus_arm == "break_after_lost" {
                    break;
                }
                if opus_arm != "lost_then_poll" {
                    continue;
                }
            }
            if opus_arm == "poll_all" && !listed_cards.iter().any(|l| l.card_position == card_position) {
                let _ = found_idle_by_a_poll(&kept);
                continue;
            }
            let Some(listed) = listed_cards''')
anchor = "    /// 账接一张 [`CardUnderTheLedger`]：没配额度、没配优先级（驱动之外照能占的算）。\n"
rep(anchor, open(model + '/gpu_model.rs').read() + open(model + '/gpu_stress.rs').read() + '\n' + anchor)
open(p, 'w').write(s)
print("打上了：gpu")
