use crate::back::init;
use crate::back::queries;

pub async fn block(args: Vec<String>) {
    if args.len() != 1 {
        println!("Usage: block <domain>");
        return;
    }
    let domain = &args[0];

    let state = init::create_app_state().await;
    if let Some(_existing) = queries::block::get(&state, domain).await {
        println!("Domain '{}' is already blocked.", domain);
        return;
    }

    queries::block::create(&state, domain).await;
    println!("Domain '{}' has been blocked.", domain);
}

pub async fn unblock(args: Vec<String>) {
    if args.len() != 1 {
        println!("Usage: unblock <domain>");
        return;
    }
    let domain = &args[0];

    let state = init::create_app_state().await;
    if queries::block::get(&state, domain).await.is_none() {
        println!("Domain '{}' is not blocked.", domain);
        return;
    }

    queries::block::delete(&state, domain).await;
    println!("Domain '{}' has been unblocked.", domain);
}

pub async fn list() {
    let state = init::create_app_state().await;
    let blocks = queries::block::get_list(&state).await;

    if blocks.is_empty() {
        println!("No blocked domains.");
        return;
    }

    println!("Blocked domains:");
    for block in blocks {
        println!("- {}", block.domain);
    }
}
