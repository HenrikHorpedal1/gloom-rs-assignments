#version 430 core
layout (location = 0) in vec3 aPos;    // Position attribute (location = 0)
layout (location = 1) in vec4 aColor;  // Color attribute (location = 1)
layout (location = 2) in vec3 aNormal;

out vec4 vertexColor;  // Output variable to pass color to fragment shader
out vec3 normal;


uniform mat4 transformationmat;
norm vec4;
void main()
{
    gl_Position = transformationmat * vec4(aPos, 1.0);  // Convert vec3 to vec4 for position
    vertexColor = aColor;  // Pass the color to the fragment shader

    // mat3 normalMatrix = mat3(transpose(inverse(transformationmat)));

    // normal = normalize(normalMatrix * aNormal);
    norm = transformationmat * vec4(aNormal, 1.0);
    normal = norm.xyz;
}
