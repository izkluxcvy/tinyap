use crate::back::init;
use crate::back::queries;
use crate::back::user;

pub async fn passwd(args: Vec<String>) {
    if args.len() != 2 {
        println!("Usage: passwd <username> <password>");
        return;
    }

    let username = &args[0];
    let password = &args[1];

    let state = init::create_app_state().await;
    let Some(user) = queries::user::get_by_username(&state, username).await else {
        println!("User not found.");
        return;
    };

    user::update_password(&state, user.id, password).await;
    println!("Password updated successfully.");
}
