use crate::diagn::{Span, RcReport};
use std::rc::Rc;


#[derive(Debug, Clone)]
pub struct Token
{
	pub span: Span,
	pub kind: TokenKind,
	pub excerpt: Option<String>
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TokenKind
{
	Error,
	Whitespace,
	Comment,
	LineBreak,
	Identifier,
	Number,
	String,
	KeywordAsm,
	ParenOpen,
	ParenClose,
	BracketOpen,
	BracketClose,
	BraceOpen,
	BraceClose,
	Dot,
	Comma,
	Colon,
	ColonColon,
	ArrowRight,
	ArrowLeft,
	HeavyArrowRight,
	Hash,
	Equal,
	Plus,
	Minus,
	Asterisk,
	Slash,
	Percent,
	Question,
	Exclamation,
	Ampersand,
	VerticalBar,
	Circumflex,
	Tilde,
	Grave,
	At,
	AmpersandAmpersand,
	VerticalBarVerticalBar,
	EqualEqual,
	ExclamationEqual,
	LessThan,
	LessThanLessThan,
	LessThanEqual,
	GreaterThan,
	GreaterThanGreaterThan,
	GreaterThanGreaterThanGreaterThan,
	GreaterThanEqual
}


impl TokenKind
{
	fn needs_excerpt(self) -> bool
	{
		self == TokenKind::Identifier ||
		self == TokenKind::Number ||
		self == TokenKind::String
	}
	
	
	pub fn ignorable(self) -> bool
	{
		self == TokenKind::Whitespace ||
		self == TokenKind::Comment ||
		self == TokenKind::LineBreak
	}
	
	
	pub fn is_allowed_pattern_token(self) -> bool
	{
		self == TokenKind::Identifier ||
		self == TokenKind::Number ||
		self == TokenKind::KeywordAsm ||
		self == TokenKind::ParenOpen ||
		self == TokenKind::ParenClose ||
		self == TokenKind::BracketOpen ||
		self == TokenKind::BracketClose ||
		self == TokenKind::Dot ||
		self == TokenKind::Comma ||
		self == TokenKind::ArrowLeft ||
		self == TokenKind::ArrowRight ||
		self == TokenKind::Hash ||
		self == TokenKind::Plus ||
		self == TokenKind::Minus ||
		self == TokenKind::Asterisk ||
		self == TokenKind::Slash ||
		self == TokenKind::Percent ||
		self == TokenKind::Exclamation ||
		self == TokenKind::Ampersand ||
		self == TokenKind::VerticalBar ||
		self == TokenKind::Circumflex ||
		self == TokenKind::Tilde ||
		self == TokenKind::At ||
		self == TokenKind::LessThan ||
		self == TokenKind::GreaterThan
	}
	
	
	pub fn is_allowed_after_pattern_parameter(self) -> bool
	{
		self == TokenKind::ParenClose ||
		self == TokenKind::BracketClose ||
		self == TokenKind::Comma
	}
	
	
	pub fn printable(self) -> &'static str
	{
		match self
		{
			TokenKind::Error => "error",
			TokenKind::Whitespace => "whitespace",
			TokenKind::Comment => "comment",
			TokenKind::LineBreak => "line break",
			TokenKind::Identifier => "identifier",
			TokenKind::Number => "number",
			TokenKind::String => "string",
			TokenKind::KeywordAsm => "`asm` keyword",
			TokenKind::ParenOpen => "`(`",
			TokenKind::ParenClose => "`)`",
			TokenKind::BracketOpen => "`[`",
			TokenKind::BracketClose => "`]`",
			TokenKind::BraceOpen => "`{`",
			TokenKind::BraceClose => "`}`",
			TokenKind::Dot => "`.`",
			TokenKind::Comma => "`,`",
			TokenKind::Colon => "`:`",
			TokenKind::ColonColon => "`::`",
			TokenKind::ArrowRight => "`->`",
			TokenKind::ArrowLeft => "`<-`",
			TokenKind::HeavyArrowRight => "`=>`",
			TokenKind::Hash => "`#`",
			TokenKind::Equal => "`=`",
			TokenKind::Plus => "`+`",
			TokenKind::Minus => "`-`",
			TokenKind::Asterisk => "`*`",
			TokenKind::Slash => "`/`",
			TokenKind::Percent => "`%`",
			TokenKind::Question => "`?`",
			TokenKind::Exclamation => "`!`",
			TokenKind::Ampersand => "`&`",
			TokenKind::VerticalBar => "`|`",
			TokenKind::Circumflex => "`^`",
			TokenKind::Tilde => "`~`",
			TokenKind::At => "`@`",
			TokenKind::Grave => "```",
			TokenKind::AmpersandAmpersand => "`&&`",
			TokenKind::VerticalBarVerticalBar => "`||`",
			TokenKind::EqualEqual => "`==`",
			TokenKind::ExclamationEqual => "`!=`",
			TokenKind::LessThan => "`<`",
			TokenKind::LessThanLessThan => "`<<`",
			TokenKind::LessThanEqual => "`<=`",
			TokenKind::GreaterThan => "`>`",
			TokenKind::GreaterThanGreaterThan => "`>>`",
			TokenKind::GreaterThanGreaterThanGreaterThan => "`>>>`",
			TokenKind::GreaterThanEqual => "`>=`"
		}
	}
	
	
	pub fn printable_excerpt(self, excerpt: Option<&str>) -> String
	{
		if self.needs_excerpt()
			{ format!("`{}`", excerpt.unwrap()) }
		else
			{ self.printable().to_string() }
	}
}


impl Token
{
	pub fn text(&self) -> &str
	{
		match self.kind
		{
			TokenKind::KeywordAsm => "asm",
			TokenKind::ParenOpen => "(",
			TokenKind::ParenClose => ")",
			TokenKind::BracketOpen => "[",
			TokenKind::BracketClose => "]",
			TokenKind::BraceOpen => "{",
			TokenKind::BraceClose => "}",
			TokenKind::Dot => ".",
			TokenKind::Comma => ",",
			TokenKind::Colon => ":",
			TokenKind::ColonColon => "::",
			TokenKind::ArrowRight => "->",
			TokenKind::ArrowLeft => "<-",
			TokenKind::HeavyArrowRight => "=>",
			TokenKind::Hash => "#",
			TokenKind::Equal => "=",
			TokenKind::Plus => "+",
			TokenKind::Minus => "-",
			TokenKind::Asterisk => "*",
			TokenKind::Slash => "/",
			TokenKind::Percent => "%",
			TokenKind::Question => "?",
			TokenKind::Exclamation => "!",
			TokenKind::Ampersand => "&",
			TokenKind::VerticalBar => "|",
			TokenKind::Circumflex => "^",
			TokenKind::Tilde => "~",
			TokenKind::At => "@",
			TokenKind::Grave => "`",
			TokenKind::AmpersandAmpersand => "&&",
			TokenKind::VerticalBarVerticalBar => "||",
			TokenKind::EqualEqual => "==",
			TokenKind::ExclamationEqual => "!=",
			TokenKind::LessThan => "<",
			TokenKind::LessThanLessThan => "<<",
			TokenKind::LessThanEqual => "<=",
			TokenKind::GreaterThan => ">",
			TokenKind::GreaterThanGreaterThan => ">>",
			TokenKind::GreaterThanGreaterThanGreaterThan => ">>>",
			TokenKind::GreaterThanEqual => ">=",
			_ => self.excerpt.as_ref().unwrap()
		}
	}
}


pub fn tokenize<S>(report: RcReport, src_filename: S, src: &[char]) -> Result<Vec<Token>, ()>
where S: Into<String>
{
	let filename = Rc::new(src_filename.into());
	let mut tokens = Vec::new();
	let mut index = 0;
	let mut had_error = false;
	
	while index < src.len()
	{
		// Decide what are the next token's kind and length.
		let (kind, length) =
			check_for_whitespace(&src[index..]).unwrap_or_else(||
			check_for_comment   (&src[index..]).unwrap_or_else(||
			check_for_fixed     (&src[index..]).unwrap_or_else(||
			check_for_identifier(&src[index..]).unwrap_or_else(||
			check_for_number    (&src[index..]).unwrap_or_else(||
			check_for_string    (&src[index..]).unwrap_or_else(||
			(TokenKind::Error, 1)))))));
		
		let span = Span::new(filename.clone(), index, index + length);
		
		// Get the source excerpt for variable tokens (e.g. identifiers).
		let excerpt = match kind.needs_excerpt()
		{
			true => Some(src[index..].iter().cloned().take(length).collect()),
			false => None
		};
		
		// Report unexpected characters.
		if kind == TokenKind::Error
		{
			report.error_span("unexpected character", &span);
			had_error = true;
		}
		
		// Add to the token list.
		let token = Token
		{
			span: span,
			kind: kind,
			excerpt: excerpt
		};
		
		tokens.push(token);
		
		index += length;
	}

	if had_error
		{ return Err(()); }
	
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
	c == '.' ||
	c == '\''
}
