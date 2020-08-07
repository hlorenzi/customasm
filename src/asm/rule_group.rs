use crate::*;


pub struct RuleGroup
{
    pub rules: Vec<asm::Rule>,
}


impl RuleGroup
{
    pub fn new() -> RuleGroup
    {
        RuleGroup
        {
            rules: Vec::new(),
        }
    }
}