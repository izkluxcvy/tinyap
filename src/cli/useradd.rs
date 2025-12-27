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
    let res = user::add(&state, username, password).await;
    match res {
        Ok(_) => println!("User added successfully."),
        Err(e) => println!("Error creating user: {}", e),
    }
}
