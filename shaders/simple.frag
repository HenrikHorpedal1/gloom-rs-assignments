how to modify this to demonstrate smooth and noperspective:

#version 430 core

in vec4 vertexColor;  // Input variable from the vertex shader

out vec4 FragColor;  // Output color of the fragment

void main()
{
    FragColor = vertexColor;  // Use the interpolated vertex color
}
