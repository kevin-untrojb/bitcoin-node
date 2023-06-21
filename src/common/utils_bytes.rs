use crate::errores::NodoBitcoinError;

pub const PREFIX_FD: u8 = 0xfd;
pub const PREFIX_FE: u8 = 0xfe;
pub const PREFIX_FF: u8 = 0xff;

pub fn parse_varint(bytes: &[u8]) -> (usize, usize) {
    let prefix = bytes[0];
    match prefix {
        PREFIX_FD => (3, u16::from_le_bytes([bytes[1], bytes[2]]) as usize),
        PREFIX_FE => (
            5,
            u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize,
        ),
        PREFIX_FF => (
            9,
            u64::from_le_bytes([
                bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            ]) as usize,
        ),
        _ => (1, u64::from(prefix) as usize),
    }
}

pub fn from_amount_bytes_to_prefix(nbytes: usize) -> u8 {
    match nbytes {
        3 => PREFIX_FD,
        5 => PREFIX_FE,
        9 => PREFIX_FF,
        _ => 1,
    }
}

pub fn _build_varint_bytes(prefix: u8, value: usize) -> Result<Vec<u8>, NodoBitcoinError> {
    match prefix {
        PREFIX_FD => {
            let value_bytes = (value as u16).to_le_bytes();
            let bytes = vec![0xfd, value_bytes[0], value_bytes[1]];
            Ok(bytes)
        }
        PREFIX_FE => {
            let value_bytes = (value as u32).to_le_bytes();
            let bytes = vec![
                0xfe,
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
            ];
            Ok(bytes)
        }
        PREFIX_FF => {
            let value_bytes = (value as u64).to_le_bytes();
            let bytes = vec![
                0xff,
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
                value_bytes[4],
                value_bytes[5],
                value_bytes[6],
                value_bytes[7],
            ];
            Ok(bytes)
        }
        _ => {
            if value <= 255 {
                let value_byte = value as u8;
                let bytes = vec![value_byte];
                Ok(bytes)
            } else {
                Err(NodoBitcoinError::ValorFueraDeRango)
            }
        }
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
        let bytes3: [u8; 9] = [0xff, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let (size3, value3) = parse_varint(&bytes3);
        assert_eq!(size3, 9);
        assert_eq!(value3, 0xefcdab8967452301);

        // Test case 4: Prefix = 0x01 (1 byte)
        let bytes4: [u8; 1] = [0x01];
        let (size4, value4) = parse_varint(&bytes4);
        assert_eq!(size4, 1);
        assert_eq!(value4, 0x01);
    }

    #[test]
    fn test_build_varint_bytes() {
        // Prefix: 0xfd, Value: 123
        let value_fd = 123;
        let expected_bytes_fd = vec![0xfd, 0x7b, 0x00];

        // Prefix: 0xfe, Value: 987654321
        let value_fe = 102003;
        let expected_bytes_fe = vec![0xfe, 0x73, 0x8E, 0x01, 0x00];

        // Prefix: 0xff, Value: 1234567890123456789
        let value_ff = 1234567890123456789;
        let expected_bytes_ff = vec![0xff, 0x15, 0x81, 0xe9, 0x7d, 0xf4, 0x10, 0x22, 0x11];

        // Prefix: Default (0x01), Value: 42
        let prefix_default = 0x01;
        let value_default = 42;
        let expected_bytes_default = vec![0x2a];

        // Value too large
        let prefix_large = 0x03;
        let value_large = usize::max_value();

        // Test _build_varint_bytes function
        let result_fd = _build_varint_bytes(PREFIX_FD, value_fd).unwrap();
        assert_eq!(result_fd, expected_bytes_fd);

        let result_fe = _build_varint_bytes(PREFIX_FE, value_fe).unwrap();
        assert_eq!(result_fe, expected_bytes_fe);

        let result_ff = _build_varint_bytes(PREFIX_FF, value_ff).unwrap();
        assert_eq!(result_ff, expected_bytes_ff);

        let result_default = _build_varint_bytes(prefix_default, value_default).unwrap();
        assert_eq!(result_default, expected_bytes_default);

        let result_large = _build_varint_bytes(prefix_large, value_large);
        assert!(result_large.is_err());
        assert_eq!(
            result_large.unwrap_err(),
            NodoBitcoinError::ValorFueraDeRango
        );
    }

    #[test]
    fn testfrom_amount_bytes_to_prefix() {
        assert_eq!(from_amount_bytes_to_prefix(3), PREFIX_FD);
        assert_eq!(from_amount_bytes_to_prefix(5), PREFIX_FE);
        assert_eq!(from_amount_bytes_to_prefix(9), PREFIX_FF);
        assert_eq!(from_amount_bytes_to_prefix(2), 1);
    }
}
