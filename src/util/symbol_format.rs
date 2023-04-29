use crate::*;


impl util::SymbolManager<asm::Symbol>
{
    pub fn format_default(
        &self,
        decls: &asm::ItemDecls,
        defs: &asm::ItemDefs)
        -> String
	{
        self.format(
            decls,
            defs,
            &mut |result, _symbol_decl, name, bigint|
            {
                result.push_str(name);
                result.push_str(&format!(" = 0x{:x}\n", bigint));
            })
    }


    pub fn format_mesen_mlb(
        &self,
        decls: &asm::ItemDecls,
        defs: &asm::ItemDefs)
        -> String
	{
        self.format(
            decls,
            defs,
            &mut |result, symbol_decl, name, bigint|
            {
                if let util::SymbolKind::Constant = symbol_decl.kind
                {
                    return;
                }

                let symbol = defs.symbols.get(symbol_decl.item_ref);
                let bankdef_ref = {
                    match symbol.bankdef_ref
                    {
                        Some(r) => r,
                        None => return,
                    }
                };

                let bankdef = defs.bankdefs.get(bankdef_ref);
                if let Some(output_offset) = bankdef.output_offset
                {
                    if let Some(addr) = bigint.maybe_into::<usize>()
                    {
                        if let Some(addr_start) = bankdef.addr_start.maybe_into::<usize>()
                        {
                            let prg_offset = addr - addr_start + output_offset / 8 - 0x10;
                            result.push_str("P:");
                            result.push_str(&format!("{:x}", prg_offset));
                            result.push_str(":");
                            result.push_str(&name.replace(".", "_"));
                            result.push_str("\n");
                        }
                    }
                }
                else
                {
                    result.push_str("R:");
                    result.push_str(&format!("{:x}", bigint));
                    result.push_str(":");
                    result.push_str(&name.replace(".", "_"));
                    result.push_str("\n");
                }
            })
    }


    pub fn format<FnFormat>(
        &self,
        decls: &asm::ItemDecls,
        defs: &asm::ItemDefs,
        formatter: &mut FnFormat)
        -> String
        where FnFormat: FnMut(
            &mut String,
            &util::SymbolDecl<asm::Symbol>,
            &str,
            &util::BigInt)
            -> ()
	{
		let mut result = String::new();

		for (name, item_ref) in &self.globals
		{
            self.format_recursive(
                decls,
                defs,
                &mut result,
                &mut vec![name.clone()],
                self.get(*item_ref),
                formatter);
		}

		result
    }


    fn format_recursive<FnFormat>(
        &self,
        decls: &asm::ItemDecls,
        defs: &asm::ItemDefs,
        result: &mut String,
        hierarchy: &mut Vec<String>,
        symbol_decl: &util::SymbolDecl<asm::Symbol>,
        formatter: &mut FnFormat)
        where FnFormat: FnMut(
            &mut String,
            &util::SymbolDecl<asm::Symbol>,
            &str,
            &util::BigInt)
            -> ()
    {
        if true//data.emit
        {
            let symbol = defs.symbols.get(symbol_decl.item_ref);
            match symbol.value
            {
                expr::Value::Integer(ref bigint) =>
                {
                    let mut name = String::new();

                    for i in 0..hierarchy.len()
                    {
                        if i > 0
                        {
                            name.push_str(".");
                        }

                        name.push_str(&format!("{}", hierarchy[i]));
                    }

                    formatter(result, symbol_decl, &name, &bigint);
                }
                _ => {}
            }
        }

        for (child_name, child_ref) in &symbol_decl.children
        {
            hierarchy.push(child_name.clone());

            self.format_recursive(
                decls,
                defs,
                result,
                hierarchy,
                &self.get(*child_ref),
                formatter);

            hierarchy.pop();
        }
    }
}