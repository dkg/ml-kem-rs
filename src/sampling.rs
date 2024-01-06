use sha3::digest::XofReader;

use crate::Q;
//use crate::byte_fns::bytes_to_bits;
//use crate::helpers::ensure;
use crate::types::Z256;

/// Algorithm 6 `SampleNTT(B)` on page 20.
/// If the input is a stream of uniformly random bytes, the output is a uniformly random element of `T_q`.
#[must_use]
pub fn sample_ntt(mut byte_stream_b: impl XofReader) -> [Z256; 256] {
    // Input: byte stream B ∈ B^{∗}
    // Output: array a_hat ∈ Z^{256}_q              ▷ the coeffcients of the NTT of a polynomial
    let mut array_a_hat = [Z256(0); 256];
    let mut bbb = [0u8; 3]; // Space for 3 random (byte) draws

    // 1: i ← 0 (not needed as three bytes are repeatedly drawn from the rng bytestream via bbb)

    // 2: j ← 0
    let mut j = 0;

    // 3: while j < 256 do
    while j < 256 {
        //
        byte_stream_b.read(&mut bbb); // Draw 3 bytes

        // 4: d1 ← B[i] + 256 · (B[i + 1] mod 16)
        let d1 = u32::from(bbb[0]) + 256 * (u32::from(bbb[1]) & 0x0F);

        // 5: d2 ← ⌊B[i + 1]/16⌋ + 16 · B[i + 2]
        let d2 = u32::from(bbb[1]) / 16 + 16 * u32::from(bbb[2]);

        // 6: if d1 < q then
        if d1 < Q {
            //
            // 7: a_hat[j] ← d1         ▷ a_hat ∈ Z256
            array_a_hat[j] = Z256(u16::try_from(d1).unwrap());

            // 8: j ← j+1
            j += 1;
            //
        } // 9: end if

        // 10: if d2 < q and j < 256 then
        if (d2 < Q) & (j < 256) {
            //
            // 11: a_hat[j] ← d2
            array_a_hat[j] = Z256(u16::try_from(d2).unwrap());

            // 12: j ← j+1
            j += 1;
            //
        } // 13: end if

        // 14: i ← i+3  (not needed as we draw 3 more bytes next time
    } // 15: end while

    array_a_hat // 16: return a_hat
}


/// Algorithm 7 `SamplePolyCBDη(B)` on page 20.
/// If the input is a stream of uniformly random bytes, outputs a sample from the distribution Dη (Rq ).
#[allow(clippy::unnecessary_wraps)]  // TODO: revisit
pub fn sample_poly_cbd(eta: u32, byte_array_b: &[u8]) -> Result<[Z256; 256], &'static str> {
    let mut array_f: [Z256; 256] = [Z256(0); 256];
    let mut temp = 0;
    let mut int_index = 0;
    let mut bit_index = 0;
    for byte in byte_array_b {
        temp |= (*byte as u64) << bit_index;
        bit_index += 8;
        while bit_index >= 2 * (eta as usize) {
            let tmask_x = temp & (2u64.pow(eta) - 1);
            let x = (tmask_x as u8).count_ones();
            let tmask_y = (temp >> eta) & (2u64.pow(eta) - 1);
            let y = (tmask_y as u8).count_ones();
            array_f[int_index] = Z256(x as u16).sub(Z256(y as u16));
            bit_index -= 2 * (eta as usize);
            temp >>= 2 * (eta as usize);
            int_index += 1;
        }
    }
    Ok(array_f)
}

// /// Algorithm 7 `SamplePolyCBDη(B)` on page 20.
// /// If the input is a stream of uniformly random bytes, outputs a sample from the distribution Dη (Rq ).
// #[allow(dead_code)]
// pub fn sample_poly_cbd2<const ETA: usize, const ETA_512: usize>(
//     byte_array_b: &[u8],
// ) -> Result<[Z256; 256], &'static str> {
//     // Input: byte array B ∈ B^{64η}
//     // Output: array f ∈ Z^{256}_q
//     ensure!(ETA * 512 == ETA_512, "Alg7: const probs");
//     ensure!(byte_array_b.len() == ETA * 64, "Alg7: bytes len not ETA * 64");
//
//     let mut array_f: [Z256; 256] = [Z256(0); 256];
//     let mut bit_array = [0u8; ETA_512];
//
//     // 1: b ← BytesToBits(B)
//     bytes_to_bits(byte_array_b, &mut bit_array)?;
//
//     // 2: for (i ← 0; i < 256; i ++)
//     for i in 0..256 {
//         //
//         // 3: x ← ∑_{j=0}^{η-1} b[2iη + j]
//         let x = (0..ETA).fold(0, |acc: u32, j| acc + u32::from(bit_array[2 * i * ETA + j]));
//
//         // 4: y ← ∑_{j=0}^{η-1} b[2iη + η + j]
//         let y = (0..ETA).fold(0, |acc: u32, j| acc + u32::from(bit_array[2 * i * ETA + ETA + j]));
//
//         // 5: f [i] ← x − y mod q
//         array_f[i] = Z256(u16::try_from(x).unwrap()).sub(Z256(u16::try_from(y).unwrap()));
//
//         //
//     } // 6: end for
//
//     Ok(array_f) // 7: return f
// }
