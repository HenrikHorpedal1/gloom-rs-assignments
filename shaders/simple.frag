#version 430 core

in vec4 vertexColor;  // Input variable from the vertex shader
in vec3 normal;

out vec4 FragColor;  // Output color of the fragment

void main() {
    vec3 norm = normalize(normal);
    vec3 light_dir = normalize(vec3(0.8, -0.5, 0.6));
    float intensity = max(0.0, dot(norm, -light_dir));
    FragColor = vec4(vertexColor.rgb * intensity, vertexColor.a);
    //FragColor = vec4(norm,1);
}

