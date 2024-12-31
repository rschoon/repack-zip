use clap::builder::PossibleValue;


#[derive(Clone, Debug)]
pub struct ProcessParams {
    pub dry_run: bool,
    pub compress_threshold: u64,
    pub sort: Option<Sort>,
}

#[derive(Copy, Clone, Debug)]
pub enum Sort {
    Normal,
    IgnoreCase
}

impl clap::ValueEnum for Sort {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Normal, Self::IgnoreCase]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Normal => PossibleValue::new("normal"),
            Self::IgnoreCase => PossibleValue::new("ignore-case"),
        })
    }
}

