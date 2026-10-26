// 扫描里的校验和核对，两份独立核里按 32 位字逐位移算的那一份（D26（后台整理与放置回收） 已定项 13 第 8 条：两份核不共用核函数）。
// 每条一个调用：这一条开头不在字边界上的几个字节逐字节吸进寄存器，中间整字一次异或进寄存器再移 32 位，尾巴几个字节再逐字节吸，
// 每一位都按「最低位为 1 就异或多项式」用掩码算，不查表、不用工作组内存。
// 另一份（checksum_kernel_table_driven.wgsl）按字节查表；两份只共用缓冲的布局，布局由 src/checksum_kernels.rs 定。

struct DescriptorOfAnEntry {
    starting_byte: u32,
    length_in_bytes: u32,
    crc32c_stored_for_the_entry: u32,
}

struct OutcomeOfAnEntry {
    crc32c_of_the_bytes: u32,
    word_saying_whether_it_matches: u32,
}

@group(0) @binding(0) var<uniform> number_of_entries_in_the_chunk: u32;
@group(0) @binding(1) var<storage, read> packed_unit_bytes: array<u32>;
@group(0) @binding(2) var<storage, read> descriptors_of_the_entries: array<DescriptorOfAnEntry>;
@group(0) @binding(3) var<storage, read_write> outcomes_of_the_entries: array<OutcomeOfAnEntry>;

// CRC-32C（Castagnoli）反射多项式。
const REFLECTED_CASTAGNOLI_POLYNOMIAL: u32 = 0x82F63B78u;

// 移一位：最低位是 1 时掩码全 1、异或多项式，是 0 时掩码全 0。
fn shift_the_register_by_one_bit(register_before: u32) -> u32 {
    let mask_when_the_lowest_bit_is_set = 0u - (register_before & 1u);
    return (register_before >> 1u) ^ (REFLECTED_CASTAGNOLI_POLYNOMIAL & mask_when_the_lowest_bit_is_set);
}

fn absorb_one_byte(register_before: u32, byte_value: u32) -> u32 {
    var register_after = register_before ^ byte_value;
    for (var shifted_bits = 0u; shifted_bits < 8u; shifted_bits = shifted_bits + 1u) {
        register_after = shift_the_register_by_one_bit(register_after);
    }
    return register_after;
}

fn byte_at(byte_address: u32) -> u32 {
    let word_holding_it = packed_unit_bytes[byte_address / 4u];
    return extractBits(word_holding_it, (byte_address % 4u) * 8u, 8u);
}

@compute @workgroup_size(64)
fn crc32c_by_shifting_whole_words(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let entry_number = invocation.x;
    if (entry_number >= number_of_entries_in_the_chunk) {
        return;
    }
    let descriptor = descriptors_of_the_entries[entry_number];
    let byte_address_past_the_entry = descriptor.starting_byte + descriptor.length_in_bytes;
    var crc_register = 0xFFFFFFFFu;
    var byte_address = descriptor.starting_byte;
    // 开头不在字边界上的那几个字节
    while (byte_address < byte_address_past_the_entry && (byte_address % 4u) != 0u) {
        crc_register = absorb_one_byte(crc_register, byte_at(byte_address));
        byte_address = byte_address + 1u;
    }
    // 中间的整字：小端的一个字等于依次吸进它的四个字节
    while (byte_address_past_the_entry - byte_address >= 4u) {
        crc_register = crc_register ^ packed_unit_bytes[byte_address / 4u];
        for (var shifted_bits = 0u; shifted_bits < 32u; shifted_bits = shifted_bits + 1u) {
            crc_register = shift_the_register_by_one_bit(crc_register);
        }
        byte_address = byte_address + 4u;
    }
    // 尾巴
    while (byte_address < byte_address_past_the_entry) {
        crc_register = absorb_one_byte(crc_register, byte_at(byte_address));
        byte_address = byte_address + 1u;
    }
    let crc32c_of_the_bytes = ~crc_register;
    outcomes_of_the_entries[entry_number] = OutcomeOfAnEntry(
        crc32c_of_the_bytes,
        select(0u, 1u, crc32c_of_the_bytes == descriptor.crc32c_stored_for_the_entry),
    );
}
