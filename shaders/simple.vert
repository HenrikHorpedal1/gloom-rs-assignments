#version 430 core
layout (location = 0) in vec3 aPos;    // Position attribute
layout (location = 1) in vec4 aColor;  // Color attribute
layout (location = 2) in vec3 aNormal; // Normal attribute

out vec4 vertexColor;  // Output variable to pass color to fragment shader
out vec3 normal;       // Transformed normal vector

uniform mat4 transformationmat;

void main()
{
    gl_Position = transformationmat * vec4(aPos, 1.0);  // Transform position
    vertexColor = aColor;  // Pass the color to the fragment shader

    // Transform the normal directly using the 3x3 part of the transformation matrix
    vec3 transformedNormal = mat3(transformationmat) * aNormal;

    // Normalize the normal to correct any non-uniform scaling issues
    normal = normalize(transformedNormal);
}
