#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aTexCoord;

uniform float offset_x = 0.0;
uniform vec3 pos = vec3(0.0, 0.0, 0.0);

out vec3 vertexColor, vertexPosition;
out vec2 texCoord;

void main() {
    gl_Position = vec4(aPos, 1.0);
    gl_Position.x = gl_Position.x + offset_x;
    gl_Position.y = -gl_Position.y;
    
    vertexPosition = gl_Position.xyz;
    vertexColor = aColor;
    texCoord = aTexCoord;
}