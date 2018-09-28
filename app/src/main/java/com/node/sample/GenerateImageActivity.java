package com.node.sample;

import android.Manifest;
import android.annotation.SuppressLint;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.os.Bundle;
import android.support.v7.app.AppCompatActivity;
import android.widget.Button;
import android.widget.ImageView;

import com.tbruyelle.rxpermissions2.RxPermissions;

public class GenerateImageActivity extends AppCompatActivity {

    static {
        System.loadLibrary("image-gen");
    }

    public native void generateJuliaFractal(String path, Observable listener);

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.processing_image);

        final RxPermissions rxPermissions = new RxPermissions(this);
        rxPermissions.setLogging(true);

        ImageView imageView = findViewById(R.id.imageView);
        Button btnGenImage = findViewById(R.id.btnGenImage);

        @SuppressLint("SdCardPath") final String path = "/sdcard/Download/fractal.png";
        btnGenImage.setOnClickListener(view -> rxPermissions
                .request(Manifest.permission.WRITE_EXTERNAL_STORAGE)
                .subscribe(granted -> {
                    if (granted) {
                        generateJuliaFractal(path, new Observable() {
                            @Override
                            public void subscribe() {
                                runOnUiThread(() -> {
                                    Bitmap bitmap = BitmapFactory.decodeFile(path);
                                    imageView.setImageBitmap(bitmap);
                                });
                            }
                        });
                    }
                }));
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
    }
}
