#version 430 core
layout (location = 0) in vec3 aPos;

out vec3 texCoords;

layout (std140, binding = 0) uniform Matrices {
    mat4 modelMat;
    mat4 viewMat;
    mat4 projMat;
    mat3 normalMat;
};

void main() {
    texCoords = aPos;
    gl_Position = (projMat * viewMat * vec4(aPos, 1.0)).xyww;
}