package com.node.sample;

import android.Manifest;
import android.annotation.SuppressLint;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.net.Uri;
import android.os.Bundle;
import android.provider.MediaStore;
import android.support.v7.app.AppCompatActivity;
import android.widget.Button;
import android.widget.ImageView;

import com.tbruyelle.rxpermissions2.RxPermissions;

import java.io.IOException;

public class GenerateImageActivity extends AppCompatActivity {

    static {
        System.loadLibrary("image-gen");
    }

    public native void generateJuliaFractal(String path, Observable listener);

    public native void blendBitmap(Bitmap bmp);

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.processing_image);

        final RxPermissions rxPermissions = new RxPermissions(this);
        rxPermissions.setLogging(true);

        // Load the bitmap into an array
        // Bitmap bmp = BitmapFactory.decodeResource(getResources(), R.drawable.macro_cover);

        ImageView imageView = findViewById(R.id.imageView);
        Uri bitmapPath = Uri.parse("android.resource://com.node.sample/drawable/macro_cover");

        Bitmap bmp = null;

        try {
            bmp = MediaStore.Images.Media.getBitmap(this.getContentResolver(), bitmapPath);
            imageView.setImageBitmap(bmp);
        } catch (IOException e) {
            e.printStackTrace();
        }

        Button btnGenImage = findViewById(R.id.btnGenImage);

        Bitmap finalBmp = bmp;
        btnGenImage.setOnClickListener(view -> {

            try {
                new Thread(() -> blendBitmap(finalBmp));
            } catch (Exception e) {
                e.printStackTrace();
            }

            /*generateJuliaFractal(path, new Observable() {
                @Override
                public void subscribe() {
                    runOnUiThread(() -> {
                        Bitmap bitmap = BitmapFactory.decodeFile(path);
                        imageView.setImageBitmap(bitmap);
                    });
                }

            });*/
        });
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
    }
}
