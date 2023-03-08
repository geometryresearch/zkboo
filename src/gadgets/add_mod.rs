use std::{
    fmt::Display,
    ops::{BitAnd, BitXor},
};

use crate::{
    gf2_word::{Bit, BitUtils, BytesInfo, GF2Word, GenRand},
    party::Party,
};

pub fn adder<T>(x: T, y: T) -> T
where
    T: Copy
        + Default
        + Display
        + BitAnd<Output = T>
        + BitXor<Output = T>
        + BitUtils
        + BytesInfo
        + GenRand,
{
    let mut carry = T::zero();

    for i in 0..T::bytes_len() * 8 - 1 {
        let a = (x ^ carry).get_bit(i);
        let b = (y ^ carry).get_bit(i);

        let ci = (a & b) ^ carry.get_bit(i);
        carry = carry.set_bit(i + 1, ci.inner());
    }

    x ^ y ^ carry
}

// fn adder2(x: u8, y: u8) -> u8 {
//     let mut carry: u8 = 0;

//     for i in 0u8..7 {
//         let a = get_bit(x ^ carry, i);
//         let b = get_bit(y ^ carry, i);

//         let ci = (a & b) ^ get_bit(carry, i);
//         carry = set_bit(carry, i + 1, ci);
//     }

//     x ^ y ^ carry
// }

/// Binary multiplication gate from p.12 of https://eprint.iacr.org/2016/163.pdf
fn bit_and(input_p1: (Bit, Bit), input_p2: (Bit, Bit), r_p1: Bit, r_p2: Bit) -> Bit {
    (input_p1.0 & input_p1.1)
        ^ (input_p1.0 & input_p2.1)
        ^ (input_p1.1 & input_p2.0)
        ^ (r_p1 ^ r_p2)
}

pub fn mpc_add_mod_k<T>(
    input_p1: GF2Word<T>,
    input_p2: GF2Word<T>,
    input_p3: GF2Word<T>,
    k: GF2Word<T>,
    p1: &mut Party<T>,
    p2: &mut Party<T>,
    p3: &mut Party<T>,
) -> (GF2Word<T>, GF2Word<T>, GF2Word<T>)
where
    T: Copy
        + Default
        + Display
        + BitAnd<Output = T>
        + BitXor<Output = T>
        + BitUtils
        + BytesInfo
        + GenRand,
{
    let rand_p1 = p1.read_tape();
    let rand_p2 = p2.read_tape();
    let rand_p3 = p3.read_tape();

    let mut carry_p1: GF2Word<T> = T::zero().into();
    let mut carry_p2: GF2Word<T> = T::zero().into();
    let mut carry_p3: GF2Word<T> = T::zero().into();

    for i in 0..T::bytes_len() * 8 - 1 {
        let ri_p1 = rand_p1.value.get_bit(i);
        let ri_p2 = rand_p2.value.get_bit(i);
        let ri_p3 = rand_p3.value.get_bit(i);

        let a_p1 = (input_p1.value ^ carry_p1.value).get_bit(i);
        let b_p1 = (k.value ^ carry_p1.value).get_bit(i);

        let a_p2 = (input_p2.value ^ carry_p2.value).get_bit(i);
        let b_p2 = (k.value ^ carry_p2.value).get_bit(i);

        let a_p3 = (input_p3.value ^ carry_p3.value).get_bit(i);
        let b_p3 = (k.value ^ carry_p3.value).get_bit(i);

        let ci_p1 = bit_and((a_p1, b_p1), (a_p2, b_p2), ri_p1, ri_p2) ^ carry_p1.value.get_bit(i);
        let ci_p2 = bit_and((a_p2, b_p2), (a_p3, b_p3), ri_p2, ri_p3) ^ carry_p2.value.get_bit(i);
        let ci_p3 = bit_and((a_p3, b_p3), (a_p1, b_p1), ri_p3, ri_p1) ^ carry_p3.value.get_bit(i);

        carry_p1 = carry_p1.value.set_bit(i + 1, ci_p1.inner()).into();
        carry_p2 = carry_p2.value.set_bit(i + 1, ci_p2.inner()).into();
        carry_p3 = carry_p3.value.set_bit(i + 1, ci_p3.inner()).into();
    }

    p1.view.send_msg(carry_p1);
    p2.view.send_msg(carry_p2);
    p3.view.send_msg(carry_p3);

    let o1 = input_p1 ^ k ^ carry_p1;
    let o2 = input_p2 ^ k ^ carry_p2;
    let o3 = input_p3 ^ k ^ carry_p3;

    (o1, o2, o3)
}

/// Performs addition modulo 2^(T::bits_size)
/// Works bit by bit and appends full carry in view, that's why it's counted as just one gate
pub fn mpc_add_mod<T>(
    input_p1: (GF2Word<T>, GF2Word<T>),
    input_p2: (GF2Word<T>, GF2Word<T>),
    input_p3: (GF2Word<T>, GF2Word<T>),
    p1: &mut Party<T>,
    p2: &mut Party<T>,
    p3: &mut Party<T>,
) -> (GF2Word<T>, GF2Word<T>, GF2Word<T>)
where
    T: Copy
        + Default
        + Display
        + BitAnd<Output = T>
        + BitXor<Output = T>
        + BitUtils
        + BytesInfo
        + GenRand,
{
    let rand_p1 = p1.read_tape();
    let rand_p2 = p2.read_tape();
    let rand_p3 = p3.read_tape();

    let mut carry_p1: GF2Word<T> = T::zero().into();
    let mut carry_p2: GF2Word<T> = T::zero().into();
    let mut carry_p3: GF2Word<T> = T::zero().into();

    for i in 0..T::bytes_len() * 8 - 1 {
        let ri_p1 = rand_p1.value.get_bit(i);
        let ri_p2 = rand_p2.value.get_bit(i);
        let ri_p3 = rand_p3.value.get_bit(i);

        let a_p1 = (input_p1.0.value ^ carry_p1.value).get_bit(i);
        let b_p1 = (input_p1.1.value ^ carry_p1.value).get_bit(i);

        let a_p2 = (input_p2.0.value ^ carry_p2.value).get_bit(i);
        let b_p2 = (input_p2.1.value ^ carry_p2.value).get_bit(i);

        let a_p3 = (input_p3.0.value ^ carry_p3.value).get_bit(i);
        let b_p3 = (input_p3.1.value ^ carry_p3.value).get_bit(i);

        let ci_p1 = bit_and((a_p1, b_p1), (a_p2, b_p2), ri_p1, ri_p2) ^ carry_p1.value.get_bit(i);
        let ci_p2 = bit_and((a_p2, b_p2), (a_p3, b_p3), ri_p2, ri_p3) ^ carry_p2.value.get_bit(i);
        let ci_p3 = bit_and((a_p3, b_p3), (a_p1, b_p1), ri_p3, ri_p1) ^ carry_p3.value.get_bit(i);

        carry_p1 = carry_p1.value.set_bit(i + 1, ci_p1.inner()).into();
        carry_p2 = carry_p2.value.set_bit(i + 1, ci_p2.inner()).into();
        carry_p3 = carry_p3.value.set_bit(i + 1, ci_p3.inner()).into();
    }

    p1.view.send_msg(carry_p1);
    p2.view.send_msg(carry_p2);
    p3.view.send_msg(carry_p3);

    let o1 = input_p1.0 ^ input_p1.1 ^ carry_p1;
    let o2 = input_p2.0 ^ input_p2.1 ^ carry_p2;
    let o3 = input_p3.0 ^ input_p3.1 ^ carry_p3;

    (o1, o2, o3)
}

pub fn add_mod_verify<T>(
    input_p: (GF2Word<T>, GF2Word<T>),
    input_p_next: (GF2Word<T>, GF2Word<T>),
    p: &mut Party<T>,
    p_next: &mut Party<T>,
) -> (GF2Word<T>, GF2Word<T>)
where
    T: Copy
        + Default
        + Display
        + BitAnd<Output = T>
        + BitXor<Output = T>
        + BitUtils
        + BytesInfo
        + GenRand,
{
    let ri = p.read_tape();
    let ri_next = p_next.read_tape();

    let mut carry_p = T::zero().into();
    let carry_p_next = p_next.view.read_next();

    for i in 0..T::bytes_len() * 8 - 1 {
        let ri_p = ri.value.get_bit(i);
        let ri_p_next = ri_next.value.get_bit(i);

        let a_p = (input_p.0 ^ carry_p).value.get_bit(i);
        let b_p = (input_p.1 ^ carry_p).value.get_bit(i);

        let a_p_next = (input_p_next.0 ^ carry_p_next).value.get_bit(i);
        let b_p_next = (input_p_next.1 ^ carry_p_next).value.get_bit(i);

        let ci_p =
            bit_and((a_p, b_p), (a_p_next, b_p_next), ri_p, ri_p_next) ^ carry_p.value.get_bit(i);

        carry_p = carry_p.value.set_bit(i + 1, ci_p.inner()).into();
    }

    p.view.send_msg(carry_p);

    let o1 = input_p.0 ^ input_p.1 ^ carry_p;
    let o2 = input_p_next.0 ^ input_p_next.1 ^ carry_p_next;

    (o1, o2)
}

pub fn add_mod_verify_k<T>(
    input_p: GF2Word<T>,
    input_p_next: GF2Word<T>,
    k: GF2Word<T>,
    p: &mut Party<T>,
    p_next: &mut Party<T>,
) -> (GF2Word<T>, GF2Word<T>)
where
    T: Copy
        + Default
        + Display
        + BitAnd<Output = T>
        + BitXor<Output = T>
        + BitUtils
        + BytesInfo
        + GenRand,
{
    let ri = p.read_tape();
    let ri_next = p_next.read_tape();

    let mut carry_p = T::zero().into();
    let carry_p_next = p_next.view.read_next();

    for i in 0..T::bytes_len() * 8 - 1 {
        let ri_p = ri.value.get_bit(i);
        let ri_p_next = ri_next.value.get_bit(i);

        let a_p = (input_p ^ carry_p).value.get_bit(i);
        let b_p = (k ^ carry_p).value.get_bit(i);

        let a_p_next = (input_p_next ^ carry_p_next).value.get_bit(i);
        let b_p_next = (k ^ carry_p_next).value.get_bit(i);

        let ci_p =
            bit_and((a_p, b_p), (a_p_next, b_p_next), ri_p, ri_p_next) ^ carry_p.value.get_bit(i);

        carry_p = carry_p.value.set_bit(i + 1, ci_p.inner()).into();
    }

    p.view.send_msg(carry_p);

    let o1 = input_p ^ k ^ carry_p;
    let o2 = input_p_next ^ k ^ carry_p_next;

    (o1, o2)
}

#[cfg(test)]
mod adder_tests {
    use std::{
        fmt::{Debug, Display},
        marker::PhantomData,
        ops::{BitAnd, BitXor},
    };

    use crate::{
        circuit::{Circuit, Output},
        error::Error,
        gadgets::add_mod::{add_mod_verify, add_mod_verify_k, adder, mpc_add_mod, mpc_add_mod_k},
        gf2_word::{BitUtils, BytesInfo, GF2Word, GenRand},
        party::Party,
    };

    use super::*;

    pub struct AddModKCircuit<T>
    where
        T: Copy
            + Default
            + Display
            + Debug
            + PartialEq
            + BitAnd<Output = T>
            + BitXor<Output = T>
            + BitUtils
            + BytesInfo
            + GenRand,
    {
        pub k: GF2Word<T>,
    }

    impl<T> Circuit<T> for AddModKCircuit<T>
    where
        T: Copy
            + Default
            + Display
            + Debug
            + PartialEq
            + BitAnd<Output = T>
            + BitXor<Output = T>
            + BitUtils
            + BytesInfo
            + GenRand,
    {
        fn compute(&self, input: &[GF2Word<T>]) -> Vec<GF2Word<T>> {
            assert_eq!(input.len(), 1);
            let res = adder(input[0].value, self.k.value);
            vec![res.into()]
        }

        fn compute_23_decomposition(
            &self,
            p1: &mut Party<T>,
            p2: &mut Party<T>,
            p3: &mut Party<T>,
        ) -> (Vec<GF2Word<T>>, Vec<GF2Word<T>>, Vec<GF2Word<T>>) {
            assert_eq!(p1.view.input.len(), 1);
            assert_eq!(p2.view.input.len(), 1);
            assert_eq!(p3.view.input.len(), 1);

            let input_p1 = p1.view.input[0];
            let input_p2 = p2.view.input[0];
            let input_p3 = p3.view.input[0];

            let (o1, o2, o3) = mpc_add_mod_k(input_p1, input_p2, input_p3, self.k, p1, p2, p3);
            (vec![o1], vec![o2], vec![o3])
        }

        fn simulate_two_parties(
            &self,
            p: &mut Party<T>,
            p_next: &mut Party<T>,
        ) -> Result<(Output<T>, Output<T>), Error> {
            assert_eq!(p.view.input.len(), 1);
            assert_eq!(p_next.view.input.len(), 1);

            let (o1, o2) =
                add_mod_verify_k(p.view.input[0], p_next.view.input[0], self.k, p, p_next);
            Ok((vec![o1], vec![o2]))
        }

        fn party_output_len(&self) -> usize {
            1
        }

        fn num_of_mul_gates(&self) -> usize {
            1
        }
    }

    #[cfg(test)]
    mod test_adder {
        use rand::{rngs::ThreadRng, thread_rng};
        use rand_chacha::ChaCha20Rng;
        use sha3::Keccak256;

        use crate::{circuit::Circuit, gf2_word::GF2Word, prover::Prover, verifier::Verifier};

        use super::AddModKCircuit;

        #[test]
        fn test_circuit() {
            let mut rng = thread_rng();
            const SIGMA: usize = 80;
            let input: Vec<GF2Word<u32>> = [4294u32].iter().map(|&vi| vi.into()).collect();

            let circuit = AddModKCircuit::<u32> {
                k: 3490903u32.into(),
            };

            let output = circuit.compute(&input);

            let proof = Prover::<u32, ChaCha20Rng, Keccak256>::prove::<ThreadRng, SIGMA>(
                &mut rng, &input, &circuit, &output,
            )
            .unwrap();

            Verifier::<u32, ChaCha20Rng, Keccak256>::verify(&proof, &circuit, &output).unwrap();
        }
    }
}
