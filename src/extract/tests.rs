use std::collections::HashMap;

use crate::extract::{
    Ctx, DataTable, DocString, FromStep, Param, StepArgument, StepContext,
};
use crate::handler::call;
use crate::response::{ParamRejection, StepError, StepOutcome};
use crate::statement::{EmptyCaptures, SliceCaptures};

#[derive(Default)]
struct World {
    balances: HashMap<String, u64>,
}

struct Currency(u64);

impl<'s, C> FromStep<'s, C> for Currency {
    type Rejection = ParamRejection;

    fn from_step(ctx: &mut StepContext<'s, C>) -> Result<Self, Self::Rejection> {
        let raw = ctx.next_capture().ok_or(ParamRejection::Missing)?;
        let amount = super::parse_from_str::<f64>(raw)?;
        Ok(Currency((amount * 100.0).round() as u64))
    }
}

fn withdraw(
    mut ctx: Ctx<'_, World>,
    person: &str,
    amount: Currency,
) -> Result<(), StepError> {
    let balance = ctx.balances.entry(person.to_owned()).or_default();
    match balance.checked_sub(amount.0) {
        Some(next) => {
            *balance = next;
            Ok(())
        }
        None => Err(StepError::new("insufficient funds")),
    }
}

#[tokio::test]
async fn withdraw_with_borrowed_extractors() {
    let mut world = World {
        balances: HashMap::from([("Alice".into(), 500)]),
    };
    let captures = ["Alice", "1.25"];
    let ctx = StepContext::new(&mut world, SliceCaptures::new(&captures), None);

    let outcome = call(withdraw, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
    assert_eq!(world.balances.get("Alice"), Some(&375));
}

#[tokio::test]
async fn ctx_taken_twice_rejects() {
    fn twice(_a: Ctx<'_, World>, _b: Ctx<'_, World>) {}

    let mut world = World::default();
    let ctx = StepContext::new(&mut world, EmptyCaptures, None);
    let outcome = call(twice, ctx).await;
    assert!(matches!(outcome, StepOutcome::Failed(_)));
}

#[tokio::test]
async fn missing_capture_rejects() {
    fn needs_param(_: u32) {}

    let mut world = World::default();
    let ctx = StepContext::new(&mut world, EmptyCaptures, None);
    let outcome = call(needs_param, ctx).await;
    assert!(matches!(outcome, StepOutcome::Failed(_)));
}

#[tokio::test]
async fn result_param_absorbs_parse_errors() {
    fn maybe(value: Result<u32, ParamRejection>) -> Result<(), StepError> {
        match value {
            Ok(n) => {
                assert_eq!(n, 0); // unreachable in this test
                Ok(())
            }
            Err(ParamRejection::Parse { .. }) => Ok(()),
            Err(other) => Err(StepError::new(other.to_string())),
        }
    }

    let mut world = World::default();
    let captures = ["abc"];
    let ctx = StepContext::new(&mut world, SliceCaptures::new(&captures), None);
    let outcome = call(maybe, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
}

#[tokio::test]
async fn docstring_take_once() {
    fn read_doc(doc: DocString<'_>) -> Result<(), StepError> {
        assert_eq!(doc.0, "hello");
        Ok(())
    }

    let mut world = World::default();
    let ctx = StepContext::new(
        &mut world,
        EmptyCaptures,
        Some(StepArgument::DocString("hello")),
    );
    assert!(matches!(call(read_doc, ctx).await, StepOutcome::Ok));

    fn read_two(_a: DocString<'_>, _b: DocString<'_>) {}
    let mut world = World::default();
    let ctx = StepContext::new(
        &mut world,
        EmptyCaptures,
        Some(StepArgument::DocString("hello")),
    );
    assert!(matches!(call(read_two, ctx).await, StepOutcome::Failed(_)));
}

#[tokio::test]
async fn data_table_extractor() {
    fn read_table(table: DataTable<'_>) {
        assert_eq!(table.0.len(), 2);
        assert_eq!(table.0[0], ["a", "b"]);
    }

    let row0: &[&str] = &["a", "b"];
    let row1: &[&str] = &["c", "d"];
    let rows: &[&[&str]] = &[row0, row1];

    let mut world = World::default();
    let ctx = StepContext::new(&mut world, EmptyCaptures, Some(StepArgument::DataTable(rows)));
    assert!(matches!(call(read_table, ctx).await, StepOutcome::Ok));
}

#[tokio::test]
async fn bare_std_from_str_params() {
    fn add_ok(a: u32, b: u32) {
        assert_eq!(a + b, 5);
    }

    let mut world = World::default();
    let captures = ["2", "3"];
    let ctx = StepContext::new(&mut world, SliceCaptures::new(&captures), None);
    let outcome = call(add_ok, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
}

#[tokio::test]
async fn async_handler_body() {
    async fn greets(name: String) {
        assert_eq!(name, "bob");
    }

    let mut world = World::default();
    let captures = ["bob"];
    let ctx = StepContext::new(&mut world, SliceCaptures::new(&captures), None);
    let outcome = call(greets, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
}

#[tokio::test]
async fn owned_handler_trait_object_style() {
    use crate::handler::Handler;

    fn add_ok(a: u32, b: u32) {
        assert_eq!(a + b, 5);
    }

    let mut world = World::default();
    let captures = ["2", "3"];
    let ctx = StepContext::new(&mut world, SliceCaptures::new(&captures), None);
    let outcome = <fn(u32, u32) as Handler<(u32, u32), World>>::call(add_ok, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
}

#[test]
fn currency_float_to_cents() {
    let mut world = World::default();
    let captures = ["12.34"];
    let mut ctx = StepContext::new(&mut world, SliceCaptures::new(&captures), None);
    let Currency(cents) = Currency::from_step(&mut ctx).unwrap();
    assert_eq!(cents, 1234);
}

#[derive(Debug, PartialEq, Eq)]
struct Color(String);

impl std::str::FromStr for Color {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Color(s.to_owned()))
    }
}

#[tokio::test]
async fn custom_from_str_via_param() {
    fn paint(Param(color): Param<Color>) {
        assert_eq!(color.0, "red");
    }

    let mut world = World::default();
    let captures = ["red"];
    let ctx = StepContext::new(&mut world, SliceCaptures::new(&captures), None);
    assert!(matches!(call(paint, ctx).await, StepOutcome::Ok));
}

#[tokio::test]
async fn data_table_typed_rows() {
    use crate::response::RowRejection;

    fn ingest(table: DataTable<'_>) -> Result<(), StepError> {
        let rows: Result<Vec<_>, _> = table.rows::<(&str, u32, String)>().collect();
        let rows = rows?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], ("alice", 3, "note".to_owned()));
        assert_eq!(rows[1], ("bob", 1, "x".to_owned()));
        Ok(())
    }

    let row0: &[&str] = &["alice", "3", "note"];
    let row1: &[&str] = &["bob", "1", "x"];
    let rows: &[&[&str]] = &[row0, row1];

    let mut world = World::default();
    let ctx = StepContext::new(&mut world, EmptyCaptures, Some(StepArgument::DataTable(rows)));
    assert!(matches!(call(ingest, ctx).await, StepOutcome::Ok));

    let bad: &[&str] = &["only"];
    let bad_rows: &[&[&str]] = &[bad];
    let table = DataTable(bad_rows);
    let err = table.rows::<(&str, u32)>().next().unwrap().unwrap_err();
    assert!(matches!(
        err,
        RowRejection::Arity {
            expected: 2,
            got: 1
        }
    ));

    let bad_parse: &[&str] = &["alice", "nope"];
    let bad_parse_rows: &[&[&str]] = &[bad_parse];
    let table = DataTable(bad_parse_rows);
    let err = table.rows::<(&str, u32)>().next().unwrap().unwrap_err();
    assert!(matches!(err, RowRejection::Cell { column: 1, .. }));
}
