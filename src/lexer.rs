use std::{
    str::Chars,
};

use language_reporting::{
    Diagnostic,
    Label,
};

use mltt_span::{
    ByteIndex,
    ByteSize,
    File,
    FileSpan,
};

use crate::types::{
    Token,
    TokenKind,
};

const KEYWORDS: [&str; 4] = [
    "true",
    "yes",
    "false",
    "no",
];

fn is_symbol(ch: char) -> bool {
    match ch {
        '~' | '!' | '@' | ':' | '|' | '&' | '#' | '^' => true,
        _ => false,
    }
}

fn is_identifier_start(ch: char) -> bool {
    match ch {
        'a'..='z' | 'A'..='Z' | '_' => true,
        _ => false,
    }
}

/// An iterator over a source string yielding `Token`s for subsequent use by
/// a `Parser` instance.
pub struct Lexer<'file> {
    /// The file being lexed
    file: &'file File,

    /// An iterator of unicode characters to consume
    chars: Chars<'file>,

    /// One character of lookahead
    peeked: Option<char>,

    /// Start position of the next token to be emitted
    token_start: ByteIndex,

    /// End position of the next token to be emitted
    // I *think* this is actually "end + 1", see https://gitter.im/pikelet-lang/Lobby?at=5c65912a28c89123cbcb0614
    token_end: ByteIndex,

    /// Diagnostics accumulated during lexing
    diagnostics: Vec<Diagnostic<FileSpan>>,
}

impl<'file> Lexer<'file> {
    /// Create a new `Lexer` from a source file
    pub fn new(file: &'file File) -> Lexer<'file> {
        let mut chars = file.contents().chars();
        let peeked = chars.next();

        Self {
            file,
            chars,
            peeked,
            token_start: ByteIndex::from(0),
            token_end: ByteIndex::from(0),
            diagnostics: Vec::new(),
        }
    }

    /// Record a diagnostic
    fn add_diagnostic(&mut self, diagnostic: Diagnostic<FileSpan>) {
        log::debug!("diagnostic added @ {}..{}: {:?}", self.token_span().start().to_usize(), self.token_span().end().to_usize(), diagnostic.message);
        self.diagnostics.push(diagnostic);
    }

    /// Take the diagnostics from the lexer, leaving an empty collection
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic<FileSpan>> {
        std::mem::replace(&mut self.diagnostics, Vec::new())
    }

    /// The next character, if any
    fn peek(&self) -> Option<char> {
        self.peeked
    }

    /// Consume the current character and load the new one into the internal state, returning the just-consumed character
    fn advance(&mut self) -> Option<char> {
        let cur = std::mem::replace(&mut self.peeked, self.chars.next());
        // TODO: This causes single-char tokens to have a span of 2 bytes
        // though this may be intentional (see the non-doc comment on self.token_end).
        self.token_end += cur.map_or(ByteSize::from(0), ByteSize::from_char_len_utf8);
        cur
    }

    fn span(&self, start: ByteIndex, end: ByteIndex) -> FileSpan {
        FileSpan::new(self.file.id(), start, end)
    }

    /// Returns the span of the current token in the source file
    fn token_span(&self) -> FileSpan {
        self.span(self.token_start, self.token_end)
    }

    /// Returns the string slice of the current token
    ///
    /// Panics if `self.token_start` or `self.token_end` are out of bounds of `self.file.contents()`
    fn token_slice(&self) -> &'file str {
        &self.file.contents()[self.token_start.to_usize()..self.token_end.to_usize()]
    }

    /// Emit a token and reset the start position, ready for the next token
    fn emit(&mut self, kind: TokenKind) -> Token<'file> {
        let slice = self.token_slice();
        let span = self.token_span();
        self.token_start = self.token_end;

        Token {
            kind,
            slice,
            span,
        }
    }

    /// Consume a token, returning its tag or `None` if end-of-file has been reached
    fn consume_token(&mut self) -> Option<TokenKind> {
        self.advance().map(|ch| match ch {
            // We put non-composite symbols here (instead of in `consume_symbol`)
            // so they don't get combined.
            '~' => TokenKind::Tilde,
            '!' => TokenKind::Bang,
            '@' => TokenKind::At,
            '^' => {
                // NOTE: We want to move this code into the, eventual, Parser
                //       since at that point we'll be operating on a token
                //       stream which gives us a more accurate view of the
                //       file contents and better spans.
                match self.peek() {
                    Some(c) if !is_identifier_start(c) => {
                        let is_whitespace = c.is_whitespace();
                        let span = self.token_span();

                        self.add_diagnostic(
                            Diagnostic::new_error(format!(
                                "expected an identifier after `^`, found {}",
                                if is_whitespace { "whitespace".into() } else { format!("`{}`", c) }
                            ))
                            .with_label(Label::new_primary(span))
                            .with_code("E0001")
                        );

                        self.add_diagnostic(
                            Diagnostic::new_help(format!(
                                "remove this {}",
                                if is_whitespace { "whitespace" } else { "character" }
                            ))
                            .with_label(Label::new_secondary(span))
                        );

                        TokenKind::Error
                    },
                    // None => { unexpected eof },
                    _ => TokenKind::Caret,
                }
            },
            ':' => TokenKind::Colon,
            _ if is_symbol(ch) => self.consume_symbol(),
            _ if ch.is_whitespace() => self.consume_whitespace(),
            _ if is_identifier_start(ch) => self.consume_identifier(),
            _ => {
                self.add_diagnostic(
                    Diagnostic::new_error(format!("unexpected character `{}`", ch))
                        .with_label(Label::new_primary(self.token_span()))
                );

                TokenKind::Error
            }
        })
    }

    /// Consume a symbol
    fn consume_symbol(&mut self) -> TokenKind {
        self.skip_while(is_symbol);

        match self.token_slice() {
            "&&" => TokenKind::LogicalAnd,
            "||" => TokenKind::LogicalOr,
            slice if slice.starts_with("#") => self.consume_comment(),
            _ => TokenKind::Symbol,
        }
    }

    fn consume_comment(&mut self) -> TokenKind {
        // TODO: What about `\r\n`?
        self.skip_while(|ch| ch != '\n');

        TokenKind::Comment
    }

    /// Consume an identifier
    fn consume_identifier(&mut self) -> TokenKind {
        self.skip_while(is_identifier_start);

        if KEYWORDS.contains(&self.token_slice()) {
            TokenKind::Keyword
        } else {
            TokenKind::Identifier
        }
    }

    /// Consume whitespace
    fn consume_whitespace(&mut self) -> TokenKind {
        self.skip_while(char::is_whitespace);
        TokenKind::Whitespace
    }

    /// Skip characters while the predicate matches the lookahead character.
    fn skip_while(&mut self, mut keep_going: impl FnMut(char) -> bool) {
        while self.peek().map_or(false, |ch| keep_going(ch)) {
            self.advance();
        }
    }
}

/// This is where the magic happens.
///
/// `Lexer`-using code will call `lexer.collect()` to actually run the lexer
/// and collect the resultant token stream.
impl<'file> Iterator for Lexer<'file> {
    type Item = Token<'file>;

    fn next(&mut self) -> Option<Self::Item> {
        let opt_token = self.consume_token()
            .map(|tag| self.emit(tag));

        match &opt_token {
            Some(token) => log::debug!("emit {:?}", token),
            _ => log::debug!("eof"),
        }

        opt_token
    }
}

#[cfg(test)]
mod tests {
    use language_reporting::Severity;
    use mltt_span::Files;
    use super::*;

    /// A handy macro to give us a nice syntax for declaring test cases
    ///
    /// This was inspired by the tests in the LALRPOP lexer
    macro_rules! test {
        ($src:expr, $($span:expr => $token:expr,)*) => {{
            let _ = simple_logger::init(); // ignore failure

            let mut files = Files::new();
            let file_id = files.add("test", $src);
            let lexed_tokens: Vec<_> = Lexer::new(&files[file_id])
                .map(|result| result)
                .collect();
            let expected_tokens = vec![$({
                let (kind, slice) = $token;
                let start = ByteIndex::from($span.find("~").unwrap());
                let end = ByteIndex::from($span.rfind("~").unwrap()) + ByteSize::from(1);
                let span = FileSpan::new(file_id, start, end);
                Token { kind, slice, span }
            }),*];

            assert_eq!(lexed_tokens, expected_tokens);
        }};
    }

    #[test]
    fn data() {
        test! {
            "wowza",
            "~~~~~" => (TokenKind::Identifier, "wowza"),
        }

        test! {
            " wowza ",
            "~      " => (TokenKind::Whitespace, " "),
            " ~~~~~ " => (TokenKind::Identifier, "wowza"),
            "      ~" => (TokenKind::Whitespace, " "),
        }
    }

    #[test]
    fn non_ident_after_caret_diagnostic() {
        // Arrange
        let mut files = Files::new();
        let src = "^:";
        let file_id = files.add("test", src);
        let mut lexer = Lexer::new(&files[file_id]);

        let span_caret = FileSpan::new(file_id, ByteIndex::from(0), ByteIndex::from(1));

        // Act
        let tokens = lexer.by_ref().collect::<Vec<_>>();

        // Assert
        assert_eq!(tokens, vec![
            Token {
                // error on this token because we haven't lexed the next token yet so don't have its span
                kind: TokenKind::Error,
                slice: "^",
                span: span_caret,
            },
            Token {
                kind: TokenKind::Colon,
                slice: ":",
                span: FileSpan::new(file_id, ByteIndex::from(1), ByteIndex::from(2)),
            },
        ]);

        let diags = lexer.take_diagnostics();
        let diag = diags.first().expect("Lexer should have a diagnostic");

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(&diag.message, "expected an identifier after `^`, found `:`");

        let label = diag.labels.first().expect("Diagnostic should have a label");
        assert_eq!(label.span, span_caret);
    }
}
