#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aTexCoord;

uniform mat4 transform;

out vec3 vertexColor, vertexPosition;
out vec2 texCoord;

void main() {
    gl_Position = vec4(aPos, 1.0);
    gl_Position = transform * gl_Position;
    
    vertexPosition = gl_Position.xyz;
    vertexColor = aColor;
    texCoord = aTexCoord;
}