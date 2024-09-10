#version 430 core

out vec4 color;

void main()
{
    float x = gl_FragCoord.x;
    float y = gl_FragCoord.y;
    
    float checker_size = 20.0;

    if (mod(floor(x / checker_size) + floor(y / checker_size), 2.0) == 0.0)
    {
        color = vec4(1.0, 1.0, 1.0, 1.0); 
    }
    else
    {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
