#version 430 core

layout (location = 0) in vec3 aPos;    // Position attribute (location = 0)
layout (location = 1) in vec4 aColor;  // Color attribute (location = 1)

out vec4 vertexColor;  // Output variable to pass color to fragment shader

void main()
{
    gl_Position = vec4(aPos, 1.0);  // Convert vec3 to vec4 for position
    vertexColor = aColor;  // Pass the color to the fragment shader
}
