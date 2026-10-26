// 扫描里的校验和核对，两份独立核里按字节查表的那一份（D26（后台整理与放置回收） 已定项 13 第 8 条：两份核不共用核函数）。
// 每条一个调用：从输入里取这一条的字节，逐字节查 256 项余式表算 CRC-32C，与期望值比，写回算出的值与判定。
// 余式表不抄常数：每个工作组开头按多项式现算进工作组内存（每个调用算 4 项），屏障之后再用。
// 另一份（checksum_kernel_bitwise.wgsl）按 32 位字逐位移算，不查表；两份只共用缓冲的布局，布局由 src/checksum_kernels.rs 定。

struct EntryDescriptor {
    byte_offset: u32,
    byte_length: u32,
    expected_crc32c: u32,
}

struct KernelOutcome {
    computed_crc32c: u32,
    verdict_word: u32,
}

@group(0) @binding(0) var<uniform> entry_count_of_the_chunk: u32;
@group(0) @binding(1) var<storage, read> input_words: array<u32>;
@group(0) @binding(2) var<storage, read> entry_descriptors: array<EntryDescriptor>;
@group(0) @binding(3) var<storage, read_write> kernel_outcomes: array<KernelOutcome>;

// CRC-32C（Castagnoli）反射多项式。
const CASTAGNOLI_REFLECTED_POLYNOMIAL: u32 = 0x82F63B78u;
const INVOCATIONS_PER_WORKGROUP: u32 = 64u;
const REMAINDER_TABLE_ENTRIES: u32 = 256u;
const VERDICT_WORD_MATCHES: u32 = 1u;
const VERDICT_WORD_DOES_NOT_MATCH: u32 = 0u;

var<workgroup> byte_remainder_table: array<u32, 256>;

@compute @workgroup_size(64)
fn crc32c_by_byte_remainder_table(
    @builtin(global_invocation_id) global_invocation: vec3<u32>,
    @builtin(local_invocation_index) invocation_in_the_workgroup: u32,
) {
    for (var table_slot = invocation_in_the_workgroup; table_slot < REMAINDER_TABLE_ENTRIES; table_slot = table_slot + INVOCATIONS_PER_WORKGROUP) {
        var remainder = table_slot;
        for (var bit_round = 0u; bit_round < 8u; bit_round = bit_round + 1u) {
            if ((remainder & 1u) == 1u) {
                remainder = (remainder >> 1u) ^ CASTAGNOLI_REFLECTED_POLYNOMIAL;
            } else {
                remainder = remainder >> 1u;
            }
        }
        byte_remainder_table[table_slot] = remainder;
    }
    workgroupBarrier();

    let entry_position = global_invocation.x;
    if (entry_position >= entry_count_of_the_chunk) {
        return;
    }
    let descriptor = entry_descriptors[entry_position];
    let end_of_the_entry = descriptor.byte_offset + descriptor.byte_length;
    var crc_register = 0xFFFFFFFFu;
    for (var byte_position = descriptor.byte_offset; byte_position < end_of_the_entry; byte_position = byte_position + 1u) {
        let containing_word = input_words[byte_position >> 2u];
        let byte_of_the_entry = (containing_word >> ((byte_position & 3u) * 8u)) & 0xFFu;
        crc_register = (crc_register >> 8u) ^ byte_remainder_table[(crc_register ^ byte_of_the_entry) & 0xFFu];
    }
    let computed_crc32c = crc_register ^ 0xFFFFFFFFu;
    var verdict_word = VERDICT_WORD_DOES_NOT_MATCH;
    if (computed_crc32c == descriptor.expected_crc32c) {
        verdict_word = VERDICT_WORD_MATCHES;
    }
    kernel_outcomes[entry_position] = KernelOutcome(computed_crc32c, verdict_word);
}
