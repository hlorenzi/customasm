use crate::*;


pub const RULEDEF_MAP_PREFIX_SIZE: usize = 4;


pub type RuledefMapPrefix = [char; RULEDEF_MAP_PREFIX_SIZE];


#[derive(Debug)]
pub struct RuledefMap
{
    /// Instructions that start with a mnemonic
    prefixed: std::collections::HashMap<RuledefMapPrefix, Vec<RuledefMapEntry>>,
    
    /// All other instructions, e.g. the ones that start
    /// with a parameter
    unprefixed: Vec<RuledefMapEntry>,
}


#[derive(Copy, Clone, Debug)]
pub struct RuledefMapEntry
{
    pub ruledef_ref: util::ItemRef<asm::Ruledef>,
    pub rule_ref: util::ItemRef<asm::Rule>,
}


impl RuledefMap
{
    pub fn new() -> RuledefMap
    {
        RuledefMap {
            prefixed: std::collections::HashMap::new(),
            unprefixed: Vec::new(),
        }
    }


    pub fn build(
        &mut self,
        ruledefs: &asm::defs::DefList<asm::Ruledef>)
    {
        for i in 0..ruledefs.defs.len()
        {
            let ruledef_ref = util::ItemRef::<asm::Ruledef>::new(i);
            let ruledef = ruledefs.get(ruledef_ref);

            if ruledef.is_subruledef
            {
                continue;
            }
    
            for rule_ref in ruledef.iter_rule_refs()
            {
                let rule = &ruledef.get_rule(rule_ref);

                self.insert(
                    ruledef_ref,
                    rule_ref,
                    rule);
            }
        }
    }


    fn insert(
        &mut self,
        ruledef_ref: util::ItemRef<asm::Ruledef>,
        rule_ref: util::ItemRef<asm::Rule>,
        rule: &asm::Rule)
    {
        let mut prefix: RuledefMapPrefix = ['\0'; RULEDEF_MAP_PREFIX_SIZE];
        let mut prefix_index = 0;

        for part in &rule.pattern
        {
            if prefix_index >= RULEDEF_MAP_PREFIX_SIZE
            {
                break;
            }
            else if let asm::RulePatternPart::Exact(c) = part
            {
                prefix[prefix_index] = c.to_ascii_lowercase();
                prefix_index += 1;
            }
            else
            {
                break;
            }
        }

        let entry = RuledefMapEntry {
            ruledef_ref,
            rule_ref,
        };

        if prefix_index > 0
        {
            self.prefixed
                .entry(prefix)
                .or_insert_with(|| Vec::new())
                .push(entry);
        }
        else
        {
            self.unprefixed.push(entry);
        }
    }


    pub fn parse_prefix(
        walker: &syntax::TokenWalker)
        -> RuledefMapPrefix
    {
        let mut prefix: RuledefMapPrefix = ['\0'; RULEDEF_MAP_PREFIX_SIZE];
        let mut prefix_index = 0;

        let mut walker_index = 0;

        while prefix_index < RULEDEF_MAP_PREFIX_SIZE
        {
            let token = walker.next_nth(walker_index);
            walker_index += 1;

            if token.kind.is_allowed_pattern_token()
            {
                for c in token.text().chars()
                {
                    if prefix_index >= RULEDEF_MAP_PREFIX_SIZE
                    {
                        break;
                    }

                    prefix[prefix_index] = c.to_ascii_lowercase();
                    prefix_index += 1;
                }
            }
            else
            {
                break;
            }
        }

        prefix
    }


    pub fn query_prefixed(
        &self,
        prefix: RuledefMapPrefix)
        -> &[RuledefMapEntry]
    {
        match self.prefixed.get(&prefix)
        {
            Some(entries) => entries,
            None => &[],
        }
    }


    pub fn query_unprefixed(
        &self)
        -> &[RuledefMapEntry]
    {
        &self.unprefixed
    }
}