use std::sync::Arc;

use liblumen_alloc::erts::exception::system::Alloc;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::{code, Process};
use liblumen_alloc::erts::term::{atom_unchecked, Term};
use liblumen_alloc::ModuleFunctionArity;

pub fn place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    parent: Term,
    old_child: Term,
) -> Result<(), Alloc> {
    process.stack_push(old_child)?;
    process.stack_push(parent)?;
    process.place_frame(frame(), placement);

    Ok(())
}

// Private

// ```elixir
// # label 4
// # pushed to stack: (parent. old_child)
// # returned form call: :ok
// # full stack: (:ok, parent, old_child)
// # returns: {:error, :hierarchy_request}
// {:error, :hierarchy_request} = Lumen.Web.replace_child(parent, old_child, parent)
// ```
fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let ok = arc_process.stack_pop().unwrap();
    assert_eq!(ok, atom_unchecked("ok"));
    let parent = arc_process.stack_pop().unwrap();
    assert!(parent.is_resource_reference());
    let old_child = arc_process.stack_pop().unwrap();
    assert!(old_child.is_resource_reference());

    lumen_web::node::replace_child_3::place_frame_with_arguments(
        arc_process,
        Placement::Replace,
        parent,
        parent,
        old_child,
    )?;

    Process::call_code(arc_process)
}

fn frame() -> Frame {
    let module_function_arity = Arc::new(ModuleFunctionArity {
        module: super::module(),
        function: super::function(),
        arity: 0,
    });

    Frame::new(module_function_arity, code)
}
