#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;

uniform float offset_x = 0.0;

out vec3 vertexColor, vertexPosition;

void main() {
    gl_Position = vec4(aPos, 1.0); // we give a vec3 to vec4's constructor
    gl_Position.x = gl_Position.x + offset_x;
    gl_Position.y = -gl_Position.y;
    vertexPosition = gl_Position.xyz;
    vertexColor = aColor;//vec4(0.5, 0.0, 0.0, 1.0); // output variable to dark-red
}