use std::ops::Div;

use byteorder::{ByteOrder, BigEndian};
use typenum::consts::{U8, U64, U128, U224, U256, U384, U512};

use digest::Digest;
use utils::buffer::{FixedBuf, FixedBuffer64, FixedBuffer128, StandardPadding};

const SHA224_INIT: [u32; 8] = [0xc1059ed8, 0x367cd507, 0x3070dd17, 0xf70e5939, 0xffc00b31,
                               0x68581511, 0x64f98fa7, 0xbefa4fa4];
const SHA256_INIT: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f,
                               0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
const U32_ROUNDS: [u32; 64] = [0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b,
                               0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01,
                               0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7,
                               0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
                               0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152,
                               0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
                               0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
                               0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
                               0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819,
                               0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08,
                               0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f,
                               0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
                               0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2];

const SHA384_INIT: [u64; 8] = [0xcbbb9d5dc1059ed8,
                               0x629a292a367cd507,
                               0x9159015a3070dd17,
                               0x152fecd8f70e5939,
                               0x67332667ffc00b31,
                               0x8eb44a8768581511,
                               0xdb0c2e0d64f98fa7,
                               0x47b5481dbefa4fa4];
const SHA512_INIT: [u64; 8] = [0x6a09e667f3bcc908,
                               0xbb67ae8584caa73b,
                               0x3c6ef372fe94f82b,
                               0xa54ff53a5f1d36f1,
                               0x510e527fade682d1,
                               0x9b05688c2b3e6c1f,
                               0x1f83d9abfb41bd6b,
                               0x5be0cd19137e2179];
const SHA512_224_INIT: [u64; 8] = [0x8c3d37c819544da2,
                                   0x73e1996689dcd4d6,
                                   0x1dfab7ae32ff9c82,
                                   0x679dd514582f9fcf,
                                   0x0f6d2b697bd44da8,
                                   0x77e36f7304c48942,
                                   0x3f9d85a86a1d36c8,
                                   0x1112e6ad91d692a1];
const SHA512_256_INIT: [u64; 8] = [0x22312194fc2bf72c,
                                   0x9f555fa3c84c64c2,
                                   0x2393b86b6f53b151,
                                   0x963877195940eabd,
                                   0x96283ee2a88effe3,
                                   0xbe5e1e2553863992,
                                   0x2b0199fc2c85b8aa,
                                   0x0eb72ddc81c52ca2];

const U64_ROUNDS: [u64; 80] = [0x428a2f98d728ae22,
                               0x7137449123ef65cd,
                               0xb5c0fbcfec4d3b2f,
                               0xe9b5dba58189dbbc,
                               0x3956c25bf348b538,
                               0x59f111f1b605d019,
                               0x923f82a4af194f9b,
                               0xab1c5ed5da6d8118,
                               0xd807aa98a3030242,
                               0x12835b0145706fbe,
                               0x243185be4ee4b28c,
                               0x550c7dc3d5ffb4e2,
                               0x72be5d74f27b896f,
                               0x80deb1fe3b1696b1,
                               0x9bdc06a725c71235,
                               0xc19bf174cf692694,
                               0xe49b69c19ef14ad2,
                               0xefbe4786384f25e3,
                               0x0fc19dc68b8cd5b5,
                               0x240ca1cc77ac9c65,
                               0x2de92c6f592b0275,
                               0x4a7484aa6ea6e483,
                               0x5cb0a9dcbd41fbd4,
                               0x76f988da831153b5,
                               0x983e5152ee66dfab,
                               0xa831c66d2db43210,
                               0xb00327c898fb213f,
                               0xbf597fc7beef0ee4,
                               0xc6e00bf33da88fc2,
                               0xd5a79147930aa725,
                               0x06ca6351e003826f,
                               0x142929670a0e6e70,
                               0x27b70a8546d22ffc,
                               0x2e1b21385c26c926,
                               0x4d2c6dfc5ac42aed,
                               0x53380d139d95b3df,
                               0x650a73548baf63de,
                               0x766a0abb3c77b2a8,
                               0x81c2c92e47edaee6,
                               0x92722c851482353b,
                               0xa2bfe8a14cf10364,
                               0xa81a664bbc423001,
                               0xc24b8b70d0f89791,
                               0xc76c51a30654be30,
                               0xd192e819d6ef5218,
                               0xd69906245565a910,
                               0xf40e35855771202a,
                               0x106aa07032bbd1b8,
                               0x19a4c116b8d2d0c8,
                               0x1e376c085141ab53,
                               0x2748774cdf8eeb99,
                               0x34b0bcb5e19b48a8,
                               0x391c0cb3c5c95a63,
                               0x4ed8aa4ae3418acb,
                               0x5b9cca4f7763e373,
                               0x682e6ff3d6b2b8a3,
                               0x748f82ee5defb2fc,
                               0x78a5636f43172f60,
                               0x84c87814a1f0ab72,
                               0x8cc702081a6439ec,
                               0x90befffa23631e28,
                               0xa4506cebde82bde9,
                               0xbef9a3f7b2c67915,
                               0xc67178f2e372532b,
                               0xca273eceea26619c,
                               0xd186b8c721c0c207,
                               0xeada7dd6cde0eb1e,
                               0xf57d4f7fee6ed178,
                               0x06f067aa72176fba,
                               0x0a637dc5a2c898a6,
                               0x113f9804bef90dae,
                               0x1b710b35131c471b,
                               0x28db77f523047d84,
                               0x32caab7b40c72493,
                               0x3c9ebe0a15c9bebc,
                               0x431d67c49c100d4c,
                               0x4cc5d4becb3e42b6,
                               0x597f299cfc657e2a,
                               0x5fcb6fab3ad6faec,
                               0x6c44198c4a475817];

#[derive(Copy, Clone, Debug)]
struct State<T: Copy> {
    state: [T; 8],
}

impl State<u32> {
    fn process_block(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 64);

        let mut words = [0u32; 64];

        for (d, w) in data.chunks(4).zip(words.iter_mut()) {
            *w = BigEndian::read_u32(d);
        }
        for i in 16..64 {
            let s0 = words[i - 15].rotate_right(7) ^ words[i - 15].rotate_right(18) ^
                     (words[i - 15] >> 3);
            let s1 = words[i - 2].rotate_right(17) ^ words[i - 2].rotate_right(19) ^
                     (words[i - 2] >> 10);
            words[i] = words[i - 16]
                           .wrapping_add(s0)
                           .wrapping_add(words[i - 7])
                           .wrapping_add(s1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

        for (&word, &round) in words.iter().zip(U32_ROUNDS.iter()) {
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let tmp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(word).wrapping_add(round);
            let tmp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(tmp1);
            d = c;
            c = b;
            b = a;
            a = tmp1.wrapping_add(tmp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

impl State<u64> {
    fn process_block(&mut self, data: &[u8]) {
        assert_eq!(data.len(), 128);

        let mut words = [0u64; 80];

        for (d, w) in data.chunks(8).zip(words.iter_mut()) {
            *w = BigEndian::read_u64(d);
        }

        for i in 16..80 {
            let s0 = words[i - 15].rotate_right(1) ^ words[i - 15].rotate_right(8) ^
                     (words[i - 15] >> 7);
            let s1 = words[i - 2].rotate_right(19) ^ words[i - 2].rotate_right(61) ^
                     (words[i - 2] >> 6);
            words[i] = words[i - 16]
                           .wrapping_add(s0)
                           .wrapping_add(words[i - 7])
                           .wrapping_add(s1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

        for (&word, &round) in words.iter().zip(U64_ROUNDS.iter()) {
            let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ (!e & g);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let tmp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(word).wrapping_add(round);
            let tmp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(tmp1);
            d = c;
            c = b;
            b = a;
            a = tmp1.wrapping_add(tmp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

macro_rules! impl_sha(
    ($name:ident, $buffer:ty, $init:ident, $state:ty, $bsize:ty, $bits:ty) => {
        #[derive(Clone)]
        pub struct $name {
            state: State<$state>,
            buffer: $buffer,
            length: u64
        }

        impl Default for $name {
            fn default() -> Self {
                $name {
                    state: State { state: $init },
                    buffer: <$buffer>::new(),
                    length: 0
                }
            }
        }

        impl Digest for $name {
            type OutputBits = $bits;
            type OutputBytes = <$bits as Div<U8>>::Output;

            type BlockSize = $bsize;

            fn update<T: AsRef<[u8]>>(&mut self, data: T) {
                let data = data.as_ref();
                self.length += data.len() as u64;

                let state = &mut self.state;
                self.buffer.input(data, |d| state.process_block(d));
            }

            fn result<T: AsMut<[u8]>>(mut self, mut out: T) {
                let mut out = out.as_mut();
                assert!(out.len() >= Self::output_bytes());

                let state = &mut self.state;

                self.buffer.standard_padding(8, |d| state.process_block(d));
                BigEndian::write_u64(self.buffer.next(8), self.length << 3);
                state.process_block(self.buffer.full_buffer());

                for i in &mut state.state {
                    *i = i.to_be();
                }

                unsafe {
                    use std::ptr;
                    ptr::copy_nonoverlapping(
                        state.state.as_ptr() as *const u8,
                        out.as_mut_ptr(),
                        Self::output_bytes())
                };
            }
        }
    };
(low $name:ident, $init:ident, $bits:ty) => {
    impl_sha!($name, FixedBuffer64, $init, u32, U64, $bits);
};
(high $name:ident, $init:ident, $bits:ty) => {
    impl_sha!($name, FixedBuffer128, $init, u64, U128, $bits);
};
);

impl_sha!(low  Sha224, SHA224_INIT, U224);
impl_sha!(low  Sha256, SHA256_INIT, U256);
impl_sha!(high Sha384, SHA384_INIT, U384);
impl_sha!(high Sha512, SHA512_INIT, U512);

impl_sha!(high Sha512224, SHA512_224_INIT, U224);
impl_sha!(high Sha512256, SHA512_256_INIT, U256);
