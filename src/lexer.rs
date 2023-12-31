use std::error::Error;
use std::fmt::Debug;
use std::{io, ops};
use crate::read::Analyser;

/// Lexer struct which contains current cursor position and contents to analyze
///
/// # Type Parameters
/// * `T` - Any type that is Sized (has a constant size in memory), and can be compared for equality.
pub struct Lexer<T: Sized + PartialEq + Copy> {
    cursor:      usize,
    contents:    Vec<T>
}

impl<T: Sized + PartialEq + Copy> Lexer<T> {
    pub fn new<C: AsRef<[T]>>(content: C) -> Self {
        Self {
            cursor: 0,
            contents: content.as_ref().to_vec(),
        }
    }

    pub fn extract(
        &mut self,
        range: ops::Range<usize>
    ) -> Vec<T> {
        let start = range.start;
        let end = range.end;
        let extraction_result = self.contents.drain(range).collect::<Vec<T>>();

        if start <= self.cursor && self.cursor < end {
            self.cursor = start;
        } else if end < self.cursor {
            self.cursor -= end;
        }

        extraction_result
    }
}

/// Defines methods for generating a token.
pub trait Token<T: Sized + PartialEq + Copy> where Self: Sized {
    type Error: From<io::Error> + Debug;

    /// Generates the next token from Lexer.
    ///
    /// # Arguments
    ///
    /// * `lexer` - Lexer from which the token should be generated.
    fn next_token(lexer: &mut Lexer<T>) -> Result<Self, Self::Error>;
}

/// Defines methods for generating a token using a specific lexical scope (can be used for lexer-hacks).
pub trait ScopedToken<T: Sized + PartialEq + Copy> where Self: Sized {
    type Scope: Default;
    type Error: From<io::Error> + Debug;

    /// Generates the next token from Lexer using the defined Scope.
    ///
    /// # Arguments
    ///
    /// * `lexer` - Lexer from which the token should be generated.
    /// * `scope` - the scope for generating the token.
    fn next_token(lexer: &mut Lexer<T>, scope: &mut Self::Scope) -> Result<Self, Self::Error>;
}

impl<T: Sized + PartialEq + Copy, Scoped: ScopedToken<T>> Token<T> for Scoped {
    type Error = <Scoped as ScopedToken<T>>::Error;

    /// Generates the next token in the default scope.
    ///
    /// # Arguments
    ///
    /// * `lexer` - Lexer from which the token should be generated.
    fn next_token(lexer: &mut Lexer<T>) -> Result<Self, Self::Error> {
        <Scoped as ScopedToken<T>>::next_token(lexer, &mut Scoped::Scope::default())
    }
}

impl<T: Sized + PartialEq + Copy> Lexer<T> {
    pub fn tokenize_until_end<
        TokenType: Token<T>
    >(mut self) -> Result<Vec<TokenType>, TokenType::Error> {
        let mut tokens = vec![];
        while !self.is_end() {
            tokens.push(TokenType::next_token(&mut self)?)
        }
        Ok(tokens)
    }
}

impl<T: Sized + PartialEq + Copy> Analyser<T> for Lexer<T> {
    /// Get the entire sequence being analyzed
    ///
    /// # Returns
    /// Array slice of the sequence being analyzed
    fn contents(&self) -> &[T] { &self.contents[..] }

    /// Get the current position of cursor within the sequence
    ///
    /// # Returns
    /// Cursor position as usize
    fn pos(&self) -> usize { self.cursor }

    /// Consumes the analyser, returning the sequence being analyzed
    ///
    /// # Returns
    /// The sequence being analyzed as an owned vector
    fn drain(self) -> Vec<T> { self.contents }

    /// Sets the cursor to a given position
    ///
    /// # Parameters
    /// * `position: usize` - The index in sequence, where cursor will be placed
    ///
    /// # Returns
    /// `std::io::Result<()>` - Ok if operation successful, otherwise an Err with the `std::io::Error`
    fn set_pos(&mut self, position: usize) -> io::Result<()> { Ok(self.cursor = position) }
}