use std::convert::TryInto;
use std::sync::Arc;

use liblumen_alloc::erts::exception::system::Alloc;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::{code, Process};
use liblumen_alloc::erts::term::{atom_unchecked, Boxed, Term, Tuple};

use liblumen_alloc::ModuleFunctionArity;

pub fn place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    parent: Term,
    reference_child: Term,
) -> Result<(), Alloc> {
    process.stack_push(reference_child)?;
    process.stack_push(parent)?;
    process.place_frame(frame(), placement);

    Ok(())
}

// Private

// ```elixir
// # label 6
// # pushed to stack: (parent, reference_child)
// # returned form call: {:ok, new_child}
// # full stack: ({:ok, new_child}, parent, reference_child)
// # returns: {:ok, inserted_child}
// {:ok, inserted_child} = Lumen.Web.insert_before(parent, new_child, reference_child)
// ```
fn code(arc_process: &Arc<Process>) -> code::Result {
    arc_process.reduce();

    let ok_new_child = arc_process.stack_pop().unwrap();
    assert!(
        ok_new_child.is_tuple(),
        "ok_new_child ({:?}) is not a tuple",
        ok_new_child
    );
    let ok_new_child_tuple: Boxed<Tuple> = ok_new_child.try_into().unwrap();
    assert_eq!(ok_new_child_tuple.len(), 2);
    assert_eq!(ok_new_child_tuple[0], atom_unchecked("ok"));
    let new_child = ok_new_child_tuple[1];
    assert!(new_child.is_resource_reference());

    let parent = arc_process.stack_pop().unwrap();
    assert!(parent.is_resource_reference());

    let reference_child = arc_process.stack_pop().unwrap();
    assert!(reference_child.is_resource_reference());

    lumen_web::node::insert_before_3::place_frame_with_arguments(
        arc_process,
        Placement::Replace,
        parent,
        new_child,
        reference_child,
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
