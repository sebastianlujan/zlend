use poseidon::poseidon2::Poseidon2;

// OGBank Unified ZK Circuit
// Mode 0: Borrow Proof - proves ownership of ZCash UTXOs with value >= threshold
// Mode 1: Auth Proof   - proves authorization for repay/withdraw (same constraints)

fn main(
    // Private inputs
    user_secret: Field,
    value: u64,
    nonce: Field,
    // Public inputs
    mode: pub Field,
    commitment_hash: pub Field,
    nullifier: pub Field,
    threshold: pub u64,
    borrow_nullifier: pub Field,
    repay_nullifier: pub Field,
    recipient: pub Field,
) {
    assert((mode == 0) | (mode == 1));
    assert(recipient != 0);

    if mode == 0 {
        // === MODE 0: BORROW PROOF ===
        // Proves: user owns ZCash UTXOs with value >= threshold

        // H(user_secret, value, nonce) == commitment_hash
        let computed_commitment = Poseidon2::hash([user_secret, value as Field, nonce], 3);
        assert(computed_commitment == commitment_hash);

        // H(user_secret, nonce) == nullifier
        let computed_nullifier = Poseidon2::hash([user_secret, nonce], 2);
        assert(computed_nullifier == nullifier);

        // Collateral must meet threshold
        assert(value >= threshold);
    } else {
        // === MODE 1: AUTH PROOF (Repay + Withdraw) ===
        // Proves: user authorized repay/withdraw for a specific borrow
        // Contract state machine differentiates repay vs withdraw

        // H(user_secret, nonce) == borrow_nullifier
        let computed_borrow_nullifier = Poseidon2::hash([user_secret, nonce], 2);
        assert(computed_borrow_nullifier == borrow_nullifier);

        // Nullifier chain: H(user_secret, borrow_nullifier) == repay_nullifier
        let computed_repay_nullifier = Poseidon2::hash([user_secret, borrow_nullifier], 2);
        assert(computed_repay_nullifier == repay_nullifier);
    }
}

#[test]
fn test_borrow_mode() {
    let user_secret: Field = 123;
    let value: u64 = 1000;
    let nonce: Field = 456;
    let recipient: Field = 0xdeadbeef;

    let commitment_hash = Poseidon2::hash([user_secret, value as Field, nonce], 3);
    let nullifier = Poseidon2::hash([user_secret, nonce], 2);

    main(user_secret, value, nonce, 0, commitment_hash, nullifier, 500, 0, 0, recipient);
}

#[test]
fn test_borrow_exact_threshold() {
    let user_secret: Field = 42;
    let value: u64 = 750;
    let nonce: Field = 99;
    let recipient: Field = 0xcafe;

    let commitment_hash = Poseidon2::hash([user_secret, value as Field, nonce], 3);
    let nullifier = Poseidon2::hash([user_secret, nonce], 2);

    main(user_secret, value, nonce, 0, commitment_hash, nullifier, 750, 0, 0, recipient);
}

#[test]
fn test_auth_mode() {
    let user_secret: Field = 123;
    let nonce: Field = 456;
    let recipient: Field = 0xdeadbeef;

    let borrow_nullifier = Poseidon2::hash([user_secret, nonce], 2);
    let repay_nullifier = Poseidon2::hash([user_secret, borrow_nullifier], 2);

    main(user_secret, 0, nonce, 1, 0, 0, 0, borrow_nullifier, repay_nullifier, recipient);
}

#[test(should_fail)]
fn test_borrow_insufficient_collateral() {
    let user_secret: Field = 123;
    let value: u64 = 100;
    let nonce: Field = 456;
    let recipient: Field = 0xdeadbeef;

    let commitment_hash = Poseidon2::hash([user_secret, value as Field, nonce], 3);
    let nullifier = Poseidon2::hash([user_secret, nonce], 2);

    main(user_secret, value, nonce, 0, commitment_hash, nullifier, 500, 0, 0, recipient);
}

#[test(should_fail)]
fn test_auth_wrong_secret() {
    let user_secret: Field = 123;
    let wrong_secret: Field = 999;
    let nonce: Field = 456;
    let recipient: Field = 0xdeadbeef;

    let borrow_nullifier = Poseidon2::hash([user_secret, nonce], 2);
    let repay_nullifier = Poseidon2::hash([user_secret, borrow_nullifier], 2);

    main(wrong_secret, 0, nonce, 1, 0, 0, 0, borrow_nullifier, repay_nullifier, recipient);
}

#[test(should_fail)]
fn test_invalid_mode() {
    main(1, 0, 1, 2, 0, 0, 0, 0, 0, 0xdeadbeef);
}

#[test]
fn test_borrow_with_browser_values() {
    let user_secret: Field = 0x0016e07e028eac14008dc606a9a4f31f472d66588a22ac32fd2a2ea66b4100ba;
    let value: u64 = 1100000000;
    let nonce: Field = 0x003992546a70e152757995d3d41e341f37a141d43f47dc7fe1280d173d51cbea;
    let recipient: Field = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

    let commitment_hash = Poseidon2::hash([user_secret, value as Field, nonce], 3);
    let nullifier = Poseidon2::hash([user_secret, nonce], 2);

    println(f"commitment_hash: {commitment_hash}");
    println(f"nullifier: {nullifier}");

    main(user_secret, value, nonce, 0, commitment_hash, nullifier, 1100000000, 0, 0, recipient);
}

#[test(should_fail)]
fn test_zero_recipient() {
    let user_secret: Field = 123;
    let value: u64 = 1000;
    let nonce: Field = 456;

    let commitment_hash = Poseidon2::hash([user_secret, value as Field, nonce], 3);
    let nullifier = Poseidon2::hash([user_secret, nonce], 2);

    main(user_secret, value, nonce, 0, commitment_hash, nullifier, 500, 0, 0, 0);
}
