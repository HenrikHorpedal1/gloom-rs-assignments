#version 430 core

in vec4 vertexColor;  // Input variable from the vertex shader
in vec3 normal;

out vec4 FragColor;  // Output color of the fragment

void main()
{
    FragColor = vec4(normal,1);  // 
}

