use phyto_fsm::generate_fsm;

// The `naming` option points to a template file that controls how generated types are named.
// The template uses `{name}` (UpperCamelCase) as placeholder.
// The `module` value is automatically converted to snake_case.
// See `custom_naming.tmpl` next to this file for the template,
// and `src/codegen/default_naming.tmpl` for the built-in default.
generate_fsm!(
    file_path = "test/actions/actions.puml",
    naming = "./custom_naming.tmpl"
);

// With the custom template, the generated module and types use "Machine" suffixes
// and drop the default `I` prefix on traits:
//   module:       test_fsm_machine  (default: test_fsm)
//   actions trait: TestFsmMachineActions  (default: ITestFsmActions)
//   params trait:  TestFsmMachineEventParams  (default: ITestFsmEventParams)
use test_fsm_machine::{TestFsmMachineActions, TestFsmMachineEventParams};

struct MyActions;

impl TestFsmMachineEventParams for MyActions {
    type GoToBParams = ();
    type GoToAParams = ();
}

impl TestFsmMachineActions for MyActions {
    fn handle_go_to_b(&mut self, _: ()) {}
    fn handle_go_to_a(&mut self, _: ()) {}
}

#[test]
fn custom_naming_generates_correct_types() {
    let actions = MyActions;
    let mut fsm = test_fsm_machine::start(actions);
    fsm.go_to_b(());
    fsm.go_to_a(());
}
