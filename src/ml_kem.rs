use rand::random;

use crate::helpers::{g, h, j};
use crate::k_pke::k_pke_decrypt;
use crate::SharedSecretKey;

use super::k_pke::{k_pke_encrypt, k_pke_key_gen};

/// Algorithm 15 `ML-KEM.KeyGen()` on page 29
/// Generates an encapsulation key and a corresponding decapsulation key.
pub(crate) fn key_gen<const K: usize, const ETA1: usize, const ETA1_64: usize, const ETA1_256: usize>(
    ek: &mut [u8], dk: &mut [u8],
) {
    // Output: Encapsulation key ek ∈ B^{384k+32}
    // Output: Decapsulation key dk ∈ B^{768k+96}
    debug_assert_eq!(ek.len(), 384 * K + 32);
    debug_assert_eq!(dk.len(), 768 * K + 96);
    // 1: z ←− B32         ▷ z is 32 random bytes (see Section 3.3)
    let z = random::<[u8; 32]>();
    // 2: (ek_{PKE}, dk_{PKE}) ← K-PKE.KeyGen()     ▷ run key generation for K-PKE
    let p1 = 384 * K;
    k_pke_key_gen::<K, ETA1, ETA1_64, ETA1_256>(ek, &mut dk[..p1]);
    let h_ek = h(ek);
    let p2 = p1 + ek.len();
    let p3 = p2 + h_ek.len();
    dk[p1..p2].copy_from_slice(ek);
    dk[p2..p3].copy_from_slice(&h_ek);
    dk[p3..].copy_from_slice(&z);
}

pub(crate) fn encaps<
    const K: usize,
    const ETA1: usize,
    const ETA1_64: usize,
    const ETA1_256: usize,
    const ETA2: usize,
    const ETA2_64: usize,
    const ETA2_256: usize,
    const DU: usize,
    const DU_256: usize,
    const DV: usize,
    const DV_256: usize,
>(
    ek: &[u8], ct: &mut [u8],
) -> SharedSecretKey {
    let m = random::<[u8; 32]>();
    let h_ek = h(ek);
    let mut g_input = [0u8; 64];
    g_input[0..32].copy_from_slice(&m);
    g_input[32..64].copy_from_slice(&h_ek);
    let (k, r) = g(&g_input);
    k_pke_encrypt::<K, ETA1, ETA1_64, ETA1_256, ETA2, ETA2_64, ETA2_256, DU, DU_256, DV, DV_256>(ek, &m, &r, ct);
    SharedSecretKey(k)
}

pub(crate) fn decaps<
    const K: usize,
    const ETA1: usize,
    const ETA1_64: usize,
    const ETA1_256: usize,
    const ETA2: usize,
    const ETA2_64: usize,
    const ETA2_256: usize,
    const DU: usize,
    const DU_8: usize,
    const DU_32: usize,
    const DU_256: usize,
    const DV: usize,
    const DV_8: usize,
    const DV_32: usize,
    const DV_256: usize,
>(
    dk: &[u8], ct: &[u8],
) -> SharedSecretKey {
    let m_prime = k_pke_decrypt::<K, DU, DU_256, DV, DV_256>(&dk[0..384 * K], &ct);

    let mut g_input = [0u8; 55];
    g_input[0..m_prime.len()].copy_from_slice(&m_prime);
    g_input[0..32].copy_from_slice(&dk[768 * K + 32..768 * K + 64]);
    let (mut k_prime, r_prime) = g(&g_input);

    let mut j_input = [0u8; 32 + 768];
    j_input[0..32].copy_from_slice(&dk[768 * K + 64..768 * K + 96]);
    j_input[32..32 + ct.len()].copy_from_slice(&ct);
    let k_bar = j(&j_input, 32); // TODO: remove 32
    let mut c_prime = [0u8; 768];
    k_pke_encrypt::<K, ETA1, ETA1_64, ETA1_256, ETA2, ETA2_64, ETA2_256, DU, DU_256, DV, DV_256>(
        &dk[384 * K..768 * K + 32],
        &m_prime,
        &r_prime,
        &mut c_prime,
    );
    if ct != c_prime {
        k_prime = k_bar
    };
    SharedSecretKey(k_prime)
}
