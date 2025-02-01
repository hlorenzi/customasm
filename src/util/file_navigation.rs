use crate::*;


pub const STD_PATH_PREFIX: &str = "<std>/";


pub fn is_std_path(
	path: &str)
	-> bool
{
	path.starts_with(STD_PATH_PREFIX)
}


pub fn filename_validate_relative(
	report: &mut diagn::Report,
	span: diagn::Span,
	filename: &str)
	-> Result<(), ()>
{
	let pathbuf = std::path::PathBuf::from(filename);
	
	pathbuf.components().fold(Ok(()), |r, c|
	{
		if r.is_err()
		{
			return r;
		}

		match c
		{
			std::path::Component::Prefix(_) =>
			{
				report.error_span(
					"invalid filename",
					span);
				
				r.and(Err(()))
			}
			
			_ => r.and(Ok(()))
		}
	})
}


pub fn filename_navigate(
	report: &mut diagn::Report,
	span: diagn::Span,
	current: &str,
	relative: &str)
	-> Result<String, ()>
{
	if is_std_path(relative)
	{
		return Ok(relative.to_string());
	}

	let current = current.replace("\\", "/");
	let nav = relative.replace("\\", "/");

	filename_validate_relative(
		report,
		span,
		&nav)?;

	// Collect current path components, but remove the last one (the filename)
	let mut path_components = Vec::new();
	for split in current.split("/")
	{
		path_components.push(split);
	}

	path_components.remove(path_components.len() - 1);

	// Remove all current components if new path is absolute
	if nav.starts_with("/")
	{
		path_components.clear();
	}

	// Add the new path components, collapsing `.` and empty components
	let relative_components: Vec<&str> = nav
		.split("/")
		.filter(|s| s.len() > 0 && s != &".")
		.collect();

	if relative_components.len() == 0
	{
		report.error_span(
			"invalid filename",
			span);
		
		return Err(());
	}
		
	for split in relative_components
	{
		path_components.push(split);
	}

	// Collapse `..` components
	let mut new_path_components = Vec::new();
	for i in 0..path_components.len()
	{
		if path_components[i] == ".."
		{
			if new_path_components.len() == 0
			{
				report.error_span("cannot navigate out of project directory", span);
				return Err(());
			}

			new_path_components.remove(new_path_components.len() - 1);
			continue;
		}

		new_path_components.push(path_components[i]);
	}

	// Collect components into a `/`-separated string
	let mut new_filename = String::new();
	for i in 0..new_path_components.len()
	{
		new_filename.push_str(new_path_components[i]);

		if i + 1 < new_path_components.len()
		{
			new_filename.push_str("/");
		}
	}

	if new_path_components.len() == 0 ||
		new_filename == "" ||
		new_filename == "." ||
		new_filename == "/"
	{
		report.error_span(
			"invalid filename",
			span);
		
		return Err(());
	}

	Ok(new_filename)
}