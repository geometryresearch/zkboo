use std::{
    fmt::Display,
    ops::{BitAnd, BitXor},
};

use rand::{CryptoRng, RngCore, SeedableRng};

use crate::{
    gf2_word::{BitUtils, BytesUitls, GF2Word, GenRand},
    key::Key,
    tape::Tape,
    view::View,
};

/// A party in the MPC protocol has a random tape and a `View`.
pub struct Party<T>
where
    T: Copy
        + Default
        + Display
        + BitAnd<Output = T>
        + BitXor<Output = T>
        + BitUtils
        + BytesUitls
        + GenRand,
{
    pub tape: Tape<T>,
    pub view: View<T>,
}

impl<T> Party<T>
where
    T: Copy
        + Default
        + Display
        + BitAnd<Output = T>
        + BitXor<Output = T>
        + BitUtils
        + BytesUitls
        + GenRand,
{
    pub fn new<TapeR: SeedableRng<Seed = Key> + RngCore + CryptoRng>(
        share: Vec<GF2Word<T>>,
        k: Key,
        tape_len: usize,
    ) -> Self {
        let tape = Tape::<T>::from_key::<TapeR>(k, tape_len);
        let view = View::new(share);

        Self { view, tape }
    }

    pub fn from_tape_and_view(view: View<T>, tape: Tape<T>) -> Self {
        Self { tape, view }
    }

    pub fn read_tape(&mut self) -> GF2Word<T> {
        self.tape.read_next()
    }

    pub fn read_view(&mut self) -> GF2Word<T> {
        self.view.read_next()
    }
}
