package com.node.util;

import android.os.Handler;
import android.os.Looper;
import android.support.annotation.Keep;
import android.support.v7.app.AppCompatActivity;
import android.util.SparseArray;

import com.node.util.v8.Response;

import java.lang.ref.WeakReference;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.HashMap;

@Keep
public class JNIHelper {
    private static SparseArray<Class> indexToClass = new SparseArray<>();
    private static HashMap<Class, Integer> classToIndex = new HashMap<>();
    private static WeakReference<AppCompatActivity> currentActivity;
    private static Handler uiThreadHandler;

    static {
        // index to class
        indexToClass.append(0, int.class);
        indexToClass.append(1, long.class);
        indexToClass.append(2, double.class);
        indexToClass.append(3, String.class);
        indexToClass.append(4, Void.class);
        // class to index
        classToIndex.put(int.class, 0);
        classToIndex.put(long.class, 1);
        classToIndex.put(double.class, 2);
        classToIndex.put(String.class, 3);
        classToIndex.put(Void.class, 4);
    }

    public static Integer getIndexByClass(Class c) {
        return classToIndex.get(c);
    }

    static int intValue(Object object) {
        return (Integer) object;
    }

    static long longValue(Object object) {
        return (Long) object;
    }

    static double doubleValue(Object object) {
        return (Double) object;
    }

    public static WeakReference getCurrentActivity() {
        return currentActivity;
    }

    public static void setCurrentActivity(AppCompatActivity activity) {
        currentActivity = new WeakReference<>(activity);
    }

    private static Handler getMainLooper() {
        if (uiThreadHandler == null) {
            uiThreadHandler = new Handler(Looper.getMainLooper());
        }
        return uiThreadHandler;
    }

    public static void runTask(long task) {
        getMainLooper().post(new DenoRunnable(task));
    }

    public static long getLongValue(Object instance, String name) throws NoSuchFieldException, IllegalAccessException {
        Field field = instance.getClass().getDeclaredField(name);
        field.setAccessible(true);
        if (Modifier.isStatic(field.getModifiers())) {
            return field.getLong(null);
        }
        return field.getLong(instance);
    }

    public static boolean isField(Object instance, String field) {
        if (instance instanceof WeakReference) {
            instance = ((WeakReference) instance).get();
        }
        try {
            instance.getClass().getDeclaredField(field);
            return true;
        } catch (NoSuchFieldException e) {
            return false;
        }
    }

    public static boolean isMethod(Object instance, String method) {
        if (instance instanceof WeakReference) {
            instance = ((WeakReference) instance).get();
        }
        try {
            instance.getClass().getDeclaredMethod(method);
            return true;
        } catch (NoSuchMethodException | SecurityException e) {
            return false;
        }
    }

    static Object callMethod(Object instance, String name, Integer[] types, Object[] values) {

        if (instance instanceof WeakReference) {
            instance = ((WeakReference) instance).get();
        }

        Class[] classes = new Class[types.length];
        for (int i = 0; i < types.length; i++) {
            classes[i] = indexToClass.get(types[i]);
        }

        Object result;
        Method method;

        try {
            method = instance.getClass().getDeclaredMethod(name, classes);
            result = method.invoke(instance, values);
            if (null == result) {
                return new Response(null, Integer.MAX_VALUE);
            }
        } catch (NoSuchMethodException | InvocationTargetException | IllegalAccessException ex) {
            return new Response(null, Integer.MAX_VALUE);
        }

        return new Response(result, classToIndex.get(method.getReturnType()));
    }

}
