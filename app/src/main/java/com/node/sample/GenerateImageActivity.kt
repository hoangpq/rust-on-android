package com.node.sample

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.os.Bundle
import android.support.annotation.Keep
import android.support.v7.app.AppCompatActivity
import android.view.View
import android.widget.Button
import android.widget.ImageView
import android.widget.Toast

class GenerateImageActivity : AppCompatActivity(), View.OnClickListener {
    private external fun blendBitmap(imageView: ImageView?, renderType: Int, callback: (String) -> Unit)

    private var imageView: ImageView? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.processing_image)

        val genMandelbrot = findViewById<Button>(R.id.mandelbrot)
        val genFractal = findViewById<Button>(R.id.fractal)

        imageView = findViewById(R.id.imageView)
        imageView?.setImageBitmap(createImage(800, 800))

        genMandelbrot.setOnClickListener(this)
        genFractal.setOnClickListener(this)
    }

    @Keep
    fun onRenderDone(message: String) {
        Toast.makeText(applicationContext, message, Toast.LENGTH_SHORT).show()
    }

    override fun onClick(view: View?) {
        val renderType = when (view?.id) {
            R.id.mandelbrot -> 0x000001
            R.id.fractal -> 0x00002
            else -> 0x000003
        }

        blendBitmap(imageView, renderType, ::onRenderDone)
    }

    companion object {

        init {
            System.loadLibrary("image-gen")
        }

        @Keep
        @JvmStatic
        fun createImage(width: Int, height: Int, color: Int = Color.BLACK): Bitmap {
            val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
            val canvas = Canvas(bitmap)
            val paint = Paint()
            paint.color = color
            canvas.drawRect(0f, 0f, width.toFloat(), height.toFloat(), paint)
            return bitmap
        }
    }
}
