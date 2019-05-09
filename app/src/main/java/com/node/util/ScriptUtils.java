package com.node.util;

import android.content.Context;

import com.node.v8.V8Context;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.Arrays;
import java.util.Optional;

public class ScriptUtils {

    private static String readFileFromRawDirectory(Context context, int resourceId) {
        InputStream iStream = context.getResources().openRawResource(resourceId);
        ByteArrayOutputStream byteStream = null;
        try {
            byte[] buffer = new byte[iStream.available()];
            iStream.read(buffer);
            byteStream = new ByteArrayOutputStream();
            byteStream.write(buffer);
            byteStream.close();
            iStream.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
        assert byteStream != null;
        return byteStream.toString();
    }

    public static void require(Context ctx_, V8Context v8ctx_, int resourceId) {
        v8ctx_.eval(readFileFromRawDirectory(ctx_, resourceId));
    }

    public static void bulkEval(V8Context v8ctx_, String ...scripts) {
        Optional<String> script = Arrays.stream(scripts).reduce((sc, c) -> sc + c + "\n");
        script.ifPresent(v8ctx_::eval);
    }
}
