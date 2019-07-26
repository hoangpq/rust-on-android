extern crate flatbuffers;

mod user_fbs;

pub fn _load_user_buf(buf: &[u8]) -> Option<&str> {
    let user = user_fbs::users::get_root_as_user(&buf);
    user.name()
}
