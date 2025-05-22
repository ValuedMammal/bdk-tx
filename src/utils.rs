use alloc::vec::Vec;

use bitcoin::{
    absolute::{self, LockTime},
    Sequence, Transaction, WitnessVersion,
};

use crate::Input;

use rand_core::{OsRng, TryRngCore};

/// Applies BIP326 anti‐fee‐sniping
pub fn apply_anti_fee_sniping(
    tx: &mut Transaction,
    selection_inputs: &[Input],
    current_height: absolute::Height,
) {
    const MAX_RELATIVE_HEIGHT: u32 = 0xFFFF;
    const MIN_SEQUENCE_VALUE: u32 = 1;

    let mut rng = OsRng;

    // Find the Inputs of the selectin matching segwit v1 (Tr).
    let tr_inputs: Vec<(usize, &Input)> = tx
        .input
        .iter()
        .enumerate()
        .filter_map(|(input_index, txin)| {
            let outpoint = txin.previous_output;
            let input = selection_inputs
                .iter()
                .find(|input| input.prev_outpoint() == outpoint)
                .expect("we selected it");

            if input.plan()?.witness_version() == Some(WitnessVersion::V1) {
                Some((input_index, input))
            } else {
                None
            }
        })
        .collect();

    // Check always‐locktime conditions
    // - 50%
    // - Does not explicitly signal rbf
    // - any inputs
    //   - have > 65535 confirmations
    //   - are unconfirmed
    if rng.try_next_u32().unwrap() % 2 == 1
        || !tx.is_explicitly_rbf()
        || tr_inputs.is_empty()
        || !tr_inputs.iter().any(|(_, input)| {
            let age = input.confirmations(current_height);
            age == 0 || age > MAX_RELATIVE_HEIGHT
        })
    {
        // Set the locktime to the current height
        let mut height = current_height.to_consensus_u32();

        // 10% chance to pick a locktime further back.
        if rng.try_next_u32().unwrap() % 10 == 1 {
            let offset = rng.try_next_u32().unwrap() % 100;
            height = height.saturating_sub(offset);
        }
        tx.lock_time = LockTime::from_height(height).expect("we checked");
    } else {
        // We're spending tr inputs with fewer than 65535 confirmations
        // and RBF enabled.

        // Set `LockTime::ZERO` and set the sequence of a randomly chosen input
        // to its confirmations.
        tx.lock_time = LockTime::ZERO;

        let i = rng.try_next_u32().unwrap() as usize % tr_inputs.len();
        let (input_index, input) = tr_inputs.get(i).expect("should get input");
        let mut n_confs = input.confirmations(current_height);

        // 10% chance to pick a sequence further back.
        if rng.try_next_u32().unwrap() % 10 == 1 {
            let offset = rng.try_next_u32().unwrap() % 100;
            n_confs = n_confs.saturating_sub(offset).max(MIN_SEQUENCE_VALUE);
        }

        tx.input[*input_index].sequence = Sequence(n_confs);
    }
}

// Problems:
// - How are we getting the current height? fallback locktime / tx.lock_time?
// is it fair to assume the tx locktime represents the current height?
// is the `confirmations` calculation correct wrt to the "current height"?
// is this sometimes ok but potentially awkward
// or always ok?
// - How are we obtaining randomness?

#[cfg(test)]
#[allow(unused)]
mod test {
    use super::*;

    #[test]
    fn asdf() {
        // TODO: Test create_psbt with `enable_anti_fee_sniping`

        // using different descriptors

        // let n = ((1u64 << 32) - 1) as u32;
        // dbg!(n);
        // let n = 2_f64.powi(32) as f64;
        // dbg!(n);
        // let n: u32 = 0xFFFFFFFF - 1;
        // dbg!(n);

        // let mut rng = OsRng;

        // let n = rng.next_u32();
        // println!("N {n}");
        // let rem = n % 10;
        // println!("Rem {rem}");
        // if rem == 1 {
        //     println!("10%");
        // } else {
        //     println!("Not");
        // }

        // let mut ct = 0;
        // for _ in 0..100 {
        //     let b = random_probability(2); // true half the time
        //     println!("If random (2): {b}");

        //     let b = random_probability(10); // only 10% true
        //     println!("If random (10): {b}");

        //     if b {
        //         ct += 1;
        //     }
        // }
        // println!("Ct {ct}");

        // let n = random_range(100);
        // println!("random in range (0..100): {n}");
        // let mut n = 0;
        // while n != 3 {
        //     n = random_range(4);
        // }
        // println!("random in range (0..4): {n}");
    }
}
