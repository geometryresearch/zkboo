use std::{
    fmt::Display,
    ops::{BitAnd, BitXor},
};

use crate::{
    error::Error,
    gf2_word::{BitUtils, BytesInfo, GF2Word, GenRand},
    party::Party,
};

pub type Output<T> = Vec<GF2Word<T>>;
pub type TwoThreeDecOutput<T> = (Output<T>, Output<T>, Output<T>);

pub trait Circuit<T>
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
    fn compute(&self, input: &[GF2Word<T>]) -> Vec<GF2Word<T>>;
    /// Decompose this circuit into 3 branches such that the values computed in
    /// 2 branches reveals no information about the input x.
    fn compute_23_decomposition(
        &self,
        p1: &mut Party<T>,
        p2: &mut Party<T>,
        p3: &mut Party<T>,
    ) -> TwoThreeDecOutput<T>;
    fn simulate_two_parties(
        &self,
        p: &mut Party<T>,
        p_next: &mut Party<T>,
    ) -> Result<(Output<T>, Output<T>), Error>;
    fn party_output_len(&self) -> usize;
    fn num_of_mul_gates(&self) -> usize;
}

#[cfg(test)]
mod circuit_tests {
    use std::{
        fmt::{Debug, Display},
        ops::{BitAnd, BitXor},
    };

    use rand::{rngs::ThreadRng, thread_rng};
    use rand_chacha::ChaCha20Rng;
    use sha3::Keccak256;

    use super::{Circuit, Output, TwoThreeDecOutput};
    use crate::{
        error::Error,
        gadgets::{mpc_and, mpc_and_verify, mpc_xor},
        gf2_word::{BitUtils, BytesInfo, GF2Word, GenRand},
        party::Party,
        prover::Prover,
        verifier::Verifier,
    };

    // computes: (x1 ^ x2) & (x3 ^ x4) & x5
    #[derive(Clone, Copy)]
    struct SimpleCircuit1 {}

    impl<T> Circuit<T> for SimpleCircuit1
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
            assert_eq!(input.len(), 5);

            vec![(input[0] ^ input[1]) & (input[2] ^ input[3]) & input[4]]
        }

        fn compute_23_decomposition(
            &self,
            p1: &mut Party<T>,
            p2: &mut Party<T>,
            p3: &mut Party<T>,
        ) -> TwoThreeDecOutput<T> {
            assert_eq!(p1.view.input.len(), 5);
            assert_eq!(p2.view.input.len(), 5);
            assert_eq!(p3.view.input.len(), 5);

            let (x1, x2, x3, x4, x5) = (
                p1.view.input[0],
                p1.view.input[1],
                p1.view.input[2],
                p1.view.input[3],
                p1.view.input[4],
            );
            let (y1, y2, y3, y4, y5) = (
                p2.view.input[0],
                p2.view.input[1],
                p2.view.input[2],
                p2.view.input[3],
                p2.view.input[4],
            );
            let (z1, z2, z3, z4, z5) = (
                p3.view.input[0],
                p3.view.input[1],
                p3.view.input[2],
                p3.view.input[3],
                p3.view.input[4],
            );

            let (a1, a2, a3) = mpc_xor((x1, x2), (y1, y2), (z1, z2));
            let (b1, b2, b3) = mpc_xor((x3, x4), (y3, y4), (z3, z4));

            let (ab1, ab2, ab3) = mpc_and((a1, b1), (a2, b2), (a3, b3), p1, p2, p3);

            let (o1, o2, o3) = mpc_and((ab1, x5), (ab2, y5), (ab3, z5), p1, p2, p3);

            (vec![o1], vec![o2], vec![o3])
        }

        fn simulate_two_parties(
            &self,
            p: &mut Party<T>,
            p_next: &mut Party<T>,
        ) -> Result<(Output<T>, Output<T>), Error> {
            assert_eq!(p.view.input.len(), 5);
            assert_eq!(p_next.view.input.len(), 5);

            let (x1, x2, x3, x4, x5) = (
                p.view.input[0],
                p.view.input[1],
                p.view.input[2],
                p.view.input[3],
                p.view.input[4],
            );

            let (y1, y2, y3, y4, y5) = (
                p_next.view.input[0],
                p_next.view.input[1],
                p_next.view.input[2],
                p_next.view.input[3],
                p_next.view.input[4],
            );

            let a1 = x1 ^ x2;
            let b1 = x3 ^ x4;

            let a2 = y1 ^ y2;
            let b2 = y3 ^ y4;

            let (ab1, ab2) = mpc_and_verify((a1, b1), (a2, b2), p, p_next)?;
            let (o1, o2) = mpc_and_verify((ab1, x5), (ab2, y5), p, p_next)?;

            Ok((vec![o1], vec![o2]))
        }

        fn party_output_len(&self) -> usize {
            1
        }

        fn num_of_mul_gates(&self) -> usize {
            2
        }
    }

    #[test]
    fn test_full_run() {
        let mut rng = thread_rng();
        const SIGMA: usize = 40;
        let input: Vec<GF2Word<_>> = [5u32, 4, 7, 2, 9].iter().map(|&vi| vi.into()).collect();

        let circuit = SimpleCircuit1 {};
        let output = circuit.compute(&input);

        let proof = Prover::<u32, ChaCha20Rng, Keccak256>::prove::<ThreadRng, SIGMA>(
            &mut rng, &input, &circuit, &output,
        )
        .unwrap();

        Verifier::<u32, ChaCha20Rng, Keccak256>::verify(&proof, &circuit, &output).unwrap();
    }
}
