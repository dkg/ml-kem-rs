#![no_std]
#![deny(clippy::pedantic)]
#![deny(warnings)]
#![deny(missing_docs)]
#![allow(clippy::cast_lossless)] // TODO
#![allow(clippy::cast_possible_truncation)] // TODO

#![doc = include_str!("../README.md")]

/// Implements FIPS 203 draft Module-Lattice-based Key-Encapsulation Mechanism Standard.
/// See <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.ipd.pdf>
//
// Supports automatically clearing sensitive data on drop
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::traits::SerDes;

// Functionality map per FIPS 203 draft
//
// Algorithm 2 BitsToBytes(b) on page 17                    --> optimized out (byte_fns.rs)
// Algorithm 3 BytesToBits(B) on page 18                    --> optimized out (byte_fns.rs)
// Algorithm 4 ByteEncoded(F) on page 19                    --> byte_fns.rs
// Algorithm 5 ByteDecoded(B) on page 19                    --> byte_fns.rs
// Algorithm 6 SampleNTT(B) on page 20                      --> sampling.rs
// Algorithm 7 SamplePolyCBDη(B) on page 20                 --> sampling.rs
// Algorithm 8 NTT(f) on page 22                            --> ntt.rs
// Algorithm 9 NTT−1(fˆ) on page 23                         --> ntt.rs
// Algorithm 10 MultiplyNTTs(fˆ,ĝ) on page 24               --> ntt.rs
// Algorithm 11 BaseCaseMultiply(a0,a1,b0,b1,γ) on page 24  --> ntt.rs
// Algorithm 12 K-PKE.KeyGen() on page 26                   --> k_pke.rs
// Algorithm 13 K-PKE.Encrypt(ekPKE,m,r) on page 27         --> k_pke.rs
// Algorithm 14 K-PKE.Decrypt(dkPKE,c) on page 28           --> k_pke.rs
// Algorithm 15 ML-KEM.KeyGen() on page 29                  --> ml_kem.rs
// Algorithm 16 ML-KEM.Encaps(ek) on page 30                --> ml_ke.rs
// Algorithm 17 ML-KEM.Decaps(c,dk) on page 32              --> ml_kem.rs
// PRF and XOF on page 16                                   --> helpers.rs
// Three hash functions: G, H, J on page 17                 --> helpers.rs
// Compress and Decompress on page 18                       --> helpers.rs
//
// The three parameter sets are modules in this file with injected macro code
// that connects them into the functionality in ml_kem.rs. Some of the strange
// coding style is driven by clippy pedantic.

mod byte_fns;
mod helpers;
mod k_pke;
mod ml_kem;
mod ntt;
mod sampling;
mod types;

/// All functionality is covered by traits, such that consumers can utilize trait objects as desired.
pub mod traits;

// Relevant to all parameter sets
const _N: u32 = 256;
const Q: u32 = 3329;
const ZETA: u32 = 17;
const SSK_LEN: usize = 32;

/// The (opaque) secret key that can be de/serialized by each party.
#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct SharedSecretKey([u8; SSK_LEN]);

impl SerDes for SharedSecretKey {
    type ByteArray = [u8; SSK_LEN];

    fn into_bytes(self) -> Self::ByteArray { self.0 }

    fn try_from_bytes(skk: Self::ByteArray) -> Result<Self, &'static str> {
        Ok(SharedSecretKey(skk))
    }
}


// Conservative (constant-time) paranoia...
impl PartialEq for SharedSecretKey {
    fn eq(&self, other: &Self) -> bool {
        let mut result = true;
        for i in 0..self.0.len() {
            result &= self.0[i] == other.0[i];
        }
        result
    }
}


// This common functionality is injected into each parameter set module
macro_rules! functionality {
    () => {
        const ETA1_64: usize = ETA1 * 64;  // Currently, Rust does not allow expressions involving constants...
        const ETA2_64: usize = ETA2 * 64;  // ...so these are handled manually.
        const J_LEN: usize = 32 + 32 * (DU * K + DV);

        use crate::byte_fns::byte_decode;
        use crate::ml_kem::{ml_kem_decaps, ml_kem_encaps, ml_kem_key_gen};
        use crate::traits::{Decaps, Encaps, KeyGen, SerDes};
        use crate::types::Z256;
        use crate::SharedSecretKey;
        use rand_core::CryptoRngCore;
        use zeroize::{Zeroize, ZeroizeOnDrop};

        /// Correctly sized encapsulation key specific to the target security parameter set.
        #[derive(Clone, Zeroize, ZeroizeOnDrop)]
        pub struct EncapsKey([u8; EK_LEN]);

        /// Correctly sized decapsulation key specific to the target security parameter set.
        #[derive(Clone, Zeroize, ZeroizeOnDrop)]
        pub struct DecapsKey([u8; DK_LEN]);

        /// Correctly sized ciphertext specific to the target security parameter set.
        #[derive(Clone, Zeroize, ZeroizeOnDrop)]
        pub struct CipherText([u8; CT_LEN]);

        /// Per FIPS 203, the key generation algorithm `ML-KEM.KeyGen` for ML-KEM (Algorithm 15)
        /// accepts no input, utilizes randomness, and produces an encapsulation key and a
        /// decapsulation key. While the encapsulation key can be made public, the decapsulation key
        /// must remain private. This outputs of this function are opaque structs specific to a
        /// target parameter set.

        pub struct KG();

        impl KeyGen for KG {
            type DecapsKey = DecapsKey;
            type EncapsKey = EncapsKey;

            fn try_keygen_with_rng_vt(
                rng: &mut impl CryptoRngCore,
            ) -> Result<(EncapsKey, DecapsKey), &'static str> {
                let (mut ek, mut dk) = ([0u8; EK_LEN], [0u8; DK_LEN]);
                ml_kem_key_gen::<K, ETA1, ETA1_64>(rng, &mut ek, &mut dk)?;
                Ok((EncapsKey(ek), DecapsKey(dk)))
            }
        }

        impl Encaps for EncapsKey {
            type CipherText = CipherText;
            type SharedSecretKey = SharedSecretKey;

            fn try_encaps_with_rng_vt(
                &self, rng: &mut impl CryptoRngCore,
            ) -> Result<(Self::SharedSecretKey, Self::CipherText), &'static str> {
                let mut ct = [0u8; CT_LEN];
                let ssk = ml_kem_encaps::<K, ETA1, ETA1_64, ETA2, ETA2_64, DU, DV>(
                    rng, &self.0, &mut ct,
                )?;
                Ok((ssk, CipherText(ct)))
            }
        }

        impl Decaps for DecapsKey {
            type CipherText = CipherText;
            type SharedSecretKey = SharedSecretKey;

            fn try_decaps_vt(&self, ct: &CipherText) -> Result<SharedSecretKey, &'static str> {
                let ssk = ml_kem_decaps::<K, ETA1, ETA1_64, ETA2, ETA2_64, DU, DV, J_LEN, CT_LEN>(
                    &self.0, &ct.0,
                );
                ssk
            }
        }


        impl SerDes for EncapsKey {
            type ByteArray = [u8; EK_LEN];

            fn try_from_bytes(ek: Self::ByteArray) -> Result<Self, &'static str> {
                let mut ek_hat = [Z256(0); 256];
                for i in 0..K {
                    byte_decode(12, &ek[384 * i..384 * (i + 1)], &mut ek_hat)?;
                }
                Ok(EncapsKey(ek))
            }

            fn into_bytes(self) -> Self::ByteArray { self.0 }
        }


        impl SerDes for DecapsKey {
            type ByteArray = [u8; DK_LEN];

            fn try_from_bytes(dk: Self::ByteArray) -> Result<Self, &'static str> {
                // TODO: additional validation here
                Ok(DecapsKey(dk))
            }

            fn into_bytes(self) -> Self::ByteArray { self.0 }
        }

        impl SerDes for CipherText {
            type ByteArray = [u8; CT_LEN];

            fn try_from_bytes(ct: Self::ByteArray) -> Result<Self, &'static str> {
                // TODO: additional validation here
                Ok(CipherText(ct))
            }

            fn into_bytes(self) -> Self::ByteArray { self.0 }
        }
    };
}


/// Functionality for the ML-KEM-512 security parameter set, which is claimed to be in security category 1, see
/// table 2 & 3 on page 33 of spec.
#[cfg(feature = "ml-kem-512")]
pub mod ml_kem_512 {
    //!
    //! See <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.ipd.pdf>
    //!
    //! Typical usage flow entails:
    //! 1. The originator runs `try_keygen_vt()` to get an encaps key `encapsKey` and decaps key `decapsKey`.
    //! 2. The originator serializes the encaps key via `encapsKey.into_bytes()` and sends to the remote party.
    //! 3. The remote party deserializes the bytes via `try_from_bytes(<bytes>)` and runs `try_encaps_vt()` to get the
    //!    shared secret key `ssk` and ciphertext `cipherText`.
    //! 4. The remote party serializes the cipertext via `cipherText.into_bytes()` and sends to the originator.
    //! 5. The originator deserializes the ciphertext via `try_from_bytes(<bytes>)` then
    //!    runs `decapsKey.try_decaps_vt(cipherText)` to the get shared secret ket `ssk`.
    //! 6. Both the originator and remote party now have the same shared secret key `ssk`.

    const K: usize = 2;
    const ETA1: usize = 3;
    const ETA2: usize = 2;
    const DU: usize = 10;
    const DV: usize = 4;
    const EK_LEN: usize = 800;
    const DK_LEN: usize = 1632;
    const CT_LEN: usize = 768;

    functionality!();
}


/// Functionality for the ML-KEM-768 security parameter set, which is claimed to be in security category 3, see
/// table 2 & 3 on page 33 of spec.
#[cfg(feature = "ml-kem-768")]
pub mod ml_kem_768 {
    //!
    //! See <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.ipd.pdf>
    //!
    //! Typical usage flow entails:
    //! 1. The originator runs `try_keygen_vt()` to get an encaps key `encapsKey` and decaps key `decapsKey`.
    //! 2. The originator serializes the encaps key via `encapsKey.into_bytes()` and sends to the remote party.
    //! 3. The remote party deserializes the bytes via `try_from_bytes(<bytes>)` and runs `try_encaps_vt()` to get the
    //!    shared secret key `ssk` and ciphertext `cipherText`.
    //! 4. The remote party serializes the cipertext via `cipherText.into_bytes()` and sends to the originator.
    //! 5. The originator deserializes the ciphertext via `try_from_bytes(<bytes>)` then
    //!    runs `decapsKey.try_decaps_vt(cipherText)` to the get shared secret ket `ssk`.
    //! 6. Both the originator and remote party now have the same shared secret key `ssk`.

    const K: usize = 3;
    const ETA1: usize = 2;
    const ETA2: usize = 2;
    const DU: usize = 10;
    const DV: usize = 4;
    const EK_LEN: usize = 1184;
    const DK_LEN: usize = 2400;
    const CT_LEN: usize = 1088;

    functionality!();
}


/// ML-KEM-1024 is claimed to be in security category 5, see table 2 & 3 on page 33.
#[cfg(feature = "ml-kem-1024")]
pub mod ml_kem_1024 {
    //!
    //! See <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.ipd.pdf>
    //!
    //! Typical usage flow entails:
    //! 1. The originator runs `try_keygen_vt()` to get an encaps key `encapsKey` and decaps key `decapsKey`.
    //! 2. The originator serializes the encaps key via `encapsKey.into_bytes()` and sends to the remote party.
    //! 3. The remote party deserializes the bytes via `try_from_bytes(<bytes>)` and runs `try_encaps_vt()` to get the
    //!    shared secret key `ssk` and ciphertext `cipherText`.
    //! 4. The remote party serializes the cipertext via `cipherText.into_bytes()` and sends to the originator.
    //! 5. The originator deserializes the ciphertext via `try_from_bytes(<bytes>)` then
    //!    runs `decapsKey.try_decaps_vt(cipherText)` to the get shared secret ket `ssk`.
    //! 6. Both the originator and remote party now have the same shared secret key `ssk`.

    const K: usize = 4;
    const ETA1: usize = 2;
    const ETA2: usize = 2;
    const DU: usize = 11;
    const DV: usize = 5;
    const EK_LEN: usize = 1568;
    const DK_LEN: usize = 3168;
    const CT_LEN: usize = 1568;

    functionality!();
}

/*
Notes

Flamegraph
https://github.com/flamegraph-rs/flamegraph
  echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
  cargo flamegraph --test integration

Fuzzing
https://rust-fuzz.github.io/book/cargo-fuzz.html
  cd fuzz
  rustup default nightly
  head -c 3200 </dev/urandom > corpus/seed1
  cargo fuzz run fuzz_all -j 4

Dudect
https://docs.rs/dudect-bencher/latest/dudect_bencher/
  cd dudect
  cargo run --release -- --continuous encaps
  cargo run --release -- --continuous decaps

*/
