#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;

out vec3 vertexColor;

void main() {
    gl_Position = vec4(aPos, 1.0); // we give a vec3 to vec4's constructor
    vertexColor = aColor;//vec4(0.5, 0.0, 0.0, 1.0); // output variable to dark-red
}