//! Example: bank withdrawals expressed as a Gherkin feature.

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use cutecumber::{Features, Param, Printer, Regex, StepError, Suite};

#[derive(Debug, Default)]
struct World {
    balances: HashMap<String, u64>,
    last_withdrawal: Option<Withdrawal>,
}

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

type SharedWorld = Arc<Mutex<World>>;

fn new_world() -> SharedWorld {
    Arc::new(Mutex::new(World::default()))
}

fn has_balance(person: String, Param(Currency(amount)): Param<Currency>, world: SharedWorld) {
    world.lock().unwrap().balances.insert(person, amount);
}

fn try_withdraw(
    person: String,
    Param(Currency(amount)): Param<Currency>,
    world: SharedWorld,
) -> Result<(), StepError> {
    let mut world = world.lock().unwrap();
    let balance = world.balances.entry(person).or_default();
    let result = match balance.checked_sub(amount) {
        Some(next) => {
            *balance = next;
            Withdrawal::Successful
        }
        None => Withdrawal::Declined,
    };
    world.last_withdrawal = Some(result);
    Ok(())
}

fn withdrawal_successful(world: SharedWorld) -> Result<(), StepError> {
    match world.lock().unwrap().last_withdrawal {
        Some(Withdrawal::Successful) => Ok(()),
        Some(Withdrawal::Declined) => Err(StepError::new("expected successful withdrawal")),
        None => Err(StepError::new("no withdrawal attempted")),
    }
}

fn withdrawal_declined(world: SharedWorld) -> Result<(), StepError> {
    match world.lock().unwrap().last_withdrawal {
        Some(Withdrawal::Declined) => Ok(()),
        Some(Withdrawal::Successful) => Err(StepError::new("expected declined withdrawal")),
        None => Err(StepError::new("no withdrawal attempted")),
    }
}

fn main() {
    let features_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/withdrawing_cash/features");

    let features = Features::new().add_path(features_dir).build();

    let mut suite = Suite::new(new_world);
    suite
        .given(
            Regex::new(r"(\w+) has (\d+\.\d{2}) in their account").unwrap(),
            has_balance,
        )
        .when(
            Regex::new(r"(\w+) tries to withdraw (\d+\.\d{2})").unwrap(),
            try_withdraw,
        )
        .then("the withdrawal is successful", withdrawal_successful)
        .then("the withdrawal is declined", withdrawal_declined)
        .bind(Printer::stdout().symbols(true));

    let outcome = suite.run_blocking(&features);
    if !outcome.is_ok() {
        std::process::exit(1);
    }
}
