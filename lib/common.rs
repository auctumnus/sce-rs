use strum::EnumString;

#[derive(Clone, Debug, EnumString, PartialEq)]
pub enum Wildcard {
    #[strum(serialize = "*?")]
    NonGreedy,
    #[strum(serialize = "*")]
    Greedy,
    #[strum(serialize = "**?")]
    NonGreedyExtended,
    #[strum(serialize = "**")]
    GreedyExtended,
}
