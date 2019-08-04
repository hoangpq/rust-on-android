function assert(cond, msg = 'assert') {
  if (!cond) {
    throw Error(msg);
  }
}

const EPOCH = Date.now();
const APOCALYPSE = 2 ** 32 - 2;

// Timeout values > TIMEOUT_MAX are set to 1.
const TIMEOUT_MAX = 2 ** 31 - 1;

function getTime() {
  // TODO: use a monotonic clock.
  const now = Date.now() - EPOCH;
  assert(now >= 0 && now < APOCALYPSE);
  return now;
}

const promiseTable = new Map();
let nextPromiseId = 1;

function isStackEmpty() {
  return false;
}

Promise.prototype.finally = function finallyPolyfill(callback) {
  let constructor = this.constructor;

  return this.then(
    function(value) {
      return constructor.resolve(callback()).then(function() {
        return value;
      });
    },
    function(reason) {
      return constructor.resolve(callback()).then(function() {
        throw reason;
      });
    }
  );
};

function createResolvable() {
  let methods;
  const cmdId = nextPromiseId++;
  const promise = new Promise((resolve, reject) => {
    methods = { resolve, reject, cmdId };
  });
  const promise_ = Object.assign(promise, methods);
  promiseTable.set(cmdId, promise_);

  // Remove the promise
  promise.finally(() => {
    promiseTable.delete(promise.cmdId);
  });

  return promise_;
}

function resolve(promiseId, value) {
  if (promiseTable.has(promiseId)) {
    try {
      let promise = promiseTable.get(promiseId);
      promise.resolve(value);
      promiseTable.delete(promiseId);
    } catch (e) {
      console.log(e.message);
    }
  }
}

class Body {
  constructor(data) {
    this._data = data;
  }
  text() {
    return Promise.resolve(this._data);
  }
  json() {
    try {
      return Promise.resolve(this._data).then(JSON.parse);
    } catch (e) {
      throw new Error(`Can't not parse json data`);
    }
  }
}

function fetch(url) {
  const promise = createResolvable();
  $fetch(url, promise.cmdId);
  return promise.then(data => new Body(data));
}

let timerMap = new Map();
let nextTimerId = 1;

// timer implementation
async function setTimer(timerId, callback, delay, repeat, ...args) {
  const timer = {
    id: timerId,
    callback,
    repeat,
    delay
  };

  // console.log(getTime());

  // Add promise to microtask queue
  timerMap.set(timer.id, timer);
  const promise = createResolvable();

  // Send message to tokio backend
  $newTimer(promise.cmdId, timer.delay);

  // Wait util promise resolve
  await promise;
  Promise.resolve(timer.id).then(fire);
}

async function fire(id) {
  if (!timerMap.has(id)) return;

  const timer = timerMap.get(id);
  const callback = timer.callback;
  callback();

  if (!timer.repeat) {
    timeMap.delete(timer.id);
    return;
  }

  // Add new timer (setInterval fake)
  const promise = createResolvable();
  $newTimer(promise.cmdId, timer.delay, true);

  await promise;
  Promise.resolve(timer.id).then(fire);
}

function setTimeout(callback, delay) {
  const timerId = nextTimerId++;
  setTimer(timerId, callback, delay, false);
  return timerId;
}

function setInterval(callback, delay) {
  const timerId = nextTimerId++;
  setTimer(timerId, callback, delay, true);
  return timerId;
}

function _clearTimer(id) {
  id = Number(id);
  const timer = timerMap.get(id);
  if (timer === undefined) {
    return;
  }
  timerMap.delete(timer.id);
}

function clearInterval(id) {
  _clearTimer(id);
}

function clearTimeout(id) {
  _clearTimer(id);
}
