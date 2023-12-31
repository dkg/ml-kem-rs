use fips203::{ml_kem_1024, ml_kem_512, ml_kem_768};
use fips203::traits::{Decaps, Encaps, KeyGen, SerDes};

#[test]
fn test_expected_flow_512() {
    for _i in 0..100 {
        // Alice runs KeyGen, and serializes ek for Bob (to bytes)
        let (alice_ek, alice_dk) = ml_kem_512::KG::try_keygen_vt().unwrap();
        let alice_ek_bytes = alice_ek.into_bytes();

        // Alice sends ek bytes to Bob
        let bob_ek_bytes = alice_ek_bytes;

        // Bob deserializes ek bytes, runs Encaps, to get ssk and serializes ct for Alice (to bytes)
        let bob_ek = ml_kem_512::EncapsKey::try_from_bytes(bob_ek_bytes).unwrap();
        let (bob_ssk_bytes, bob_ct) = bob_ek.try_encaps_vt().unwrap();
        let bob_ct_bytes = bob_ct.into_bytes();

        // Bob sends ct bytes to Alice
        let alice_ct_bytes = bob_ct_bytes;

        // Alice deserializes runs Decaps
        let alice_ct = ml_kem_512::CipherText::try_from_bytes(alice_ct_bytes).unwrap();
        let alice_ssk_bytes = alice_dk.try_decaps_vt(&alice_ct).unwrap();

        // Alice and Bob now have the same shared secret key
        assert_eq!(bob_ssk_bytes, alice_ssk_bytes)
    }
}

#[test]
fn test_expected_flow_768() {
    for _i in 0..100 {
        // Alice runs KeyGen, and serializes ek for Bob (to bytes)
        let (alice_ek, alice_dk) = ml_kem_768::KG::try_keygen_vt().unwrap();
        let alice_ek_bytes = alice_ek.into_bytes();

        // Alice sends ek bytes to Bob
        let bob_ek_bytes = alice_ek_bytes;

        // Bob deserializes ek bytes, runs Encaps, to get ssk and serializes ct for Alice (to bytes)
        let bob_ek = ml_kem_768::EncapsKey::try_from_bytes(bob_ek_bytes).unwrap();
        let (bob_ssk_bytes, bob_ct) = bob_ek.try_encaps_vt().unwrap();
        let bob_ct_bytes = bob_ct.into_bytes();

        // Bob sends ct bytes to Alice
        let alice_ct_bytes = bob_ct_bytes;

        // Alice deserializes runs Decaps
        let alice_ct = ml_kem_768::CipherText::try_from_bytes(alice_ct_bytes).unwrap();
        let alice_ssk_bytes = alice_dk.try_decaps_vt(&alice_ct).unwrap();

        // Alice and Bob now have the same shared secret key
        assert_eq!(bob_ssk_bytes, alice_ssk_bytes)
    }
}

#[test]
fn test_expected_flow_1024() {
    for _i in 0..100 {
        // Alice runs KeyGen, and serializes ek for Bob (to bytes)
        let (alice_ek, alice_dk) = ml_kem_1024::KG::try_keygen_vt().unwrap();
        let alice_ek_bytes = alice_ek.into_bytes();

        // Alice sends ek bytes to Bob
        let bob_ek_bytes = alice_ek_bytes;

        // Bob deserializes ek bytes, runs Encaps, to get ssk and serializes ct for Alice (to bytes)
        let bob_ek = ml_kem_1024::EncapsKey::try_from_bytes(bob_ek_bytes).unwrap();
        let (bob_ssk_bytes, bob_ct) = bob_ek.try_encaps_vt().unwrap();
        let bob_ct_bytes = bob_ct.into_bytes();

        // Bob sends ct bytes to Alice
        let alice_ct_bytes = bob_ct_bytes;

        // Alice deserializes runs Decaps
        let alice_ct = ml_kem_1024::CipherText::try_from_bytes(alice_ct_bytes).unwrap();
        let alice_ssk_bytes = alice_dk.try_decaps_vt(&alice_ct).unwrap();

        // Alice and Bob now have the same shared secret key
        assert_eq!(bob_ssk_bytes, alice_ssk_bytes)
    }
}