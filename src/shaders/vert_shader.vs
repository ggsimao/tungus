#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aTexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 vertexColor, vertexPosition;
out vec2 texCoord;

void main() {
    gl_Position = vec4(aPos, 1.0);
    gl_Position = projection * view * model * gl_Position;
    
    vertexPosition = gl_Position.xyz;
    vertexColor = aColor;
    texCoord = aTexCoord;
}