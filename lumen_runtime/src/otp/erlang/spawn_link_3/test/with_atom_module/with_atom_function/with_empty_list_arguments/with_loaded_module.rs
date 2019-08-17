use super::*;

mod with_exported_function;

#[test]
fn without_exported_function_when_run_exits_undef_and_parent_exits() {
    let parent_arc_process = process::test_init();
    let arc_scheduler = Scheduler::current();

    let priority = Priority::Normal;
    let run_queue_length_before = arc_scheduler.run_queue_len(priority);

    let module_atom = Atom::try_from_str("erlang").unwrap();
    let module = unsafe { module_atom.as_term() };

    // Typo
    let function_atom = Atom::try_from_str("sel").unwrap();
    let function = unsafe { function_atom.as_term() };

    let arguments = Term::NIL;

    let result = spawn_link_3::native(&parent_arc_process, module, function, arguments);

    assert!(result.is_ok());

    let child_pid = result.unwrap();
    let child_pid_result_pid: core::result::Result<Pid, _> = child_pid.try_into();

    assert!(child_pid_result_pid.is_ok());

    let child_pid_pid = child_pid_result_pid.unwrap();

    let run_queue_length_after = arc_scheduler.run_queue_len(priority);

    assert_eq!(run_queue_length_after, run_queue_length_before + 1);

    let arc_process = pid_to_process(&child_pid_pid).unwrap();

    assert!(arc_scheduler.run_through(&arc_process));
    assert!(!arc_scheduler.run_through(&arc_process));

    assert_eq!(arc_process.code_stack_len(), 1);
    assert_eq!(
        arc_process.current_module_function_arity(),
        Some(apply_3::module_function_arity())
    );

    match *arc_process.status.read() {
        Status::Exiting(ref runtime_exception) => {
            let runtime_undef: runtime::Exception =
                undef!(&arc_process, module, function, arguments)
                    .try_into()
                    .unwrap();

            assert_eq!(runtime_exception, &runtime_undef);
        }
        ref status => panic!("ProcessControlBlock status ({:?}) is not exiting.", status),
    };
}
