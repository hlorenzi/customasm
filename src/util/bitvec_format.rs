use crate::*;


pub struct FormatListOptions
{
	pub base: usize,
	pub digits_per_group: usize,
	pub groups_per_group2: usize,
	pub str_before: String,
	pub str_after: String,
	pub str_between_groups: String,
	pub str_between_groups2: String,
}


impl util::BitVec
{
	pub fn format_binary(&self) -> Vec<u8>
	{
		let mut result = Vec::new();

		let mut index = 0;
		while index < self.len()
		{
			let mut byte: u8 = 0;
			for _ in 0..8
			{
				byte <<= 1;
				byte |= if self.read_bit(index) { 1 } else { 0 };
				index += 1;
			}

			result.push(byte);
		}

		result
    }


	pub fn format_binstr(&self) -> String
	{
		self.format_str(1)
	}


	pub fn format_hexstr(&self) -> String
	{
		self.format_str(4)
	}


	pub fn format_str(&self, bits_per_digit: usize) -> String
	{
		let mut result = String::new();

		let mut index = 0;
		while index < self.len()
		{
			let mut digit: u8 = 0;
			for _ in 0..bits_per_digit
			{
				digit <<= 1;
				digit |= if self.read_bit(index) { 1 } else { 0 };
				index += 1;
			}

			let c = if digit < 10
				{ ('0' as u8 + digit) as char }
			else
				{ ('a' as u8 + digit - 10) as char };

			result.push(c);
		}

		result
    }


    pub fn format_bindump(&self) -> String
    {
        self.format_dump(1, 8, 8)
    }


    pub fn format_hexdump(&self) -> String
    {
        self.format_dump(4, 8, 16)
    }


    pub fn format_dump(
        &self,
        digit_bits: usize,
        byte_bits: usize,
        bytes_per_line: usize)
        -> String
    {
        let mut result = String::new();

        let line_start = 0 / (byte_bits * bytes_per_line);
        let line_end = (self.len() + (bytes_per_line - 1) * byte_bits) / (byte_bits * bytes_per_line);

        let line_end = if self.len() < byte_bits { line_start + 1 } else { line_end };

        let addr_max_width = format!("{:x}", (line_end - 1) * bytes_per_line).len();

        for line_index in line_start..line_end
        {
            result.push_str(&format!(" {:01$x} | ", line_index * bytes_per_line, addr_max_width));

            for byte_index in 0..bytes_per_line
            {
                for digit_index in 0..(byte_bits / digit_bits)
                {
                    let digit_first_bit = (line_index * bytes_per_line + byte_index) * byte_bits + digit_index * digit_bits;

                    if digit_first_bit >= self.len()
                    {
                        result.push('.');
                        continue;
                    }

                    let mut digit = 0;
                    for bit_index in 0..digit_bits
                    {
                        digit <<= 1;
                        digit |= if self.read_bit(digit_first_bit + bit_index) { 1 } else { 0 };
                    }

                    let c = if digit < 10
                        { ('0' as u8 + digit) as char }
                    else
                        { ('a' as u8 + digit - 10) as char };

                    result.push(c);
                }

                result.push(' ');

                if byte_index % 4 == 3 && byte_index < bytes_per_line - 1
                    { result.push(' '); }
            }

            result.push_str("| ");

            if byte_bits == 8
            {
                for byte_index in 0..bytes_per_line
                {
                    let byte_first_bit = (line_index * bytes_per_line + byte_index) * byte_bits;

                    if byte_first_bit >= self.len()
                    {
                        result.push('.');
                        continue;
                    }

                    let mut byte = 0u8;
                    for bit_index in 0..byte_bits
                    {
                        byte <<= 1;
                        byte |= if self.read_bit(byte_first_bit + bit_index) { 1 } else { 0 };
                    }

                    let c = byte as char;

                    if c == ' ' || c == '\t' || c == '\r' || c == '\n'
                        { result.push(' '); }
                    else if c as u8 >= 0x80 || c < ' ' || c == '|'
                        { result.push('.'); }
                    else
                        { result.push(c); }
                }

                result.push_str(" |");
            }

            result.push('\n');
        }

        result
    }


	pub fn format_mif(&self) -> String
	{
		let mut result = String::new();

		let byte_num = self.len() / 8 + if self.len() % 8 != 0 { 1 } else { 0 };

		result.push_str(&format!("DEPTH = {};\n", byte_num));
		result.push_str("WIDTH = 8;\n");
		result.push_str("ADDRESS_RADIX = HEX;\n");
		result.push_str("DATA_RADIX = HEX;\n");
		result.push_str("\n");
		result.push_str("CONTENT\n");
		result.push_str("BEGIN\n");

		let addr_max_width = format!("{:x}", byte_num - 1).len();

		let mut index = 0;
		while index < self.len()
		{
			result.push_str(&format!(" {:1$X}: ", index / 8, addr_max_width));

			let mut byte: u8 = 0;
			for _ in 0..8
			{
				byte <<= 1;
				byte |= if self.read_bit(index) { 1 } else { 0 };
				index += 1;
			}

			result.push_str(&format!("{:02X};\n", byte));
		}

		result.push_str("END;");
		result
	}
	
	
	pub fn format_intelhex(&self, address_unit: usize) -> String
	{	
		let mut result = String::new();

		let mut flush_bytes = |
			read_index: usize,
			accum_index: &mut usize,
			accum_bytes: &mut Vec::<u8>|
		{
			let length = accum_bytes.len() as u8;

			if length > 0
			{
				let addr_hi = ((*accum_index / address_unit) >> 8) as u8;
				let addr_lo = (*accum_index / address_unit) as u8;

				result.push(':');
				result.push_str(&format!("{:02X}", length));
				result.push_str(&format!("{:02X}", addr_hi));
				result.push_str(&format!("{:02X}", addr_lo));
				result.push_str("00");

				let mut checksum = 0_u8;
				checksum = checksum.wrapping_add(length);
				checksum = checksum.wrapping_add(addr_hi);
				checksum = checksum.wrapping_add(addr_lo);

				for byte in accum_bytes.iter().copied()
				{
					result.push_str(&format!("{:02X}", byte));
					checksum = checksum.wrapping_add(byte);
				}
				
				result.push_str(&format!("{:02X}", (!checksum).wrapping_add(1)));
				result.push('\n');
			}

			accum_bytes.clear();
			*accum_index = read_index;
		};

		for block in self.get_blocks()
		{
			let mut read_index = block.offset;
			let mut accum_index = block.offset;
			let mut accum_bytes = Vec::<u8>::new();
	
			while read_index < block.offset + block.size
			{
				let mut byte: u8 = 0;
				for _ in 0..8
				{
					byte <<= 1;
					byte |= if self.read_bit(read_index) { 1 } else { 0 };
					read_index += 1;
				}

				accum_bytes.push(byte);

				if accum_bytes.len() >= 32
				{
					flush_bytes(read_index, &mut accum_index, &mut accum_bytes);
				}
			}

			flush_bytes(read_index, &mut accum_index, &mut accum_bytes);
		}

		result.push_str(":00000001FF");
		result
	}


	pub fn format_list(&self, opts: &FormatListOptions) -> String
	{
		let mut result = String::new();
		result.push_str(&opts.str_before);

		let bits_per_digit = (opts.base - 1).count_ones() as usize;

		let mut index = 0;
		let mut groups_output = 0;
		while index < self.len()
		{
			for _ in 0..opts.digits_per_group
			{
				let mut digit: u8 = 0;
				for _ in 0..bits_per_digit
				{
					digit <<= 1;
					digit |= if self.read_bit(index) { 1 } else { 0 };
					index += 1;
				}

				let c = {
					if digit < 10
						{ ('0' as u8 + digit) as char }
					else
						{ ('a' as u8 + digit - 10) as char }
				};
				
				result.push(c);
			}

			if index < self.len()
			{
				groups_output += 1;

				if opts.str_between_groups2.len() > 0 &&
					groups_output % opts.groups_per_group2 == 0
				{
					result.push_str(&opts.str_between_groups2);
				}
				else
				{
					result.push_str(&opts.str_between_groups);
				}
			}
		}

		result.push_str(&opts.str_after);

		result
	}


	pub fn format_separator(&self, radix: usize, separator: &str) -> String
	{
		let mut result = String::new();

		let mut index = 0;
		while index < self.len()
		{
			let mut byte: u8 = 0;
			for _ in 0..8
			{
				byte <<= 1;
				byte |= if self.read_bit(index) { 1 } else { 0 };
				index += 1;
			}

			match radix
			{
				10 => result.push_str(&format!("{}", byte)),
				16 => result.push_str(&format!("0x{:02x}", byte)),
				_  => panic!("invalid radix")
			}

			if index < self.len()
			{
				result.push_str(separator);

				if (index / 8) % 16 == 0
					{ result.push('\n'); }
			}
		}

		result
	}


	pub fn format_c_array(&self, radix: usize) -> String
	{
		let mut result = String::new();

		result.push_str("const unsigned char data[] = {\n");

		let byte_num = self.len() / 8 + if self.len() % 8 != 0 { 1 } else { 0 };
		let addr_max_width = format!("{:x}", byte_num - 1).len();

		let mut index = 0;
		result.push_str(&format!("\t/* 0x{:01$x} */ ", 0, addr_max_width));

		while index < self.len()
		{
			let mut byte: u8 = 0;
			for _ in 0..8
			{
				byte <<= 1;
				byte |= if self.read_bit(index) { 1 } else { 0 };
				index += 1;
			}

			match radix
			{
				10 => result.push_str(&format!("{}", byte)),
				16 => result.push_str(&format!("0x{:02x}", byte)),
				_  => panic!("invalid radix")
			}

			if index < self.len()
			{
				result.push_str(", ");

				if (index / 8) % 16 == 0
				{
					result.push_str(&format!("\n\t/* 0x{:01$x} */ ", index / 8, addr_max_width));
				}
			}
		}

		result.push_str("\n};");
		result
	}


	// From: https://github.com/milanvidakovic/customasm/blob/master/src/asm/binary_output.rs#L84
	pub fn format_logisim(&self, bits_per_chunk: usize) -> String
	{
		let mut result = String::new();
		result.push_str("v2.0 raw\n");

		let mut index = 0;
		while index < self.len()
		{
			let mut value: u16 = 0;
			for _ in 0..bits_per_chunk
			{
				value <<= 1;
				value |= if self.read_bit(index) { 1 } else { 0 };
				index += 1;
			}

			result.push_str(&format!("{:01$x} ", value, bits_per_chunk / 4));
			if (index / 8) % 16 == 0
				{ result.push('\n'); }
		}

		result
    }


	pub fn format_annotated(
		&self,
		fileserver: &dyn util::FileServer,
		base: usize,
		digits_per_group: usize)
		-> String
	{
		let mut result = String::new();

		let bits_per_digit = (base - 1).count_ones() as usize;
		let bits_per_group = digits_per_group * bits_per_digit;

		let mut outp_width = 2;
		let mut outp_bit_width = 1;
		let mut addr_width = 4;
		let mut content_width = (digits_per_group + 1) * 1 - 1;

		let mut sorted_spans = self.spans.clone();
        sorted_spans.sort_by(|a, b|
        	a.offset.cmp(&b.offset));

        for span in &sorted_spans
        {
            if let Some(offset) = span.offset
            {
                outp_width = std::cmp::max(
                    outp_width,
                    format!("{:x}", offset / bits_per_group).len());

				outp_bit_width = std::cmp::max(
					outp_bit_width,
					format!("{:x}", offset % bits_per_group).len());

                addr_width = std::cmp::max(
                    addr_width,
                    format!("{:x}", span.addr).len());

                let data_digits = span.size / bits_per_digit + if span.size % bits_per_digit == 0 { 0 } else { 1 };
				let this_content_width = data_digits + data_digits / digits_per_group;

				if this_content_width > 1 && this_content_width <= (digits_per_group + 1) * 5
				{
					content_width = std::cmp::max(
						content_width,
						this_content_width - 1);
				}
            }
		}

		result.push_str(&format!(" {:>1$} |", "outp", outp_width + outp_bit_width + 1));
		result.push_str(&format!(" {:>1$} |", "addr", addr_width));
		result.push_str(&format!(" data (base {})", base));
		result.push_str("\n");
		result.push_str("\n");

		let mut prev_file_handle = util::FileServerHandle::MAX;
        let mut prev_file_chars = "".to_string();

        for span in &sorted_spans
        {
            if let Some(offset) = span.offset
            {
                result.push_str(&format!(" {:1$x}", offset / bits_per_group, outp_width));
                result.push_str(&format!(":{:1$x} | ", offset % bits_per_group, outp_bit_width));
            }
            else
            {
                result.push_str(&format!(" {:>1$}", "--", outp_width));
                result.push_str(&format!(":{:>1$} | ", "-", outp_bit_width));
            }

            result.push_str(&format!("{:1$x} | ", span.addr, addr_width));

            let mut contents_str = String::new();

            let digit_num = span.size / bits_per_digit + if span.size % bits_per_digit == 0 { 0 } else { 1 };
            for digit_index in 0..digit_num
            {
                if digit_index > 0 && digit_index % digits_per_group == 0
                    { contents_str.push_str(" "); }

                let mut digit = 0;
                for bit_index in 0..bits_per_digit
                {
                    let i = span.offset.unwrap() + digit_index * bits_per_digit + bit_index;
                    let bit = self.read_bit(i);

                    digit <<= 1;
                    digit |= if bit { 1 } else { 0 };
                }

                let c = if digit < 10
                    { ('0' as u8 + digit) as char }
                else
                    { ('a' as u8 + digit - 10) as char };

                contents_str.push(c);
            }

            if span.span.file_handle != prev_file_handle
            {
                prev_file_handle = span.span.file_handle;
                prev_file_chars = fileserver
					.get_str_unwrap(prev_file_handle);
            }

            let span_location = span.span.location().unwrap();
            let char_counter = util::CharCounter::new(&prev_file_chars);

            result.push_str(&format!("{:1$}", contents_str, content_width));
            result.push_str(&format!(" ; {}", char_counter.get_excerpt(span_location.0, span_location.1)));
            result.push_str("\n");
		}

		result
	}
	

	// Turing Complete is a game in which you advance from nand gates to
	// computer architecture. Its assembly editor uses `#` comments,
	// `0b` and `0x` prefixes for binary and hex, and groups of 8 or 8x4 bytes.
	// This format produces annotated bytecode that meets these constraints.
	//
	// The implementation is a clone of format_annotated. Large portions
	// could easily be factored out and shared or this could be converted into
	// a configuration option for format_annotated with some work.
	pub fn format_tcgame(
		&self,
		fileserver: &dyn util::FileServer,
		base: usize,
		digits_per_group: usize)
		-> String
	{
		let mut result = String::new();
		assert!(base == 2 || base == 16);
		let prefix: &str = if base == 2 { "0b" } else { "0x" };
		let comment: &str = "#";

		let bits_per_digit = (base - 1).count_ones() as usize;
		let bits_per_group = digits_per_group * bits_per_digit;

		let mut outp_width = 2;
		let mut outp_bit_width = 1;
		let mut addr_width = 4;
		let mut content_width = (digits_per_group + 1) * 1 - 1;

		let mut sorted_spans = self.spans.clone();
        sorted_spans.sort_by(|a, b|
                a.offset.cmp(&b.offset));

        for span in &sorted_spans
        {
            if let Some(offset) = span.offset
            {
                outp_width = std::cmp::max(
                    outp_width,
                    format!("{:x}", offset / bits_per_group).len());

				outp_bit_width = std::cmp::max(
					outp_bit_width,
					format!("{:x}", offset % bits_per_group).len());

                addr_width = std::cmp::max(
                    addr_width,
                    format!("{:x}", span.addr).len());

                let data_digits = span.size / bits_per_digit + if span.size % bits_per_digit == 0 { 0 } else { 1 };
				let this_content_width = data_digits + data_digits / digits_per_group;

				if this_content_width > 1 && this_content_width <= (digits_per_group + 1) * 5
				{
					content_width = std::cmp::max(
						content_width,
						this_content_width - 1);
				}
            }
		}
		result.push_str(&format!("{comment} {:>1$} |", "outp", outp_width + outp_bit_width + 1));
		result.push_str(&format!(" {:>1$} |", "addr", addr_width));
		result.push_str(&format!(" data (base {})", base));
		result.push_str("\n");
		result.push_str("\n");

		let mut prev_file_handle = util::FileServerHandle::MAX;
        let mut prev_file_chars = "".to_string();

        for span in &sorted_spans
        {
			result.push_str(&format!("{comment} "));
			// offset
            if let Some(offset) = span.offset
            {
                result.push_str(&format!(" {:1$x}", offset / bits_per_group, outp_width));
                result.push_str(&format!(":{:1$x} | ", offset % bits_per_group, outp_bit_width));
            }
            else
            {
                result.push_str(&format!(" {:>1$}", "--", outp_width));
                result.push_str(&format!(":{:>1$} | ", "-", outp_bit_width));
            }

			// addr
            result.push_str(&format!("{:1$x} \n", span.addr, addr_width));

			// instruction excerpt
			if span.span.file_handle != prev_file_handle
            {
                prev_file_handle = span.span.file_handle;
                prev_file_chars = fileserver
					.get_str_unwrap(prev_file_handle);
            }
            let span_location = span.span.location().unwrap();
            let char_counter = util::CharCounter::new(&prev_file_chars);
            result.push_str(&format!("{comment} {}\n", char_counter.get_excerpt(span_location.0, span_location.1)));

			// bytecode
            let mut contents_str = String::new();
            let digit_num = span.size / bits_per_digit + if span.size % bits_per_digit == 0 { 0 } else { 1 };
            for digit_index in 0..digit_num
            {
                if digit_index % digits_per_group == 0
				{
					if digit_index > 0
					{
						contents_str.push_str(" ");
					}
					contents_str.push_str(prefix);
				}

                let mut digit = 0;
                for bit_index in 0..bits_per_digit
                {
                    let i = span.offset.unwrap() + digit_index * bits_per_digit + bit_index;
                    let bit = self.read_bit(i);

                    digit <<= 1;
                    digit |= if bit { 1 } else { 0 };
                }

                let c = if digit < 10
                    { ('0' as u8 + digit) as char }
                else
                    { ('a' as u8 + digit - 10) as char };

                contents_str.push(c);
            }
            result.push_str(&format!("{:1$}\n", contents_str, content_width));
		}
		result
	}


    pub fn format_addrspan(&self, fileserver: &dyn util::FileServer) -> String
    {
        let mut result = String::new();

        let mut sorted_spans = self.spans.clone();
        sorted_spans.sort_by(|a, b| a.offset.cmp(&b.offset));

        result.push_str("; ");
        result.push_str("physical address : bit offset | ");
        result.push_str("logical address | ");
        result.push_str("file : line start : column start : line end : column end\n");

        for span in &sorted_spans
        {
            let chars =
				fileserver.get_str(
					&mut diagn::Report::new(),
					None,
					span.span.file_handle)
				.unwrap();

            let counter = util::CharCounter::new(&chars);

            if let Some(offset) = span.offset
            {
                result.push_str(&format!("{:x}:{:x} | ", offset / 8, offset % 8));
            }
            else
            {
                result.push_str(&format!("-:- | "));
            }

            result.push_str(&format!("{:x} | ", span.addr));

            if let Some((start, end)) = span.span.location()
            {
                let (line_start, col_start) = counter.get_line_column_at_index(start);
                let (line_end, col_end) = counter.get_line_column_at_index(end);
                let filename = fileserver.get_filename(span.span.file_handle);

                result.push_str(
                    &format!("{}:{}:{}:{}:{}",
                        filename,
                        line_start, col_start,
                        line_end, col_end));
            }
            else
            {
                result.push_str(&format!("{}:-:-:-:-", &span.span.file_handle));
            };

            result.push_str("\n");
        }

        result
    }
}
