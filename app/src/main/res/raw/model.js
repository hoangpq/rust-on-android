function createUser(name) {
  try {
    const builder = new flatbuffers.Builder(0);
    const _name = builder.createString(name);

    const User = users.User;
    User.startUser(builder);
    User.addName(builder, _name);
    User.addAge(builder, 20);

    const offset = User.endUser(builder);
    builder.finish(offset);

    const bytes = builder.asUint8Array();

    const ab = new ArrayBuffer(bytes.length);
    const bufView = new Uint8Array(ab);
    bufView.set(bytes, 0);

    $send(ab)

  } catch (e) {
    $log(e.message);
  }
}
