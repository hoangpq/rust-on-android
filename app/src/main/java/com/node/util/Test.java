package com.node.util;

import java.lang.reflect.Method;

import static com.node.util.JNIUtils.getJNIMethodSignature;

class Test {

    public static void main(String[] args) throws ClassNotFoundException {
        for (Method m : Class.forName("java.util.ArrayList").getMethods()) {
            System.out.printf("%s - %s\n", m.getName(), getJNIMethodSignature(m));
        }
    }
}