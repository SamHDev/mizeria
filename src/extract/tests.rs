use std::collections::HashMap;

use crate::entity::EntityBag;
use crate::extract::{
    CtxMut, CtxRef, DataTable, DocString, FromStep, Param, StepArgument, StepContext,
};
use crate::handler::call;
use crate::lock::Locked;
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

fn withdraw(mut world: CtxMut<World>, person: String, amount: Currency) -> Result<(), StepError> {
    let balance = world.balances.entry(person).or_default();
    match balance.checked_sub(amount.0) {
        Some(next) => {
            *balance = next;
            Ok(())
        }
        None => Err(StepError::new("insufficient funds")),
    }
}

#[tokio::test]
async fn withdraw_with_world_mut() {
    let world = Locked::new(World {
        balances: HashMap::from([("Alice".into(), 500)]),
    });
    let mut entities = EntityBag::new();
    let captures = ["Alice", "1.25"];
    let ctx = StepContext::new(
        world.clone(),
        &mut entities,
        SliceCaptures::new(&captures),
        None,
    );

    let outcome = call(withdraw, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
    assert_eq!(world.try_read().unwrap().balances.get("Alice"), Some(&375));
}

#[tokio::test]
async fn ctx_mut_twice_rejects() {
    fn twice(_a: CtxMut<World>, _b: CtxMut<World>) {}

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let ctx = StepContext::new(world, &mut entities, EmptyCaptures, None);
    let outcome = call(twice, ctx).await;
    assert!(matches!(outcome, StepOutcome::Failed(_)));
}

#[tokio::test]
async fn two_ctx_ref_ok() {
    fn twice(_a: CtxRef<World>, _b: CtxRef<World>) {}

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let ctx = StepContext::new(world, &mut entities, EmptyCaptures, None);
    let outcome = call(twice, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
}

#[tokio::test]
async fn ctx_ref_and_mut_rejects() {
    fn both(_a: CtxRef<World>, _b: CtxMut<World>) {}

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let ctx = StepContext::new(world, &mut entities, EmptyCaptures, None);
    let outcome = call(both, ctx).await;
    assert!(matches!(outcome, StepOutcome::Failed(_)));
}

#[tokio::test]
async fn missing_capture_rejects() {
    fn needs_param(_: u32) {}

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let ctx = StepContext::new(world, &mut entities, EmptyCaptures, None);
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

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["abc"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    let outcome = call(maybe, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
}

#[tokio::test]
async fn docstring_take_once() {
    fn read_doc(doc: DocString) -> Result<(), StepError> {
        assert_eq!(doc.0, "hello");
        Ok(())
    }

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let ctx = StepContext::new(
        world.clone(),
        &mut entities,
        EmptyCaptures,
        Some(StepArgument::DocString("hello")),
    );
    assert!(matches!(call(read_doc, ctx).await, StepOutcome::Ok));

    fn read_two(_a: DocString, _b: DocString) {}
    let mut entities = EntityBag::new();
    let ctx = StepContext::new(
        world,
        &mut entities,
        EmptyCaptures,
        Some(StepArgument::DocString("hello")),
    );
    assert!(matches!(call(read_two, ctx).await, StepOutcome::Failed(_)));
}

#[tokio::test]
async fn data_table_extractor() {
    fn read_table(table: DataTable) {
        assert_eq!(table.len(), 2);
        assert_eq!(table.0[0], ["a", "b"]);
    }

    let rows = vec![
        vec!["a".to_owned(), "b".to_owned()],
        vec!["c".to_owned(), "d".to_owned()],
    ];

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let ctx = StepContext::new(
        world,
        &mut entities,
        EmptyCaptures,
        Some(StepArgument::DataTable(rows.as_slice())),
    );
    assert!(matches!(call(read_table, ctx).await, StepOutcome::Ok));
}

#[tokio::test]
async fn bare_std_from_str_params() {
    fn add_ok(a: u32, b: u32) {
        assert_eq!(a + b, 5);
    }

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["2", "3"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    let outcome = call(add_ok, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
}

#[tokio::test]
async fn async_handler_body() {
    async fn greets(name: String) {
        assert_eq!(name, "bob");
    }

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["bob"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    let outcome = call(greets, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
}

#[tokio::test]
async fn owned_handler_trait_object_style() {
    use crate::handler::Handler;

    fn add_ok(a: u32, b: u32) {
        assert_eq!(a + b, 5);
    }

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["2", "3"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    let outcome = <fn(u32, u32) as Handler<(u32, u32), World>>::call(add_ok, ctx).await;
    assert!(matches!(outcome, StepOutcome::Ok));
}

#[test]
fn currency_float_to_cents() {
    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["12.34"];
    let mut ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
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

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["red"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    assert!(matches!(call(paint, ctx).await, StepOutcome::Ok));
}

#[tokio::test]
async fn data_table_typed_rows() {
    use crate::response::RowRejection;

    fn ingest(table: DataTable) -> Result<(), StepError> {
        let rows: Result<Vec<_>, _> = table.rows::<(&str, u32, String)>().collect();
        let rows = rows?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], ("alice", 3, "note".to_owned()));
        assert_eq!(rows[1], ("bob", 1, "x".to_owned()));
        Ok(())
    }

    let rows = vec![
        vec!["alice".to_owned(), "3".to_owned(), "note".to_owned()],
        vec!["bob".to_owned(), "1".to_owned(), "x".to_owned()],
    ];

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let ctx = StepContext::new(
        world,
        &mut entities,
        EmptyCaptures,
        Some(StepArgument::DataTable(rows.as_slice())),
    );
    assert!(matches!(call(ingest, ctx).await, StepOutcome::Ok));

    let bad_rows = vec![vec!["only".to_owned()]];
    let table = DataTable(bad_rows);
    let err = table.rows::<(&str, u32)>().next().unwrap().unwrap_err();
    assert!(matches!(
        err,
        RowRejection::Arity {
            expected: 2,
            got: 1
        }
    ));

    let bad_parse_rows = vec![vec!["alice".to_owned(), "nope".to_owned()]];
    let table = DataTable(bad_parse_rows);
    let err = table.rows::<(&str, u32)>().next().unwrap().unwrap_err();
    assert!(matches!(err, RowRejection::Cell { column: 1, .. }));
}

#[tokio::test]
async fn data_table_records() {
    use crate::response::{RowRejection, TableRejection};

    let table = DataTable(vec![
        vec!["SKU".to_owned(), "Quantity".to_owned()],
        vec!["widget".to_owned(), "2".to_owned()],
        vec!["gizmo".to_owned(), "1".to_owned()],
    ]);

    let rows: Vec<_> = table
        .records::<(String, u32)>(&["sku", "quantity"])
        .expect("records")
        .collect::<Result<_, _>>()
        .expect("body");
    assert_eq!(
        rows,
        vec![("widget".to_owned(), 2), ("gizmo".to_owned(), 1),]
    );

    let empty_body = DataTable(vec![vec!["sku".to_owned(), "quantity".to_owned()]]);
    assert_eq!(
        empty_body
            .records::<(String, u32)>(&["sku", "quantity"])
            .unwrap()
            .len(),
        0
    );

    let empty = DataTable(vec![]);
    assert!(matches!(
        empty.records::<(String, u32)>(&["sku", "quantity"]),
        Err(TableRejection::Empty)
    ));

    let bad_header = DataTable(vec![
        vec!["name".to_owned(), "qty".to_owned()],
        vec!["widget".to_owned(), "1".to_owned()],
    ]);
    assert!(matches!(
        bad_header.records::<(String, u32)>(&["sku", "quantity"]),
        Err(TableRejection::Header { .. })
    ));

    let bad_cell = DataTable(vec![
        vec!["sku".to_owned(), "quantity".to_owned()],
        vec!["widget".to_owned(), "nope".to_owned()],
    ]);
    let err = bad_cell
        .records::<(String, u32)>(&["sku", "quantity"])
        .expect("header")
        .next()
        .unwrap()
        .unwrap_err();
    assert!(matches!(
        err,
        TableRejection::Row {
            index: 1,
            source: RowRejection::Cell { column: 1, .. }
        }
    ));
}

#[test]
fn rows_macro_named_fields() {
    let table = DataTable(vec![
        vec!["sku".to_owned(), "quantity".to_owned()],
        vec!["widget".to_owned(), "2".to_owned()],
        vec!["gizmo".to_owned(), "1".to_owned()],
    ]);

    let rows: Vec<_> = crate::rows!(for (sku: &str, quantity: u32) in &table)
        .expect("rows")
        .collect::<Result<_, _>>()
        .expect("body");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].sku, "widget");
    assert_eq!(rows[0].quantity, 2);
    assert_eq!(rows[1].sku, "gizmo");
    assert_eq!(rows[1].quantity, 1);
}

crate::params! {
    struct Transfer {
        from: String,
        amount: u32,
    }
}

crate::params! {
    struct Pair(String, u32);
}

#[tokio::test]
async fn params_macro_named_struct() {
    fn send(t: Transfer) {
        assert_eq!(t.from, "alice");
        assert_eq!(t.amount, 5);
    }

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["alice", "5"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    assert!(matches!(call(send, ctx).await, StepOutcome::Ok));
}

#[tokio::test]
async fn params_macro_tuple_struct() {
    fn send(Pair(from, amount): Pair) {
        assert_eq!(from, "bob");
        assert_eq!(amount, 3);
    }

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["bob", "3"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    assert!(matches!(call(send, ctx).await, StepOutcome::Ok));
}

#[tokio::test]
async fn params_macro_missing_capture() {
    fn send(_: Transfer) {}

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["alice"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    assert!(matches!(call(send, ctx).await, StepOutcome::Failed(_)));
}

crate::variants! {
    enum Status = {
        Open = "open",
        Closed = "closed",
    }
}

#[tokio::test]
async fn variants_macro_extracts() {
    fn check(status: Status) {
        assert_eq!(status, Status::Open);
        assert_eq!(status.to_string(), "open");
    }

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["open"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    assert!(matches!(call(check, ctx).await, StepOutcome::Ok));
}

#[tokio::test]
async fn variants_macro_rejects_unknown() {
    fn check(_: Status) {}

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let captures = ["pending"];
    let ctx = StepContext::new(world, &mut entities, SliceCaptures::new(&captures), None);
    let outcome = call(check, ctx).await;
    match outcome {
        StepOutcome::Failed(err) => {
            assert!(err.message().contains("expected one of"));
            assert!(err.message().contains("open"));
        }
        other => panic!("expected failure, got {other:?}"),
    }
}

#[tokio::test]
async fn fail_macro_returns_step_error() {
    fn boom() -> Result<(), StepError> {
        if true {
            crate::fail!("boom {}", 1);
        }
        Ok(())
    }

    let world = Locked::new(World::default());
    let mut entities = EntityBag::new();
    let ctx = StepContext::new(world, &mut entities, EmptyCaptures, None);
    match call(boom, ctx).await {
        StepOutcome::Failed(err) => assert_eq!(err.message(), "boom 1"),
        other => panic!("expected failure, got {other:?}"),
    }
}

#[test]
fn error_macro_literal_and_format() {
    let lit = crate::error!("static");
    assert_eq!(lit.message(), "static");
    let fmt = crate::error!("n={}", 7);
    assert_eq!(fmt.message(), "n=7");
    let expected = 1u32;
    let actual = 2u32;
    let captured = crate::error!("expected {expected}, got {actual}");
    assert_eq!(captured.message(), "expected 1, got 2");
}
