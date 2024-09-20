#version 430 core

in noperspective vec4 vertexColor;  // Input variable from the vertex shader

out vec4 FragColor;  // Output color of the fragment

void main()
{
    FragColor = vertexColor;  // Use the interpolated vertex color
}
