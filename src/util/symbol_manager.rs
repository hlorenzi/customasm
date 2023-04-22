use crate::*;


#[derive(Debug)]
pub struct SymbolManager<T>
{
    decls: Vec<SymbolDecl<T>>,
    pub(super) globals: std::collections::HashMap<String, util::ItemRef<T>>,
    span_refs: std::collections::HashMap<diagn::Span, util::ItemRef<T>>,
    report_as: &'static str,
}


#[derive(Debug)]
pub struct SymbolDecl<T>
{
    pub span: diagn::Span,
    pub name: String,
    pub kind: SymbolKind,
    pub depth: usize,
    pub ctx: SymbolContext,
    pub item_ref: util::ItemRef<T>,
    pub(super) children: std::collections::HashMap<String, util::ItemRef<T>>,
}


#[derive(Copy, Clone, Debug)]
pub enum SymbolKind
{
    Constant,
    Label,
    Function,
    Other,
}


#[derive(Clone, Debug)]
pub struct SymbolContext
{
    hierarchy: Vec<String>,
}


impl<T> SymbolManager<T>
{
    pub fn new(report_as: &'static str) -> SymbolManager<T>
    {
        SymbolManager {
            decls: Vec::new(),
            globals: std::collections::HashMap::new(),
            span_refs: std::collections::HashMap::new(),
            report_as,
        }
    }


    fn traverse<S>(
        &self,
        parent_ref: Option<util::ItemRef<T>>,
        hierarchy: &[S])
        -> Option<util::ItemRef<T>>
        where S: std::borrow::Borrow<str>
    {
        if hierarchy.len() == 0
        {
            return None;
        }

        match self.get_children(parent_ref).get(hierarchy[0].borrow())
        {
            None => None,
            Some(child_ref) =>
            {
                if hierarchy.len() == 1
                {
                    Some(*child_ref)
                }
                else
                {
                    self.traverse(
                        Some(*child_ref),
                        &hierarchy[1..])
                }
            }
        }
    }


    fn get_parent<S>(
        &self,
        parent_ref: Option<util::ItemRef<T>>,
        hierarchy: &[S])
        -> Option<util::ItemRef<T>>
        where S: std::borrow::Borrow<str>
    {
        if hierarchy.len() == 0
        {
            return parent_ref;
        }

        match self.get_children(parent_ref).get(hierarchy[0].borrow())
        {
            None => None,
            Some(child_ref) =>
            {
                self.get_parent(
                    Some(*child_ref),
                    &hierarchy[1..])
            }
        }
    }


    fn get_children(
        &self,
        parent_ref: Option<util::ItemRef<T>>)
        -> &std::collections::HashMap<String, util::ItemRef<T>>
    {
        match parent_ref
        {
            Some(parent_ref) => &self.get(parent_ref).children,
            None => &self.globals,
        }
    }


    fn get_children_mut(
        &mut self,
        parent_ref: Option<util::ItemRef<T>>)
        -> &mut std::collections::HashMap<String, util::ItemRef<T>>
    {
        match parent_ref
        {
            Some(parent_ref) => &mut self.get_mut(parent_ref).children,
            None => &mut self.globals,
        }
    }


    pub fn get(
        &self,
        item_ref: util::ItemRef<T>)
        -> &util::SymbolDecl<T>
    {
        &self.decls[item_ref.0]
    }


    pub fn get_mut(
        &mut self,
        item_ref: util::ItemRef<T>)
        -> &mut util::SymbolDecl<T>
    {
        &mut self.decls[item_ref.0]
    }


    pub fn get_by_name_global(
        &self,
        report: &mut diagn::Report,
        span: &diagn::Span,
        name: &str)
        -> Result<util::ItemRef<T>, ()>
    {
        self.get_by_name(
            report,
            span,
            &SymbolContext::new_global(),
            0,
            &[name])
    }


    pub fn get_by_name<S>(
        &self,
        report: &mut diagn::Report,
        span: &diagn::Span,
        ctx: &SymbolContext,
        hierarchy_level: usize,
        hierarchy: &[S])
        -> Result<util::ItemRef<T>, ()>
        where S: std::borrow::Borrow<str> + std::fmt::Debug
    {
        let maybe_symbol = {
            if hierarchy_level > ctx.hierarchy.len()
            {
                None
            }
            else
            {
                let parent = self.get_parent(
                    None,
                    &ctx.hierarchy[0..hierarchy_level]);
                
                self.traverse(
                    parent,
                    hierarchy)
            }
        };

        match maybe_symbol
        {
            Some(symbol) => Ok(symbol),
            None =>
            {
                let hierarchy_string = hierarchy
                    .iter()
                    .map(|s| s.borrow().to_string())
                    .collect::<Vec<String>>();
                
                report.error_span(
                    format!("unknown {} `{}`",
                        self.report_as,
                        self.get_displayable_name(
                            hierarchy_level,
                            &hierarchy_string)),
                    span);

                Err(())
            }
        }
    }


    pub fn get_displayable_name<S>(
        &self,
        hierarchy_level: usize,
        hierarchy: &[S])
        -> String
        where S: std::borrow::Borrow<str>
    {
        format!("{}{}",
            ".".repeat(hierarchy_level),
            hierarchy.join("."))
    }


    pub fn generate_anonymous_name(&self) -> String
    {
        format!(
            "#anonymous_{}_{}",
            self.report_as,
            self.decls.len())
    }


    pub fn declare(
        &mut self,
        report: &mut diagn::Report,
        span: &diagn::Span,
        ctx: &SymbolContext,
        name: String,
        hierarchy_level: usize,
        kind: SymbolKind)
        -> Result<util::ItemRef<T>, ()>
    {
        // Check skips in nesting level
        if hierarchy_level > ctx.hierarchy.len()
        {
            report.error_span(
                "symbol declaration skips a nesting level",
                &span);
            
            return Err(());
        }


        // Check for duplicates at the same nesting level
        let parent_ref = self.get_parent(
            None,
            &ctx.hierarchy[0..hierarchy_level]);

        let children = self.get_children(parent_ref);

        if let Some(duplicate_ref) = children.get(&name)
        {
            report.push_parent(
                format!("duplicate {} `{}`", self.report_as, name),
                span);

            report.note_span(
                "first declared here",
                &self.get(*duplicate_ref).span);

            report.pop_parent();

            return Err(());
        }


        // Generate the ItemRef
        let index = self.decls.len();
        let item_ref = util::ItemRef::<T>::new(index);


        // Insert ItemRef into the parent's children-list
        let parent_ref = self.get_parent(
            None,
            &ctx.hierarchy[0..hierarchy_level]);
        
        let children = self.get_children_mut(parent_ref);

        children.insert(
            name.clone(),
            item_ref);


        // Generate new SymbolContext
        let new_ctx = {
            let mut new_hierarchy = ctx.hierarchy[0..hierarchy_level]
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            
            new_hierarchy.push(name.clone());

            SymbolContext {
                hierarchy: new_hierarchy,
            }
        };


        // Create a new declaration and add a Span reference
        let full_name = {
            let mut s = String::new();
            if let Some(parent_ref) = parent_ref
            {
                s.push_str(&self.get(parent_ref).name);
                s.push_str(".");
            }

            s.push_str(&name);
            s
        };

        self.decls.push(SymbolDecl {
            span: span.clone(),
            name: full_name.clone(),
            kind,
            depth: hierarchy_level,
            ctx: new_ctx.clone(),
            item_ref,
            children: std::collections::HashMap::new(),
        });

        self.span_refs.insert(
            span.clone(),
            item_ref);


        Ok(item_ref)
    }


    pub fn add_span_ref(
        &mut self,
        span: diagn::Span,
        item_ref: util::ItemRef<T>)
    {
        self.span_refs.insert(span, item_ref);
    }
}


impl SymbolContext
{
    pub const fn new_global() -> SymbolContext
    {
        SymbolContext {
            hierarchy: Vec::new(),
        }
    }
}