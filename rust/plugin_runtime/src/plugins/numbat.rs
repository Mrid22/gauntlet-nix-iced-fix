use std::cell::RefCell;
use std::rc::Rc;

use anyhow::anyhow;
use deno_core::OpState;
use deno_core::op2;
use numbat::Context;
use numbat::InterpreterResult;
use numbat::markup::Formatter;
use numbat::markup::PlainTextFormatter;
use numbat::module_importer::BuiltinModuleImporter;
use numbat::pretty_print::PrettyPrint;
use numbat::resolver::CodeSource;
use serde::Serialize;

use crate::deno::GauntletJsError;

#[derive(Clone)]
pub struct NumbatContext(Rc<RefCell<Context>>);

impl NumbatContext {
    pub fn new() -> NumbatContext {
        let mut context = Context::new(BuiltinModuleImporter::default());

        context.load_currency_module_on_demand(true);

        if cfg!(feature = "release") {
            Context::prefetch_exchange_rates();
        }

        let _ = context.interpret("use prelude", CodeSource::Internal);

        NumbatContext(Rc::new(RefCell::new(context)))
    }
}

#[derive(Debug, Serialize)]
struct NumbatResult {
    left: String,
    right: String,
}

#[op2]
#[serde]
pub fn run_numbat(state: Rc<RefCell<OpState>>, #[string] input: String) -> Result<NumbatResult, GauntletJsError> {
    let context = {
        let state = state.borrow();

        let context = state.borrow::<NumbatContext>().clone();

        context
    };

    let mut context = context.0.borrow_mut();

    let (statements, result) = context
        .interpret(&input, CodeSource::Text)
        .map_err(|err| anyhow!(err))?;

    let formatter = PlainTextFormatter;

    let expression = statements
        .iter()
        .map(|s| formatter.format(&s.pretty_print(), false))
        .collect::<Vec<_>>()
        .join(" ")
        .replace('➞', "to");

    let value = match result {
        InterpreterResult::Value(value) => format!("{}", value.pretty_print()),
        InterpreterResult::Continue => Err(anyhow!("numbat returned Continue"))?,
    };

    Ok(NumbatResult {
        left: expression,
        right: value,
    })
}
