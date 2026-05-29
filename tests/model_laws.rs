use dont::model::{Status, TransitionError, flag, ignore, lock, reopen, trust, undoubt};
use proptest::prelude::*;

type Transition = fn(Status) -> Result<Status, TransitionError>;

fn any_status() -> impl Strategy<Value = Status> {
    prop_oneof![
        Just(Status::Unverified),
        Just(Status::Verified),
        Just(Status::Doubted),
        Just(Status::Ignored),
        Just(Status::Locked),
    ]
}

fn ignorable_status() -> impl Strategy<Value = Status> {
    prop_oneof![
        Just(Status::Unverified),
        Just(Status::Verified),
        Just(Status::Doubted),
    ]
}

fn project_status(result: Result<Status, TransitionError>) -> Status {
    match result {
        Ok(status) => status,
        Err(err) => err.from_status,
    }
}

proptest! {
    #[test]
    fn terminal_states_are_idempotent_under_their_own_transitions(
        terminal in prop_oneof![
            Just((Status::Doubted, trust as Transition)),
            Just((Status::Verified, flag as Transition)),
            Just((Status::Ignored, ignore as Transition)),
            Just((Status::Unverified, reopen as Transition)),
            Just((Status::Locked, lock as Transition)),
        ]
    ) {
        let (status, transition) = terminal;
        prop_assert_eq!(project_status(transition(status)), status);
    }

    #[test]
    fn ignore_and_reopen_form_a_left_inverse_on_ignorable_statuses(status in ignorable_status()) {
        let ignored = ignore(status).expect("ignorable status should transition to ignored");
        let reopened = reopen(ignored).expect("ignored status should reopen to unverified");
        prop_assert_eq!(reopened, Status::Unverified);
    }

    #[test]
    fn transition_functions_are_deterministic(status in any_status()) {
        let transitions: [Transition; 6] = [trust, flag, ignore, reopen, undoubt, lock];

        for transition in transitions {
            prop_assert_eq!(transition(status), transition(status));
        }
    }
}

#[test]
fn ignore_after_reopen_restores_ignored_state() {
    let reopened = reopen(Status::Ignored).expect("ignored should reopen to unverified");
    let ignored_again =
        ignore(reopened).expect("reopened ignored entity should be ignorable again");
    assert_eq!(ignored_again, Status::Ignored);
}
