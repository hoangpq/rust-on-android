package com.node.util;

import java.lang.reflect.Method;
import java.util.HashMap;
import java.util.Map;

/**
 * Commodity utility for JNI
 */
public class JNIUtils {
    private static final Map<Object, String> PRIMITIVE_SIGNATURES = new HashMap<>();

    static {
        PRIMITIVE_SIGNATURES.put(boolean.class, "Z");
        PRIMITIVE_SIGNATURES.put(byte.class, "B");
        PRIMITIVE_SIGNATURES.put(char.class, "C");
        PRIMITIVE_SIGNATURES.put(double.class, "D");
        PRIMITIVE_SIGNATURES.put(float.class, "F");
        PRIMITIVE_SIGNATURES.put(int.class, "I");
        PRIMITIVE_SIGNATURES.put(long.class, "J");
        PRIMITIVE_SIGNATURES.put(short.class, "S");
        PRIMITIVE_SIGNATURES.put(void.class, "V");
    }

    private JNIUtils() {
    }

    public static String getJNIMethodSignature(Object o) {
        Method m = (Method) o;
        final StringBuilder sb = new StringBuilder("(");
        for (final Class<?> p : m.getParameterTypes()) {
            sb.append(getJNIClassSignature(p));
        }
        sb.append(')').append(getJNIClassSignature(m.getReturnType()));
        return sb.toString();
    }

    private static String getJNIClassSignature(Class<?> c) {
        if (c.isArray()) {
            final Class<?> ct = c.getComponentType();
            return '[' + getJNIClassSignature(ct);
        } else if (c.isPrimitive()) {
            return PRIMITIVE_SIGNATURES.get(c);
        } else {
            return 'L' + c.getName().replace('.', '/') + ';';
        }
    }
}
