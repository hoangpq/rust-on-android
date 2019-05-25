try {
  let count = 0;
  $interval(function() {
    $invokeRef(++count);
  }, 1e3);
} catch (e) {
  $log(e.message);
}
