use candid::CandidType;
use crate::canister::RUNTIME_STATE;
use crate::model::user::User;
use crate::runtime_state::RuntimeState;
use crate::user_map::UpdateUserResult;
use ic_cdk_macros::update;
use serde::Deserialize;

const MAX_USERNAME_LENGTH: u16 = 25;
const MIN_USERNAME_LENGTH: u16 = 2;

#[update]
fn set_username(request: Request) -> Response {
    RUNTIME_STATE.with(|state| {
        set_username_impl(request, state.borrow_mut().as_mut().unwrap())
    })
}

fn set_username_impl(request: Request, runtime_state: &mut RuntimeState) -> Response {
    let username = request.username;
    
    if username.len() > MAX_USERNAME_LENGTH as usize {
        return Response::UsernameTooLong(MAX_USERNAME_LENGTH);
    }

    if username.len() < MIN_USERNAME_LENGTH as usize {
        return Response::UsernameTooShort(MIN_USERNAME_LENGTH);
    }

    if let Some(user) = runtime_state.data.users.get_by_principal(&runtime_state.env.caller()) {
        let mut user = user.clone();
        if matches!(user, User::Unconfirmed(_)) {
            Response::UserUnconfirmed
        } else {
            user.set_username(username);
            match runtime_state.data.users.update(user) {
                UpdateUserResult::Success => Response::Success,
                UpdateUserResult::PhoneNumberTaken => panic!("PhoneNumberTaken returned when updating username"),
                UpdateUserResult::UsernameTaken => Response::UsernameTaken,
                UpdateUserResult::UserNotFound => Response::UserNotFound,
            }
        }
    } else {
        Response::UserNotFound
    }
}

#[derive(Deserialize)]
pub struct Request {
    username: String
}

#[allow(dead_code)]
#[derive(CandidType)]
pub enum Response {
    Success,
    UsernameTaken,
    UserUnconfirmed,
    UserNotFound,
    UsernameInvalid,
    UsernameTooShort(u16),
    UsernameTooLong(u16),
}
