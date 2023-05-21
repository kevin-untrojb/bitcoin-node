use crate::errores::NodoBitcoinError;

pub fn parse_varint(bytes: &[u8]) -> (usize, usize) {
    let prefix = bytes[0];
    match prefix {
        0xfd => (3, u16::from_le_bytes([bytes[1], bytes[2]]) as usize),
        0xfe => (
            5,
            u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize,
        ),
        0xff => (
            9,
            u64::from_le_bytes([
                bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            ]) as usize,
        ),
        _ => (1, u64::from(prefix) as usize),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_varint() {
        // Test case 1: Prefix = 0xfd (2 bytes)
        let bytes1: [u8; 3] = [0xfd, 0xab, 0xcd];
        let (size1, value1) = parse_varint(&bytes1);
        assert_eq!(size1, 3);
        assert_eq!(value1, 0xcdab);

        // Test case 2: Prefix = 0xfe (4 bytes)
        let bytes2: [u8; 5] = [0xfe, 0x12, 0x34, 0x56, 0x78];
        let (size2, value2) = parse_varint(&bytes2);
        assert_eq!(size2, 5);
        assert_eq!(value2, 0x78563412);

        // Test case 3: Prefix = 0xff (8 bytes)
        let bytes3: [u8; 9] = [
            0xff, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        ];
        let (size3, value3) = parse_varint(&bytes3);
        assert_eq!(size3, 9);
        assert_eq!(value3, 0xefcdab8967452301);

        // Test case 4: Prefix = 0x01 (1 byte)
        let bytes4: [u8; 1] = [0x01];
        let (size4, value4) = parse_varint(&bytes4);
        assert_eq!(size4, 1);
        assert_eq!(value4, 0x01);
    }
}