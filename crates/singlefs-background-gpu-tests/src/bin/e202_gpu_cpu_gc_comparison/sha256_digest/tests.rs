use super::*;

fn sha256_hex_of(bytes: &[u8]) -> String {
    let mut digest = Sha256Digest::new();
    digest.feed(bytes);
    digest.finish_as_hex()
}

#[test]
fn sha256_of_the_standard_messages_matches_the_known_answers() {
    assert_eq!(
        sha256_hex_of(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex_of(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex_of(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn sha256_fed_in_uneven_pieces_equals_the_digest_fed_at_once() {
    let message: Vec<u8> = (0..1000_u32)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect();
    let mut digest = Sha256Digest::new();
    for piece in message.chunks(37) {
        digest.feed(piece);
    }
    assert_eq!(digest.finish_as_hex(), sha256_hex_of(&message));
}
