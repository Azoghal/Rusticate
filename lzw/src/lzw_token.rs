// To facilitate non-char features having their own codes
// For example the end and clear code
// Or non-text files e.g. images

use std::any::Any;

trait Token<T> {
    fn get_value(&self) -> Option<T>;
}

// Special Tokens

struct EndCodeToken {}

impl<T> Token<T> for EndCodeToken {
    fn get_value(&self) -> Option<T> {
        None
    }
}

struct ClearCodeToken {}

impl<T> Token<T> for ClearCodeToken {
    fn get_value(&self) -> Option<T> {
        None
    }
}

// Generic Tokens

struct GenToken<T> {
    value: T,
}

impl<T> Token<T> for GenToken<T>
where
    T: Copy,
{
    fn get_value(&self) -> Option<T> {
        Some(self.value)
    }
}

// Specific Tokens

// Type for any ascii token
// struct AsciiToken {
//     value: char,
// }

// impl Token<char> for AsciiToken {
//     fn get_value(&self) -> Option<char> {
//         Some(self.value)
//     }
// }
