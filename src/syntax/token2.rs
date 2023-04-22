use crate::*;
use syntax::{Token, TokenKind};


pub fn tokenize2<S>(
    report: &mut diagn::Report,
    src_filename: S,
    src: &[char])
    -> Result<Vec<Token>, ()>
    where S: Into<String>
{
	let filename = std::rc::Rc::new(src_filename.into());
	let mut tokens = Vec::new();
	let mut index = 0;
	
	while index < src.len()
	{
		// Decide what the next token's kind and length are.
		let (kind, length) =
			check_for_whitespace(&src[index..]).unwrap_or_else(||
			check_for_comment   (&src[index..]).unwrap_or_else(||
			check_for_fixed     (&src[index..]).unwrap_or_else(||
			check_for_identifier(&src[index..]).unwrap_or_else(||
			check_for_number    (&src[index..]).unwrap_or_else(||
			check_for_string    (&src[index..]).unwrap_or_else(||
			(TokenKind::Error, 1)))))));
		
		let span = diagn::Span::new(
            filename.clone(),
            index,
            index + length);
		
		// Get the source excerpt for variable tokens (e.g. identifiers).
		let excerpt = {
            match kind.needs_excerpt()
            {
                false => None,
                true => Some(src[index..]
                    .iter()
                    .take(length)
                    .collect()),
            }
        };
		
		// Report unexpected characters.
		if kind == TokenKind::Error
		{
			report.error_span(
                "unexpected character",
                &span);

			return Err(());
		}
		
		// Add to the token list.
		let token = Token {
			span,
			kind,
			excerpt,
		};
		
		tokens.push(token);
		
		index += length;
	}

	Ok(tokens)
}


fn check_for_whitespace(src: &[char]) -> Option<(TokenKind, usize)>
{
	let mut length = 0;
	
	if !is_whitespace(src[length])
		{ return None; }
	
	while length < src.len() && is_whitespace(src[length])
		{ length += 1; }
		
	Some((TokenKind::Whitespace, length))
}


fn check_for_comment(src: &[char]) -> Option<(TokenKind, usize)>
{
	let mut length = 0;
	
	if src[length] != ';'
		{ return None; }

	if length + 1 < src.len() && src[length + 1] == '*'
    {
		let mut nesting = 1;
		length += 2;

		loop
		{
			if length + 1 >= src.len()
				{ return None; }

			if src[length] == ';' && src[length + 1] == '*'
			{
				nesting += 1;
				length += 2;
				continue;
			}
			
			if src[length] == '*' && src[length + 1] == ';'
			{
				nesting -= 1;
				length += 2;

				if nesting == 0
					{ break; }

				continue;
			}

			length += 1;
		}

    	return Some((TokenKind::Comment, length));
    }
    else
    {
    	while length < src.len() && src[length] != '\n'
    		{ length += 1; }
    	return Some((TokenKind::Comment, length));
    }
}


fn check_for_identifier(src: &[char]) -> Option<(TokenKind, usize)>
{
	let mut length = 0;
	
	if !is_identifier_start(src[length])
		{ return None; }
	
	while length < src.len() && is_identifier_mid(src[length])
		{ length += 1; }
		
	Some((TokenKind::Identifier, length))
}


fn check_for_number(src: &[char]) -> Option<(TokenKind, usize)>
{
	let mut length = 0;
	
	if !is_number_start(src[length])
		{ return None; }
	
	while length < src.len() && is_number_mid(src[length])
		{ length += 1; }
		
	Some((TokenKind::Number, length))
}


fn check_for_string(src: &[char]) -> Option<(TokenKind, usize)>
{
	let mut length = 0;
	
	if src[length] != '\"'
		{ return None; }
		
	length += 1;
	
	while length < src.len() && src[length] != '\"'
		{ length += 1; }
		
	if length >= src.len()
		{ return None; }
		
	if src[length] != '\"'
		{ return None; }
		
	length += 1;
		
	Some((TokenKind::String, length))
}


fn check_for_fixed(src: &[char]) -> Option<(TokenKind, usize)>
{
	static POSSIBLE_TOKENS: [(&str, TokenKind); 41] =
	[
		("\n",  TokenKind::LineBreak),
		("asm", TokenKind::KeywordAsm),
		("(",   TokenKind::ParenOpen),
		(")",   TokenKind::ParenClose),
		("[",   TokenKind::BracketOpen),
		("]",   TokenKind::BracketClose),
		("{",   TokenKind::BraceOpen),
		("}",   TokenKind::BraceClose),
		(".",   TokenKind::Dot),
		(",",   TokenKind::Comma),
		("::",  TokenKind::ColonColon),
		(":",   TokenKind::Colon),
		("->",  TokenKind::ArrowRight),
		("<-",  TokenKind::ArrowLeft),
		("=>",  TokenKind::HeavyArrowRight),
		("#",   TokenKind::Hash),
		("+",   TokenKind::Plus),
		("-",   TokenKind::Minus),
		("*",   TokenKind::Asterisk),
		("/",   TokenKind::Slash),
		("%",   TokenKind::Percent),
		("^",   TokenKind::Circumflex),
		("~",   TokenKind::Tilde),
		("@",   TokenKind::At),
		("`",   TokenKind::Grave),
		("&&",  TokenKind::AmpersandAmpersand),
		("&",   TokenKind::Ampersand),
		("||",  TokenKind::VerticalBarVerticalBar),
		("|",   TokenKind::VerticalBar),
		("==",  TokenKind::EqualEqual),
		("=",   TokenKind::Equal),
		("?",   TokenKind::Question),
		("!=",  TokenKind::ExclamationEqual),
		("!",   TokenKind::Exclamation),
		("<=",  TokenKind::LessThanEqual),
		("<<",  TokenKind::LessThanLessThan),
		("<",   TokenKind::LessThan),
		(">=",  TokenKind::GreaterThanEqual),
		(">>>", TokenKind::GreaterThanGreaterThanGreaterThan),
		(">>",  TokenKind::GreaterThanGreaterThan),
		(">",   TokenKind::GreaterThan)
	];
	
	let maybe_match = POSSIBLE_TOKENS.iter().find(|op|
	{
		for (i, c) in op.0.chars().enumerate()
		{
			if i >= src.len() || src[i] != c
				{ return false; }
		}
		true
	});
	
	maybe_match.map(|s| { (s.1, s.0.chars().count()) })
}


pub fn is_whitespace(c: char) -> bool
{
	c == ' '  ||
	c == '\t' ||
	c == '\r'
}


fn is_identifier_start(c: char) -> bool
{
	(c >= 'a' && c <= 'z') ||
	(c >= 'A' && c <= 'Z') ||
	c == '_' ||
	c == '$'
}


fn is_identifier_mid(c: char) -> bool
{
	(c >= 'a' && c <= 'z') ||
	(c >= 'A' && c <= 'Z') ||
	(c >= '0' && c <= '9') ||
	c == '_' ||
	c == '$'
}


fn is_number_start(c: char) -> bool
{
	c >= '0' && c <= '9'
}


fn is_number_mid(c: char) -> bool
{
	(c >= 'a' && c <= 'z') ||
	(c >= 'A' && c <= 'Z') ||
	(c >= '0' && c <= '9') ||
	c == '_' ||
	c == '.'
}
