use octavo::digest::md4::Md4;

use digest::Test;

const TESTS: &'static [Test<'static>] =
    &[Test {
          input: b"",
          output: &[0x31, 0xd6, 0xcf, 0xe0, 0xd1, 0x6a, 0xe9, 0x31, 0xb7, 0x3c, 0x59, 0xd7, 0xe0,
                    0xc0, 0x89, 0xc0],
      },
      Test {
          input: b"a",
          output: &[0xbd, 0xe5, 0x2c, 0xb3, 0x1d, 0xe3, 0x3e, 0x46, 0x24, 0x5e, 0x05, 0xfb, 0xdb,
                    0xd6, 0xfb, 0x24],
      },
      Test {
          input: b"abc",
          output: &[0xa4, 0x48, 0x01, 0x7a, 0xaf, 0x21, 0xd8, 0x52, 0x5f, 0xc1, 0x0a, 0xe8, 0x7a,
                    0xa6, 0x72, 0x9d],
      },
      Test {
          input: b"message digest",
          output: &[0xd9, 0x13, 0x0a, 0x81, 0x64, 0x54, 0x9f, 0xe8, 0x18, 0x87, 0x48, 0x06, 0xe1,
                    0xc7, 0x01, 0x4b],
      },
      Test {
          input: b"abcdefghijklmnopqrstuvwxyz",
          output: &[0xd7, 0x9e, 0x1c, 0x30, 0x8a, 0xa5, 0xbb, 0xcd, 0xee, 0xa8, 0xed, 0x63, 0xdf,
                    0x41, 0x2d, 0xa9],
      },
      Test {
          input: b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
          output: &[0x04, 0x3f, 0x85, 0x82, 0xf2, 0x41, 0xdb, 0x35, 0x1c, 0xe6, 0x27, 0xe1, 0x53,
                    0xe7, 0xf0, 0xe4],
      },
      Test {
          input: b"12345678901234567890123456789012345678901234567890123456789012345678901234567890",
          output: &[0xe3, 0x3b, 0x4d, 0xdc, 0x9c, 0x38, 0xf2, 0x19, 0x9c, 0x3e, 0x7b, 0x16, 0x4f,
                    0xcc, 0x05, 0x36],
      }];

#[test]
fn rfc1320_test_vectors() {
    for test in TESTS {
        test.test(Md4::default());
    }
}
