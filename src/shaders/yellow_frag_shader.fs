#version 330 core
in vec3 vertexColor, vertexPosition;
out vec4 fragColor;

void main() {
    fragColor = vec4(vertexPosition, 1.0);
}