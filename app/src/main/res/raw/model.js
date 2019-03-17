function createUser(name, age = 10) {
  try {
    const builder = new flatbuffers.Builder(0);
    const _name = builder.createString(name);

    const User = users.User;
    User.startUser(builder);
    User.addName(builder, _name);
    User.addAge(builder, age);

    const offset = User.endUser(builder);
    builder.finish(offset);

    const bytes = builder.asUint8Array();

    const ab = new ArrayBuffer(bytes.length);
    const bufView = new Uint8Array(ab);
    bufView.set(bytes, 0);

    $log($send(ab));

  } catch (e) {
    $log(e.message);
  }
}
