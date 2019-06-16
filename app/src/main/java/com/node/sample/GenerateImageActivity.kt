package com.node.sample

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.os.Bundle
import android.support.v7.app.AppCompatActivity
import android.widget.Button
import android.widget.ImageView
import android.widget.Toast

import com.tbruyelle.rxpermissions2.RxPermissions

class GenerateImageActivity : AppCompatActivity() {

    private external fun blendBitmap(imageView: ImageView, pixel_size: Double, x0: Double, y0: Double)

    private fun showToast() {
        Toast.makeText(applicationContext,
                "Render successfully!", Toast.LENGTH_SHORT).show()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.processing_image)

        val rxPermissions = RxPermissions(this)
        rxPermissions.setLogging(true)

        val btnGenImage = findViewById<Button>(R.id.btnGenImage)
        val imageView = findViewById<ImageView>(R.id.imageView)

        val bmp = createImage(800, 800, Color.BLACK)
        imageView.setImageBitmap(bmp)

        btnGenImage.setOnClickListener { blendBitmap(imageView, 0.004, -2.1, -1.5) }
    }

    companion object {

        init {
            System.loadLibrary("image-gen")
        }

        fun createImage(width: Int, height: Int, color: Int): Bitmap {
            val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
            val canvas = Canvas(bitmap)
            val paint = Paint()
            paint.color = color
            canvas.drawRect(0f, 0f, width.toFloat(), height.toFloat(), paint)
            return bitmap
        }
    }
}
