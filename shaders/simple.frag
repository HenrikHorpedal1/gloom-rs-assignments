#version 430 core

in vec4 vertexColor; // Input color with noperspective interpolation
out vec4 FragColor;                // Output color of the fragment

void main()
{
    // Calculate the distance from the center of the triangle to the fragment position
    vec2 fragCoord = gl_FragCoord.xy / vec2(800.0, 600.0); // Adjust 800.0, 600.0 to your viewport size
    vec2 triangleCenter = vec2(0.5, 0.5); // Assuming the triangle is centered, adjust if needed

    // Create a border effect: set a threshold for color transition
    float borderWidth = 0.05; // Width of the border
    float distance = length(fragCoord - triangleCenter);

    // Set border color and inside color
    vec4 borderColor = vec4(1.0, 1.0, 1.0, 1.0); // White border
    vec4 insideColor = vertexColor;

    // Apply the border effect
    if (distance > 0.5 - borderWidth) {
        FragColor = borderColor;
    } else {
        FragColor = insideColor;
    }
}

