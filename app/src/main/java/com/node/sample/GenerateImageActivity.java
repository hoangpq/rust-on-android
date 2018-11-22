package com.node.sample;

import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.os.Bundle;
import android.support.v7.app.AppCompatActivity;
import android.widget.Button;
import android.widget.ImageView;
import android.widget.Toast;

import com.tbruyelle.rxpermissions2.RxPermissions;

public class GenerateImageActivity extends AppCompatActivity {

    static {
        System.loadLibrary("image-gen");
    }

    public native void blendBitmap(ImageView imageView, double pixel_size, double x0, double y0);

    public static Bitmap createImage(int width, int height, int color) {
        Bitmap bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888);
        Canvas canvas = new Canvas(bitmap);
        Paint paint = new Paint();
        paint.setColor(color);
        canvas.drawRect(0F, 0F, (float) width, (float) height, paint);
        return bitmap;
    }

    private void showToast() {
        Toast.makeText(getApplicationContext(),
                "Render successfully!", Toast.LENGTH_SHORT).show();
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.processing_image);

        final RxPermissions rxPermissions = new RxPermissions(this);
        rxPermissions.setLogging(true);

        Button btnGenImage = findViewById(R.id.btnGenImage);
        ImageView imageView = findViewById(R.id.imageView);

        Bitmap bmp = createImage(800, 800, Color.BLACK);
        imageView.setImageBitmap(bmp);

        btnGenImage.setOnClickListener(view -> {
            blendBitmap(imageView, 0.004, -2.1, -1.5);
        });
    }
}
