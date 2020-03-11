package com.node.sample;

import android.content.Intent;
import android.graphics.Color;
import android.os.Bundle;
import android.support.annotation.Keep;
import android.support.v7.app.AppCompatActivity;
import android.util.Log;
import android.view.Gravity;
import android.widget.Button;
import android.widget.TextView;
import android.widget.Toast;

import com.node.util.JNIHelper;
import com.node.util.ResourceUtil;
import com.node.util.Util;

@Keep
public class MainActivity extends AppCompatActivity {
    private TextView txtMessage = null;

    static {
        System.loadLibrary("native-lib");
    }

    public native void initVM(Observable callbackObj);

    public native void releaseVM();

    public native void demoMain();

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        // FIXME: register class to dex helper
        Util.createReference("com/node/sample/MainActivity");
        Util.createReference("com/node/util/Util");
        Util.createReference("com/node/util/JNIHelper");

        JNIHelper.setCurrentActivity(this);
        ResourceUtil.setContext(this);

        final Button btnImageProcessing = findViewById(R.id.btImageProcessing);
        final Button evalScriptButton = findViewById(R.id.evalScriptBtn);
        txtMessage = findViewById(R.id.txtMessage);

        // Listeners
        evalScriptButton.setOnClickListener(view -> {
        });

        btnImageProcessing.setOnClickListener(view -> startActivity(
                new Intent(MainActivity.this, GenerateImageActivity.class)));

        // Init VM
        initVM(new Observable() {
            @Override
            public void subscribe(String arg) {
                runOnUiThread(() -> {
                    Toast toast = Toast.makeText(getApplicationContext(), arg, Toast.LENGTH_SHORT);
                    TextView tv = toast.getView().findViewById(android.R.id.message);
                    if (tv != null) {
                        tv.setGravity(Gravity.CENTER);
                    }
                    toast.show();
                });
            }
        });

        // initNodeJS();

        new Thread(this::demoMain).start();

    }

    @Keep
    public void setText(String text) {
        txtMessage.setText(text);
    }

    @Keep
    public int setTextColor(String color) {
        try {
            int colorCode = Color.parseColor(color);
            txtMessage.setTextColor(colorCode);
            return colorCode;
        } catch (Exception ex) {
            Log.d("Kotlin", ex.getLocalizedMessage());
        }
        return 1;
    }

    @Override
    protected void onDestroy() {
        releaseVM();
        super.onDestroy();
    }

}