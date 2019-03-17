extern crate flatbuffers;

mod user_fbs;

pub unsafe fn load_user_proto(buf: Vec<u8>) {
    let user = user_fbs::users::get_root_as_user(&buf[..]);
    adb_debug!(user.name().unwrap());
}
