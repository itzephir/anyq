#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseSource {
    Override,
    Environment,
    Configuration,
    Default,
}

impl DatabaseSource {
    pub fn select(explicit: bool, environment: bool, configuration: bool) -> Self {
        if explicit {
            Self::Override
        } else if environment {
            Self::Environment
        } else if configuration {
            Self::Configuration
        } else {
            Self::Default
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Override => "override",
            Self::Environment => "environment",
            Self::Configuration => "configuration",
            Self::Default => "default",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DatabaseSource;

    #[test]
    fn all_precedence_combinations_select_the_first_available_source() {
        for (explicit, environment, configuration, expected) in [
            (false, false, false, DatabaseSource::Default),
            (false, false, true, DatabaseSource::Configuration),
            (false, true, false, DatabaseSource::Environment),
            (false, true, true, DatabaseSource::Environment),
            (true, false, false, DatabaseSource::Override),
            (true, false, true, DatabaseSource::Override),
            (true, true, false, DatabaseSource::Override),
            (true, true, true, DatabaseSource::Override),
        ] {
            assert_eq!(
                DatabaseSource::select(explicit, environment, configuration),
                expected
            );
        }
    }
}
