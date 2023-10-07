use crate::*;


const MAX_PREFIX_SIZE: usize = 4;


pub type RuledefMapPrefix = [char; MAX_PREFIX_SIZE];


#[derive(Debug)]
pub struct RuledefMap
{
    /// Each rule is considered to have a prefix
    /// consisting of the first few "exact" pattern-part characters,
    /// stopping before parameter and subrule slots, and also
    /// cut off by the constant maximum prefix size declared above.
    prefixes_to_rules: std::collections::HashMap<RuledefMapPrefix, Vec<RuledefMapEntry>>,
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
            prefixes_to_rules: std::collections::HashMap::new(),
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
        let mut prefix: RuledefMapPrefix = ['\0'; MAX_PREFIX_SIZE];
        let mut prefix_index = 0;

        for part in &rule.pattern
        {
            if prefix_index >= MAX_PREFIX_SIZE
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

        self.prefixes_to_rules
            .entry(prefix)
            .or_insert_with(|| Vec::new())
            .push(entry);
    }


    pub fn parse_prefix(
        walker: &syntax::TokenWalker)
        -> RuledefMapPrefix
    {
        let mut prefix: RuledefMapPrefix = ['\0'; MAX_PREFIX_SIZE];
        let mut prefix_index = 0;

        let mut walker_index = 0;

        while prefix_index < MAX_PREFIX_SIZE
        {
            let token = walker.next_nth(walker_index);
            walker_index += 1;

            if token.kind.is_allowed_pattern_token()
            {
                for c in token.text().chars()
                {
                    if prefix_index >= MAX_PREFIX_SIZE
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
        -> [&[RuledefMapEntry]; MAX_PREFIX_SIZE + 1]
    {
        // Try querying for every possible prefix,
        // including the empty prefix,
        // i.e. "abcde" will query for "", "a", "ab", "abc", and "abcd".
        let mut results: [&[RuledefMapEntry]; MAX_PREFIX_SIZE + 1] =
            [&[]; MAX_PREFIX_SIZE + 1];

        for i in 0..(MAX_PREFIX_SIZE + 1)
        {
            let mut subprefix = prefix;
            for j in i..MAX_PREFIX_SIZE
            {
                subprefix[j] = '\0';
            }

            if let Some(entries) = self.prefixes_to_rules.get(&subprefix)
            {
                results[i] = entries;
            }

            if i < MAX_PREFIX_SIZE &&
                prefix[i] == '\0'
            {
                break;
            }
        }

        results
    }
}