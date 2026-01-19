pub fn show() {
    let help = r#"TinyAP - A tiny activitypub micro-blogging

Options:
    --help                  Show help
    --version               Show version

    serve                   Start server
    useradd      <U> <P>    Add user
    passwd       <U> <P>    Change user password
    block        <D>        Block domain
    unblock      <D>        Unblock domain
    blocklist               List blocked domains"#;
    println!("{}", help)
}
