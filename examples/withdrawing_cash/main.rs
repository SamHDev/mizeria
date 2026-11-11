//! Example: bank withdrawals expressed as a Gherkin feature.

use std::path::PathBuf;
use std::str::FromStr;

use mizeria::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Balance(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Withdrawal {
    Successful,
    Declined,
}

/// Amount in cents, parsed from a decimal capture such as `234.56`.
struct Currency(u64);

impl FromStr for Currency {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (upper, lower) = s
            .split_once('.')
            .map(|(u, l)| (u, Some(l)))
            .unwrap_or((s, None));
        let mut val = upper.parse::<u64>()? * 100;
        if let Some(lower) = lower {
            let lower_val = lower.parse::<u64>()?.min(100);
            val += lower_val;
        }
        Ok(Self(val))
    }
}

fn has_balance(person: String, Param(Currency(amount)): Param<Currency>) -> InsertNamed<Balance> {
    InsertNamed::named(person, Balance(amount))
}

fn try_withdraw(
    mut balance: NamedMut<Balance>,
    Param(Currency(amount)): Param<Currency>,
) -> Result<Put<Withdrawal>, StepError> {
    match balance.0.checked_sub(amount) {
        Some(next) => {
            balance.0 = next;
            Ok(Put::new(Withdrawal::Successful))
        }
        None => Ok(Put::new(Withdrawal::Declined)),
    }
}

fn withdrawal_successful(w: GetRef<Withdrawal>) -> Result<(), StepError> {
    match *w {
        Withdrawal::Successful => Ok(()),
        Withdrawal::Declined => fail!("expected successful withdrawal"),
    }
}

fn withdrawal_declined(w: GetRef<Withdrawal>) -> Result<(), StepError> {
    match *w {
        Withdrawal::Declined => Ok(()),
        Withdrawal::Successful => fail!("expected declined withdrawal"),
    }
}

fn main() {
    let features_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/withdrawing_cash/features");

    let features = Features::builder()
        .add_path(features_dir)
        .build()
        .expect("load features");

    let mut suite = Suite::new(|| ());
    suite
        .given(re(r"(\w+) has (\d+\.\d{2}) in their account"), has_balance)
        .when(re(r"(\w+) tries to withdraw (\d+\.\d{2})"), try_withdraw)
        .then("the withdrawal is successful", withdrawal_successful)
        .then("the withdrawal is declined", withdrawal_declined)
        .with_reporter(Printer::stdout().symbols(true));

    let outcome = suite.run_blocking(&features);
    if !outcome.is_ok() {
        std::process::exit(1);
    }
}
