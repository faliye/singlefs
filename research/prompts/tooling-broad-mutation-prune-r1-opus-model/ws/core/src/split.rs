/// S1 被改文件：内容长度按容量切成几个单元（照 write_request_split::data_unit_count_of_a_sequential_write）。不引用任何常量。
pub fn units(length: u64, capacity: u64) -> u64 {
    if length == 0 {
        1
    } else {
        (length + capacity - 1) / capacity
    }
}
