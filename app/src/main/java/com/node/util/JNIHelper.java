package com.node.util;

import android.support.annotation.Keep;
import android.util.SparseArray;

import com.node.util.v8.Response;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.HashMap;

@Keep
public class JNIHelper {
    private static SparseArray<Class> indexToClass = new SparseArray<>();
    private static HashMap<Class, Integer> classToIndex = new HashMap<>();

    static {
        // index to class
        indexToClass.append(0, int.class);
        indexToClass.append(1, long.class);
        indexToClass.append(2, double.class);
        indexToClass.append(3, String.class);
        // class to index
        classToIndex.put(int.class, 0);
        classToIndex.put(long.class, 1);
        classToIndex.put(double.class, 2);
        classToIndex.put(String.class, 3);
    }

    synchronized static Object callMethod(Object instance, String name, Integer[] types, Object[] values)
            throws NoSuchMethodException, InvocationTargetException, IllegalAccessException {

        Class[] classes = new Class[types.length];
        for (int i = 0; i < types.length; i++) {
            classes[i] = indexToClass.get(types[i]);
        }

        Method method = instance.getClass().getDeclaredMethod(name, classes);
        Object result = method.invoke(instance, values);

        return new Response(result, classToIndex.get(method.getReturnType()));
    }

}
