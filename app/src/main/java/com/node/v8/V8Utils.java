package com.node.v8;

import android.os.Handler;
import android.os.Looper;

public class V8Utils {

    private static Handler handler_;

    static Handler getHandler() {
        Looper looper = Looper.getMainLooper();
        if (handler_ == null) {
            handler_ = new Handler(looper);
        }
        return handler_;
    }

}
