const STYLE_DEFAULT: &'static str = "\u{001b}[0m";
const STYLE_GRAY:    &'static str = "\u{001b}[90m";
const STYLE_RED:     &'static str = "\u{001b}[91m";
const STYLE_YELLOW:  &'static str = "\u{001b}[93m";
const STYLE_CYAN:    &'static str = "\u{001b}[96m";
const STYLE_WHITE:   &'static str = "\u{001b}[97m";
const STYLE_BOLD:    &'static str = "\u{001b}[1m";


pub struct StringStyler
{
    pub result: String,
    pub use_colors: bool,
}


impl StringStyler
{
    pub fn new(
        use_colors: bool)
        -> StringStyler
    {
        StringStyler {
            result: String::new(),
            use_colors,
        }
    }


    pub fn add(
        &mut self,
        string: &str)
    {
        self.result.push_str(string);
    }


    pub fn add_char(
        &mut self,
        ch: char)
    {
        self.result.push(ch);
    }


    pub fn addln(
        &mut self,
        string: &str)
    {
        self.result.push_str(string);
        self.result.push_str("\n");
    }
	
	
	pub fn indent(
		&mut self,
		indent: usize)
	{
		for _ in 0..indent
		{
			self.add(" ");
		}
	}


    pub fn add_styled(
        &mut self,
        string: &str,
        pattern: &str,
        fn_start: &mut dyn FnMut(&mut StringStyler),
        fn_end: &mut dyn FnMut(&mut StringStyler))
    {
        let mut index = 0;

        for (pos, substr) in string.match_indices(pattern)
        {
            while index < pos
            {
                self.add(string.get(index..pos).unwrap());
                index = pos + substr.len();
            }

            fn_start(self);
            self.add(substr);
            fn_end(self);
        }

        if index < string.len()
        {
            self.add(string.get(index..).unwrap());
        }
    }


    pub fn add_style(
        &mut self,
        style: &str)
    {
        if self.use_colors
        {
            self.add(style);
        }
    }


    pub fn reset(&mut self)
    {
        self.add_style(STYLE_DEFAULT);
    }


    pub fn gray(&mut self)
    {
        self.add_style(STYLE_GRAY);
    }


    pub fn white(&mut self)
    {
        self.add_style(STYLE_WHITE);
    }


    pub fn red(&mut self)
    {
        self.add_style(STYLE_RED);
    }


    pub fn yellow(&mut self)
    {
        self.add_style(STYLE_YELLOW);
    }


    pub fn cyan(&mut self)
    {
        self.add_style(STYLE_CYAN);
    }


    pub fn bold(&mut self)
    {
        self.add_style(STYLE_BOLD);
    }
}