import re,sys,os,json
T='/tmp/claude-1000/m3-leftover-code-r1/tree/'
R='/home/fy5090/code/singlefs/'
def norm(s): return re.sub(r'\s+',' ',s.replace('**','').replace('`','')).strip()
K={}
def add(key,path,quote,nth=None,root=T):
    K[key]=(root+path if not path.startswith('/') else path,quote,nth)
# --- mount.rs
M='crates/singlefs-core/src/mount.rs'
add('m_true',M,'| WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => true,')
add('m_lt',M,'< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE')
add('m_cmp',M,'let comparison = if some_system_configuration_slot_unread {')
add('m_wfn',M,'fn newer_publish_witness<Reader: PoolReader + ?Sized>(')
add('m_reread_true',M,'if reread_against_the_witness.witnesses_a_publish_newer_than_the_selected_version() {')
add('m_second_fn',M,'fn refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(')
add('m_err_device',M,'return Err(device);')
add('m_iso',M,'fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(')
add('m_unread_inc',M,'unreadable += 1;')
add('m_computed',M,'fn placements_referenced_by_root_computed_from_the_mapping<Device: BlockDevice>(')
add('m_account_err',M,'StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {',nth=1)
add('m_restore',M,'Ok(computed) => computed.abandoned_roots_unreadable,')
add('m_rebuilt',M,'let rebuilt = rebuilt_allocator(')
add('m_acq',M,'let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(')
add('m_devices_without',M,'let devices_without = devices_without_the_selected_version(')
# --- transaction.rs
X='crates/singlefs-core/src/transaction.rs'
add('t_fn',X,'fn self_verified_system_configurations_of_every_device<Device: BlockDevice>(')
add('t_lt',X,'< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE')
add('t_call1',X,'self_verified_system_configurations_of_every_device(pool)?;')
add('t_witness',X,'highest_journal_tail_of(&self_verified_system_configurations_of_every_device(pool)?);')
# --- recovery.rs
V='crates/singlefs-core/src/recovery.rs'
add('r_fn',V,'fn effective_rollback_floor_of_the_roots_read<Reader: PoolReader + ?Sized>(')
add('r_ab',V,'let highest_on_the_abandoned_roots = roots_with_ring_slots')
add('r_chain',V,'.chain(highest_on_the_abandoned_roots)')
add('r_recover',V,'pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {')
add('r_choose',V,'pub fn choose_root(')
# --- walk.rs
W='crates/singlefs-checker/src/walk.rs'
add('w_ab',W,'let highest_floor_on_the_abandoned_roots = roots')
add('w_max',W,'.max(highest_floor_on_the_abandoned_roots);')
add('w_defer_fn',W,'fn deferred_slots_referenced_only_below_the_floor_per_device(')
add('w_gen',W,'if record.is_released && record.generation > effective_rollback_floor {')
add('w_walked',W,'if !walked_slots.contains(&key) && released_above_the_floor.contains(&key) {')
add('w_sum',W,'+ deferred_below_the_floor;')
add('w_below',W,'&& root.checkpoint_txg < effective_rollback_floor')
# --- device_log.rs
D='crates/singlefs-checker-tier/src/device_log.rs'
add('d_findmap',D,'.find_map(|(declared_offset, declared_length)| {')
add('d_chunk',D,'let chunk_index = (position - fill_start) / ZERO_FILL_CHUNK_BYTES;')
add('d_guard',D,'if declared_length == 0 {')
add('d_cont',D,'let continues_at_cursor = piece.start == covered_up_to && piece.end <= fill_end;')
add('d_ahead',D,'piece.start > covered_up_to && piece.end <= chunk_of_cursor.end;')
add('d_overlap',D,'.any(|earlier| earlier.start < piece.end && piece.start < earlier.end);')
add('d_assert',D,'covered_up_to, fill_end,')
add('d_hash',D,'if *content_hash != fnv1a_64_of_zeros(*length) {')
add('d_first',D,'fn zero_fill_folds_back_into_one_event_however_the_block_layer_split_it() {')
add('d_decl',D,'pub fn declared_zero_fills(')
B='crates/singlefs-core/src/block_device.rs'
add('b_loop',B,'let this_chunk = (length - written).min(length_of(zeros));')
add('b_scratch',B,'let scratch = AlignedScratch::new(zero_fill_chunk_length(length));')
add('b_len',B,'fn zero_fill_chunk_length(length: u64) -> usize {')
add('b_const',B,'pub const ZERO_FILL_CHUNK_BYTES: u64 = 4 * 1024 * 1024;')
add('b_punch',B,'HolePunchFailure::PunchingUnsupported => {')
# --- crash_amplification.rs
C='crates/singlefs-checker-tier/src/crash_amplification.rs'
add('ca_fn',C,'fn assert_journal_recovery_of_the_base_image_lands_on_its_newest_root(')
add('ca_rec',C,'let landed_root = recover(&base_with_no_write_persisted, JournalPolicy::Consult)')
add('ca_newest',C,'fn newest_root_in_the_base_image(base: &MemoryPool) -> Option<(CheckpointTxg, InstanceGeneration)> {')
add('ca_magic',C,'bytes.len() >= ROOT_RECORD_IDENTITY_END && bytes[..ROOT_MAGIC.len()] == ROOT_MAGIC')
add('ca_call',C,'let newest_root_of_the_base = newest_root_in_the_base_image(flow.base);')
add('ca_test','crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs','fn a_flow_whose_base_image_is_a_crash_state_with_a_journal_record_past_its_newest_root_is_refused_at_planning(')
# --- crash_identity.rs
I='crates/singlefs-checker-tier/src/crash_identity.rs'
add('ci_table',I,'pub const RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED: &[RandomHistoryThroughALibraryModuleAllowed] = &[')
add('ci_islib',I,'fn is_a_library_module(file: &str) -> bool {')
add('ci_tt',I,'return RandomHistoryIntroducerVerdict::TestTargetFile;')
add('ci_crate',I,'name if crates.contains_key(name) => {')
add('ci_mod',I,'if text_at(position) == Some("mod") && text_at(position + 2) == Some(";") && is_test_crate {')
add('ci_dead',I,'登记表里 {} 经 {} 引随机历史脚手架那一行，在这条用例的判法闭包里没引到')
# --- numeric / history
N='crates/singlefs-harness/src/numeric_environment_variable.rs'
H='crates/singlefs-harness/src/history.rs'
add('n_trim',N,'match text.trim().parse::<Number>() {')
add('h_trim',H,'match text.trim().parse::<Number>() {')
add('h_count',H,'number_given_in_reproduction_environment::<std::num::NonZeroU64>(')
add('h_choice',H,'.find(|choice| environment_value_of(*choice) == value)')
add('h_ops',H,'let operations_per_history_given = count_from_reproduction_environment(')
J='crates/singlefs-checker-tier/src/crash_injection.rs'
add('j_none',J,'let draw = if short_segment_writes == SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE {')
add('j_nz',J,'crash_points_per_history: std::num::NonZeroUsize::new(points)')
add('j_pos',J,'segment_writes: positive_count_from_the_variable_text(')
add('j_seeds',J,'let seed_count = count_from_reproduction_environment_panicking_when_unreadable(')
add('j_first',J,'let first_seed = u64_from_reproduction_environment(')
add('f_faults','crates/singlefs-harness/tests/fault_injection_fast_tier.rs','let faults_given = count_from_reproduction_environment(')
add('b_seed','crates/singlefs-checker-tier/src/bad_disk_input.rs','let seed_count = count_from_reproduction_environment(')
add('rh_nz','crates/singlefs-harness/tests/random_histories.rs','number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroU64>(name, value)')
# --- pool
P='crates/singlefs-harness/tests/common_admission_pool/mod.rs'
add('p_384',P,'const UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 384;')
add('p_815',P,'const RING_WINDOW_ALLOWANCE_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 815;')
add('p_ring',P,'const JOURNAL_RING_BYTES_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 6 << 20;')
add('p_start',P,'pub fn start_after_the_first_file_on_devices_of(')
add('h_384',H,'const SMALL_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE: u64 = 384;')
add('h_815',H,'const SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 815;')
add('h_ring',H,'const SMALL_DEVICE_JOURNAL_RING_BYTES: u64 = 6 << 20;')
Q='crates/singlefs-harness/tests/common_admission/mod.rs'
add('q_assert',Q,'assert_the_384_slot_copy_matches_the_random_history_width();',nth=1)
add('q_widths',Q,'device_width.device_bytes(),')
add('q_path',Q,'#[path = "../common_admission_pool/mod.rs"]')
add('mut_384','crates/mutations.tsv','准入池抄的 384 槽那一档单元区槽数与 history.rs 那一档对不上')
add('mut_jia','crates/mutations.tsv','C331 甲（用户 2026-09-28 定）：可写挂载见证读缺一槽不判不出')
add('mut_v4','crates/mutations.tsv','C331 甲 V4（用户 2026-09-28 定）：取号那一刻缺一槽照样取号')
add('mut_b','crates/mutations.tsv','C393 (b)-从映射现算（用户 2026-09-28 定）：从映射一律现算不成')
add('mut_defer','crates/mutations.tsv','收口表第 43 行丁-defer（用户 2026-09-28 定）：I-3.1 的遍历不并 F 之下仍在 defer 的槽')
add('mut_f1','crates/mutations.tsv','收口表第 ② 行算进（用户 2026-09-28 定）：实现的 F_生效 不算被抛弃根带的 F')
add('mut_f2','crates/mutations.tsv','收口表第 ② 行算进（用户 2026-09-28 定）：checker 的 F_生效 不算被抛弃根带的 F')
add('mut_narrow','crates/mutations.tsv','崩溃放量 B0 前提 排计划时不断言基镜像照 journal 恢复落在它里面最新的根')
add('mut_wl','crates/mutations.tsv','判法闭包 随机历史脚手架的库模块白名单一律放行')
add('mut_dev_hash','crates/mutations.tsv','设备侧日志折清零段时不查每条是不是全 0')
add('mut_dev_past','crates/mutations.tsv','设备侧日志折清零段时最后一块不截到段尾')
add('t_jia_file','crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs','fn pool_with_one_system_configuration_slot_zeroed(')
add('t_c393_refuse','crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs','fn an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking(')
add('fi_left','crates/singlefs-harness/src/fault_injection.rs','pub fn left_by_the_lying_device(')
add('fi_refused','crates/singlefs-harness/src/fault_injection.rs','ModelDisagreementAspect::RefusedWhenModelRequiresSuccess',nth=1)
add('mc_expl','crates/singlefs-harness/src/model_comparison.rs','StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {',nth=1)
add('mk_slots','crates/singlefs-core/src/make_filesystem.rs','for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {')
# --- kb
D23='.claude/kb/decisions/23-journal的角色与格式.md'
D22='.claude/kb/decisions/22-单元原子性怎么合成.md'
D16='.claude/kb/decisions/16-发布语义.md'
add('k23_impl',D23,'这里的实现还是 2026-09-28 取甲的那一版',root=R)
add('k23_refuse',D23,'判不出就在第一个取号写之前拒',root=R)
add('k23_shadow',D23,'被抛弃根的账（树表或分配记录树）读不出时不跳过：从中央映射现算这条根引用的落点，只在内存里隔离、不写盘',root=R)
add('k23_shadow2',D23,'映射也现算不成就重读一次，仍不成在取号之前拒可写、只读照常',root=R)
add('k23_cost',D23,'撕裂的系统配置轮换（普通断电就造得出，被撕裂的那次发布没确认过）不再因判不出拒可写挂载',root=R)
add('k23_cost2',D23,'C393（被抛弃根的账读不出时修复没有条款） 从映射也现算不成的那一形同样只剩只读挂载',root=R)
add('k22_r2',D22,'判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（R2，用户定，被攻过零轮，原话在变更史）',root=R)
add('k22_promise',D22,'容错承诺是单故障：每盘一槽系统配置坏在承诺里',root=R)
add('k22_double',D22,'每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里（用户 2026-09-28 定不罩）',root=R)
add('k22_loss',D22,'丢写的界按类写：全池读回一个值的纯形丢写 120 格，一盘报读错、另一盘读回全 0 的混合形丢写 0 格',root=R)
add('k22_dec',D22,'用户定案：改取 R2（原话在变更史）',root=R)
add('k16_floor',D16,'恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 系统配置里读得出的 F 的最大值)',root=R)
add('k16_abn',D16,'被抛弃时间线上的根带的 F 算进 F_生效（用户 2026-09-28 定',root=R)
INV='.claude/kb/invariants.md'
add('inv_defer',INV,'F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于 F_生效 的',root=R)
add('inv_blind',INV,'那一类错，没拿变异试过',root=R)
CO='.claude/kb/checks-owed.md'
add('co_r2',CO,'定案改取 R2、实现在途：见证读判不出 ⟺ 有槽设备报读错且没有一块盘两槽都自证过',root=R)
add('co_grid',CO,'混合形（拒可写 / 可写不丢 / 可写丢）甲-读错-每盘 888 / 831 / 240，R2 1244 / 715 / 0',root=R)
add('co_notchanged',CO,'D23（journal 的角色与格式） 已定项 14 N-配置续那一段与实现还是 2026-09-28 取甲的那一版',root=R)
add('co_c596',CO,'R2 之后仍判不出的那一形（有槽读错、没有一块盘两槽都自证过）一次可写挂载都做不成',root=R)
add('co_c600',CO,'没有条款，用户 2026-09-29 定先记欠账',root=R)
add('co_c662',CO,'R2 进代码之后这一族生成的历史与条款不再是同一套写路径',root=R)
add('co_c600_row',CO,'| C600 | 吞写之后可写挂载被拒而模型说该成 |',root=R)
add('ms_c600','.claude/kb/milestone/03-third-txn.md','| 五·6 | 大档故障注入 2 条新发现',root=R)
add('rec_c331','records/2026-09-28-收口表对齐与三笔欠账复查.md','选甲 但是不需要再开三轮了 等之后开',root=R) 
add('rec_ding','records/2026-09-28-收口表对齐与三笔欠账复查.md','丁-defer（推荐）',root=R)
add('rec_suan','records/2026-09-28-收口表对齐与三笔欠账复查.md','算进（推荐）',root=R)
add('rec_narrow','records/2026-10-01-对接上游测试脚本规范.md','（只断言对基镜像照 journal 恢复落到的就是它里面最新的根）；第 3 条取',root=R)
add('rec_repro','records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md','种子 7463871032432355222 吞掉起点段的一次系统配置槽写（整池第 19 次写）',root=R)
add('rec_setup','records/2026-10-01-对接上游测试脚本规范.md','坐实。弹窗，用户 2026-10-02 定',root=R)

def resolve(key):
    path,quote,nth=K[key]
    lines=open(path,encoding='utf-8',errors='replace').read().splitlines()
    nq=norm(quote)
    hits=[i+1 for i,l in enumerate(lines) if nq in norm(l)]
    if not hits: raise SystemExit(f'NO HIT {key}: {quote}')
    if nth is None:
        if len(hits)!=1: raise SystemExit(f'{len(hits)} HITS {key}: {hits[:6]}')
        n=hits[0]
    else:
        if nth>len(hits): raise SystemExit(f'NTH {nth} > {len(hits)} for {key}')
        n=hits[nth-1]
    # 报告里写的路径：tree 绝对路径；仓内的写相对路径
    shown=path if path.startswith(T) else path[len(R):]
    return f'「{quote}」（`{shown}:{n}`）', shown, n

def render(text):
    def sub(m):
        key=m.group(1)
        return resolve(key)[0]
    return re.sub(r'\[\[(\w+)\]\]',sub,text)

def loc(key):
    c,shown,n=resolve(key)
    return f'`{shown}:{n}`'

if __name__=='__main__':
    bad=0
    for k in K:
        try:
            c,shown,n=resolve(k)
        except SystemExit as e:
            print(e); bad+=1
    print('keys',len(K),'bad',bad)
