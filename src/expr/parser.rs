use crate::*;

pub fn parse(report: &mut diagn::Report, walker: &mut syntax::TokenWalker)
    -> Result<expr::Expr, ()>
{
    ExpressionParser::new(report, walker).parse_expr()
}

pub fn parse_optional(walker: &mut syntax::TokenWalker) -> Option<expr::Expr>
{
    let mut dummy_report = diagn::Report::new();
    parse(&mut dummy_report, walker).ok()
}

struct ExpressionParser<'a, 'tokens: 'a>
{
    report: &'a mut diagn::Report,
    walker: &'a mut syntax::TokenWalker<'tokens>,
    recursion_depth: usize,
}

impl<'a, 'tokens> ExpressionParser<'a, 'tokens>
{
    pub fn new(
        report: &'a mut diagn::Report,
        walker: &'a mut syntax::TokenWalker<'tokens>,
    ) -> ExpressionParser<'a, 'tokens>
    {
        ExpressionParser {
            report,
            walker,
            recursion_depth: 0,
        }
    }

    pub fn check_recursion_limit(&mut self) -> Result<(), ()>
    {
        if self.recursion_depth > expr::PARSE_RECURSION_DEPTH_MAX
        {
            self.report
                .message_with_parents_dedup(diagn::Message::error_span(
                    "expression recursion depth limit reached",
                    self.walker.get_span_after_prev(),
                ));

            return Err(());
        }

        Ok(())
    }

    pub fn parse_expr(&mut self) -> Result<expr::Expr, ()>
    {
        self.recursion_depth += 1;

        self.check_recursion_limit()?;

        let maybe_expr = self.parse_ternary_conditional();

        self.recursion_depth -= 1;

        maybe_expr
    }

    fn parse_unary_ops<F>(
        &mut self,
        ops: &[(syntax::TokenKind, expr::UnaryOp)],
        parse_inner: F,
    ) -> Result<expr::Expr, ()>
    where
        F: Fn(&mut ExpressionParser<'a, 'tokens>) -> Result<expr::Expr, ()>,
    {
        for op in ops
        {
            let tk_span = {
                match self.walker.maybe_expect(op.0)
                {
                    Some(tk) => tk.span,
                    None => continue,
                }
            };

            self.recursion_depth += 1;
            self.check_recursion_limit()?;

            let inner = self.parse_unary_ops(ops, parse_inner)?;
            let span = tk_span.join(inner.span());

            self.recursion_depth -= 1;

            return Ok(expr::Expr::UnaryOp(span, tk_span, op.1, Box::new(inner)));
        }

        parse_inner(self)
    }

    fn parse_binary_ops<F>(
        &mut self,
        ops: &[(syntax::TokenKind, expr::BinaryOp)],
        parse_inner: F,
    ) -> Result<expr::Expr, ()>
    where
        F: Fn(&mut ExpressionParser<'a, 'tokens>) -> Result<expr::Expr, ()>,
    {
        let mut lhs = parse_inner(self)?;

        loop
        {
            if self.walker.next_is_linebreak()
            {
                break;
            }

            let mut op_match = None;

            for op in ops
            {
                if let Some(tk) = self.walker.maybe_expect(op.0)
                {
                    op_match = Some((tk.span, op.1));
                    break;
                }
            }

            if let Some(op_match) = op_match
            {
                let rhs = parse_inner(self)?;
                let span = lhs.span().join(rhs.span());

                lhs = expr::Expr::BinaryOp(
                    span,
                    op_match.0,
                    op_match.1,
                    Box::new(lhs),
                    Box::new(rhs),
                );
            }
            else
            {
                break;
            }
        }

        Ok(lhs)
    }

    fn parse_right_associative_binary_ops<F>(
        &mut self,
        ops: &[(syntax::TokenKind, expr::BinaryOp)],
        parse_inner: F,
    ) -> Result<expr::Expr, ()>
    where
        F: Fn(&mut ExpressionParser<'a, 'tokens>) -> Result<expr::Expr, ()>,
    {
        let mut lhs = parse_inner(self)?;

        let mut op_match = None;

        for op in ops
        {
            if let Some(tk) = self.walker.maybe_expect(op.0)
            {
                op_match = Some((tk.span, op.1));
                break;
            }
        }

        if let Some(op_match) = op_match
        {
            let rhs = self.parse_expr()?;
            let span = lhs.span().join(rhs.span());

            lhs = expr::Expr::BinaryOp(span, op_match.0, op_match.1, Box::new(lhs), Box::new(rhs));
        }

        Ok(lhs)
    }

    fn parse_ternary_conditional(&mut self) -> Result<expr::Expr, ()>
    {
        let cond = self.parse_assignment()?;

        if self
            .walker
            .maybe_expect(syntax::TokenKind::Question)
            .is_some()
        {
            let true_branch = self.parse_expr()?;

            let false_branch = {
                if self.walker.maybe_expect(syntax::TokenKind::Colon).is_some()
                {
                    self.parse_expr()?
                }
                else
                {
                    expr::Expr::Block(true_branch.span(), Vec::new())
                }
            };

            Ok(expr::Expr::TernaryOp(
                cond.span().join(false_branch.span()),
                Box::new(cond),
                Box::new(true_branch),
                Box::new(false_branch),
            ))
        }
        else
        {
            Ok(cond)
        }
    }

    fn parse_assignment(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_right_associative_binary_ops(
            &[(syntax::TokenKind::Equal, expr::BinaryOp::Assign)],
            |s| s.parse_concat(),
        )
    }

    fn parse_concat(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(&[(syntax::TokenKind::At, expr::BinaryOp::Concat)], |s| {
            s.parse_lazy_or()
        })
    }

    fn parse_lazy_or(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(
            &[(
                syntax::TokenKind::VerticalBarVerticalBar,
                expr::BinaryOp::LazyOr,
            )],
            |s| s.parse_lazy_and(),
        )
    }

    fn parse_lazy_and(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(
            &[(
                syntax::TokenKind::AmpersandAmpersand,
                expr::BinaryOp::LazyAnd,
            )],
            |s| s.parse_relational(),
        )
    }

    fn parse_relational(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(
            &[
                (syntax::TokenKind::EqualEqual, expr::BinaryOp::Eq),
                (syntax::TokenKind::ExclamationEqual, expr::BinaryOp::Ne),
                (syntax::TokenKind::LessThan, expr::BinaryOp::Lt),
                (syntax::TokenKind::LessThanEqual, expr::BinaryOp::Le),
                (syntax::TokenKind::GreaterThan, expr::BinaryOp::Gt),
                (syntax::TokenKind::GreaterThanEqual, expr::BinaryOp::Ge),
            ],
            |s| s.parse_binary_or(),
        )
    }

    fn parse_binary_or(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(
            &[(syntax::TokenKind::VerticalBar, expr::BinaryOp::Or)],
            |s| s.parse_binary_xor(),
        )
    }

    fn parse_binary_xor(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(
            &[(syntax::TokenKind::Circumflex, expr::BinaryOp::Xor)],
            |s| s.parse_binary_and(),
        )
    }

    fn parse_binary_and(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(
            &[(syntax::TokenKind::Ampersand, expr::BinaryOp::And)],
            |s| s.parse_shifts(),
        )
    }

    fn parse_shifts(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(
            &[
                (syntax::TokenKind::LessThanLessThan, expr::BinaryOp::Shl),
                (
                    syntax::TokenKind::GreaterThanGreaterThan,
                    expr::BinaryOp::Shr,
                ),
            ],
            |s| s.parse_addition(),
        )
    }

    fn parse_addition(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(
            &[
                (syntax::TokenKind::Plus, expr::BinaryOp::Add),
                (syntax::TokenKind::Minus, expr::BinaryOp::Sub),
            ],
            |s| s.parse_multiplication(),
        )
    }

    fn parse_multiplication(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_binary_ops(
            &[
                (syntax::TokenKind::Asterisk, expr::BinaryOp::Mul),
                (syntax::TokenKind::Slash, expr::BinaryOp::Div),
                (syntax::TokenKind::Percent, expr::BinaryOp::Mod),
            ],
            |s| s.parse_bitslice(),
        )
    }

    fn parse_bitslice(&mut self) -> Result<expr::Expr, ()>
    {
        let inner = self.parse_size()?;

        if self.walker.next_is_linebreak()
        {
            return Ok(inner);
        }

        let tk_open = match self.walker.maybe_expect(syntax::TokenKind::BracketOpen)
        {
            Some(tk) => tk,
            None => return Ok(inner),
        };

        let leftmost = {
            let tk_leftmost = self.walker.expect(self.report, syntax::TokenKind::Number)?;
            syntax::excerpt_as_usize(
                self.report,
                tk_leftmost.span,
                tk_leftmost.excerpt.as_ref().unwrap(),
            )?
        };

        self.walker.expect(self.report, syntax::TokenKind::Colon)?;

        let rightmost = {
            let tk_rightmost = self.walker.expect(self.report, syntax::TokenKind::Number)?;
            syntax::excerpt_as_usize(
                self.report,
                tk_rightmost.span,
                tk_rightmost.excerpt.as_ref().unwrap(),
            )?
        };

        let tk_close_span = self
            .walker
            .expect(self.report, syntax::TokenKind::BracketClose)?
            .span;

        let slice_span = tk_open.span.join(tk_close_span);
        let span = inner.span().join(tk_close_span);

        if leftmost < rightmost
        {
            self.report
                .error_span("invalid bit slice range", slice_span);

            return Err(());
        }

        Ok(expr::Expr::BitSlice(
            span,
            slice_span,
            leftmost + 1,
            rightmost,
            Box::new(inner),
        ))
    }

    fn parse_size(&mut self) -> Result<expr::Expr, ()>
    {
        let inner = self.parse_unary()?;

        if self.walker.next_is_linebreak()
        {
            return Ok(inner);
        }

        let tk_grave_span = match self.walker.maybe_expect(syntax::TokenKind::Grave)
        {
            Some(tk) => tk.span,
            None => return Ok(inner),
        };

        let tk_size = self.walker.expect(self.report, syntax::TokenKind::Number)?;
        let size =
            syntax::excerpt_as_usize(self.report, tk_size.span, tk_size.excerpt.as_ref().unwrap())?;

        let span = inner.span().join(tk_size.span);
        let size_span = tk_grave_span.join(tk_size.span);

        Ok(expr::Expr::BitSlice(
            span,
            size_span,
            size,
            0,
            Box::new(inner),
        ))
    }

    fn parse_unary(&mut self) -> Result<expr::Expr, ()>
    {
        self.parse_unary_ops(
            &[
                (syntax::TokenKind::Exclamation, expr::UnaryOp::Not),
                (syntax::TokenKind::Minus, expr::UnaryOp::Neg),
            ],
            |s| s.parse_call(),
        )
    }

    fn parse_call(&mut self) -> Result<expr::Expr, ()>
    {
        let leaf = self.parse_leaf()?;

        if self.walker.next_is_linebreak()
        {
            return Ok(leaf);
        }

        if self
            .walker
            .maybe_expect(syntax::TokenKind::ParenOpen)
            .is_none()
        {
            return Ok(leaf);
        }

        let mut args = Vec::new();
        while !self.walker.next_is(0, syntax::TokenKind::ParenClose)
        {
            args.push(self.parse_expr()?);

            if self.walker.next_is(0, syntax::TokenKind::ParenClose)
            {
                break;
            }

            self.walker.expect(self.report, syntax::TokenKind::Comma)?;
        }

        let tk_close = self
            .walker
            .expect(self.report, syntax::TokenKind::ParenClose)?;

        Ok(expr::Expr::Call(
            leaf.span().join(tk_close.span),
            Box::new(leaf),
            args,
        ))
    }

    fn parse_leaf(&mut self) -> Result<expr::Expr, ()>
    {
        if self.walker.next_is(0, syntax::TokenKind::BraceOpen)
        {
            self.parse_block()
        }
        else if self.walker.next_is(0, syntax::TokenKind::ParenOpen)
        {
            self.parse_parenthesized()
        }
        else if self.walker.next_is(0, syntax::TokenKind::Identifier)
        {
            self.parse_variable()
        }
        else if self.walker.next_is(0, syntax::TokenKind::Dot)
        {
            self.parse_variable()
        }
        else if self.walker.next_is(0, syntax::TokenKind::Number)
        {
            self.parse_number()
        }
        else if self.walker.next_is(0, syntax::TokenKind::String)
        {
            self.parse_string()
        }
        else if self.walker.next_is(0, syntax::TokenKind::KeywordAsm)
        {
            self.parse_asm()
        }
        else if self.walker.next_is(0, syntax::TokenKind::KeywordTrue)
        {
            self.parse_boolean_true()
        }
        else if self.walker.next_is(0, syntax::TokenKind::KeywordFalse)
        {
            self.parse_boolean_false()
        }
        else
        {
            self.report
                .error_span("expected expression", self.walker.get_span_after_prev());

            Err(())
        }
    }

    fn parse_block(&mut self) -> Result<expr::Expr, ()>
    {
        let tk_open_span = self
            .walker
            .expect(self.report, syntax::TokenKind::BraceOpen)?
            .span;

        let mut exprs = Vec::new();
        while !self.walker.next_is(0, syntax::TokenKind::BraceClose)
        {
            exprs.push(self.parse_expr()?);

            if self.walker.maybe_expect_linebreak().is_some()
            {
                continue;
            }

            if self.walker.next_is(0, syntax::TokenKind::BraceClose)
            {
                break;
            }

            self.walker.expect(self.report, syntax::TokenKind::Comma)?;
        }

        let tk_close = self
            .walker
            .expect(self.report, syntax::TokenKind::BraceClose)?;

        Ok(expr::Expr::Block(tk_open_span.join(tk_close.span), exprs))
    }

    fn parse_parenthesized(&mut self) -> Result<expr::Expr, ()>
    {
        self.walker
            .expect(self.report, syntax::TokenKind::ParenOpen)?;
        let expr = self.parse_expr()?;
        self.walker
            .expect(self.report, syntax::TokenKind::ParenClose)?;
        Ok(expr)
    }

    fn parse_variable(&mut self) -> Result<expr::Expr, ()>
    {
        let mut span = diagn::Span::new_dummy();
        let mut hierarchy_level = 0;

        self.walker.clear_linebreak();

        loop
        {
            if self.walker.next_is_linebreak()
            {
                break;
            }

            if let Some(tk_dot) = self.walker.maybe_expect(syntax::TokenKind::Dot)
            {
                hierarchy_level += 1;
                span = span.join(tk_dot.span);
                continue;
            }

            break;
        }

        let mut hierarchy = Vec::new();

        loop
        {
            if self.walker.next_is_linebreak()
            {
                break;
            }

            let tk_name = self
                .walker
                .expect(self.report, syntax::TokenKind::Identifier)?;
            let name = tk_name.excerpt.clone().unwrap();
            hierarchy.push(name);
            span = span.join(tk_name.span);

            if self.walker.next_is_linebreak()
            {
                break;
            }

            if self.walker.maybe_expect(syntax::TokenKind::Dot).is_none()
            {
                break;
            }
        }

        Ok(expr::Expr::Variable(span, hierarchy_level, hierarchy))
    }

    fn parse_number(&mut self) -> Result<expr::Expr, ()>
    {
        let tk_number = self.walker.expect(self.report, syntax::TokenKind::Number)?;
        let number = tk_number.excerpt.clone().unwrap();

        let bigint = syntax::excerpt_as_bigint(Some(self.report), tk_number.span, &number)?;

        let expr = expr::Expr::Literal(tk_number.span, expr::Value::Integer(bigint));

        Ok(expr)
    }

    fn parse_boolean_true(&mut self) -> Result<expr::Expr, ()>
    {
        let tk_true = self
            .walker
            .expect(self.report, syntax::TokenKind::KeywordTrue)?;

        let expr = expr::Expr::Literal(tk_true.span, expr::Value::Bool(true));

        Ok(expr)
    }

    fn parse_boolean_false(&mut self) -> Result<expr::Expr, ()>
    {
        let tk_true = self
            .walker
            .expect(self.report, syntax::TokenKind::KeywordFalse)?;

        let expr = expr::Expr::Literal(tk_true.span, expr::Value::Bool(false));

        Ok(expr)
    }

    fn parse_string(&mut self) -> Result<expr::Expr, ()>
    {
        let tk_str = self.walker.expect(self.report, syntax::TokenKind::String)?;

        let string = syntax::excerpt_as_string_contents(
            self.report,
            tk_str.span,
            tk_str.excerpt.as_ref().unwrap(),
        )?;

        let expr = expr::Expr::Literal(
            tk_str.span,
            expr::Value::String(expr::ExprString {
                utf8_contents: string,
                encoding: "utf8".to_string(),
            }),
        );

        Ok(expr)
    }

    fn parse_asm(&mut self) -> Result<expr::Expr, ()>
    {
        let tk_asm_span = self
            .walker
            .expect(self.report, syntax::TokenKind::KeywordAsm)?
            .span;

        self.walker
            .expect(self.report, syntax::TokenKind::BraceOpen)?;

        let skipped = self
            .walker
            .skip_until_token_over_nested_braces(syntax::TokenKind::BraceClose);

        let tk_brace_close = self
            .walker
            .expect(self.report, syntax::TokenKind::BraceClose)?;

        let expr = expr::Expr::Asm(
            tk_asm_span.join(tk_brace_close.span),
            skipped.get_cloned_tokens(),
        );

        Ok(expr)
    }
}
