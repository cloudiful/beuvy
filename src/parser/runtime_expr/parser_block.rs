use super::RuntimeExprParser;
use crate::ast::DeclarativeRuntimeStmt;
use crate::parser::{DeclarativeUiAssetLoadError, attr_error};

impl<'a, 'input> RuntimeExprParser<'a, 'input> {
    pub(crate) fn parse_block_statements(
        &mut self,
    ) -> Result<Vec<DeclarativeRuntimeStmt>, DeclarativeUiAssetLoadError> {
        self.skip_ws();
        self.expect_char('{', "expected `{` to start computed block")?;
        let mut statements = Vec::new();
        loop {
            self.skip_ws();
            if self.consume_char('}') {
                break;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<DeclarativeRuntimeStmt, DeclarativeUiAssetLoadError> {
        self.skip_ws();
        if self.starts_with("const") && self.word_boundary_after("const") {
            self.index += 5;
            self.skip_ws();
            let name = self.parse_identifier()?;
            self.skip_ws();
            self.expect_char('=', "expected `=` in const declaration")?;
            let expr = self.parse_expression()?;
            self.skip_ws();
            self.expect_char(';', "expected `;` after const declaration")?;
            return Ok(DeclarativeRuntimeStmt::Const { name, expr });
        }
        if self.starts_with("if") && self.word_boundary_after("if") {
            self.index += 2;
            self.skip_ws();
            self.expect_char('(', "expected `(` after `if`")?;
            let condition = self.parse_expression()?;
            self.skip_ws();
            self.expect_char(')', "expected `)` after `if` condition")?;
            let then_branch = self.parse_statement_block()?;
            self.skip_ws();
            let else_branch = if self.starts_with("else") && self.word_boundary_after("else") {
                self.index += 4;
                self.skip_ws();
                if self.starts_with("if") && self.word_boundary_after("if") {
                    vec![self.parse_statement()?]
                } else {
                    self.parse_statement_block()?
                }
            } else {
                Vec::new()
            };
            return Ok(DeclarativeRuntimeStmt::If {
                condition,
                then_branch,
                else_branch,
            });
        }
        if self.starts_with("return") && self.word_boundary_after("return") {
            self.index += 6;
            let expr = self.parse_expression()?;
            self.skip_ws();
            self.expect_char(';', "expected `;` after return")?;
            return Ok(DeclarativeRuntimeStmt::Return(expr));
        }
        Err(attr_error(
            self.node,
            self.attr_name,
            self.remaining(),
            "computed blocks only support `const`, `if`, and `return` statements",
        ))
    }

    fn parse_statement_block(
        &mut self,
    ) -> Result<Vec<DeclarativeRuntimeStmt>, DeclarativeUiAssetLoadError> {
        self.skip_ws();
        self.expect_char('{', "expected `{` to start statement block")?;
        let mut statements = Vec::new();
        loop {
            self.skip_ws();
            if self.consume_char('}') {
                break;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    pub(crate) fn parse_identifier(&mut self) -> Result<String, DeclarativeUiAssetLoadError> {
        self.skip_ws();
        let start = self.index;
        let Some(first) = self.peek_char() else {
            return Err(attr_error(
                self.node,
                self.attr_name,
                self.raw,
                "expected identifier",
            ));
        };
        if !(first.is_ascii_alphabetic() || first == '_') {
            return Err(attr_error(
                self.node,
                self.attr_name,
                self.remaining(),
                "expected identifier",
            ));
        }
        self.index += 1;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.index += 1;
            } else {
                break;
            }
        }
        Ok(self.slice(start, self.index).to_string())
    }

    pub(crate) fn expect_eof(&mut self) -> Result<(), DeclarativeUiAssetLoadError> {
        self.skip_ws();
        if self.index == self.chars.len() {
            return Ok(());
        }
        Err(attr_error(
            self.node,
            self.attr_name,
            self.remaining(),
            "unsupported runtime expression syntax",
        ))
    }

    pub(crate) fn expect_char(
        &mut self,
        expected: char,
        message: &str,
    ) -> Result<(), DeclarativeUiAssetLoadError> {
        self.skip_ws();
        if self.consume_char(expected) {
            return Ok(());
        }
        Err(attr_error(self.node, self.attr_name, self.raw, message))
    }

    pub(crate) fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn starts_with(&self, raw: &str) -> bool {
        self.remaining().starts_with(raw)
    }

    pub(crate) fn word_boundary_after(&self, word: &str) -> bool {
        let tail = &self.remaining()[word.len()..];
        tail.chars()
            .next()
            .is_none_or(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
    }

    pub(crate) fn skip_ws(&mut self) {
        while self.peek_char().is_some_and(char::is_whitespace) {
            self.index += 1;
        }
    }

    pub(crate) fn peek_char(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    pub(crate) fn remaining(&self) -> &str {
        self.slice(self.index, self.chars.len())
    }

    pub(crate) fn slice(&self, start: usize, end: usize) -> &str {
        let start_byte = self.chars[..start].iter().map(|ch| ch.len_utf8()).sum();
        let end_byte = self.chars[..end].iter().map(|ch| ch.len_utf8()).sum();
        &self.raw[start_byte..end_byte]
    }
}
