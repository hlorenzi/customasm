use crate::*;
use std::collections::HashMap;


#[derive(Debug)]
pub struct SymbolManager
{
    globals: HashMap<String, Symbol>,
    cur_ctx: SymbolContext,
}


#[derive(Debug)]
pub struct Symbol
{
    pub value: expr::Value,
    pub decl_span: diagn::Span,
    children: HashMap<String, Symbol>,
}


#[derive(Clone, Debug)]
pub struct SymbolContext
{
    hierarchy: Vec<String>,
}


impl SymbolManager
{
    pub fn new() -> SymbolManager
    {
        SymbolManager
        {
            globals: HashMap::new(),
            cur_ctx: SymbolContext
            {
                hierarchy: Vec::new(),
            },
        }
    }


    fn traverse<'from>(
        from: &'from HashMap<String, Symbol>,
        hierarchy: &[String])
        -> Option<&'from Symbol>
    {
        if hierarchy.len() == 0
        {
            return None;
        }

        match from.get(&hierarchy[0])
        {
            None => None,
            Some(child) =>
            {
                if hierarchy.len() == 1
                {
                    Some(child)
                }
                else
                {
                    SymbolManager::traverse(&child.children, &hierarchy[1..])
                }
            }
        }
    }


    fn traverse_mut<'from>(
        from: &'from mut HashMap<String, Symbol>,
        hierarchy: &[String])
        -> Option<&'from mut Symbol>
    {
        match from.get_mut(&hierarchy[0])
        {
            None => None,
            Some(child) =>
            {
                if hierarchy.len() == 1
                {
                    Some(child)
                }
                else
                {
                    SymbolManager::traverse_mut(&mut child.children, &hierarchy[1..])
                }
            }
        }
    }


    fn get_parent<'from>(
        from: &'from HashMap<String, Symbol>,
        hierarchy: &[String])
        -> Option<&'from HashMap<String, Symbol>>
    {
        if hierarchy.len() == 0
        {
            return Some(from);
        }

        match from.get(&hierarchy[0])
        {
            None => None,
            Some(child) => SymbolManager::get_parent(&child.children, &hierarchy[1..])
        }
    }


    fn get_mut_parent<'from>(
        from: &'from mut HashMap<String, Symbol>,
        hierarchy: &[String])
        -> Option<&'from mut HashMap<String, Symbol>>
    {
        if hierarchy.len() == 0
        {
            return Some(from);
        }

        match from.get_mut(&hierarchy[0])
        {
            None => None,
            Some(child) => SymbolManager::get_mut_parent(&mut child.children, &hierarchy[1..])
        }
    }


    pub fn get(&self, ctx: &SymbolContext, hierarchy_level: usize, hierarchy: &[String]) -> Option<&Symbol>
    {
        let parent = SymbolManager::get_parent(&self.globals, &ctx.hierarchy[0..hierarchy_level])?;
        SymbolManager::traverse(parent, hierarchy)
    }


    pub fn get_mut(&mut self, ctx: &SymbolContext, hierarchy_level: usize, hierarchy: &[String]) -> Option<&mut Symbol>
    {
        let parent = SymbolManager::get_mut_parent(&mut self.globals, &ctx.hierarchy[0..hierarchy_level])?;
        SymbolManager::traverse_mut(parent, hierarchy)
    }


    pub fn get_ctx(&self) -> SymbolContext
    {
        self.cur_ctx.clone()
    }


    pub fn create(
        &mut self,
        ctx: &SymbolContext,
        name: String,
        hierarchy_level: usize,
        value: expr::Value,
        report: diagn::RcReport,
        span: &diagn::Span)
        -> Result<(), ()>
    {
        if hierarchy_level > ctx.hierarchy.len()
        {
            report.error_span("symbol declaration skips a nesting level", &span);
            return Err(());
        }

        let parent = SymbolManager::get_mut_parent(
                &mut self.globals,
                &ctx.hierarchy[0..hierarchy_level])
            .unwrap();

        if let Some(duplicate) = parent.get(&name)
        {
            let _guard = report.push_parent("duplicate symbol", span);
            report.note_span("first declared here", &duplicate.decl_span);
            return Err(());
        }

        parent.insert(name.clone(), Symbol
        {
           value, 
           decl_span: span.clone(),
           children: HashMap::new(),
        });

        self.cur_ctx.hierarchy = ctx.hierarchy[0..hierarchy_level].iter().cloned().collect();
        self.cur_ctx.hierarchy.push(name);

        Ok(())
    }


    pub fn format_output(&self) -> String
	{
		let mut result = String::new();

		for (name, data) in &self.globals
		{
            self.format_output_recursive(
                &mut result,
                &mut vec![name.clone()],
                data);
		}

		result
    }


    fn format_output_recursive(
        &self,
        result: &mut String,
        hierarchy: &mut Vec<String>,
        data: &Symbol)
    {
        match &data.value
        {
            expr::Value::Integer(ref bigint) =>
            {
                for i in 0..hierarchy.len()
                {
                    if i > 0
                    {
                        result.push_str(".");
                    }

                    result.push_str(&format!("{}", hierarchy[i]));
                }

                result.push_str(&format!(" = 0x{:x}\n", bigint));
            }
            _ => {}
        }

        for (child_name, child_data) in &data.children
        {
            hierarchy.push(child_name.clone());

            self.format_output_recursive(
                result,
                hierarchy,
                &child_data);

            hierarchy.pop();
        }
    }
}