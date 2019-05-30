use futures::sync::mpsc;
use futures::Future;

use reqwest::r#async::Client;
use reqwest::r#async::Response;

use serde::Deserialize;

lazy_static! {
    pub static ref CLIENT: reqwest::r#async::Client = reqwest::r#async::Client::new();
}

#[derive(Deserialize, Debug)]
pub struct User {
    name: String,
}

pub fn fetch_async() -> Box<impl Future<Item = (), Error = ()>> {
    let json = |mut res: Response| res.json::<User>();

    Box::new(
        CLIENT
            .get("https://api.github.com/users/hoangpq")
            .send()
            .and_then(json)
            .and_then(move |user| {
                adb_debug!(user.name);
                Ok(())
            })
            .map_err(|_| ()),
    )
}
