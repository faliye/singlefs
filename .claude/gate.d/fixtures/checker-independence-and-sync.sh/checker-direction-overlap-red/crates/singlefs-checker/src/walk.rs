use singlefs_format::{slot_offset, SLOT_BYTES};

use crate::rebuild::rebuild_index;
use crate::{read_u64, ImageReader, HEADER_MAGIC};

pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<String> {
    judge_pool_image(reader)
}

fn judge_pool_image(reader: &dyn ImageReader) -> Vec<String> {
    let mut verdicts = vec![judge_root_record(reader)];
    let scanned = scanned_units(reader);
    verdicts.push(judge_rebuild_agrees_with_the_walked_index(reader, &scanned));
    verdicts
}

fn root_record_is_well_formed(root: &[u8]) -> bool {
    root.len() >= 8 && read_u64(root, 0) != 0
}

fn judge_root_record(reader: &dyn ImageReader) -> String {
    let root = reader.read(slot_offset(0)).unwrap_or_default();
    if root_record_is_well_formed(&root) {
        String::from("I-7.2 成立")
    } else {
        String::from("I-7.2 违例")
    }
}

fn walked_index(reader: &dyn ImageReader) -> Vec<u64> {
    let root = reader.read(SLOT_BYTES).unwrap_or_default();
    vec![read_u64(&root, 0)]
}

fn judge_rebuild_agrees_with_the_walked_index(reader: &dyn ImageReader, scanned: &[Vec<u8>]) -> String {
    if walked_index(reader) == rebuild_index(scanned) {
        String::from("I-1.13 成立")
    } else {
        String::from("I-1.13 违例")
    }
}

fn unit_of_header(header: &[u8]) -> Option<Vec<u8>> {
    if header.len() < 8 || &header[..4] != HEADER_MAGIC {
        return None;
    }
    if !root_record_is_well_formed(header) {
        return None;
    }
    Some(header.to_vec())
}

fn scanned_units(reader: &dyn ImageReader) -> Vec<Vec<u8>> {
    (0..4)
        .filter_map(|slot| reader.read(slot_offset(slot)))
        .filter_map(|header| unit_of_header(&header))
        .collect()
}
