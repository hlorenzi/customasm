use crate::*;


#[derive(Clone)]
pub struct Walker<'src>
{
	src: &'src str,
    file_handle: util::FileServerHandle,
    span_offset: usize,

    /// The current byte index into the `src` string.
    cursor_index: usize,
    /// The walker ignores characters from this byte index onward.
    cursor_limit: usize,
}


impl<'src> Walker<'src>
{
	pub fn new(
        src: &'src str,
        src_file_handle: util::FileServerHandle,
        src_byte_offset: usize)
        -> Walker<'src>
	{
		let walker = Walker {
            src,
            file_handle: src_file_handle,
            span_offset: src_byte_offset,

            cursor_index: 0,
            cursor_limit: src.len(),
		};
		
		walker
	}


    pub fn slice(
        &self,
        start_byte_index: usize,
        end_byte_index: usize)
        -> Walker<'src>
    {
        let src = &self.src[start_byte_index..end_byte_index];

		let walker = Walker {
            src,
            file_handle: self.file_handle,
            span_offset: self.span_offset + start_byte_index,

            cursor_index: 0,
            cursor_limit: src.len(),
		};
		
		walker
    }
	
	
	pub fn is_over(&self) -> bool
	{
		self.cursor_index >= self.cursor_limit
	}


    pub fn advance_to_token_end(
        &mut self,
        token: &syntax::Token)
    {
        self.cursor_index = self.get_index_at_span_end(token.span);
    }
	
	
	pub fn skip_ignorable(&mut self)
	{
        loop
        {
            if self.is_over()
                { break; }
            
            let token = self.token_at(self.cursor_index);

            if !token.kind.is_ignorable()
                { break; }

            self.advance_to_token_end(&token);
        }
	}


    pub fn get_cursor_index(&self) -> usize
    {
        self.cursor_index
    }


    pub fn get_index_at_span_start(
        &self,
        span: diagn::Span)
        -> usize
    {
        span.location().unwrap().0 - self.span_offset
    }


    pub fn get_index_at_span_end(
        &self,
        span: diagn::Span)
        -> usize
    {
        span.location().unwrap().1 - self.span_offset
    }


    pub fn next_useful_index(&self) -> usize
    {
        let token = self.next_useful_token();
        self.get_index_at_span_start(token.span)
    }


    pub fn get_span(
        &self,
        start_byte_index: usize,
        end_byte_index: usize)
        -> diagn::Span
    {
        let start =
            self.span_offset + start_byte_index;
        
        let end =
            self.span_offset + end_byte_index;
        
        diagn::Span::new(
            self.file_handle,
            start,
            end)
    }


    pub fn get_cursor_span(&self) -> diagn::Span
    {
        self.get_span(self.cursor_index, self.cursor_index)
    }


    pub fn get_full_span(&self) -> diagn::Span
    {
        self.get_span(0, self.cursor_limit)
    }


    pub fn get_excerpt(
        &self,
        start_byte_index: usize,
        end_byte_index: usize)
        -> &'src str
    {
        &self.src[start_byte_index..end_byte_index]
    }


    pub fn get_full_excerpt(&self) -> &'src str
    {
        &self.src[0..self.cursor_limit]
    }


    pub fn get_span_excerpt(
        &self,
        span: diagn::Span)
        -> &'src str
    {
        let start = self.get_index_at_span_start(span);
        let end = self.get_index_at_span_end(span);
        &self.src[start..end]
    }


    pub fn get_cursor_limit(
        &self)
        -> usize
    {
        self.cursor_limit
    }


    pub fn set_cursor_limit(
        &mut self,
        end: usize)
    {
        self.cursor_limit = end;
    }


    fn char_at(
        &self,
        byte_index: usize)
        -> char
    {
        if byte_index >= self.cursor_limit
        {
            '\0'
        }
        else
        {
            self.src[byte_index..self.cursor_limit]
                .chars()
                .next()
                .unwrap_or('\0')
        }
    }


    pub fn next_char(
        &self)
        -> char
    {
        self.char_at(self.cursor_index)
    }


    fn token_at(
        &self,
        byte_index: usize)
        -> syntax::Token
    {
        if byte_index >= self.cursor_limit
        {
            let span_index =
                self.span_offset + self.cursor_limit;
            
            let span = diagn::Span::new(
                self.file_handle,
                span_index,
                span_index);

            return syntax::Token {
                kind: syntax::TokenKind::LineBreak,
                span,
            };
        }

        let src_next = &self.src[byte_index..self.cursor_limit];
        let (kind, length) = syntax::decide_next_token(src_next);

        let end = byte_index + length;

        let span = diagn::Span::new(
            self.file_handle,
            self.span_offset + byte_index,
            self.span_offset + end);
        
        syntax::Token {
            kind,
            span,
        }
    }


    pub fn next_token(
        &self)
        -> syntax::Token
    {
        self.token_at(self.cursor_index)
    }


    pub fn next_nth_token(
        &self,
        mut nth: usize)
        -> syntax::Token
    {
        let mut byte_index = self.cursor_index;

        loop
        {
            let token = self.token_at(byte_index);

            if nth == 0
                { return token; }

            if byte_index >= self.cursor_limit
                { return token; }

            nth -= 1;
            byte_index += token.span.length();
        }
    }


    pub fn next_nth_useful_token(
        &self,
        mut nth: usize)
        -> syntax::Token
    {
        let mut byte_index = self.cursor_index;

        loop
        {
            let token = self.token_at(byte_index);

            if byte_index >= self.cursor_limit
                { return token; }

            if !token.kind.is_ignorable()
            {
                if nth == 0
                    { return token; }
                
                nth -= 1;
            }

            byte_index += token.span.length();
        }
    }


    fn next_useful_token(
        &self)
        -> syntax::Token
    {
        self.next_nth_useful_token(0)
    }
	
	
	pub fn next_linebreak(
        &self)
        -> Option<syntax::Token>
	{
        let mut byte_index = self.cursor_index;

        loop
        {
            let token = self.token_at(byte_index);

            if token.kind == syntax::TokenKind::LineBreak
                { return Some(token); }

            if !token.kind.is_ignorable()
                { return None; }

            byte_index += token.span.length();
        }
	}

	
	pub fn next_useful_is(
		&mut self,
        nth: usize,
		kind: syntax::TokenKind)
		-> bool
	{
        let token = self.next_nth_useful_token(nth);
        token.kind == kind
	}

	
	pub fn maybe_expect(
		&mut self,
		kind: syntax::TokenKind)
		-> Option<syntax::Token>
	{
        let token = self.next_useful_token();
        if token.kind == kind
		{
            let token = token.clone();
            self.advance_to_token_end(&token);
			Some(token)
		}
		else
        {
            None
        }
	}
	
	
	pub fn expect(
		&mut self,
		report: &mut diagn::Report,
		kind: syntax::TokenKind)
		-> Result<syntax::Token, ()>
	{
		match self.maybe_expect(kind)
		{
			Some(token) => Ok(token),
			None =>
			{
				report.error_span(
                    format!("expected {}", kind.printable()),
                    self.get_cursor_span());
                
				Err(())
			}
		}
	}

	
	pub fn maybe_expect_char(
		&mut self,
		wanted_char: char)
		-> bool
	{
        let index = self.next_useful_index();

        let c = self.char_at(index);

        if c.eq_ignore_ascii_case(&wanted_char)
		{
            self.cursor_index =
                index +
                c.len_utf8();
            
			true
		}
		else
        {
            false
        }
	}
	
	
	pub fn expect_linebreak(
        &mut self,
		report: &mut diagn::Report)
        -> Result<(), ()>
	{
		match self.maybe_expect_linebreak()
		{
			Some(()) => Ok(()),
			None =>
			{
				report.error_span(
                    format!(
                        "expected {}",
                        syntax::TokenKind::LineBreak.printable()),
                    self.get_cursor_span());
                
				Err(())
			}
		}
	}
	
	
	pub fn maybe_expect_linebreak(&mut self) -> Option<()>
	{
		if let Some(token) = self.next_linebreak()
		{
            self.advance_to_token_end(&token);
			Some(())
		}
		else
		{
			None
		}
	}


	pub fn advance_until_closing_brace(
		&mut self)
		-> Walker<'src>
	{
		let start = self.cursor_index;

		let mut brace_nesting = 0;

		while !self.is_over()
		{
            let c = self.next_char();

            if c == '{'
			{
				brace_nesting += 1;
			}
			else if c == '}'
			{
                if brace_nesting == 0
                    { break; }
                
				brace_nesting -= 1;
			}

            self.cursor_index += c.len_utf8();
		}

		let end = self.cursor_index;

        self.slice(start, end)
	}


	pub fn advance_until_linebreak(
		&mut self)
		-> Walker<'src>
	{
		let start = self.cursor_index;
		let mut end = self.cursor_index;

		let mut brace_nesting = 0;

		while !self.is_over()
		{
            let token = self.next_token();

            if token.kind == syntax::TokenKind::LineBreak &&
                brace_nesting == 0
            {
                break;
            }
            else if token.kind == syntax::TokenKind::BraceOpen
			{
				brace_nesting += 1;
			}
			else if token.kind == syntax::TokenKind::BraceClose
			{
                if brace_nesting == 0
                    { break; }
                
				brace_nesting -= 1;
			}

            self.advance_to_token_end(&token);

            if !token.kind.is_ignorable()
            {
                end = self.cursor_index;
            }
		}

        self.slice(start, end)
	}


	pub fn find_lookahead_char_index(
		&self,
        wanted_char: char)
		-> Option<usize>
	{
		let mut byte_index = self.cursor_index;

        let mut seen_tokens = false;
        let mut paren_nesting = 0;
		let mut brace_nesting = 0;

		while byte_index < self.cursor_limit
		{
            let c = self.char_at(byte_index);

            if c.eq_ignore_ascii_case(&wanted_char) &&
                seen_tokens &&
                paren_nesting == 0 &&
                brace_nesting == 0
            {
                return Some(byte_index);
            }
            else if c == '('
			{
				paren_nesting += 1;
			}
			else if c == ')'
			{
                if paren_nesting == 0
                    { break; }
                
                paren_nesting -= 1;
			}
            else if c == '{'
			{
				brace_nesting += 1;
			}
			else if c == '}'
			{
                if brace_nesting == 0
                    { break; }
                
				brace_nesting -= 1;
			}


            byte_index += c.len_utf8();

            if !syntax::token::is_whitespace(c)
            {
                seen_tokens = true;
            }
		}

        None
	}
}