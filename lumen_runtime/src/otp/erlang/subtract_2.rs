// wasm32 proptest cannot be compiled at the same time as non-wasm32 proptest, so disable tests that
// use proptest completely for wasm32
//
// See https://github.com/rust-lang/cargo/issues/4866
#[cfg(all(not(target_arch = "wasm32"), test))]
mod test;

use std::sync::Arc;

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::exception::system::Alloc;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::code::{self, result_from_exception};
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::{Atom, Term};
use liblumen_alloc::ModuleFunctionArity;

/// `-/2` infix operator
pub fn native(process: &Process, minuend: Term, subtrahend: Term) -> exception::Result {
    number_infix_operator!(minuend, subtrahend, process, checked_sub, -)
}

/// `-/2` infix operator
pub fn place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    minuend: Term,
    subtrahend: Term,
) -> Result<(), Alloc> {
    process.stack_push(subtrahend)?;
    process.stack_push(minuend)?;
    process.place_frame(frame(), placement);

    Ok(())
}

// Private

fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let minuend = arc_process.stack_pop().unwrap();
    let subtrahend = arc_process.stack_pop().unwrap();

    match native(arc_process, minuend, subtrahend) {
        Ok(sum) => {
            arc_process.return_from_call(sum)?;

            Process::call_code(arc_process)
        }
        Err(exception) => result_from_exception(arc_process, exception),
    }
}

fn frame() -> Frame {
    Frame::new(module_function_arity(), code)
}

fn function() -> Atom {
    Atom::try_from_str("self").unwrap()
}

fn module_function_arity() -> Arc<ModuleFunctionArity> {
    Arc::new(ModuleFunctionArity {
        module: super::module(),
        function: function(),
        arity: 0,
    })
}
