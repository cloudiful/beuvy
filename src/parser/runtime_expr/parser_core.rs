use super::{expr_binding_path, parse_runtime_number};
use crate::ast::{
    DeclarativeBinaryOp, DeclarativeLiteral, DeclarativeNumber, DeclarativeRuntimeExpr,
};
use crate::parser::{DeclarativeUiAssetLoadError, attr_error};
use roxmltree::Node as XmlNode;

pub(crate) struct RuntimeExprParser<'a, 'input> {
    pub(crate) node: XmlNode<'a, 'input>,
    pub(crate) attr_name: &'a str,
    pub(crate) raw: &'a str,
    pub(crate) chars: Vec<char>,
    pub(crate) index: usize,
}

impl<'a, 'input> RuntimeExprParser<'a, 'input> {
    pub(crate) fn new(node: XmlNode<'a, 'input>, attr_name: &'a str, raw: &'a str) -> Self {
        Self {
            node,
            attr_name,
            raw: raw.trim(),
            chars: raw.trim().chars().collect(),
            index: 0,
        }
    }

    pub(crate) fn parse_expression(
        &mut self,
    ) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        self.parse_conditional()
    }

    fn parse_conditional(&mut self) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        let condition = self.parse_equality()?;
        self.skip_ws();
        if !self.consume_char('?') {
            return Ok(condition);
        }
        let then_expr = self.parse_expression()?;
        self.skip_ws();
        self.expect_char(':', "expected `:` in conditional expression")?;
        let else_expr = self.parse_expression()?;
        Ok(DeclarativeRuntimeExpr::Conditional {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        })
    }

    fn parse_equality(&mut self) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        self.parse_binary_chain(
            Self::parse_comparison,
            &[
                ("===", DeclarativeBinaryOp::Equal),
                ("!==", DeclarativeBinaryOp::NotEqual),
                ("==", DeclarativeBinaryOp::Equal),
                ("!=", DeclarativeBinaryOp::NotEqual),
            ],
        )
    }

    fn parse_comparison(&mut self) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        self.parse_binary_chain(
            Self::parse_additive,
            &[
                ("<=", DeclarativeBinaryOp::LessThanOrEqual),
                (">=", DeclarativeBinaryOp::GreaterThanOrEqual),
                ("<", DeclarativeBinaryOp::LessThan),
                (">", DeclarativeBinaryOp::GreaterThan),
            ],
        )
    }

    fn parse_additive(&mut self) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        self.parse_binary_chain(
            Self::parse_multiplicative,
            &[
                ("+", DeclarativeBinaryOp::Add),
                ("-", DeclarativeBinaryOp::Subtract),
            ],
        )
    }

    fn parse_multiplicative(
        &mut self,
    ) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        self.parse_binary_chain(
            Self::parse_unary,
            &[
                ("*", DeclarativeBinaryOp::Multiply),
                ("/", DeclarativeBinaryOp::Divide),
            ],
        )
    }

    fn parse_binary_chain(
        &mut self,
        subparser: fn(&mut Self) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError>,
        operators: &[(&str, DeclarativeBinaryOp)],
    ) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        let mut expr = subparser(self)?;
        loop {
            self.skip_ws();
            let Some((token, op)) = operators
                .iter()
                .find(|(token, _)| self.starts_with(token))
                .copied()
            else {
                break;
            };
            self.index += token.len();
            let right = subparser(self)?;
            expr = DeclarativeRuntimeExpr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        self.skip_ws();
        if self.consume_char('!') {
            return Ok(DeclarativeRuntimeExpr::UnaryNot {
                expr: Box::new(self.parse_unary()?),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        let mut expr = self.parse_primary()?;
        loop {
            self.skip_ws();
            if self.consume_char('.') {
                let field = self.parse_identifier()?;
                self.skip_ws();
                if field == "getBoundingClientRect" && self.consume_char('(') {
                    self.skip_ws();
                    self.expect_char(')', "getBoundingClientRect() does not take arguments")?;
                    let Some(target_path) = expr_binding_path(&expr) else {
                        return Err(attr_error(
                            self.node,
                            self.attr_name,
                            self.raw,
                            "getBoundingClientRect() receiver must be a binding path",
                        ));
                    };
                    expr = DeclarativeRuntimeExpr::GetBoundingClientRect {
                        target_path: target_path.to_string(),
                    };
                    continue;
                }
                expr = DeclarativeRuntimeExpr::FieldAccess {
                    base: Box::new(expr),
                    field,
                };
                continue;
            }
            if self.consume_char('(') {
                let args = self.parse_call_args()?;
                expr = self.parse_supported_call(expr, args)?;
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_supported_call(
        &self,
        callee: DeclarativeRuntimeExpr,
        args: Vec<DeclarativeRuntimeExpr>,
    ) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        match callee {
            DeclarativeRuntimeExpr::BindingPath(name) if name == "anchorPopup" => {
                if args.len() != 6 {
                    return Err(attr_error(
                        self.node,
                        self.attr_name,
                        self.raw,
                        "expected 6 arguments for anchorPopup",
                    ));
                }
                let mut args = args.into_iter();
                Ok(DeclarativeRuntimeExpr::AnchorPopup {
                    anchor_rect: Box::new(args.next().unwrap()),
                    shell_rect: Box::new(args.next().unwrap()),
                    popup_width: Box::new(args.next().unwrap()),
                    popup_min_height: Box::new(args.next().unwrap()),
                    gap: Box::new(args.next().unwrap()),
                    margin: Box::new(args.next().unwrap()),
                })
            }
            DeclarativeRuntimeExpr::FieldAccess { base, field } => match (*base, field.as_str()) {
                (DeclarativeRuntimeExpr::BindingPath(name), "min") if name == "Math" => {
                    Ok(DeclarativeRuntimeExpr::MathMin { args })
                }
                (DeclarativeRuntimeExpr::BindingPath(name), "max") if name == "Math" => {
                    Ok(DeclarativeRuntimeExpr::MathMax { args })
                }
                _ => Err(attr_error(
                    self.node,
                    self.attr_name,
                    self.raw,
                    "unsupported runtime method or function in expression",
                )),
            },
            _ => Err(attr_error(
                self.node,
                self.attr_name,
                self.raw,
                "unsupported runtime method or function in expression",
            )),
        }
    }

    fn parse_call_args(
        &mut self,
    ) -> Result<Vec<DeclarativeRuntimeExpr>, DeclarativeUiAssetLoadError> {
        let mut args = Vec::new();
        self.skip_ws();
        if self.consume_char(')') {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expression()?);
            self.skip_ws();
            if self.consume_char(')') {
                return Ok(args);
            }
            self.expect_char(',', "expected `,` or `)` in function call")?;
        }
    }

    fn parse_primary(&mut self) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        self.skip_ws();
        if self.consume_char('(') {
            let expr = self.parse_expression()?;
            self.skip_ws();
            self.expect_char(')', "expected `)`")?;
            return Ok(expr);
        }
        if self.peek_char() == Some('[') {
            return self.parse_array_literal();
        }
        if self.peek_char() == Some('{') {
            return self.parse_object_literal();
        }
        if let Some(string) = self.parse_string_literal()? {
            return Ok(DeclarativeRuntimeExpr::Literal(DeclarativeLiteral::String(
                string,
            )));
        }
        if let Some(number) = self.parse_number_literal()? {
            return Ok(DeclarativeRuntimeExpr::NumberLiteral(number));
        }
        if self.starts_with("true") && self.word_boundary_after("true") {
            self.index += 4;
            return Ok(DeclarativeRuntimeExpr::Literal(DeclarativeLiteral::Bool(
                true,
            )));
        }
        if self.starts_with("false") && self.word_boundary_after("false") {
            self.index += 5;
            return Ok(DeclarativeRuntimeExpr::Literal(DeclarativeLiteral::Bool(
                false,
            )));
        }
        let identifier = self.parse_identifier()?;
        Ok(DeclarativeRuntimeExpr::BindingPath(identifier))
    }

    fn parse_object_literal(
        &mut self,
    ) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        self.expect_char('{', "expected `{`")?;
        let mut fields = Vec::new();
        loop {
            self.skip_ws();
            if self.consume_char('}') {
                break;
            }
            let key = if let Some(string) = self.parse_string_literal()? {
                string
            } else {
                self.parse_identifier()?
            };
            self.skip_ws();
            let value = if self.consume_char(':') {
                self.parse_expression()?
            } else {
                DeclarativeRuntimeExpr::BindingPath(key.clone())
            };
            fields.push((key, value));
            self.skip_ws();
            if self.consume_char('}') {
                break;
            }
            self.expect_char(',', "expected `,` or `}` in object literal")?;
        }
        Ok(DeclarativeRuntimeExpr::ObjectLiteral(fields))
    }

    fn parse_array_literal(
        &mut self,
    ) -> Result<DeclarativeRuntimeExpr, DeclarativeUiAssetLoadError> {
        self.expect_char('[', "expected `[`")?;
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.consume_char(']') {
                break;
            }
            items.push(self.parse_expression()?);
            self.skip_ws();
            if self.consume_char(']') {
                break;
            }
            self.expect_char(',', "expected `,` or `]` in array literal")?;
        }
        Ok(DeclarativeRuntimeExpr::ArrayLiteral(items))
    }

    pub(crate) fn parse_string_literal(
        &mut self,
    ) -> Result<Option<String>, DeclarativeUiAssetLoadError> {
        self.skip_ws();
        let Some(quote @ ('\'' | '"')) = self.peek_char() else {
            return Ok(None);
        };
        self.index += 1;
        let mut value = String::new();
        while let Some(ch) = self.peek_char() {
            self.index += 1;
            if ch == quote {
                return Ok(Some(value));
            }
            if ch == '\\' {
                let Some(escaped) = self.peek_char() else {
                    break;
                };
                self.index += 1;
                value.push(escaped);
                continue;
            }
            value.push(ch);
        }
        Err(attr_error(
            self.node,
            self.attr_name,
            self.raw,
            "unterminated string literal",
        ))
    }

    fn parse_number_literal(
        &mut self,
    ) -> Result<Option<DeclarativeNumber>, DeclarativeUiAssetLoadError> {
        self.skip_ws();
        let start = self.index;
        let mut seen_digit = false;
        let mut seen_dot = false;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                seen_digit = true;
                self.index += 1;
                continue;
            }
            if ch == '.' && !seen_dot {
                seen_dot = true;
                self.index += 1;
                continue;
            }
            break;
        }
        if !seen_digit {
            self.index = start;
            return Ok(None);
        }
        let raw = self.slice(start, self.index);
        parse_runtime_number(raw)
            .map(Some)
            .map_err(|_| attr_error(self.node, self.attr_name, raw, "invalid number literal"))
    }
}
