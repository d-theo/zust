use crate::dir::Weights;

pub const DEFAULT_TUI_CHOICES: usize = 10;
pub const MAX_TUI_CHOICES: usize = 99;

/// User settings, read from environment variables.
#[derive(Debug, PartialEq)]
pub struct Config {
    /// Number of choices shown by the TUI (`ZUST_TUI_CHOICES`, 1..=99).
    pub tui_choices: usize,
    /// Frecency weights (`ZUST_WEIGHTS="hour,day,week,older"`).
    pub weights: Weights,
}

impl Config {
    /// Invalid values are reported on stderr and replaced by the defaults.
    pub fn from_env() -> Config {
        let (config, errors) = Config::parse(|name| std::env::var(name).ok());
        for error in errors {
            eprintln!("zust: {error}");
        }
        config
    }

    fn parse(var: impl Fn(&str) -> Option<String>) -> (Config, Vec<String>) {
        let mut errors = Vec::new();
        let tui_choices = match var("ZUST_TUI_CHOICES") {
            None => DEFAULT_TUI_CHOICES,
            Some(value) => parse_choices(&value).unwrap_or_else(|| {
                errors.push(format!(
                    "ZUST_TUI_CHOICES={value:?} ignored: expected an integer from 1 to {MAX_TUI_CHOICES}"
                ));
                DEFAULT_TUI_CHOICES
            }),
        };
        let weights = match var("ZUST_WEIGHTS") {
            None => Weights::default(),
            Some(value) => parse_weights(&value).unwrap_or_else(|| {
                errors.push(format!(
                    "ZUST_WEIGHTS={value:?} ignored: expected 4 non-negative numbers \"hour,day,week,older\""
                ));
                Weights::default()
            }),
        };
        (Config { tui_choices, weights }, errors)
    }
}

fn parse_choices(value: &str) -> Option<usize> {
    value.trim().parse().ok().filter(|n| (1..=MAX_TUI_CHOICES).contains(n))
}

fn parse_weights(value: &str) -> Option<Weights> {
    let numbers: Vec<f64> = value
        .split(',')
        .map(|w| w.trim().parse::<f64>().ok().filter(|w| w.is_finite() && *w >= 0.0))
        .collect::<Option<_>>()?;
    let [hour, day, week, older] = numbers[..] else { return None };
    Some(Weights { hour, day, week, older })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(vars: &[(&str, &str)]) -> (Config, Vec<String>) {
        Config::parse(|name| vars.iter().find(|(n, _)| *n == name).map(|(_, v)| v.to_string()))
    }

    #[test]
    fn defaults_when_unset() {
        let (config, errors) = parse(&[]);
        assert_eq!(config, Config { tui_choices: 10, weights: Weights::default() });
        assert!(errors.is_empty());
    }

    #[test]
    fn reads_tui_choices_within_range() {
        assert_eq!(parse(&[("ZUST_TUI_CHOICES", "1")]).0.tui_choices, 1);
        assert_eq!(parse(&[("ZUST_TUI_CHOICES", " 99 ")]).0.tui_choices, 99);
    }

    #[test]
    fn invalid_tui_choices_falls_back_with_error() {
        for value in ["0", "100", "-3", "ten", ""] {
            let (config, errors) = parse(&[("ZUST_TUI_CHOICES", value)]);
            assert_eq!(config.tui_choices, DEFAULT_TUI_CHOICES, "{value:?}");
            assert_eq!(errors.len(), 1, "{value:?}");
        }
    }

    #[test]
    fn reads_weights() {
        let (config, errors) = parse(&[("ZUST_WEIGHTS", "8, 3,1,0")]);
        assert_eq!(config.weights, Weights { hour: 8.0, day: 3.0, week: 1.0, older: 0.0 });
        assert!(errors.is_empty());
    }

    #[test]
    fn invalid_weights_fall_back_with_error() {
        for value in ["4,2,0.5", "4,2,0.5,0.25,1", "4,2,x,1", "4,-1,1,1", "inf,1,1,1", ""] {
            let (config, errors) = parse(&[("ZUST_WEIGHTS", value)]);
            assert_eq!(config.weights, Weights::default(), "{value:?}");
            assert_eq!(errors.len(), 1, "{value:?}");
        }
    }
}
