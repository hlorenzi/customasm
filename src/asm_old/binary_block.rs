use crate::asm::BitRangeSpan;
use crate::diagn::Span;
use crate::diagn::RcReport;
use crate::util::FileServer;
use crate::util::CharCounter;


pub struct BinaryBlock
{
	pub bits: Vec<bool>,
	pub spans: Vec<BitRangeSpan>
}


impl BinaryBlock
{
	pub fn new() -> BinaryBlock
	{
		BinaryBlock
		{
			bits: Vec::new(),
			spans: Vec::new()
		}
	}
	
	
	pub fn append(&mut self, bit: bool, maybe_range: Option<(usize, &Span)>)
	{
		let index = self.len();
		self.write(index, bit, maybe_range);
	}
	
	
	pub fn write(&mut self, index: usize, bit: bool, maybe_range: Option<(usize, &Span)>)
	{
		while self.bits.len() <= index
			{ self.bits.push(false); }
			
		self.bits[index] = bit;
		
		if let Some(range) = maybe_range
		{
			let spans_len = self.spans.len();
			
			let addr = range.0;
			let span = range.1;
			
			if self.spans.len() > 0 && *span == self.spans[spans_len - 1].span && index == self.spans[spans_len - 1].end
				{ self.spans[spans_len - 1].end = index + 1; }
			else
			{
				self.spans.push(BitRangeSpan
				{
					start: index,
					end: index + 1,
					addr,
					span: span.clone()
				})
			}
		}
	}
	
	
	pub fn read(&self, bit_index: usize) -> bool
	{
		if bit_index >= self.bits.len()
			{ false }
		else
			{ self.bits[bit_index] }
	}
	
	
	pub fn truncate(&mut self, new_len: usize)
	{
		while self.bits.len() > new_len
			{ self.bits.pop(); }
	}
	
	
	pub fn len(&self) -> usize
	{
		self.bits.len()
	}
	
	
	pub fn mark_label(&mut self, index: usize, addr: usize, span: &Span)
	{
		self.spans.push(BitRangeSpan
		{
			start: index,
			end: index,
			addr,
			span: span.clone()
		})
	}
	
	
	pub fn generate_binstr(&self, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_str(1, start_bit, end_bit)
	}
	
	
	pub fn generate_hexstr(&self, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_str(4, start_bit, end_bit)
	}
	
	
	pub fn generate_str(&self, digit_bits: usize, start_bit: usize, end_bit: usize) -> String
	{
		let mut result = String::new();
		
		let mut index = start_bit;
		while index < end_bit
		{
			let mut digit: u8 = 0;
			for _ in 0..digit_bits
			{
				digit <<= 1;
				digit |= if self.read(index) { 1 } else { 0 };
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
	
	
	pub fn generate_bindump(&self, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_dump(1, start_bit, end_bit, 8, 8)
	}
	
	
	pub fn generate_hexdump(&self, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_dump(4, start_bit, end_bit, 8, 16)
	}
	
	
	pub fn generate_dump(&self, digit_bits: usize, start_bit: usize, end_bit: usize, byte_bits: usize, bytes_per_line: usize) -> String
	{
		let mut result = String::new();
		
		let line_start = start_bit / (byte_bits * bytes_per_line);
		let line_end = (end_bit + (bytes_per_line - 1) * byte_bits) / (byte_bits * bytes_per_line);
		
		let line_end = if end_bit - start_bit < byte_bits { line_start + 1 } else { line_end };
		
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
						digit |= if self.read(digit_first_bit + bit_index) { 1 } else { 0 };
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
						byte |= if self.read(byte_first_bit + bit_index) { 1 } else { 0 };
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
	
	
	pub fn generate_binary(&self, start_bit: usize, end_bit: usize) -> Vec<u8>
	{
		let mut result = Vec::new();
		
		let mut index = start_bit;
		while index < end_bit
		{
			let mut byte: u8 = 0;
			for _ in 0..8
			{
				byte <<= 1;
				byte |= if self.read(index) { 1 } else { 0 };
				index += 1;
			}
			
			result.push(byte);
		}
		
		result
	}
	
	
	pub fn generate_mif(&self, start_bit: usize, end_bit: usize) -> String
	{
		let mut result = String::new();
		
		let byte_num = (end_bit - start_bit) / 8 + if (end_bit - start_bit) % 8 != 0 { 1 } else { 0 };
		
		result.push_str(&format!("DEPTH = {};\n", byte_num));
		result.push_str("WIDTH = 8;\n");
		result.push_str("ADDRESS_RADIX = HEX;\n");
		result.push_str("DATA_RADIX = HEX;\n");
		result.push_str("\n");
		result.push_str("CONTENT\n");
		result.push_str("BEGIN\n");
		
		let addr_max_width = format!("{:x}", byte_num - 1).len();
		
		let mut index = start_bit;
		while index < end_bit
		{
			result.push_str(&format!(" {:1$X}: ", index / 8, addr_max_width));
			
			let mut byte: u8 = 0;
			for _ in 0..8
			{
				byte <<= 1;
				byte |= if self.read(index) { 1 } else { 0 };
				index += 1;
			}
			
			result.push_str(&format!("{:02X};\n", byte));
		}
		
		result.push_str("END;");
		result
	}
	
	
	pub fn generate_intelhex(&self, start_bit: usize, end_bit: usize) -> String
	{
		let mut result = String::new();
		
		let mut bytes_left = (end_bit - start_bit) / 8 + if (end_bit - start_bit) % 8 != 0 { 1 } else { 0 };
		
		let mut index = start_bit;
		while index < end_bit
		{
			let bytes_in_row = if bytes_left > 32 { 32 } else { bytes_left };
			
			result.push(':');
			result.push_str(&format!("{:02X}", bytes_in_row));
			result.push_str(&format!("{:04X}", index / 8));
			result.push_str("00");
			
			let mut checksum = 0_u8;
			checksum = checksum.wrapping_add(bytes_in_row as u8);
			checksum = checksum.wrapping_add(((index / 8) >> 8) as u8);
			checksum = checksum.wrapping_add((index / 8) as u8);
			
			for _ in 0..bytes_in_row
			{
				let mut byte: u8 = 0;
				for _ in 0..8
				{
					byte <<= 1;
					byte |= if self.read(index) { 1 } else { 0 };
					index += 1;
				}
				
				result.push_str(&format!("{:02X}", byte));
				checksum = checksum.wrapping_add(byte);
			}
			
			bytes_left -= bytes_in_row;
			result.push_str(&format!("{:02X}", (!checksum).wrapping_add(1)));
			result.push('\n');
		}
		
		result.push_str(":00000001FF");
		result
	}
	
	
	pub fn generate_comma(&self, start_bit: usize, end_bit: usize, radix: usize) -> String
	{
		let mut result = String::new();
		
		let mut index = start_bit;
		while index < end_bit
		{
			let mut byte: u8 = 0;
			for _ in 0..8
			{
				byte <<= 1;
				byte |= if self.read(index) { 1 } else { 0 };
				index += 1;
			}
			
			match radix
			{
				10 => result.push_str(&format!("{}", byte)),
				16 => result.push_str(&format!("0x{:02x}", byte)),
				_  => panic!("invalid radix")
			}
			
			if index < end_bit
			{ 
				result.push_str(", ");
				
				if (index / 8) % 16 == 0
					{ result.push('\n'); }
			}
		}
		
		result
	}
	
	
	pub fn generate_c_array(&self, start_bit: usize, end_bit: usize, radix: usize) -> String
	{
		let mut result = String::new();
		
		result.push_str("const unsigned char data[] = {\n");
		
		let byte_num = (end_bit - start_bit) / 8 + if (end_bit - start_bit) % 8 != 0 { 1 } else { 0 };
		let addr_max_width = format!("{:x}", byte_num - 1).len();
		
		let mut index = start_bit;
		result.push_str(&format!("\t/* 0x{:01$x} */ ", 0, addr_max_width));
		
		while index < end_bit
		{
			let mut byte: u8 = 0;
			for _ in 0..8
			{
				byte <<= 1;
				byte |= if self.read(index) { 1 } else { 0 };
				index += 1;
			}
			
			match radix
			{
				10 => result.push_str(&format!("{}", byte)),
				16 => result.push_str(&format!("0x{:02x}", byte)),
				_  => panic!("invalid radix")
			}
			
			if index < end_bit
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
	pub fn generate_logisim(&self, start_bit: usize, end_bit: usize, bits_per_chunk: usize) -> String
	{
		let mut result = String::new();
		result.push_str("v2.0 raw\n");
		
		let mut index = start_bit;
		while index < end_bit
		{
			let mut value: u16 = 0;
			for _ in 0..bits_per_chunk
			{
				value <<= 1;
				value |= if self.read(index) { 1 } else { 0 };
				index += 1;
			}
			
			result.push_str(&format!("{:01$x} ", value, bits_per_chunk / 4));
			if (index / 8) % 16 == 0
				{ result.push('\n'); }
		}
		
		result
	}
	
	
	pub fn generate_annotated_bin(&self, fileserver: &dyn FileServer, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_annotated(fileserver, 1, start_bit, end_bit, 8)
	}
	
	
	pub fn generate_annotated_hex(&self, fileserver: &dyn FileServer, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_annotated(fileserver, 4, start_bit, end_bit, 2)
	}
	
	
	pub fn generate_annotated(&self, fileserver: &dyn FileServer, digit_bits: usize, start_bit: usize, end_bit: usize, byte_digits: usize) -> String
	{
		let mut result = String::new();
		
		let byte_bits = byte_digits * digit_bits;
		
		let mut outp_width = 2;
		let outp_bit_width = format!("{:x}", digit_bits - 1).len();
		let mut addr_width = 4;
		let content_width = (byte_digits + 1) * 5 - 1;
						
		let mut sorted_spans = self.spans.clone();
		sorted_spans.sort_by(|a, b|
			if a.start == b.start
				{ (b.end - b.start).cmp(&(a.end - a.start)) }
			else
				{ a.start.cmp(&b.start) }
		);
		
		//for bitrange in &sorted_spans
		//	{ println!("{:?}", bitrange); }
		
		let mut sorted_span_index = 0;
		
		for index in start_bit..end_bit
		{
			let bitrange_index = sorted_spans[sorted_span_index..].iter().position(|s| index >= s.start && index < s.end);
			let bitrange = bitrange_index.map(|i| &sorted_spans[sorted_span_index + i]);
			
			if let Some(bitrange_index) = bitrange_index
				{ sorted_span_index += bitrange_index; }
			
			if let Some(bitrange) = bitrange
			{
				outp_width = std::cmp::max(outp_width, format!("{:x}", bitrange.start / byte_bits).len());
				addr_width = std::cmp::max(addr_width, format!("{:x}", bitrange.addr).len());
			}
		}
		
		result.push_str(&format!(" {:>1$} |", "outp", outp_width + outp_bit_width + 1));
		result.push_str(&format!(" {:>1$} | data", "addr", addr_width));
		result.push_str("\n");
		result.push_str("\n");
		
		let mut prev_bitrange = None;
		let mut accum_bits = Vec::<bool>::new();
		
		sorted_span_index = 0;
		
		let mut prev_filename = "";
		let mut prev_file_chars = Vec::new();
		
		for index in start_bit..=end_bit
		{
			let bitrange_index = sorted_spans[sorted_span_index..].iter().position(|s| index >= s.start && index < s.end);
			let bitrange = bitrange_index.map(|i| &sorted_spans[sorted_span_index + i]);
			
			if let Some(bitrange_index) = bitrange_index
			{
				for label_bitrange in &sorted_spans[sorted_span_index..(sorted_span_index + bitrange_index)]
				{
					if label_bitrange.start != label_bitrange.end
						{ continue; }
						
					if &*label_bitrange.span.file != prev_filename
					{
						prev_filename = &*label_bitrange.span.file;
						prev_file_chars = fileserver.get_chars(RcReport::new(), &prev_filename, None).ok().unwrap();
					}
					
					let span_location = label_bitrange.span.location.unwrap();
					let char_counter = CharCounter::new(&prev_file_chars);
					
					result.push_str(&format!(" {:1$x}", label_bitrange.start / byte_bits, outp_width));
					result.push_str(&format!(":{:1$x} | ", label_bitrange.start % byte_bits, outp_bit_width));
					result.push_str(&format!("{:1$x} | ", label_bitrange.addr, addr_width));
					result.push_str(&format!("{:1$}", "", content_width));
					result.push_str(&format!(" ; {}", char_counter.get_excerpt(span_location.0, span_location.1).iter().collect::<String>()));
					result.push_str("\n");
				}
				
				sorted_span_index += bitrange_index;
			}
				
			if prev_bitrange != bitrange
			{
				if let Some(prev_bitrange) = prev_bitrange
				{
					result.push_str(&format!(" {:1$x}", prev_bitrange.start / byte_bits, outp_width));
					result.push_str(&format!(":{:1$x} | ", prev_bitrange.start % byte_bits, outp_bit_width));
					result.push_str(&format!("{:1$x} | ", prev_bitrange.addr, addr_width));
					
					let mut contents_str = String::new();
					
					let digit_num = accum_bits.len() / digit_bits + if accum_bits.len() % digit_bits == 0 { 0 } else { 1 };
					for digit_index in 0..digit_num
					{
						if digit_index > 0 && digit_index % byte_digits == 0
							{ contents_str.push_str(" "); }
					
						let mut digit = 0;
						for bit_index in 0..digit_bits
						{
							let i = digit_index * digit_bits + bit_index;
							let bit = if i < accum_bits.len() { accum_bits[i] } else { false };
							
							digit <<= 1;
							digit |= if bit { 1 } else { 0 };
						}
						
						let c = if digit < 10
							{ ('0' as u8 + digit) as char }
						else
							{ ('a' as u8 + digit - 10) as char };
						
						contents_str.push(c);
					}
					
					if &*prev_bitrange.span.file != prev_filename
					{
						prev_filename = &*prev_bitrange.span.file;
						prev_file_chars = fileserver.get_chars(RcReport::new(), &prev_filename, None).ok().unwrap();
					}
					
					let span_location = prev_bitrange.span.location.unwrap();
					let char_counter = CharCounter::new(&prev_file_chars);
					
					result.push_str(&format!("{:1$}", contents_str, content_width));
					result.push_str(&format!(" ; {}", char_counter.get_excerpt(span_location.0, span_location.1).iter().collect::<String>()));
					result.push_str("\n");
				}
				
				prev_bitrange = bitrange;
				accum_bits.clear();
			}
			
			if bitrange.is_some()
				{ accum_bits.push(self.read(index)); }
		}
		
		result
	}
}