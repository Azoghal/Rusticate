// To facilitate non-char features having their own codes
// For example the end and clear code
// Or non-text files e.g. images

pub trait HashableToken: Eq + PartialEq + Copy + Hash {}
impl<T: Eq + PartialEq + Copy + Hash> HashableToken for T {}

use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

// https://stackoverflow.com/questions/26070559/is-there-any-way-to-create-a-type-alias-for-multiple-traits

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum ControlToken {
    END,
    CLEAR,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct Token<T: HashableToken> {
    value: Option<T>,
    control_token: Option<ControlToken>,
}

impl<T: HashableToken> Token<T> {
    pub fn new(value: T) -> Token<T> {
        Token {
            value: Some(value),
            control_token: None,
        }
    }

    pub fn new_control(control_token: ControlToken) -> Token<T> {
        Token {
            value: None,
            control_token: Some(control_token),
        }
    }

    fn get_value(&self) -> Option<T> {
        self.value
    }
}

// Specific Tokens

// pub type AsciiToken = Token<char>;
