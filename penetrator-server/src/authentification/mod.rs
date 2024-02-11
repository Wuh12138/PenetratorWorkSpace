use crate::config;
#[derive(Debug)]
pub enum Error {
    RuleNotFound,
    RuleAlreadyExists,
    RuleNotMatch,
    RuleMatch,
    RuleNotExists,
    RuleExists,
    RuleNotMatched,
    RuleMatched,
    RuleNotAdded,
    RuleAdded,
    RuleNotRemoved,
    RuleRemoved,
    RuleNotUpdated,
    RuleUpdated,
    RuleNotChecked,
    RuleChecked,
}
pub fn check(rule: &config::Rule) -> Result<bool, Error> {
    // TODO:
    Ok(true)
}
