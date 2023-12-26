use crate::*;

#[derive(Clone)]
pub struct TokenWalker<'tokens>
{
    tokens: &'tokens [syntax::Token],
    index: usize,
    index_prev: usize,
    read_linebreak: bool,
    read_whitespace_index: Option<usize>,
    read_whitespace_acknowledged: bool,
    partial_index: usize,
    dummy_token: syntax::Token,
}

impl<'tokens> TokenWalker<'tokens>
{
    pub fn new(tokens: &'tokens [syntax::Token]) -> TokenWalker<'tokens>
    {
        let dummy_span = {
            if let Some(tk_last) = tokens.last()
            {
                tk_last.span
            }
            else
            {
                diagn::Span::new_dummy()
            }
        };

        let dummy_token = syntax::Token {
            kind: syntax::TokenKind::LineBreak,
            span: dummy_span,
            excerpt: None,
        };

        let mut parser = TokenWalker {
            tokens: tokens,
            index: 0,
            index_prev: 0,
            read_linebreak: false,
            read_whitespace_index: None,
            read_whitespace_acknowledged: true,
            partial_index: 0,
            dummy_token,
        };

        parser.skip_ignorable();
        parser
    }

    pub fn get_current_token_index(&self) -> usize
    {
        self.index
    }

    pub fn get_previous_token_index(&self) -> usize
    {
        self.index_prev
    }

    pub fn get_full_span(&self) -> diagn::Span
    {
        if self.tokens.len() == 0
        {
            diagn::Span::new_dummy()
        }
        else
        {
            self.tokens[0].span.join(self.tokens.last().unwrap().span)
        }
    }

    pub fn get_cloned_tokens(&self) -> Vec<syntax::Token>
    {
        let mut result = Vec::new();

        for token in self.tokens
        {
            result.push(token.clone());
        }

        result
    }

    pub fn get_cloned_tokens_by_index(&self, start: usize, end: usize) -> Vec<syntax::Token>
    {
        let mut result = Vec::new();

        for token in &self.tokens[start..end]
        {
            result.push(token.clone());
        }

        if let Some(last_token) = result.last()
        {
            if last_token.kind.is_ignorable()
            {
                result.pop();
            }
        }

        result
    }

    pub fn debug_remaining(&self) -> String
    {
        let mut result = String::new();

        for i in self.index..self.tokens.len()
        {
            result.push_str(&self.tokens[i].text());
        }

        result
    }

    pub fn get_next_spans(&self, count: usize) -> diagn::Span
    {
        if self.index >= self.tokens.len()
        {
            return diagn::Span::new_dummy();
        }

        let mut span = self.tokens[self.index].span;

        let mut i = 1;
        while i <= count && self.index + i < self.tokens.len()
        {
            span = span.join(self.tokens[self.index + i].span);
            i += 1;
        }

        span
    }

    pub fn get_span_after_prev(&self) -> diagn::Span
    {
        if self.index_prev >= self.tokens.len()
        {
            return diagn::Span::new_dummy();
        }

        self.tokens[self.index_prev].span.after()
    }

    pub fn clone_slice<'b>(&'b self, start: usize, end: usize) -> TokenWalker<'tokens>
    {
        TokenWalker::new(&self.tokens[start..end])
    }

    pub fn skip_until_linebreak_over_nested_braces<'b>(&'b mut self) -> TokenWalker<'tokens>
    {
        let start = self.get_current_token_index();
        let mut brace_nesting = 0;

        while !self.is_over() && (!self.next_is_linebreak() || brace_nesting > 0)
        {
            if self.next_is(0, syntax::TokenKind::BraceOpen)
            {
                brace_nesting += 1;
                self.advance();
                continue;
            }

            if self.next_is(0, syntax::TokenKind::BraceClose) && brace_nesting > 0
            {
                brace_nesting -= 1;
                self.advance();
                continue;
            }

            if brace_nesting > 0
            {
                self.advance();
                continue;
            }

            self.advance();
        }

        if self.get_current_token_index() == start
        {
            self.clone_slice(start, start)
        }
        else
        {
            self.clone_slice(start, self.get_previous_token_index() + 1)
        }
    }

    pub fn skip_until_token_over_nested_braces<'b>(
        &'b mut self,
        kind: syntax::TokenKind,
    ) -> TokenWalker<'tokens>
    {
        let start = self.get_current_token_index();
        let mut brace_nesting = 0;

        while !self.is_over() && (!self.next_is(0, kind) || brace_nesting > 0)
        {
            if self.next_is(0, syntax::TokenKind::BraceOpen)
            {
                brace_nesting += 1;
                self.advance();
                continue;
            }

            if self.next_is(0, syntax::TokenKind::BraceClose) && brace_nesting > 0
            {
                brace_nesting -= 1;
                self.advance();
                continue;
            }

            if brace_nesting > 0
            {
                self.advance();
                continue;
            }

            self.advance();
        }

        if self.get_current_token_index() == start
        {
            self.clone_slice(start, start)
        }
        else
        {
            self.clone_slice(start, self.get_previous_token_index() + 1)
        }
    }

    pub fn try_lookahead_until_char_over_nested_parens<'b>(
        &'b self,
        c: char,
    ) -> Option<TokenWalker<'tokens>>
    {
        let mut lookahead = self.clone();
        let start = lookahead.get_current_token_index();

        let mut paren_nesting = 0;

        loop
        {
            if lookahead.is_over()
            {
                break;
            }

            if lookahead.next_partial() == c
                && paren_nesting == 0
                && lookahead.get_current_token_index() > start
            {
                break;
            }

            if lookahead.next_is(0, syntax::TokenKind::ParenOpen)
            {
                paren_nesting += 1;
                lookahead.advance();
                continue;
            }

            if lookahead.next_is(0, syntax::TokenKind::ParenClose) && paren_nesting > 0
            {
                paren_nesting -= 1;
                lookahead.advance();
                continue;
            }

            if paren_nesting > 0
            {
                lookahead.advance();
                continue;
            }

            lookahead.advance_partial();
        }

        let end = lookahead.get_previous_token_index() + 1;

        if lookahead.is_at_partial() || start >= end
        {
            None
        }
        else
        {
            let mut new_walker = self.clone();
            new_walker.tokens = &self.tokens[0..end];
            Some(new_walker)
        }
    }

    pub fn copy_state_from(&mut self, other: &TokenWalker)
    {
        self.index = other.index;
        self.index_prev = other.index_prev;
        self.read_linebreak = other.read_linebreak;
        self.read_whitespace_index = other.read_whitespace_index;
        self.read_whitespace_acknowledged = other.read_whitespace_acknowledged;
        self.partial_index = other.partial_index;
        self.skip_ignorable();
    }

    pub fn is_over(&self) -> bool
    {
        self.index >= self.tokens.len()
    }

    pub fn skip_ignorable(&mut self)
    {
        while self.index < self.tokens.len() && self.tokens[self.index].kind.is_ignorable()
        {
            if self.tokens[self.index].kind == syntax::TokenKind::LineBreak
            {
                self.read_linebreak = true;
            }

            if self.tokens[self.index].kind == syntax::TokenKind::Whitespace
            {
                self.read_whitespace_index = Some(self.index);
                self.read_whitespace_acknowledged = false;
            }

            self.index += 1;
        }
    }

    pub fn advance(&mut self) -> &'tokens syntax::Token
    {
        if self.is_at_partial()
        {
            panic!("trying to advance TokenWalker at partial token");
        }

        self.index_prev = self.index;

        let token = &self.tokens[self.index];

        if self.index < self.tokens.len()
        {
            self.index += 1;
        }

        self.read_linebreak = false;

        self.skip_ignorable();
        token
    }

    pub fn advance_partial(&mut self) -> char
    {
        if self.index >= self.tokens.len()
        {
            return '\0';
        }

        let sliced = self.tokens[self.index]
            .text()
            .get(self.partial_index..)
            .unwrap();

        let mut char_indices = sliced.char_indices();
        let c = char_indices.next().unwrap().1;

        if let Some((index, _)) = char_indices.next()
        {
            self.partial_index += index;
        }
        else
        {
            self.partial_index = 0;
            self.advance();
        }

        c
    }

    pub fn skip_until_linebreak(&mut self)
    {
        while !self.is_over() && !self.next_is_linebreak()
        {
            self.advance();
        }
    }

    pub fn next(&self) -> &'tokens syntax::Token
    {
        &self.tokens[self.index]
    }

    pub fn next_nth(&self, nth: usize) -> &syntax::Token
    {
        if self.index + nth >= self.tokens.len()
        {
            return &self.dummy_token;
        }

        &self.tokens[self.index + nth]
    }

    pub fn next_partial(&mut self) -> char
    {
        if self.index >= self.tokens.len()
        {
            return '\0';
        }

        if self.tokens[self.index].kind == syntax::TokenKind::Whitespace
        {
            return ' ';
        }

        let sliced = self.tokens[self.index]
            .text()
            .get(self.partial_index..)
            .unwrap();

        let mut char_indices = sliced.char_indices();
        char_indices.next().unwrap().1
    }

    pub fn prev(&self) -> &'tokens syntax::Token
    {
        &self.tokens[self.index_prev]
    }

    pub fn is_at_partial(&self) -> bool
    {
        self.partial_index != 0
    }

    pub fn next_is(&self, mut nth: usize, kind: syntax::TokenKind) -> bool
    {
        let mut index = self.index;

        while nth > 0 && index < self.tokens.len()
        {
            nth -= 1;
            index += 1;
            while index < self.tokens.len() && self.tokens[index].kind.is_ignorable()
            {
                index += 1;
            }
        }

        if index >= self.tokens.len()
        {
            return false;
        }

        self.tokens[index].kind == kind
    }

    pub fn maybe_expect(&mut self, kind: syntax::TokenKind) -> Option<&'tokens syntax::Token>
    {
        if self.next_is(0, kind)
        {
            self.acknowledge_whitespace();
            Some(self.advance())
        }
        else
        {
            None
        }
    }

    pub fn expect(
        &mut self,
        report: &mut diagn::Report,
        kind: syntax::TokenKind,
    ) -> Result<&'tokens syntax::Token, ()>
    {
        match self.maybe_expect(kind)
        {
            Some(token) => Ok(&token),
            None =>
            {
                let descr = format!("expected {}", kind.printable());
                let span = self.tokens[self.index_prev].span.after();
                report.error_span(descr, span);
                Err(())
            }
        }
    }

    pub fn expect_msg<S: Into<String>>(
        &mut self,
        report: &mut diagn::Report,
        kind: syntax::TokenKind,
        descr: S,
    ) -> Result<&'tokens syntax::Token, ()>
    {
        match self.maybe_expect(kind)
        {
            Some(token) => Ok(&token),
            None =>
            {
                report.error_span(descr, self.tokens[self.index_prev].span.after());

                Err(())
            }
        }
    }

    pub fn acknowledge_whitespace(&mut self)
    {
        self.read_whitespace_acknowledged = true;
    }

    pub fn is_whitespace_acknowledged(&self) -> bool
    {
        self.read_whitespace_acknowledged
    }

    pub fn next_is_whitespace(&self) -> bool
    {
        if self.is_over()
        {
            true
        }
        else if let Some(index) = self.read_whitespace_index
        {
            if index + 1 == self.index
            {
                true
            }
            else
            {
                false
            }
        }
        else
        {
            false
        }
    }

    pub fn maybe_expect_whitespace(&mut self) -> Option<&syntax::Token>
    {
        if let Some(index) = self.read_whitespace_index
        {
            self.acknowledge_whitespace();
            Some(&self.tokens[index])
        }
        else if self.is_over()
        {
            Some(&self.dummy_token)
        }
        else
        {
            None
        }
    }

    pub fn maybe_expect_unacknowledged_whitespace(&mut self) -> Option<&'tokens syntax::Token>
    {
        if self.read_whitespace_acknowledged
        {
            return None;
        }

        if let Some(index) = self.read_whitespace_index
        {
            self.acknowledge_whitespace();
            Some(&self.tokens[index])
        }
        else
        {
            None
        }
    }

    pub fn clear_linebreak(&mut self)
    {
        self.read_linebreak = false;
    }

    pub fn next_is_linebreak(&self) -> bool
    {
        self.read_linebreak || self.is_over()
    }

    pub fn maybe_expect_linebreak(&mut self) -> Option<()>
    {
        if self.next_is_linebreak()
        {
            self.clear_linebreak();
            Some(())
        }
        else
        {
            None
        }
    }

    pub fn expect_linebreak(&mut self, report: &mut diagn::Report) -> Result<(), ()>
    {
        if self.maybe_expect_linebreak().is_some()
        {
            Ok(())
        }
        else
        {
            report.error_span(
                "expected line break",
                self.tokens[self.index_prev].span.after(),
            );

            Err(())
        }
    }

    pub fn expect_linebreak_or(
        &mut self,
        report: &mut diagn::Report,
        kind: syntax::TokenKind,
    ) -> Result<(), ()>
    {
        if self.maybe_expect(kind).is_some()
        {
            Ok(())
        }
        else if self.maybe_expect_linebreak().is_some()
        {
            Ok(())
        }
        else
        {
            report.error_span(
                "expected line break",
                self.tokens[self.index_prev].span.after(),
            );

            Err(())
        }
    }

    pub fn expect_usize(
        &mut self,
        report: &mut diagn::Report,
    ) -> Result<(&'tokens syntax::Token, usize), ()>
    {
        let tk = self.expect(report, syntax::TokenKind::Number)?;

        let value = syntax::excerpt_as_usize(report, tk.span, &tk.excerpt.as_ref().unwrap())?;

        Ok((tk, value))
    }

    pub fn maybe_expect_partial_usize(&mut self) -> Option<usize>
    {
        let mut value: usize = 0;
        let mut advance_count: usize = 0;

        while !self.is_over()
        {
            let c = self.next_partial();

            let digit = match c.to_digit(10)
            {
                Some(d) => d,
                None => break,
            };

            value = match value.checked_mul(10)
            {
                Some(v) => v,
                None => break,
            };

            value = match value.checked_add(digit as usize)
            {
                Some(v) => v,
                None => break,
            };

            self.advance_partial();
            advance_count += 1;
        }

        if advance_count == 0
        {
            return None;
        }

        Some(value)
    }
}
