use byteorder::{ByteOrder, LittleEndian};
use typenum::consts::{U20, U64, U160};

use digest::Digest;
use utils::buffer::{FixedBuf, FixedBuffer64, StandardPadding};

const LEFT_PICK: [usize; 80] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 7, 4, 13, 1,
                                10, 6, 15, 3, 12, 0, 9, 5, 2, 14, 11, 8, 3, 10, 14, 4, 9, 15, 8,
                                1, 2, 7, 0, 6, 13, 11, 5, 12, 1, 9, 11, 10, 0, 8, 12, 4, 13, 3, 7,
                                15, 14, 5, 6, 2, 4, 0, 5, 9, 7, 12, 2, 10, 14, 1, 3, 8, 11, 6, 15,
                                13];
const RIGHT_PICK: [usize; 80] = [5, 14, 7, 0, 9, 2, 11, 4, 13, 6, 15, 8, 1, 10, 3, 12, 6, 11, 3,
                                 7, 0, 13, 5, 10, 14, 15, 8, 12, 4, 9, 1, 2, 15, 5, 1, 3, 7, 14,
                                 6, 9, 11, 8, 12, 2, 10, 0, 4, 13, 8, 6, 4, 1, 3, 11, 15, 0, 5,
                                 12, 2, 13, 9, 7, 10, 14, 12, 15, 10, 4, 1, 5, 8, 7, 6, 2, 13, 14,
                                 0, 3, 9, 11];

const LEFT_ROTATE: [u32; 80] = [11, 14, 15, 12, 5, 8, 7, 9, 11, 13, 14, 15, 6, 7, 9, 8, 7, 6, 8,
                                13, 11, 9, 7, 15, 7, 12, 15, 9, 11, 7, 13, 12, 11, 13, 6, 7, 14,
                                9, 13, 15, 14, 8, 13, 6, 5, 12, 7, 5, 11, 12, 14, 15, 14, 15, 9,
                                8, 9, 14, 5, 6, 8, 6, 5, 12, 9, 15, 5, 11, 6, 8, 13, 12, 5, 12,
                                13, 14, 11, 8, 5, 6];
const RIGHT_ROTATE: [u32; 80] = [8, 9, 9, 11, 13, 15, 15, 5, 7, 7, 8, 11, 14, 14, 12, 6, 9, 13,
                                 15, 7, 12, 8, 9, 11, 7, 7, 12, 7, 6, 15, 13, 11, 9, 7, 15, 11, 8,
                                 6, 6, 14, 12, 13, 5, 14, 13, 13, 7, 5, 15, 5, 8, 11, 14, 14, 6,
                                 14, 6, 9, 12, 9, 12, 5, 15, 8, 8, 5, 12, 9, 12, 5, 14, 6, 8, 13,
                                 6, 5, 15, 13, 11, 11];

const LEFT_CONST: [u32; 5] = [0x00000000, 0x5A827999, 0x6ED9EBA1, 0x8F1BBCDC, 0xA953FD4E];
const RIGHT_CONST: [u32; 5] = [0x50A28BE6, 0x5C4DD124, 0x6D703EF3, 0x7A6D76E9, 0x00000000];

#[derive(Copy, Clone, Debug)]
struct State {
    state: [u32; 5],
}

impl State {
    fn new() -> Self {
        State { state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0] }
    }

    fn process_block(&mut self, block: &[u8]) {
        assert_eq!(block.len(), 64);

        fn ff(x: u32, y: u32, z: u32) -> u32 {
            x ^ y ^ z
        }
        fn gg(x: u32, y: u32, z: u32) -> u32 {
            (x & y) | (!x & z)
        }
        fn hh(x: u32, y: u32, z: u32) -> u32 {
            (x | !y) ^ z
        }
        fn ii(x: u32, y: u32, z: u32) -> u32 {
            (x & z) | (y & !z)
        }
        fn jj(x: u32, y: u32, z: u32) -> u32 {
            x ^ (y | !z)
        }

        let funcs: [fn(u32, u32, u32) -> u32; 5] = [ff, gg, hh, ii, jj];

        fn process<F>(block: &mut [u32], value: u32, rot: u32, c: u32, func: F)
            where F: Fn(u32, u32, u32) -> u32
        {
            let tmp = block[0]
                          .wrapping_add(func(block[1], block[2], block[3]))
                          .wrapping_add(value)
                          .wrapping_add(c)
                          .rotate_left(rot)
                          .wrapping_add(block[4]);
            block[0] = block[4];
            block[4] = block[3];
            block[3] = block[2].rotate_left(10);
            block[2] = block[1];
            block[1] = tmp;
        }

        let mut data = [0u32; 16];

        for (c, v) in block.chunks(4).zip(data.iter_mut()) {
            *v = LittleEndian::read_u32(c);
        }

        let mut left = self.state.clone();
        let mut right = self.state.clone();

        for i in 0..80 {
            process(&mut left,
                    data[LEFT_PICK[i]],
                    LEFT_ROTATE[i],
                    LEFT_CONST[i / 16],
                    funcs[i / 16]);
            process(&mut right,
                    data[RIGHT_PICK[i]],
                    RIGHT_ROTATE[i],
                    RIGHT_CONST[i / 16],
                    funcs[4 - (i / 16)]);
        }

        let tmp = self.state[1].wrapping_add(left[2]).wrapping_add(right[3]);
        self.state[1] = self.state[2].wrapping_add(left[3]).wrapping_add(right[4]);
        self.state[2] = self.state[3].wrapping_add(left[4]).wrapping_add(right[0]);
        self.state[3] = self.state[4].wrapping_add(left[0]).wrapping_add(right[1]);
        self.state[4] = self.state[0].wrapping_add(left[1]).wrapping_add(right[2]);
        self.state[0] = tmp;
    }
}

#[derive(Clone)]
pub struct Ripemd160 {
    state: State,
    length: u64,
    buffer: FixedBuffer64,
}

impl Default for Ripemd160 {
    fn default() -> Self {
        Ripemd160 {
            state: State::new(),
            length: 0,
            buffer: FixedBuffer64::new(),
        }
    }
}

impl Digest for Ripemd160 {
    type OutputBits = U160;
    type OutputBytes = U20;

    type BlockSize = U64;

    fn update<T>(&mut self, update: T)
        where T: AsRef<[u8]>
    {
        let update = update.as_ref();
        self.length += update.len() as u64;

        let state = &mut self.state;
        self.buffer.input(update, |d| state.process_block(d));
    }

    fn result<T: AsMut<[u8]>>(mut self, mut out: T) {
        let state = &mut self.state;

        self.buffer.standard_padding(8, |d| state.process_block(d));
        LittleEndian::write_u64(self.buffer.next(8), self.length << 3);
        state.process_block(self.buffer.full_buffer());

        let mut out = out.as_mut();
        assert!(out.len() >= Self::output_bytes());
        for (c, &v) in out.chunks_mut(4).zip(&state.state) {
            LittleEndian::write_u32(c, v);
        }
    }
}
