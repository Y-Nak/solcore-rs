use logos::Logos;

use super::{MAX_SYNTAX_NESTING, errors::lex_error, recovery::trace_recovery};
use crate::{
    lexer::{LexedCommentKind, Token},
    types::*,
};

#[cfg(test)]
pub(super) fn tokenize<'src>(src: &'src str) -> (Vec<(Token<'src>, LexSpan)>, Vec<ParsedError>) {
    let (mut tokens, _, mut errors) = tokenize_impl(src, 0);

    truncate_excessive_nesting(&mut tokens, &mut errors);
    (tokens, errors)
}

pub(super) fn tokenize_with_comments<'src>(
    src: &'src str,
) -> (
    Vec<(Token<'src>, LexSpan)>,
    Vec<ParsedSourceComment<'src>>,
    Vec<ParsedError>,
) {
    let (mut tokens, comments, mut errors) = tokenize_impl(src, 0);
    truncate_excessive_nesting(&mut tokens, &mut errors);
    (tokens, comments, errors)
}

fn truncate_excessive_nesting(
    tokens: &mut Vec<(Token<'_>, LexSpan)>,
    errors: &mut Vec<ParsedError>,
) {
    let mut closers = Vec::new();
    for (token, span) in tokens.iter() {
        match token {
            Token::LParen | Token::LBracket | Token::LBrace => {
                closers.push(match token {
                    Token::LParen => Token::RParen,
                    Token::LBracket => Token::RBracket,
                    _ => Token::RBrace,
                });
                if closers.len() > MAX_SYNTAX_NESTING {
                    let span = *span;
                    trace_recovery("nesting_limit", span);
                    errors.push(ParsedError::new(
                        span,
                        format!(
                            "delimiter nesting exceeds the compiler limit of {MAX_SYNTAX_NESTING}"
                        ),
                    ));
                    tokens.clear();
                    return;
                }
            }
            Token::RParen | Token::RBrace | Token::RBracket if closers.last() == Some(token) => {
                // An unrelated closing token cannot end a recursive grammar
                // scope. Recovery can skip a stray `)` and continue parsing
                // inside a block, so counting it against an open `{` would let
                // malformed input evade this bound before recovery runs.
                closers.pop();
            }
            _ => {}
        }
    }
}

pub(super) fn tokenize_with_base<'src>(
    src: &'src str,
    base_offset: usize,
) -> (
    Vec<(Token<'src>, LexSpan)>,
    Vec<ParsedError>,
    Vec<ParsedError>,
) {
    let (mut tokens, _, lexer_errors) = tokenize_impl(src, base_offset);
    let mut nesting_errors = Vec::new();
    truncate_excessive_nesting(&mut tokens, &mut nesting_errors);
    (tokens, lexer_errors, nesting_errors)
}

fn tokenize_impl<'src>(
    src: &'src str,
    base_offset: usize,
) -> (
    Vec<(Token<'src>, LexSpan)>,
    Vec<ParsedSourceComment<'src>>,
    Vec<ParsedError>,
) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut lexer = Token::lexer(src).spanned();

    for (tok, span) in lexer.by_ref() {
        let raw_span = span.clone();
        let span = LexSpan::from((span.start + base_offset)..(span.end + base_offset));
        match tok {
            Ok(tok) => tokens.push((tok, span)),
            Err(err) => {
                trace_recovery("invalid_token", span);
                errors.push(lex_error(src, raw_span.start, raw_span.end, span, err));
            }
        }
    }

    let comments = lexer
        .extras
        .comments
        .iter()
        .map(|comment| {
            let (kind, text_start, text_end) = match comment.kind {
                LexedCommentKind::Line => (
                    ParsedSourceCommentKind::Line,
                    comment.range.start + 2,
                    comment.range.end,
                ),
                LexedCommentKind::Block => (
                    ParsedSourceCommentKind::Block,
                    comment.range.start + 2,
                    comment.range.end - 2,
                ),
            };
            let text = src
                .get(text_start..text_end)
                .expect("lexer produced a comment range outside its source");
            ParsedSourceComment {
                kind,
                text,
                span: LexSpan::from(
                    (comment.range.start + base_offset)..(comment.range.end + base_offset),
                ),
            }
        })
        .collect();

    (tokens, comments, errors)
}
