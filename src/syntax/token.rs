use crate::*;


#[derive(Clone, Debug)]
pub struct Token
{
	pub span: diagn::Span,
	pub kind: TokenKind,
	pub excerpt: Option<String>,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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
	KeywordTrue,
	KeywordFalse,
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
	DoubleAmpersand,
	DoubleVerticalBar,
	DoubleEqual,
	ExclamationEqual,
	LessThan,
	DoubleLessThan,
	LessThanEqual,
	GreaterThan,
	DoubleGreaterThan,
	TripleGreaterThan,
	GreaterThanEqual
}


impl TokenKind
{
	pub fn needs_excerpt(self) -> bool
	{
		self == TokenKind::Identifier ||
		self == TokenKind::Number ||
		self == TokenKind::String
	}
	
	
	pub fn is_ignorable(self) -> bool
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
		self == TokenKind::KeywordTrue ||
		self == TokenKind::KeywordFalse ||
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
			TokenKind::KeywordTrue => "`true` keyword",
			TokenKind::KeywordFalse => "`false` keyword",
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
			TokenKind::DoubleAmpersand => "`&&`",
			TokenKind::DoubleVerticalBar => "`||`",
			TokenKind::DoubleEqual => "`==`",
			TokenKind::ExclamationEqual => "`!=`",
			TokenKind::LessThan => "`<`",
			TokenKind::DoubleLessThan => "`<<`",
			TokenKind::LessThanEqual => "`<=`",
			TokenKind::GreaterThan => "`>`",
			TokenKind::DoubleGreaterThan => "`>>`",
			TokenKind::TripleGreaterThan => "`>>>`",
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
			TokenKind::Error => "�",
			TokenKind::Comment => ";* *;",
			TokenKind::Whitespace => " ",
			TokenKind::LineBreak => "\\n",
			TokenKind::KeywordAsm => "asm",
			TokenKind::KeywordTrue => "true",
			TokenKind::KeywordFalse => "false",
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
			TokenKind::DoubleAmpersand => "&&",
			TokenKind::DoubleVerticalBar => "||",
			TokenKind::DoubleEqual => "==",
			TokenKind::ExclamationEqual => "!=",
			TokenKind::LessThan => "<",
			TokenKind::DoubleLessThan => "<<",
			TokenKind::LessThanEqual => "<=",
			TokenKind::GreaterThan => ">",
			TokenKind::DoubleGreaterThan => ">>",
			TokenKind::TripleGreaterThan => ">>>",
			TokenKind::GreaterThanEqual => ">=",
			_ => self.excerpt.as_ref().unwrap()
		}
	}
}


pub fn decide_next_token(
	src: &str)
	-> (TokenKind, usize)
{
	check_for_whitespace(src).unwrap_or_else(||
	check_for_comment   (src).unwrap_or_else(||
	check_for_number    (src).unwrap_or_else(||
	check_for_identifier(src).unwrap_or_else(||
	check_for_special   (src).unwrap_or_else(||
	check_for_string    (src).unwrap_or_else(||
	(TokenKind::Error, 1)))))))
}


#[derive(Clone)]
struct CharWalker<'a>
{
	src: &'a str,
	char_indices: std::str::CharIndices<'a>,
	current: char,
	length: usize,
}


impl<'a> CharWalker<'a>
{
	pub fn new(src: &'a str) -> CharWalker<'a>
	{
		let mut walker = CharWalker {
			src,
			char_indices: src.char_indices(),
			current: '\0',
			length: 0,
		};

		walker.advance();
		walker
	}


	pub fn ended(&self) -> bool
	{
		self.length >= self.src.len()
	}


	pub fn advance(&mut self)
	{
		match self.char_indices.next()
		{
			None =>
			{
				self.current = '\0';
				self.length = self.src.len();
			}
			Some((index, c)) =>
			{
				self.current = c;
				self.length = index;
			}
		}
	}


	pub fn consume_if(&mut self, fn_test: fn(char) -> bool) -> bool
	{
		if fn_test(self.current)
		{
			self.advance();
			true
		}
		else
		{
			false
		}
	}


	pub fn consume_char(&mut self, wanted: char) -> bool
	{
		if self.current == wanted
		{
			self.advance();
			true
		}
		else
		{
			false
		}
	}


	pub fn consume_str(&mut self, wanted: &str) -> bool
	{
		let mut cloned = self.clone();

		for c in wanted.chars()
		{
			if !cloned.consume_char(c)
			{
				return false;
			}
		}

		*self = cloned;
		true
	}


	pub fn consume_while(
		&mut self,
		fn_start: fn(char) -> bool,
		fn_mid: fn(char) -> bool)
		-> bool
	{
		if !self.consume_if(fn_start)
		{
			return false;
		}

		while self.consume_if(fn_mid) {}

		true
	}


	pub fn consume_until_char(&mut self, wanted: char)
	{
		while !self.ended() && self.current != wanted
		{
			self.advance();
		}
	}
}


fn check_for_whitespace(src: &str) -> Option<(TokenKind, usize)>
{
	let mut walker = CharWalker::new(src);
	
	if !walker.consume_while(
		is_whitespace,
		is_whitespace)
	{
		return None;
	}

	Some((TokenKind::Whitespace, walker.length))
}


fn check_for_comment(src: &str) -> Option<(TokenKind, usize)>
{
	let mut walker = CharWalker::new(src);
	
	if !walker.consume_char(';')
		{ return None; }

	if walker.consume_char('*')
    {
		let mut nesting = 0;

		loop
		{
			if walker.ended()
			{
				break;
			}

			else if walker.consume_str(";*")
			{
				nesting += 1;
			}
			
			else if walker.consume_str("*;")
			{
				if nesting == 0
					{ break; }

				nesting -= 1;
			}

			else
			{
				walker.advance();
			}
		}

    	return Some((TokenKind::Comment, walker.length));
    }
    else
    {
		walker.consume_until_char('\n');
    	return Some((TokenKind::Comment, walker.length));
    }
}


fn check_for_identifier(src: &str) -> Option<(TokenKind, usize)>
{
	let mut walker = CharWalker::new(src);

	if walker.consume_if(|c| c == '$')
	{
		return Some((TokenKind::Identifier, walker.length));
	}
	
	if !walker.consume_while(
		is_identifier_start,
		is_identifier_mid)
	{
		return None;
	}

	let length = walker.length;

	let ident = src.get(0..length).unwrap();

	static KEYWORDS: [(&str, TokenKind); 3] =
	[
		("asm", TokenKind::KeywordAsm),
		("true", TokenKind::KeywordTrue),
		("false", TokenKind::KeywordFalse),
	];

	for keyword in KEYWORDS
	{
		if ident == keyword.0
		{
			return Some((keyword.1, length));
		}
	}
		
	Some((TokenKind::Identifier, length))
}


fn check_for_number(src: &str) -> Option<(TokenKind, usize)>
{
	let mut walker = CharWalker::new(src);

	if walker.consume_while(
		is_number_start,
		is_number_mid)
	{
		return Some((TokenKind::Number, walker.length));
	}

	else if walker.consume_char('$')
	{
		if walker.consume_while(
			is_hex_number_mid,
			is_hex_number_mid)
		{
			return Some((TokenKind::Number, walker.length));
		}
	}

	else if walker.consume_char('%')
	{
		if walker.consume_while(
			is_bin_number_mid,
			is_bin_number_mid)
		{
			return Some((TokenKind::Number, walker.length));
		}
	}

	None
}


fn check_for_string(src: &str) -> Option<(TokenKind, usize)>
{
	let mut walker = CharWalker::new(src);

	if !walker.consume_char('\"')
		{ return None; }
		
	walker.consume_until_char('\"');
		
	if !walker.consume_char('\"')
		{ return None; }
		
	Some((TokenKind::String, walker.length))
}


fn check_for_special(src: &str) -> Option<(TokenKind, usize)>
{
	static TOKENS: [(&str, TokenKind); 40] =
	[
		("\n",  TokenKind::LineBreak),
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
		("&&",  TokenKind::DoubleAmpersand),
		("&",   TokenKind::Ampersand),
		("||",  TokenKind::DoubleVerticalBar),
		("|",   TokenKind::VerticalBar),
		("==",  TokenKind::DoubleEqual),
		("=",   TokenKind::Equal),
		("?",   TokenKind::Question),
		("!=",  TokenKind::ExclamationEqual),
		("!",   TokenKind::Exclamation),
		("<=",  TokenKind::LessThanEqual),
		("<<",  TokenKind::DoubleLessThan),
		("<",   TokenKind::LessThan),
		(">=",  TokenKind::GreaterThanEqual),
		(">>>", TokenKind::TripleGreaterThan),
		(">>",  TokenKind::DoubleGreaterThan),
		(">",   TokenKind::GreaterThan)
	];

	let mut walker = CharWalker::new(src);

	for tk in TOKENS
	{
		if walker.consume_str(tk.0)
		{
			return Some((tk.1, walker.length));
		}
	}
	
	None
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
	c == '_'
}


fn is_identifier_mid(c: char) -> bool
{
	(c >= 'a' && c <= 'z') ||
	(c >= 'A' && c <= 'Z') ||
	(c >= '0' && c <= '9') ||
	c == '_'
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
	c == '_'
}


fn is_bin_number_mid(c: char) -> bool
{
	(c >= '0' && c <= '1') ||
	c == '_'
}


fn is_hex_number_mid(c: char) -> bool
{
	(c >= 'a' && c <= 'f') ||
	(c >= 'A' && c <= 'F') ||
	(c >= '0' && c <= '9') ||
	c == '_'
}