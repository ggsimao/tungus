#version 430 core
layout (location = 0) in vec3 aPos;

out vec3 texCoords;

uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;

void main() {
    texCoords = aPos;
    gl_Position = (projectionMatrix * viewMatrix * vec4(aPos, 1.0)).xyww;
}