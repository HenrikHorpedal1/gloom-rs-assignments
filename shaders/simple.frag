#version 430 core

// Input from the vertex shader
in vec4 vertexColor; // Interpolated color from the vertex shader

// Output color of the fragment
out vec4 FragColor;

void main()
{
    // Compute barycentric coordinates
    // Assuming the triangle vertices are at positions (0,0), (1,0), (0,1)
    vec2 p = gl_FragCoord.xy / vec2(800.0, 600.0); // Normalize coordinates (adjust for your viewport)
    vec2 barycentric = vec2(p.x + p.y, p.y);

    // Define border thresholds
    float borderThreshold = 0.05; // Adjust the value to change border sharpness

    // Create sharp color borders
    if (barycentric.x > 1.0 - borderThreshold || barycentric.y > 1.0 - borderThreshold) {
        FragColor = vec4(1.0, 1.0, 1.0, 1.0); // White border color
    } else {
        FragColor = vertexColor; // Original color
    }
}

