use crate::back::init;
use crate::back::user;

pub async fn useradd(args: Vec<String>) {
    if args.len() != 2 {
        println!("Usage: useradd <username> <password>");
        return;
    }

    let username = &args[0];
    let password = &args[1];

    let state = init::create_app_state().await;
    user::add_user(&state, username, password).await;
}
