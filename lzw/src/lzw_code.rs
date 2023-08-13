use mutable_trie::TrieVal;

use crate::LzwSpec;
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct Code {
    code: u32,
    used_bits: u8, // how many bits of the 32 bit integer actually constitute the code
}

impl Code {
    pub fn new(code: u32, used_bits: u8) -> Code {
        assert_eq!(code >> used_bits, 0);
        Code { code, used_bits }
    }

    pub fn get_code(self) -> u32 {
        self.code
    }

    pub fn get_used_bits(self) -> u8 {
        self.used_bits
    }
}

impl PartialEq for Code {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Code({},{})", self.code, self.used_bits)
    }
}

#[derive()]
pub struct CodeGenerator {
    current_code: u32,
    variable_width: bool,
    width: u8,
    min_width: u8,
    max_width: u8,
    pack_msb_first: bool,
}

impl CodeGenerator {
    pub fn new(lzw_spec: LzwSpec) -> CodeGenerator {
        // Perform any neccessary validation
        CodeGenerator {
            current_code: 0,
            variable_width: lzw_spec.variable_width,
            width: lzw_spec.width,
            min_width: lzw_spec.min_width,
            max_width: lzw_spec.max_width,
            pack_msb_first: lzw_spec.pack_msb_first,
        }
    }

    pub fn get_next_code(&mut self) -> Option<Code> {
        // Check that current bit width is respected
        if self.current_code >> self.width != 0 {
            tracing::debug!(
                "All codes for current code width {} already used",
                self.width,
            );
            None
        } else {
            // current fits within bit width
            let res = Some(Code::new(self.current_code, self.width));
            self.current_code += 1;
            res
        }
    }
}

impl Iterator for CodeGenerator {
    type Item = Code;
    fn next(&mut self) -> Option<Self::Item> {
        // Check that current bit width is respected
        if self.current_code >> self.width != 0 {
            tracing::debug!(
                "All codes for current code width {} already used",
                self.width,
            );
            None
        } else {
            // current fits within bit width
            let res = Some(Code::new(self.current_code, self.width));
            self.current_code += 1;
            res
        }
    }
}
