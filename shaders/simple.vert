#version 430 core

layout (location = 0) in vec3 aPos;    // Position attribute (location = 0)
layout (location = 1) in vec4 aColor;  // Color attribute (location = 1)

out vec4 vertexColor;  // Output variable to pass color to fragment shader

uniform float sin_value;
void main()
{
    mat4 transformationmat;

    transformationmat[0] = vec4(sin_value, 0.0, 0.0, 0.0);  // First column
    transformationmat[1] = vec4(0.0, 1.0, 0.0, 0.0);  // Second column
    transformationmat[2] = vec4(0.0, 0.0, 1.0, 0.0);  // Third column
    transformationmat[3] = vec4(0.0, 0.0, 0.0, 1.0);  // Fourth column

    gl_Position = transformationmat * vec4(aPos, 1.0);  // Convert vec3 to vec4 for position
    vertexColor = aColor;  // Pass the color to the fragment shader
}
