// §11.2.1.1: 24-byte non-time packet, zero Data Length, checksum
// EB25 + 0018 + 0900 = F43D. Use valid input for output/parser tests.
pub const NO_TIME: [u8; 24] = [
    0x25, 0xEB, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0x3D, 0xF4,
];
