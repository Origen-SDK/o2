//! Helpers for working with BigUnits

use crate::Result;
use num_bigint::BigUint;
use num_bigint::ToBigUint;
use num_traits::Zero;

/// Returns the value of the given BigUint for the given bit field range.
/// Notes:
///  * start_bit and stop_bit can be given in either high/low order, but they mean the same
///  * start_bit == stop_bit will be handled correctly
///  * The size of the input number is assumed to be infinite and references to
pub fn bit_slice(value: &BigUint, start_bit: usize, stop_bit: usize) -> Result<BigUint> {
    // Shortcut the common case of value == 0
    if value.is_zero() {
        let result: BigUint = Zero::zero();
        return Ok(result);
    }

    // Clean up input args
    let lower;
    let upper;
    if start_bit > stop_bit {
        lower = stop_bit;
        upper = start_bit;
    } else {
        lower = start_bit;
        upper = stop_bit;
    }

    // If the whole slice is above the MSB just return 0
    if value.bits() <= lower {
        return Ok(0.to_biguint().unwrap());
    }

    // Note that this also gets rid of all lower bytes outside of the given range, so
    // bytes[0] contains the start of the requested slice, though LSB0 may not be in
    // bit0 and the offset variable is created to account for that.
    let start_byte = lower / 8;
    let bytes = &value.to_bytes_le()[start_byte..];
    let offset = lower % 8;

    // Now create a byte array containing the result
    let mut result_bytes: Vec<u8> = Vec::new();
    // Handle 1 bit to be returned as a special case
    if lower == upper {
        match bytes.get(0) {
            Some(x) => result_bytes.push((x >> offset) & 1),
            None => result_bytes.push(0),
        }
    } else {
        let num_bytes = ((upper + 1 - lower) as f32 / 8.0).ceil() as usize;
        let mut byte: u8 = 0;
        let mut last_byte_done = false;
        for i in 0..num_bytes {
            let next: u8;
            match bytes.get(i) {
                Some(x) => next = *x,
                None => {
                    next = 0;
                    last_byte_done = i < num_bytes - 1;
                }
            }
            if i != 0 {
                byte = byte | (next.clone().checked_shl(8 - offset as u32).unwrap_or(0));
                result_bytes.push(byte);
            }
            byte = next >> offset;
            if last_byte_done {
                break;
            }
        }

        if !last_byte_done {
            // For the last byte  we also need to account for throwing away excess upper bits
            let x = 7 - (upper % 8);
            let last_byte_mask: u8 = 0xFF >> x;

            match bytes.get(num_bytes) {
                // This means that 'next' is the last byte, and so the mask gets applied to that
                // and then it is merged with the current byte
                Some(x) => {
                    let next = *x & last_byte_mask;
                    byte = byte | (next.checked_shl(8 - offset as u32).unwrap_or(0));
                }
                // This means that 'byte' is the last byte, and so the mask gets applied to that
                None => {
                    byte = byte & last_byte_mask;
                }
            };
            result_bytes.push(byte);
        }
    }

    Ok(BigUint::from_bytes_le(&result_bytes))
}

#[cfg(test)]
mod tests {
    use crate::utility::big_uint_helpers::*;
    use num_bigint::ToBigUint;

    #[test]
    fn bit_slice_works() {
        assert_eq!(
            bit_slice(&0.to_biguint().unwrap(), 15, 31).unwrap(),
            0.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0x1234.to_biguint().unwrap(), 0, 0).unwrap(),
            0x0.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0x1234.to_biguint().unwrap(), 1, 1).unwrap(),
            0x0.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0x1234.to_biguint().unwrap(), 2, 2).unwrap(),
            0x1.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0x1234.to_biguint().unwrap(), 8, 8).unwrap(),
            0x0.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0x1234.to_biguint().unwrap(), 9, 9).unwrap(),
            0x1.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0xF234.to_biguint().unwrap(), 0, 7).unwrap(),
            0x34.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0xF234.to_biguint().unwrap(), 7, 0).unwrap(),
            0x34.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0xF234.to_biguint().unwrap(), 15, 8).unwrap(),
            0xF2.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0xF234.to_biguint().unwrap(), 15, 0).unwrap(),
            0xF234.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0xF234.to_biguint().unwrap(), 14, 0).unwrap(),
            0x7234.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0xFFFF.to_biguint().unwrap(), 15, 0).unwrap(),
            0xFFFF.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0xFFFF.to_biguint().unwrap(), 8, 0).unwrap(),
            0x1FF.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&0xFFFF.to_biguint().unwrap(), 9, 0).unwrap(),
            0x3FF.to_biguint().unwrap()
        );
        let big_val = BigUint::parse_bytes(
            b"ADE811244939F9E2EDBB07BE627CF1F94ACE820AF6FDCC4EDA910FE7CF681073",
            16,
        )
        .unwrap();
        assert_eq!(
            bit_slice(&big_val, 255, 0).unwrap(),
            BigUint::parse_bytes(
                b"ADE811244939F9E2EDBB07BE627CF1F94ACE820AF6FDCC4EDA910FE7CF681073",
                16
            )
            .unwrap()
        );
        assert_eq!(
            bit_slice(&big_val, 226, 75).unwrap(),
            BigUint::parse_bytes(b"89273F3C5DB760F7CC4F9E3F2959D0415EDFB9", 16).unwrap()
        );
        assert_eq!(
            bit_slice(&big_val, 555, 75).unwrap(),
            BigUint::parse_bytes(b"15BD022489273F3C5DB760F7CC4F9E3F2959D0415EDFB9", 16).unwrap()
        );
        assert_eq!(
            bit_slice(&big_val, 555, 495).unwrap(),
            0.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&big_val, 555, 256).unwrap(),
            0.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&big_val, 555, 255).unwrap(),
            1.to_biguint().unwrap()
        );
        assert_eq!(
            bit_slice(&big_val, 111, 149).unwrap(),
            BigUint::parse_bytes(b"79E3F2959D", 16).unwrap()
        );
    }
}
