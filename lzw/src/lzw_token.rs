// To facilitate non-char features having their own codes
// For example the end and clear code
// Or non-text files e.g. images

use std::cmp::{Eq, PartialEq};
use std::hash::Hash;

pub trait HashableToken: Eq + PartialEq + Copy + Hash + std::fmt::Debug {}
impl<T: Eq + PartialEq + Copy + Hash + std::fmt::Debug> HashableToken for T {}

pub trait ValToken: HashableToken {}
impl<T: HashableToken> ValToken for T {}

// An LzwToken is either a value token (subject of compression) or a control character
// Any value token must be hashable etc.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum LzwToken<T>
where
    T: ValToken,
{
    Control(ControlToken),
    Value(T),
}

impl<T: ValToken> LzwToken<T> {
    pub fn new_control_end() -> LzwToken<T> {
        LzwToken::Control(ControlToken::End)
    }

    pub fn new_control_clear() -> LzwToken<T> {
        LzwToken::Control(ControlToken::Clear)
    }

    pub fn new(val: T) -> LzwToken<T> {
        LzwToken::Value(val)
    }
}

// https://stackoverflow.com/questions/26070559/is-there-any-way-to-create-a-type-alias-for-multiple-traits

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum ControlToken {
    End,
    Clear,
}

// Specific Tokens

pub type AsciiToken = LzwToken<char>;

// pub type AsciiToken = Token<char>;
