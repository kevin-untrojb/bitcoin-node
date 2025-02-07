use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, BitOr, Mul},
};

const NUM_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uint256([u8; NUM_BYTES]);

impl Default for Uint256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Uint256 {
    pub fn new() -> Uint256 {
        Uint256([0; NUM_BYTES])
    }

    pub fn _from_u128(value: u128) -> Uint256 {
        let mut bytes = [0; NUM_BYTES];
        bytes[16..32].copy_from_slice(&value.to_be_bytes());
        Uint256(bytes)
    }

    pub fn _from_u64(value: u64) -> Uint256 {
        let mut bytes = [0; NUM_BYTES];
        bytes[24..32].copy_from_slice(&value.to_be_bytes());
        Uint256(bytes)
    }

    pub fn _from_u32(value: u32) -> Uint256 {
        let mut bytes = [0; NUM_BYTES];
        bytes[28..32].copy_from_slice(&value.to_be_bytes());
        Uint256(bytes)
    }

    pub fn _from_u16(value: u16) -> Uint256 {
        let mut bytes = [0; NUM_BYTES];
        bytes[30..32].copy_from_slice(&value.to_be_bytes());
        Uint256(bytes)
    }

    pub fn from_be_bytes(bytes: [u8; NUM_BYTES]) -> Uint256 {
        Uint256(bytes)
    }

    pub fn _reverse_endian(&mut self) -> Uint256 {
        let bytes = self.get_bytes();
        Uint256::from_le_bytes(bytes)
    }

    pub fn from_le_bytes(bytes: [u8; NUM_BYTES]) -> Uint256 {
        let mut bytes = bytes;
        bytes.reverse();
        Uint256(bytes)
    }

    pub fn get_bytes(&self) -> [u8; NUM_BYTES] {
        self.0
    }

    pub fn _get_le_bytes(&self) -> [u8; NUM_BYTES] {
        let mut bytes = self.get_bytes();
        bytes.reverse();
        bytes
    }

    pub fn pow(&self, exponent: u32) -> Uint256 {
        let mut result = Uint256::_from_u64(1);
        let base = *self;

        for _i in 0..exponent {
            result = base * result;
        }

        result
    }

    pub fn _to_hexa_be_string(&self) -> String {
        let mut result = String::new();
        for i in 0..NUM_BYTES {
            result.push_str(&format!("{:02x}", self.0[i]));
        }
        result
    }

    pub fn to_hexa_le_string(self) -> String {
        let mut result = String::new();
        for i in 0..NUM_BYTES {
            result.push_str(&format!("{:02x}", self.0[NUM_BYTES - i - 1]));
        }
        result
    }
}

impl PartialOrd for Uint256 {
    fn partial_cmp(&self, other: &Uint256) -> Option<Ordering> {
        for i in 0..NUM_BYTES {
            match self.0[i] > other.0[i] {
                true => return Some(Ordering::Greater),
                false => match self.0[i] < other.0[i] {
                    true => return Some(Ordering::Less),
                    false => continue,
                },
            }
        }
        Some(Ordering::Equal)
    }
}

impl Ord for Uint256 {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,
            None => Ordering::Equal,
        }
    }
}

impl Add for Uint256 {
    type Output = Uint256;
    fn add(self, other: Uint256) -> Uint256 {
        let mut result = Uint256::new();
        let mut carry = 0u16;

        for i in (0..NUM_BYTES).rev() {
            let sum = u16::from(self.0[i]) + u16::from(other.0[i]) + carry;
            result.0[i] = sum as u8;
            carry = sum >> 8;
        }

        result
    }
}

impl Mul<Uint256> for Uint256 {
    type Output = Uint256;

    fn mul(self, other: Uint256) -> Uint256 {
        // quiero una variable que sea un vector de tamaño NUM_BYTES de Uint256 que vaya guardando las sumas parciales
        let mut partial_sum = Vec::new();

        //let mut partial_sum = Vec<Uint256::new()
        for i in (0..NUM_BYTES).rev() {
            // recorro la fila del multiplcando (el de abajo de la cuentita)
            let mut iteration_sum = Uint256::new();
            let mut carry = 0;
            let mut columna_actual = i + 1;

            for j in (0..NUM_BYTES).rev() {
                columna_actual -= 1;
                // recorro la fila del multiplicador (el de arriba de la cuentita)
                let byte_multiplicador = self.0[j];
                let byte_multiplicando = other.0[i];
                let multiplicacion = byte_multiplicador as u16 * byte_multiplicando as u16 + carry;
                carry = multiplicacion >> 8;

                // agrego el byte en la columna_actual
                iteration_sum.0[columna_actual] = multiplicacion as u8;
                if columna_actual == 0 {
                    break;
                }
            }
            partial_sum.push(iteration_sum); // 1 por cada byte del multiplicando
        }

        partial_sum
            .into_iter()
            .fold(Uint256::new(), |acc, x| acc.add(x))
    }
}

impl BitOr for Uint256 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut result = Uint256([0; NUM_BYTES]);

        for i in 0..NUM_BYTES {
            result.0[i] = self.0[i] | rhs.0[i];
        }

        result
    }
}

impl fmt::Display for Uint256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formato hexadecimal por byte
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::uint256::Uint256;

    use super::NUM_BYTES;

    #[test]
    fn test_nulti() {
        let a = Uint256([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00,
        ]);

        let b = Uint256([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ]);
        let result = a * b;
        assert_eq!(result, a);
    }

    #[test]
    fn test_compare() {
        let a = Uint256::_from_u64(1);
        let b = Uint256::_from_u64(2);
        assert!(a < b);
        assert!(b > a);
        assert!(b >= a);
        assert!(a <= b);
        assert_eq!(a > b, false);
    }

    #[test]
    fn test_compare_2() {
        let a = Uint256::_from_u64(25896);
        let b = Uint256::_from_u64(2);
        assert!(a > b);
        assert!(b < a);
        assert!(b <= a);
        assert!(a >= b);
        assert_eq!(a < b, false);
    }

    #[test]
    fn test_mul() {
        let a = Uint256::_from_u64(256);
        let b = Uint256::_from_u64(16);
        let c = a * b;
        assert_eq!(c, Uint256::_from_u64(4096));
    }

    #[test]
    fn test_add() {
        let a = Uint256::_from_u64(150);
        let b = Uint256::_from_u64(550);
        let c = a + b;
        assert_eq!(c, Uint256::_from_u64(700));
    }

    #[test]
    fn test_pow() {
        let a = Uint256::_from_u64(256);
        let b = a.pow(21);

        let valor = Uint256([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);

        assert_eq!(b, valor);
    }

    #[test]
    fn test_display_hexa() {
        let a = Uint256::_from_u64(123456789);
        let format = "Uint256([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 5B, CD, 15])".to_string();
        assert_eq!(format!("{:X?}", a), format);
    }

    #[test]
    fn test_reverse_endian() {
        let mut a = Uint256::_from_u64(123456789);
        let b = a._reverse_endian();
        for i in 0..NUM_BYTES {
            assert_eq!(a.0[i], b.0[NUM_BYTES - i - 1]);
        }
    }

    #[test]
    fn test_get_bytes_from_le_bytes() {
        let a = Uint256::_from_u64(123456789);

        let bytes = a.get_bytes();
        let from_be = Uint256::from_be_bytes(bytes);

        for i in 0..NUM_BYTES {
            assert_eq!(a.0[i], from_be.0[i]);
        }
    }

    #[test]
    fn test_to_hexa_be_string() {
        let valor = Uint256([
            0x01, 0x00, 0x00, 0x00, 0x01, 0x81, 0x3f, 0x79, 0x01, 0x1a, 0xcb, 0x80, 0x92, 0x5d,
            0xfe, 0x69, 0xb3, 0xde, 0xf3, 0x55, 0xfe, 0x91, 0x4b, 0xd1, 0xd9, 0x6a, 0x3f, 0x5f,
            0x71, 0xbf, 0x83, 0x03,
        ]);

        let hexa_string =
            "0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303".to_string();

        assert_eq!(valor._to_hexa_be_string(), hexa_string);
    }

    #[test]
    fn test_to_hexa_le_string() {
        let bytes: [u8; 32] = [
            0x01, 0x00, 0x00, 0x00, 0x01, 0x81, 0x3f, 0x79, 0x01, 0x1a, 0xcb, 0x80, 0x92, 0x5d,
            0xfe, 0x69, 0xb3, 0xde, 0xf3, 0x55, 0xfe, 0x91, 0x4b, 0xd1, 0xd9, 0x6a, 0x3f, 0x5f,
            0x71, 0xbf, 0x83, 0x03,
        ];

        let valor = Uint256::from_le_bytes(bytes);

        let hexa_string =
            "0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303".to_string();

        assert_eq!(valor.to_hexa_le_string(), hexa_string);
    }
}
