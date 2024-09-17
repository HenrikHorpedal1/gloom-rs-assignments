#version 430 core

in vec4 vertexColor; // Input color from vertex shader
out vec4 FragColor; // Output color of the fragment

void main()
{
    // Get the normalized device coordinates (NDC) of the fragment
    vec2 ndc = gl_FragCoord.xy / vec2(800.0, 600.0); // Assuming window size 800x600

    // Define the size of the checkerboard squares
    float squareSize = 0.1; // Size of each square

    // Compute the checkerboard pattern
    bool isWhite = mod(floor(ndc.x / squareSize) + floor(ndc.y / squareSize), 2.0) > 0.0;

    // Set the color based on the checkerboard pattern
    vec4 color = isWhite ? vec4(1.0, 1.0, 1.0, 1.0) : vertexColor; // White or vertex color

    FragColor = color; // Output the color
}
