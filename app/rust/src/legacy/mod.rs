use core::borrow::BorrowMut;
use curl::easy::Easy;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
}

#[allow(dead_code)]
fn fetch_user() -> User {
    let mut handle = Easy::new();
    handle.ssl_verify_peer(false).unwrap();

    handle
        .url("https://my-json-server.typicode.com/typicode/demo/profile")
        .unwrap();

    let mut json = Vec::new();
    {
        let mut transfer = handle.transfer();
        transfer
            .borrow_mut()
            .write_function(|data| {
                json.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    let json = json.to_owned();
    assert_eq!(200, handle.response_code().unwrap());
    serde_json::from_slice(&json).unwrap()
}
